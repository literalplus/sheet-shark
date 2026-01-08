use std::sync::Mutex;

use color_eyre::{Result, eyre::Context};
use copypasta::{ClipboardContext, ClipboardProvider};
use crossterm::event::{KeyCode, KeyEvent};
use educe::Educe;
use lazy_static::lazy_static;
use ratatui::prelude::*;
use time::{Date, Duration, OffsetDateTime, format_description};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{
    action::{Action, Page, RelevantKey},
    layout::LayoutSlot,
    persist::{self, Command, Event, TimeEntry},
    shared::summary::{SummaryJson, TimesheetSummary},
};

mod widgets;
use widgets::TimesheetCalendar;

mod export;

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
    entries: Vec<TimeEntry>,
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
                if let Some(_summary) = &self.summary {
                    let summary_json = SummaryJson::from_entries(self.entries.clone());
                    let json = serde_json::to_string(&summary_json)
                        .context("serializing timesheet summary")?;
                    let mut clip = CLIPBOARD.lock().expect("clipboard mutex not poisoned");
                    match clip.set_contents(json) {
                        Ok(_) => Ok(Some(Action::SetStatusLine("Summary copied!".into()))),
                        Err(_) => Ok(Some(Action::SetStatusLine("Failed to copy".into()))),
                    }
                } else {
                    Ok(Some(Action::SetStatusLine("No summary available".into())))
                }
            }
            KeyCode::Char('e') => {
                if let Some(summary) = &self.summary {
                    match export::export(self.day, summary) {
                        Ok(()) => Ok(Some(Action::SetStatusLine("Exported!".into()))),
                        Err(e) => Ok(Some(Action::SetStatusLine(format!("Export failed: {e}")))),
                    }
                } else {
                    Ok(Some(Action::SetStatusLine(
                        "No timesheet data to export".into(),
                    )))
                }
            }
            KeyCode::Char('f') => {
                let data_dir = crate::config::get_data_dir();
                match std::process::Command::new("xdg-open")
                    .arg(&data_dir)
                    .spawn()
                {
                    Ok(_) => Ok(Some(Action::SetStatusLine("Opened data directory".into()))),
                    Err(e) => Ok(Some(Action::SetStatusLine(format!(
                        "Failed to open directory: {e}"
                    )))),
                }
            }
            KeyCode::Char('F') => {
                let config_dir = crate::config::get_config_dir();
                match std::process::Command::new("xdg-open")
                    .arg(&config_dir)
                    .spawn()
                {
                    Ok(_) => Ok(Some(Action::SetStatusLine(
                        "Opened config directory".into(),
                    ))),
                    Err(e) => Ok(Some(Action::SetStatusLine(format!(
                        "Failed to open directory: {e}"
                    )))),
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
                self.entries = entries.clone();
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
        RelevantKey::new("e", "Export to Jira"),
    ];
    static ref CLIPBOARD: Mutex<ClipboardContext> = ClipboardContext::new()
        .expect("init clipboard context")
        .into();
}
