use std::collections::HashMap;

use color_eyre::{Result, eyre::Context};
use serde::Serialize;
use time::Date;

use crate::{components::home::state::TimeItem, config::Config, shared::BREAK_PROJECT_KEY};

use super::get_project_key;

#[derive(Serialize)]
struct JsonExport {
    meta: JsonMeta,
    projects: HashMap<String, JsonProject>,
    entries: Vec<JsonEntry>,
}

#[derive(Serialize)]
struct JsonMeta {
    day: String,
    exported_at: String,
}

#[derive(Serialize)]
enum ProjectKind {
    AdHoc,
    Configured,
    SpecialBreak,
}

#[derive(Serialize)]
struct JsonProject {
    internal_name: String,
    kind: ProjectKind,
}

#[derive(Serialize)]
struct JsonEntry {
    start: String,
    end: String,
    project_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    ticket: Option<String>,
    duration_mins: u64,
    description: String,
}

pub fn generate_json_content(items: &[TimeItem], day: Date) -> Result<String> {
    let config = Config::get();

    let meta = JsonMeta {
        day: day.to_string(),
        exported_at: chrono::Utc::now().to_rfc3339(),
    };

    let used_projects: std::collections::HashSet<String> = items
        .iter()
        .filter(|item| !item.duration.is_zero())
        .map(|item| get_project_key(&item.project))
        .collect();

    let projects = used_projects
        .into_iter()
        .map(|project_key| {
            let config = config.projects.get(&project_key);
            let internal_name = if let Some(config) = config {
                config.internal_name.clone()
            } else {
                project_key.clone()
            };
            let kind = match config {
                _ if project_key == BREAK_PROJECT_KEY => ProjectKind::SpecialBreak,
                Some(_) => ProjectKind::Configured,
                _ => ProjectKind::AdHoc,
            };
            let project_info = JsonProject {
                internal_name,
                kind,
            };
            (project_key, project_info)
        })
        .collect();

    let entries: Vec<JsonEntry> = items
        .iter()
        .filter(|item| !item.duration.is_zero())
        .map(|item| {
            let start_time = item.start_time;
            let end_time = item.next_start_time();
            let project_key = get_project_key(&item.project);
            let duration_mins = item.duration.as_secs().div_ceil(60);

            JsonEntry {
                start: start_time.format("%H:%M").to_string(),
                end: end_time.format("%H:%M").to_string(),
                project_key,
                ticket: if item.ticket.is_empty() {
                    None
                } else {
                    Some(item.ticket.clone())
                },
                duration_mins,
                description: item.description.clone(),
            }
        })
        .collect();

    let json_export = JsonExport {
        meta,
        projects,
        entries,
    };

    serde_json::to_string_pretty(&json_export).context("Failed to serialize JSON export")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        components::home::state::TimeItem, config::Config, persist::TimeEntryId,
        shared::DataVersion,
    };
    use chrono::NaiveTime;
    use std::time::Duration;
    use time::macros::date;

    fn setup_test_config() {
        use std::sync::Once;
        static INIT: Once = Once::new();

        INIT.call_once(|| {
            let mut projects = std::collections::HashMap::new();
            projects.insert(
                "TEST-PROJECT".to_string(),
                crate::config::ProjectConfig {
                    internal_name: "Test Project".to_string(),
                    jira_url: Some("https://test.atlassian.net".to_string()),
                },
            );
            projects.insert(
                "W".to_string(),
                crate::config::ProjectConfig {
                    internal_name: "Work Project".to_string(),
                    jira_url: None,
                },
            );

            let test_config = Config {
                default_project_key: "TEST-PROJECT".to_string(),
                projects,
                ..Default::default()
            };

            if let Err(_) = std::panic::catch_unwind(|| {
                Config::set_for_tests(test_config.clone());
            }) {
                // Config was already set, that's fine for our tests
            }
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
        let start_time = NaiveTime::from_hms_opt(start_hour, start_minute, 0).expect("Valid time");
        let duration = Duration::from_secs(duration_minutes * 60);

        TimeItem {
            id: TimeEntryId::new(),
            start_time,
            project: project.to_string(),
            ticket: ticket.to_string(),
            description: description.to_string(),
            duration,
            version: DataVersion::fresh(),
        }
    }

    #[test]
    fn test_generate_json_content_basic() {
        setup_test_config();

        let items = vec![
            create_test_item(8, 40, 20, "", "SCRUM-17", "post vacation catchup"),
            create_test_item(9, 0, 15, "W", "TICKET-123", "work task"),
        ];

        let day = date!(2025 - 09 - 22);
        let json_content = generate_json_content(&items, day).unwrap();

        // Parse the JSON to verify structure
        let json_value: serde_json::Value = serde_json::from_str(&json_content).unwrap();

        // Check meta
        assert_eq!(json_value["meta"]["day"], "2025-09-22");
        assert!(json_value["meta"]["exported_at"].is_string());

        // Check projects
        assert!(json_value["projects"]["TEST-PROJECT"].is_object());
        assert_eq!(
            json_value["projects"]["TEST-PROJECT"]["internal_name"],
            "Test Project"
        );
        assert_eq!(
            json_value["projects"]["TEST-PROJECT"]["is_configured"],
            true
        );

        assert!(json_value["projects"]["W"].is_object());
        assert_eq!(json_value["projects"]["W"]["internal_name"], "Work Project");
        assert_eq!(json_value["projects"]["W"]["is_configured"], true);

        // Check entries
        let entries = json_value["entries"].as_array().unwrap();
        assert_eq!(entries.len(), 2);

        // First entry
        assert_eq!(entries[0]["start"], "08:40");
        assert_eq!(entries[0]["end"], "09:00");
        assert_eq!(entries[0]["project_key"], "TEST-PROJECT");
        assert_eq!(entries[0]["ticket"], "SCRUM-17");
        assert_eq!(entries[0]["duration_mins"], 20);
        assert_eq!(entries[0]["description"], "post vacation catchup");

        // Second entry
        assert_eq!(entries[1]["start"], "09:00");
        assert_eq!(entries[1]["end"], "09:15");
        assert_eq!(entries[1]["project_key"], "W");
        assert_eq!(entries[1]["ticket"], "TICKET-123");
        assert_eq!(entries[1]["duration_mins"], 15);
        assert_eq!(entries[1]["description"], "work task");
    }

    #[test]
    fn test_generate_json_content_pause_conversion() {
        setup_test_config();

        let items = vec![create_test_item(12, 5, 50, "x", "", "lunch break")];

        let day = date!(2025 - 09 - 22);
        let json_content = generate_json_content(&items, day).unwrap();

        let json_value: serde_json::Value = serde_json::from_str(&json_content).unwrap();
        let entries = json_value["entries"].as_array().unwrap();

        assert_eq!(entries[0]["project_key"], "x");
        assert!(entries[0]["ticket"].is_null());
    }

    #[test]
    fn test_generate_json_content_skip_zero_duration() {
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

        let day = date!(2025 - 09 - 22);
        let json_content = generate_json_content(&items, day).unwrap();

        let json_value: serde_json::Value = serde_json::from_str(&json_content).unwrap();
        let entries = json_value["entries"].as_array().unwrap();

        // Should have 2 entries (zero duration item skipped)
        assert_eq!(entries.len(), 2);
        assert!(!json_content.contains("should be skipped"));
    }
}
