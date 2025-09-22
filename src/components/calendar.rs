use std::{collections::HashMap, sync::Mutex};

use color_eyre::{Result, eyre::Context};
use copypasta::{ClipboardContext, ClipboardProvider};
use crossterm::event::{KeyCode, KeyEvent};
use educe::Educe;
use lazy_static::lazy_static;
use ratatui::prelude::*;
use serde::Serialize;
use time::{Date, Duration, OffsetDateTime, format_description};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{
    action::{Action, Page, RelevantKey},
    config::Config,
    layout::LayoutSlot,
    persist::{self, Command, Event, TimeEntry},
};

mod widgets;
use widgets::TimesheetCalendar;

#[derive(Educe)]
#[educe(Default)]
pub struct Calendar {
    action_tx: Option<UnboundedSender<Action>>,
    persist_tx: Option<UnboundedSender<Command>>,
    suspended: bool,

    #[educe(Default(expression= OffsetDateTime::now_local()
            .expect("find local offset for date")
            .date()))]
    day: Date,
    days_with_timesheets: Vec<Date>,
    summary: Option<TimesheetSummary>,
}

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
    fn new(entries: Vec<TimeEntry>) -> Self {
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
}

impl Component for Calendar {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn register_persist_handler(&mut self, tx: UnboundedSender<persist::Command>) -> Result<()> {
        self.persist_tx = Some(tx);
        Ok(())
    }

    fn is_suspended(&self) -> bool {
        self.suspended
    }

    fn init(&mut self, _area: Size) -> Result<()> {
        self.fetch_for_new_day()?;
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        match key.code {
            _ if self.handle_day_movement(key) => Ok(None),
            KeyCode::Enter => Ok(Some(Action::SetActivePage(Page::Home { day: self.day }))),
            KeyCode::Char('c') => {
                let json =
                    serde_json::to_string(&self.summary).context("seralizing timesheet summary")?;
                let mut clip = CLIPBOARD.lock().expect("clipboard mutex not poisoned");
                match clip.set_contents(json) {
                    Ok(_) => Ok(Some(Action::SetStatusLine("Summary copied!".into()))),
                    Err(_) => Ok(Some(Action::SetStatusLine("Failed to copy".into()))),
                }
            }
            _ => Ok(None),
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let area = crate::layout::main_vert(LayoutSlot::MainCanvas, area);

        let calendar_widget =
            TimesheetCalendar::new(self.day, &self.days_with_timesheets, self.summary.as_ref());
        frame.render_widget(calendar_widget, area);

        Ok(())
    }

    fn handle_persisted(&mut self, event: persist::Event) -> Result<Option<Action>> {
        match event {
            Event::TimesheetsOfMonthLoaded { day, timesheets } if day == self.day => {
                self.days_with_timesheets = vec![];
                let format = format_description::parse("[year]-[month]-[day]")?;
                for timesheet in timesheets {
                    if let Ok(day) = Date::parse(&timesheet.day, &format) {
                        self.days_with_timesheets.push(day);
                    }
                }
            }
            Event::TimesheetLoaded {
                day,
                timesheet: _,
                entries,
            } if day == self.day => {
                self.summary = Some(TimesheetSummary::new(entries));
            }
            _ => {}
        }
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::SetActivePage(Page::Calendar { day }) => {
                self.action_tx
                    .as_mut()
                    .unwrap()
                    .send(Action::SetRelevantKeys(KEYS.to_vec()))
                    .expect("sent initial keys");
                self.day = day;
                self.suspended = false;
                self.fetch_for_new_day()?;
            }
            Action::SetActivePage(_) => {
                self.suspended = true;
            }
            _ => {}
        }
        Ok(None)
    }
}

impl Calendar {
    fn handle_day_movement(&mut self, key: KeyEvent) -> bool {
        let new_day = match key.code {
            KeyCode::PageUp => self.day.checked_sub(Duration::days(365)),
            KeyCode::Up => self.day.checked_sub(Duration::days(7)),
            KeyCode::Left => self.day.checked_sub(Duration::days(1)),
            KeyCode::Right => self.day.checked_add(Duration::days(1)),
            KeyCode::Down => self.day.checked_add(Duration::days(7)),
            KeyCode::PageDown => self.day.checked_add(Duration::days(365)),
            _ => return false,
        }
        .expect("date math not to overflow");
        self.day = new_day;
        let _ = self.fetch_for_new_day();
        true
    }

    fn fetch_for_new_day(&mut self) -> Result<()> {
        self.persist_tx
            .as_mut()
            .expect("persist tx")
            .send(Command::LoadTimesheetsOfMonth { day: self.day })?;
        self.persist_tx
            .as_mut()
            .expect("persist tx")
            .send(Command::LoadTimesheet { day: self.day })?;
        self.days_with_timesheets = vec![];
        Ok(())
    }
}

lazy_static! {
    static ref KEYS: Vec<RelevantKey> = vec![
        RelevantKey::new("Enter", "Select"),
        RelevantKey::new("c", "Copy summary"),
    ];
    static ref CLIPBOARD: Mutex<ClipboardContext> = ClipboardContext::new()
        .expect("init clipboard context")
        .into();
}
