use std::time::Duration;

use chrono::NaiveTime;
use humantime::{format_duration, parse_duration};
use ratatui::{
    text::Text,
    widgets::{Row, TableState},
};

#[derive(Default)]
pub struct TimeItem {
    pub start_time: NaiveTime,
    pub ticket: String,
    pub description: String,
    pub duration: Duration,
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
                    start_time: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
                    ticket: "(W)SCRUM-17".into(),
                    description: "daily".into(),
                    duration: parse_duration("1h").unwrap(),
                },
                TimeItem {
                    start_time: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
                    ticket: "(W)XAMPL-569".into(),
                    description: "implementation".into(),
                    duration: parse_duration("1h").unwrap(),
                },
                TimeItem {
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
        let idx = self.table.selected().unwrap_or(0);
        self.items.get(idx).expect("the selected item to exist")
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
