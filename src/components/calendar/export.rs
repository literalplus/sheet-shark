use color_eyre::{Result, eyre::Context};
use time::{Date, format_description};

use crate::shared::{BREAK_PROJECT_KEY, summary::TimesheetSummary};

pub fn export_to_jira(day: Date, summary: &TimesheetSummary) -> Result<u32> {
    let mut url_count = 0;
    let date_str = format_date(day)?;
    let time_str = get_start_time(summary);

    for (project_key, project_summary) in &summary.projects {
        if project_key == BREAK_PROJECT_KEY {
            continue;
        }

        url_count += export_project_tickets(project_summary, &date_str, &time_str)?;
    }

    Ok(url_count)
}

fn format_date(day: Date) -> Result<String> {
    let date_format = format_description::parse("[day].[month].[year repr:last_two]")?;
    Ok(day.format(&date_format)?)
}

fn get_start_time(summary: &TimesheetSummary) -> String {
    summary
        .start_time
        .clone()
        .unwrap_or_else(|| "09:00".to_string())
}

fn export_project_tickets(
    project_summary: &crate::shared::summary::ProjectSummary,
    date_str: &str,
    time_str: &str,
) -> Result<u32> {
    let project_config = match &project_summary.config {
        Some(config) => config,
        None => return Ok(0), // Early return if no config
    };

    let jira_base_url = match &project_config.jira_url {
        Some(url) => url,
        None => return Ok(0), // Early return if no jira_url
    };

    let mut count = 0;

    for (ticket_key, duration) in &project_summary.ticket_sums {
        if ticket_key == "-" || duration.is_zero() {
            continue;
        }

        let minutes = duration.whole_minutes();
        let booking_url =
            format_booking_url(jira_base_url, ticket_key, minutes, date_str, time_str);

        open_url(&booking_url)?;
        count += 1;
    }

    Ok(count)
}

fn format_booking_url(
    jira_base_url: &str,
    ticket_key: &str,
    minutes: i64,
    date_str: &str,
    time_str: &str,
) -> String {
    format!(
        "{}/browse/{}?xxLogTime={}m&xxLogDate={}%20{}",
        jira_base_url.trim_end_matches('/'),
        ticket_key,
        minutes,
        date_str,
        time_str,
    )
}

fn open_url(url: &str) -> Result<()> {
    std::process::Command::new("xdg-open")
        .arg(url)
        .spawn()
        .context("Failed to open URL in browser")?;
    Ok(())
}
