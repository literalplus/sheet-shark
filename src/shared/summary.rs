use std::collections::HashMap;

use serde::Serialize;
use time::Duration;

use crate::{config::Config, persist::TimeEntry, shared::BREAK_PROJECT_KEY};

#[derive(Serialize)]
pub struct ProjectSummary {
    pub internal_name: Option<String>,
    pub ticket_sums: HashMap<String, Duration>,
}

#[derive(Serialize)]
pub struct TimesheetSummary {
    pub projects: HashMap<String, ProjectSummary>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

impl TimesheetSummary {
    pub fn new(entries: Vec<TimeEntry>) -> Self {
        let config = Config::get();
        let mut projects: HashMap<String, ProjectSummary> = HashMap::new();

        let mut start_time: Option<String> = None;
        let mut end_time: Option<String> = None;

        for entry in entries.iter() {
            let duration = Duration::minutes(entry.duration_mins as i64);
            if duration.is_zero() {
                continue;
            }

            let project_key = &entry.project_key;
            let ticket = entry.ticket_key.as_deref().unwrap_or("-").to_string();

            let project_summary = projects
                .entry(project_key.clone())
                .or_insert_with(|| Self::create_project_summary(project_key, config));

            *project_summary
                .ticket_sums
                .entry(ticket)
                .or_insert(Duration::ZERO) += duration;
        }

        // Calculate start and end times from non-zero duration entries
        let working_entries: Vec<_> = entries
            .iter()
            .filter(|entry| entry.duration_mins > 0)
            .collect();

        if !working_entries.is_empty() {
            // Find earliest start time
            start_time = working_entries
                .iter()
                .map(|entry| &entry.start_time)
                .min()
                .cloned();

            // Find latest end time (start_time + duration)
            end_time = working_entries
                .iter()
                .filter_map(|entry| {
                    // Parse start_time and add duration to get end_time
                    let start_parts: Vec<&str> = entry.start_time.split(':').collect();
                    if start_parts.len() == 2 {
                        if let (Ok(hours), Ok(minutes)) =
                            (start_parts[0].parse::<u32>(), start_parts[1].parse::<u32>())
                        {
                            let start_minutes = hours * 60 + minutes;
                            let end_minutes = start_minutes + entry.duration_mins as u32;
                            let end_hours = end_minutes / 60;
                            let end_mins = end_minutes % 60;
                            Some(format!("{:02}:{:02}", end_hours, end_mins))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .max();
        }

        Self {
            projects,
            start_time,
            end_time,
        }
    }

    fn create_project_summary(project_key: &str, config: &Config) -> ProjectSummary {
        let internal_name = config
            .projects
            .get(project_key)
            .map(|p| p.internal_name.clone());

        ProjectSummary {
            internal_name,
            ticket_sums: HashMap::new(),
        }
    }

    pub fn calculate_total_duration(&self) -> Duration {
        self.projects
            .iter()
            .filter(|(project_key, _)| project_key != &BREAK_PROJECT_KEY)
            .flat_map(|(_, project_summary)| project_summary.ticket_sums.values())
            .sum()
    }
}
