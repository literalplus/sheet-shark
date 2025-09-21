use std::{fs, path::PathBuf};

use color_eyre::{Result, eyre::Context};
use time::Date;

use crate::{
    components::home::state::TimeItem,
    config::{Config, get_data_dir},
};

pub mod csv;
pub mod json;

pub fn export_timesheet(items: &[TimeItem], day: Date) -> Result<()> {
    let csv_path = build_export_file_path(day, "csv")?;
    let json_path = build_export_file_path(day, "json")?;

    if let Some(parent) = csv_path.parent() {
        fs::create_dir_all(parent).wrap_err("Failed to create export directory")?;
    }

    let csv_file = fs::File::create(&csv_path)
        .with_context(|| format!("Failed to create CSV file at {}", csv_path.display()))?;
    csv::generate_csv_content(items, csv_file)?;

    let json_content = json::generate_json_content(items, day)?;
    fs::write(&json_path, json_content)
        .with_context(|| format!("Failed to write JSON file at {}", json_path.display()))?;

    Ok(())
}

fn build_export_file_path(day: Date, extension: &str) -> Result<PathBuf> {
    let data_dir = get_data_dir();
    let year = day.year();
    let month = u8::from(day.month());
    let day_num = day.day();

    let filename = format!("{year:04}-{month:02}-{day_num:02}.{extension}");
    let file_path = data_dir
        .join("exports")
        .join(year.to_string())
        .join(format!("{month:02}"))
        .join(filename);

    Ok(file_path)
}

pub(super) fn get_project_key(project: &str) -> String {
    if project.is_empty() {
        Config::get().default_project_key.clone()
    } else {
        project.to_string()
    }
}
