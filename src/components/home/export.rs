use std::{fs, io::Write, path::PathBuf};

use chrono::{NaiveTime, Timelike};
use color_eyre::{Result, eyre::Context};
use csv::WriterBuilder;
use time::Date;

use crate::{
    components::home::state::TimeItem,
    config::{Config, get_data_dir},
};

pub fn export_to_csv(items: &[TimeItem], day: Date) -> Result<()> {
    let file_path = get_export_file_path(day)?;

    // Create directory structure if it doesn't exist
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).wrap_err("Failed to create export directory")?;
    }

    let file = fs::File::create(&file_path)
        .wrap_err_with(|| format!("Failed to create CSV file at {}", file_path.display()))?;

    generate_csv_content(items, file)?;

    Ok(())
}

pub fn generate_csv_content<W: Write>(items: &[TimeItem], writer: W) -> Result<()> {
    let mut wtr = WriterBuilder::new().has_headers(false).from_writer(writer);

    write_csv_header(&mut wtr)?;

    for item in items {
        if item.duration.is_zero() {
            continue; // Skip items with zero duration
        }

        let start_time = item.start_time;
        let end_time = item.next_start_time();

        let project_key = get_project_key(&item.project);

        write_csv_record(
            &mut wtr,
            start_time,
            end_time,
            &project_key,
            &item.ticket,
            &item.description,
            item.duration.as_secs(),
        )?;
    }

    wtr.flush().wrap_err("Failed to flush CSV writer")?;

    Ok(())
}

fn get_project_key(project: &str) -> String {
    if project.is_empty() {
        Config::get().default_project_key.clone()
    } else {
        project.to_string()
    }
}

fn write_csv_header<W: Write>(wtr: &mut csv::Writer<W>) -> Result<()> {
    wtr.write_record([
        "", // empty column
        "start",
        "",
        "",
        "", // start columns
        "end",
        "",
        "",
        "", // end columns
        "proj",
        "tracking code",
        "",
        "",
        "", // project and tracking columns
        "duration",
        "min",
        "h", // duration columns
    ])
    .wrap_err("Failed to write CSV header")
}

fn write_csv_record<W: Write>(
    wtr: &mut csv::Writer<W>,
    start_time: NaiveTime,
    end_time: NaiveTime,
    project_key: &str,
    ticket: &str,
    description: &str,
    duration_secs: u64,
) -> Result<()> {
    let start_hour = start_time.hour();
    let start_minute = start_time.minute();
    let start_formatted = start_time.format("%H:%M:%S").to_string();

    let end_hour = end_time.hour();
    let end_minute = end_time.minute();
    let end_formatted = end_time.format("%H:%M:%S").to_string();

    let duration_minutes = duration_secs.div_ceil(60); // Round up to next minute
    let duration_hours = duration_secs as f64 / 3600.0;
    let duration_formatted = format_duration_hms(duration_secs);

    let project_key = if project_key == "Pau" {
        "Pause"
    } else {
        project_key
    };

    // Format: ,start_hour,start_minute,start_time,,end_hour,end_minute,end_time,,,project_key,ticket,description,,duration_formatted,duration_minutes,duration_hours
    wtr.write_record([
        "", // empty column
        &start_hour.to_string(),
        &start_minute.to_string(),
        &start_formatted,
        "", // empty column
        &end_hour.to_string(),
        &end_minute.to_string(),
        &end_formatted,
        "", // empty column
        project_key,
        ticket,
        "", // empty column
        description,
        "", // empty column
        &duration_formatted,
        &duration_minutes.to_string(),
        &duration_hours.to_string(),
    ])
    .wrap_err("Failed to write CSV record")?;

    Ok(())
}

fn format_duration_hms(duration_secs: u64) -> String {
    let hours = duration_secs / 3600;
    let minutes = (duration_secs % 3600) / 60;
    let seconds = duration_secs % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

fn get_export_file_path(day: Date) -> Result<PathBuf> {
    let data_dir = get_data_dir();
    let year = day.year();
    let month = day.month() as u8;
    let day_num = day.day();

    let filename = format!("{:04}-{:02}-{:02}.csv", year, month, day_num);
    let file_path = data_dir
        .join("exports")
        .join(year.to_string())
        .join(format!("{:02}", month))
        .join(filename);

    Ok(file_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime;
    use std::time::Duration;

    fn setup_test_config() {
        use std::sync::Once;
        static INIT: Once = Once::new();

        INIT.call_once(|| {
            let test_config = Config {
                default_project_key: "TEST-PROJECT".to_string(),
                ..Default::default()
            };
            Config::set_for_tests(test_config);
        });
    }

    fn create_test_item(
        start_hour: u32,
        start_minute: u32,
        duration_minutes: u64,
        project: &str,
        ticket: &str,
        description: &str,
    ) -> TimeItem {
        let start_time = NaiveTime::from_hms_opt(start_hour, start_minute, 0).unwrap();
        let duration = Duration::from_secs(duration_minutes * 60);

        TimeItem {
            id: crate::persist::TimeEntryId::new(),
            start_time,
            project: project.to_string(),
            ticket: ticket.to_string(),
            description: description.to_string(),
            duration,
            version: crate::shared::DataVersion::fresh(),
        }
    }

    #[test]
    fn test_generate_csv_content_basic() {
        setup_test_config();

        let items = vec![
            create_test_item(8, 40, 20, "", "SCRUM-17", "post vacation catchup"),
            create_test_item(9, 0, 15, "", "SCRUM-17", "abst clemens+"),
        ];

        let mut output = Vec::new();
        generate_csv_content(&items, &mut output).unwrap();

        let csv_string = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = csv_string.lines().collect();

        // Check header
        assert_eq!(
            lines[0],
            ",start,,,,end,,,,proj,tracking code,,,,duration,min,h"
        );

        // Check first data row - should use TEST-PROJECT as default
        assert!(lines[1].contains("8,40,08:40:00"));
        assert!(lines[1].contains("9,0,09:00:00"));
        assert!(lines[1].contains("TEST-PROJECT"));
        assert!(lines[1].contains("SCRUM-17"));
        assert!(lines[1].contains("post vacation catchup"));
        assert!(lines[1].contains("00:20:00"));
        assert!(lines[1].contains("20"));
        // Just check that the fractional hour is approximately 1/3
        assert!(lines[1].contains("0.333333"));

        // Check second data row
        assert!(lines[2].contains("9,0,09:00:00"));
        assert!(lines[2].contains("9,15,09:15:00"));
        assert!(lines[2].contains("TEST-PROJECT"));
        assert!(lines[2].contains("SCRUM-17"));
        assert!(lines[2].contains("abst clemens+"));
        assert!(lines[2].contains("00:15:00"));
        assert!(lines[2].contains("15"));
        assert!(lines[2].contains("0.25"));
    }

    #[test]
    fn test_generate_csv_content_pause_conversion() {
        setup_test_config();

        let items = vec![create_test_item(12, 5, 50, "Pau", "", "lunch break")];

        let mut output = Vec::new();
        generate_csv_content(&items, &mut output).unwrap();

        let csv_string = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = csv_string.lines().collect();

        // Check that "Pau" is converted to "Pause"
        assert!(lines[1].contains("Pause"));
        assert!(!lines[1].contains("Pau,"));
    }

    #[test]
    fn test_generate_csv_content_skip_zero_duration() {
        setup_test_config();

        let items = vec![
            create_test_item(8, 40, 20, "", "SCRUM-17", "real work"),
            TimeItem {
                id: crate::persist::TimeEntryId::new(),
                start_time: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
                project: "".to_string(),
                ticket: "EMPTY-1".to_string(),
                description: "should be skipped".to_string(),
                duration: Duration::from_secs(0),
                version: crate::shared::DataVersion::fresh(),
            },
            create_test_item(9, 0, 15, "", "SCRUM-17", "more work"),
        ];

        let mut output = Vec::new();
        generate_csv_content(&items, &mut output).unwrap();

        let csv_string = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = csv_string.lines().collect();

        // Should have header + 2 data rows (zero duration item skipped)
        assert_eq!(lines.len(), 3);
        assert!(!csv_string.contains("should be skipped"));
        assert!(csv_string.contains("real work"));
        assert!(csv_string.contains("more work"));
    }

    #[test]
    fn test_format_duration_hms() {
        assert_eq!(format_duration_hms(3661), "01:01:01"); // 1 hour, 1 minute, 1 second
        assert_eq!(format_duration_hms(3600), "01:00:00"); // 1 hour
        assert_eq!(format_duration_hms(60), "00:01:00"); // 1 minute
        assert_eq!(format_duration_hms(1), "00:00:01"); // 1 second
        assert_eq!(format_duration_hms(0), "00:00:00"); // 0 seconds
    }

    #[test]
    fn test_csv_format_structure() {
        setup_test_config();

        let items = vec![create_test_item(
            8,
            40,
            20,
            "PROJECT-1",
            "TICKET-123",
            "test description",
        )];

        let mut output = Vec::new();
        generate_csv_content(&items, &mut output).unwrap();

        let csv_string = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = csv_string.lines().collect();

        // Parse the data row and check column count and structure
        let data_row = lines[1];
        let columns: Vec<&str> = data_row.split(',').collect();

        // Should have 17 columns total
        assert_eq!(columns.len(), 17);

        // Check specific column positions
        assert_eq!(columns[0], ""); // empty
        assert_eq!(columns[1], "8"); // start hour
        assert_eq!(columns[2], "40"); // start minute
        assert_eq!(columns[3], "08:40:00"); // start time
        assert_eq!(columns[4], ""); // empty
        assert_eq!(columns[5], "9"); // end hour
        assert_eq!(columns[6], "0"); // end minute
        assert_eq!(columns[7], "09:00:00"); // end time
        assert_eq!(columns[8], ""); // empty
        assert_eq!(columns[9], "PROJECT-1"); // project
        assert_eq!(columns[10], "TICKET-123"); // ticket
        assert_eq!(columns[11], ""); // empty
        assert_eq!(columns[12], "test description"); // description
        assert_eq!(columns[13], ""); // empty
        assert_eq!(columns[14], "00:20:00"); // duration formatted
        assert_eq!(columns[15], "20"); // duration minutes
        // Check that fractional hours is approximately correct (20 minutes = 1/3 hour)
        let hours: f64 = columns[16].parse().unwrap();
        assert!((hours - 0.3333333333333333).abs() < 0.0001);
    }
}
