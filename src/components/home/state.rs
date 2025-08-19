use std::time::Duration;
use std::{ops::Range, str::FromStr};

use chrono::NaiveTime;
use color_eyre::eyre::Context;
use humantime::format_duration;
use ratatui::{
    text::Text,
    widgets::{Row, TableState},
};

use crate::persist::{self, TimeEntryId, Timesheet};
use crate::shared::DataVersion;

pub struct TimeItem {
    pub id: TimeEntryId,
    pub start_time: NaiveTime,
    pub project: String,
    pub ticket: String,
    pub description: String,
    pub duration: Duration,
    pub version: DataVersion,
}

impl TimeItem {
    pub fn new(duration: Duration, start_time: NaiveTime) -> Self {
        Self {
            id: persist::TimeEntryId::new(),
            start_time,
            duration,
            ticket: Default::default(),
            project: Default::default(),
            description: Default::default(),
            version: DataVersion::fresh(),
        }
    }

    pub fn to_persist(&self, day: &str) -> persist::TimeEntry {
        let duration_mins = self.duration.as_secs().div_ceil(60) as i32;
        persist::TimeEntry {
            id: self.id.to_string(),
            timesheet_day: day.to_string(),
            duration_mins,
            ticket_key: Some(self.ticket.to_string()).filter(|it| !it.is_empty()),
            project_key: Some(self.project.to_string()).filter(|it| !it.is_empty()),
            description: self.description.to_string(),
            start_time: self.start_time.format("%H:%M").to_string(),
        }
    }
}

impl TryFrom<&persist::TimeEntry> for TimeItem {
    type Error = color_eyre::Report;

    fn try_from(value: &persist::TimeEntry) -> Result<Self, Self::Error> {
        Ok(Self {
            id: TimeEntryId::from_str(&value.id).wrap_err("TimeEntryId")?,
            start_time: NaiveTime::from_str(&value.start_time).wrap_err("start_time")?,
            ticket: value.ticket_key.clone().unwrap_or_default(),
            project: value.project_key.clone().unwrap_or_default(),
            description: value.description.to_string(),
            duration: Duration::from_secs(value.duration_mins as u64 * 60),
            version: DataVersion::loaded(),
        })
    }
}

pub const TIME_ITEM_WIDTH: usize = 5;

impl TimeItem {
    pub fn as_row<'a>(&'a self) -> Row<'a> {
        Row::new(self.as_cells())
    }

    /// Needed because ratatui's Row doesn't expose its contents
    pub fn as_cells<'a>(&'a self) -> [Text<'a>; TIME_ITEM_WIDTH] {
        let formatted_duration = if self.duration.is_zero() {
            "".to_string()
        } else {
            format!("{}", format_duration(self.duration))
        };
        [
            Text::from(self.start_time.format("%H:%M").to_string()),
            Text::from(&self.project as &str),
            Text::from(&self.ticket as &str),
            Text::from(&self.description as &str),
            Text::from(formatted_duration),
        ]
    }

    pub fn next_start_time(&self) -> NaiveTime {
        self.start_time + self.duration
    }
}

pub struct HomeState {
    pub table: TableState,
    pub timesheet: Option<Timesheet>,
    pub items: Vec<TimeItem>,
    pub items_to_delete: Vec<TimeItem>,
}

impl Default for HomeState {
    fn default() -> Self {
        Self {
            table: TableState::default(),
            timesheet: None,
            items: vec![TimeItem {
                id: TimeEntryId::from_uuid(
                    "791d98c7-3be0-455f-8bfb-94769131243c".try_into().unwrap(),
                ),
                start_time: NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
                ticket: "".into(),
                project: "".into(),
                description: "Loading...".into(),
                duration: Default::default(),
                version: DataVersion::fresh(),
            }],
            items_to_delete: vec![],
        }
    }
}

impl HomeState {
    pub fn expect_selected_item(&self) -> &TimeItem {
        self.maybe_selected_item().expect("an item to be selected")
    }

    pub fn maybe_selected_item(&self) -> Option<&TimeItem> {
        self.table.selected().and_then(|idx| self.items.get(idx))
    }

    pub fn expect_selected_item_mut(&mut self) -> &mut TimeItem {
        let idx = self.table.selected().unwrap_or(0);
        let item = self.items.get_mut(idx).expect("the selected item to exist");
        item.version.touch();
        item
    }

    pub fn ensure_column_selected(&mut self) {
        if self.table.selected_column().is_none() {
            self.table.select_first_column();
        }
    }

    pub fn ensure_row_selected(&mut self) {
        if self.table.selected().is_none() {
            self.table.select_first();
        }
    }

    pub fn is_last_column_selected(&self) -> bool {
        self.table.selected_column() == Some(TIME_ITEM_WIDTH - 1)
    }

    pub fn is_last_row_selected(&self) -> bool {
        self.table.selected() == Some(self.items.len() - 1)
    }

    pub fn drain_items(&mut self, range: Range<usize>) {
        self.items_to_delete.extend(self.items.drain(range));
    }
}
