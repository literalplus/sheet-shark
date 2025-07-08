use std::time::Duration;

use chrono::NaiveTime;
use humantime::{format_duration, parse_duration};
use ratatui::{
    text::Text,
    widgets::{Row, TableState},
};

pub struct TimeItem {
    pub start_time: NaiveTime,
    pub ticket: String,
    pub description: String,
    pub duration: Duration,
}

impl TimeItem {
    pub fn as_row<'a>(&'a self) -> Row<'a> {
        Row::new(self.as_cells())
    }

    /// Needed because ratatui's Row doesn't expose its contents
    pub fn as_cells<'a>(&'a self) -> [Text<'a>; 4] {
        [
            Text::from(self.start_time.format("%H:%M").to_string()),
            Text::from(&self.ticket as &str),
            Text::from(&self.description as &str),
            Text::from(format!("{}", format_duration(self.duration))),
        ]
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
                    start_time: NaiveTime::from_hms_opt(9, 15, 0).unwrap(),
                    ticket: "(W)SCRUM-17".into(),
                    description: "daily".into(),
                    duration: parse_duration("15m").unwrap(),
                },
                TimeItem {
                    start_time: NaiveTime::from_hms_opt(9, 30, 0).unwrap(),
                    ticket: "(W)XAMPL-568".into(),
                    description: "tech analysis".into(),
                    duration: parse_duration("90m").unwrap(),
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
}
