use std::collections::VecDeque;

use serde::Serialize;

use crate::shared::{BREAK_PROJECT_KEY, summary::TimesheetSummary};

#[derive(Serialize, Debug, Clone)]
pub struct DefragmentedEntry {
    pub project_key: String,
    pub ticket_key: String,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Debug, Clone)]
struct ProjectTicket {
    project_key: String,
    ticket_key: String,
    duration_mins: u32,
}

#[derive(Debug, Clone)]
struct Break {
    start_mins: u32,
    duration_mins: u32,
}

impl Break {
    fn has_started_at(&self, ref_time_minutes: u32) -> bool {
        self.start_mins <= ref_time_minutes
    }
}

pub fn calculate(summary: &TimesheetSummary) -> Vec<DefragmentedEntry> {
    let start_time = match &summary.start_time {
        Some(time) => time.as_str(),
        None => return Vec::new(),
    };

    let start_minutes = parse_time_to_minutes(start_time).expect("Valid start time");
    let project_tickets = collect_project_tickets_in_chronological_order(summary);

    if project_tickets.is_empty() {
        return Vec::new();
    }

    let breaks = parse_breaks(summary);
    allocate_project_tickets_with_breaks(project_tickets, breaks, start_minutes)
}

/// Collects project tickets sorted by the first start time of each project
fn collect_project_tickets_in_chronological_order(
    summary: &TimesheetSummary,
) -> Vec<ProjectTicket> {
    let mut project_keys: Vec<(&String, &Option<String>)> = summary
        .projects
        .iter()
        .filter(|(project_key, _)| *project_key != BREAK_PROJECT_KEY)
        .map(|(key, summary)| (key, &summary.first_start))
        .collect();

    project_keys.sort_by_key(|(_, first_start)| *first_start);

    let mut project_tickets = Vec::new();
    for (project_key, _) in project_keys {
        let project_summary = &summary.projects[project_key];
        for (ticket_key, duration) in &project_summary.ticket_sums {
            let minutes = duration.whole_minutes();
            if minutes > 0 {
                project_tickets.push(ProjectTicket {
                    project_key: project_key.clone(),
                    ticket_key: ticket_key.clone(),
                    duration_mins: minutes as u32,
                });
            }
        }
    }

    project_tickets
}

/// Converts breaks from the summary into sorted internal format
fn parse_breaks(summary: &TimesheetSummary) -> Vec<Break> {
    let mut breaks: Vec<Break> = summary
        .breaks
        .iter()
        .filter_map(|b| {
            Some(Break {
                start_mins: parse_time_to_minutes(&b.start_time)?,
                duration_mins: b.duration_mins,
            })
        })
        .collect();

    breaks.sort_by_key(|b| b.start_mins);
    breaks
}

fn allocate_project_tickets_with_breaks(
    project_tickets: Vec<ProjectTicket>,
    breaks: Vec<Break>,
    start_minutes: u32,
) -> Vec<DefragmentedEntry> {
    let mut result = Vec::new();
    let mut breaks = VecDeque::from(breaks);
    let mut current_minutes = start_minutes;

    for project_ticket in project_tickets {
        let mut remaining_minutes = project_ticket.duration_mins;

        while remaining_minutes > 0 {
            while let Some(next_break) = breaks.front()
                && next_break.has_started_at(current_minutes)
            {
                current_minutes += next_break.duration_mins;
                breaks.pop_front();
            }

            // Now we know that current_minutes is not during a break

            let mut next_end = current_minutes + remaining_minutes;
            if let Some(next_break) = breaks.front()
                && next_break.has_started_at(next_end)
            {
                next_end = next_break.start_mins; // >0 because current_minutes not_in break
            }
            let next_duration = next_end - current_minutes;

            result.push(DefragmentedEntry {
                project_key: project_ticket.project_key.clone(),
                ticket_key: project_ticket.ticket_key.clone(),
                start_time: minutes_to_string(current_minutes),
                end_time: minutes_to_string(current_minutes + next_duration),
            });

            remaining_minutes -= next_duration;
            current_minutes += next_duration;
        }
    }

    result
}

fn parse_time_to_minutes(time: &str) -> Option<u32> {
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() == 2 {
        if let (Ok(hours), Ok(minutes)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
            return Some(hours * 60 + minutes);
        }
    }
    None
}

fn minutes_to_string(minutes: u32) -> String {
    let hours = minutes / 60;
    let mins = minutes % 60;
    format!("{:02}:{:02}", hours, mins)
}

#[cfg(test)]
mod tests {
    use crate::config::Config;

    use super::*;
    use crate::persist::TimeEntry;

    #[test]
    fn test_no_breaks() {
        Config::set_for_tests(Default::default());
        let entries = vec![
            TimeEntry {
                id: "1".to_string(),
                timesheet_day: "2026-01-08".to_string(),
                start_time: "09:00".to_string(),
                duration_mins: 120,
                project_key: "PROJECT1".to_string(),
                ticket_key: Some("TICKET-1".to_string()),
                description: String::new(),
            },
            TimeEntry {
                id: "2".to_string(),
                timesheet_day: "2026-01-08".to_string(),
                start_time: "11:00".to_string(),
                duration_mins: 60,
                project_key: "PROJECT2".to_string(),
                ticket_key: Some("TICKET-2".to_string()),
                description: String::new(),
            },
        ];

        let summary = TimesheetSummary::new(entries);
        let result = calculate(&summary);

        assert_eq!(result.len(), 2); // Now we have 2 separate entries

        // Projects should be ordered by first_start time
        // PROJECT1 starts at 09:00, PROJECT2 starts at 11:00
        assert_eq!(result[0].project_key, "PROJECT1");
        assert_eq!(result[0].ticket_key, "TICKET-1");
        assert_eq!(result[0].start_time, "09:00");
        assert_eq!(result[0].end_time, "11:00"); // 120 minutes

        assert_eq!(result[1].project_key, "PROJECT2");
        assert_eq!(result[1].ticket_key, "TICKET-2");
        assert_eq!(result[1].start_time, "11:00");
        assert_eq!(result[1].end_time, "12:00"); // 60 minutes
    }

    #[test]
    fn test_with_break() {
        Config::set_for_tests(Default::default());
        let entries = vec![
            TimeEntry {
                id: "1".to_string(),
                timesheet_day: "2026-01-08".to_string(),
                start_time: "09:00".to_string(),
                duration_mins: 120,
                project_key: "PROJECT1".to_string(),
                ticket_key: Some("TICKET-1".to_string()),
                description: String::new(),
            },
            TimeEntry {
                id: "2".to_string(),
                timesheet_day: "2026-01-08".to_string(),
                start_time: "11:00".to_string(),
                duration_mins: 30,
                project_key: BREAK_PROJECT_KEY.to_string(),
                ticket_key: None,
                description: String::new(),
            },
            TimeEntry {
                id: "3".to_string(),
                timesheet_day: "2026-01-08".to_string(),
                start_time: "11:30".to_string(),
                duration_mins: 60,
                project_key: "PROJECT2".to_string(),
                ticket_key: Some("TICKET-2".to_string()),
                description: String::new(),
            },
        ];

        let summary = TimesheetSummary::new(entries);
        let result = calculate(&summary);

        // Should have 2 entries: PROJECT1 before break, PROJECT2 after break
        assert_eq!(result.len(), 2);

        // PROJECT1 starts at 09:00, gets allocated first
        assert_eq!(result[0].project_key, "PROJECT1");
        assert_eq!(result[0].ticket_key, "TICKET-1");
        assert_eq!(result[0].start_time, "09:00");
        assert_eq!(result[0].end_time, "11:00"); // Stops at break

        // PROJECT2 starts at 11:30, gets allocated after the break
        assert_eq!(result[1].project_key, "PROJECT2");
        assert_eq!(result[1].ticket_key, "TICKET-2");
        assert_eq!(result[1].start_time, "11:30"); // Resumes after break
        assert_eq!(result[1].end_time, "12:30"); // 60 minutes
    }

    #[test]
    fn test_break_cuts_allocation() {
        Config::set_for_tests(Default::default());
        let entries = vec![
            TimeEntry {
                id: "1".to_string(),
                timesheet_day: "2026-01-08".to_string(),
                start_time: "09:00".to_string(),
                duration_mins: 180, // 3 hours of project work
                project_key: "PROJECT1".to_string(),
                ticket_key: Some("TICKET-1".to_string()),
                description: String::new(),
            },
            TimeEntry {
                id: "2".to_string(),
                timesheet_day: "2026-01-08".to_string(),
                start_time: "10:30".to_string(),
                duration_mins: 30, // 30 min break in the middle
                project_key: BREAK_PROJECT_KEY.to_string(),
                ticket_key: None,
                description: String::new(),
            },
        ];

        let summary = TimesheetSummary::new(entries);
        let result = calculate(&summary);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].project_key, "PROJECT1");
        assert_eq!(result[0].ticket_key, "TICKET-1");
        assert_eq!(result[0].start_time, "09:00");
        assert_eq!(result[0].end_time, "10:30"); // 90 minutes before break
        assert_eq!(result[1].project_key, "PROJECT1");
        assert_eq!(result[1].ticket_key, "TICKET-1");
        assert_eq!(result[1].start_time, "11:00"); // Resume after break
        assert_eq!(result[1].end_time, "12:30"); // 90 remaining minutes
    }
}
