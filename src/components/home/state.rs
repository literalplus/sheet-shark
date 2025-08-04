use std::time::Duration;

use chrono::NaiveTime;
use humantime::{format_duration, parse_duration};
use ratatui::{
    text::Text,
    widgets::{Row, TableState},
};

use crate::persist::{self, TimeEntryId};

pub struct TimeItem {
    pub id: TimeEntryId,
    pub start_time: NaiveTime,
    pub ticket: String,
    pub description: String,
    pub duration: Duration,
}

impl TimeItem {
    pub fn new(duration: Duration, start_time: NaiveTime) -> Self {
        Self {
            id: persist::TimeEntryId::new(),
            start_time,
            duration,
            ticket: Default::default(),
            description: Default::default(),
        }
    }

    pub fn to_persist(&self, day: &str) -> persist::TimeEntry {
        let duration_mins = self.duration.as_secs().div_ceil(60) as i32;
        persist::TimeEntry {
            id: self.id.to_string(),
            timesheet_day: day.to_string(),
            duration_mins,
            description: self.description.to_string(),
            start_time: self.start_time.format("%H:%M").to_string(),
        }
    }
}

pub const TIME_ITEM_WIDTH: usize = 4;

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
    pub items: Vec<TimeItem>,
}

impl Default for HomeState {
    fn default() -> Self {
        Self {
            table: TableState::default(),
            items: vec![
                TimeItem {
                    id: TimeEntryId::from_uuid(
                        "fbbd6f4f-6c6d-40d2-bd9c-4f83b9af1d2b".try_into().unwrap(),
                    ),
                    start_time: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
                    ticket: "(W)SCRUM-17".into(),
                    description: "daily".into(),
                    duration: parse_duration("1h").unwrap(),
                },
                TimeItem {
                    id: TimeEntryId::from_uuid(
                        "830a5556-adc8-4a5f-8082-f3186bfdd10c".try_into().unwrap(),
                    ),
                    start_time: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
                    ticket: "(W)XAMPL-569".into(),
                    description: "implementation".into(),
                    duration: parse_duration("1h").unwrap(),
                },
                TimeItem {
                    id: TimeEntryId::from_uuid(
                        "17382f77-c973-41e4-a16f-5b4c7fa0193f".try_into().unwrap(),
                    ),
                    start_time: NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
                    ticket: "(W)REFI-12".into(),
                    description: "tech analysis".into(),
                    duration: Duration::default(),
                },
            ],
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
        self.items.get_mut(idx).expect("the selected item to exist")
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
}
