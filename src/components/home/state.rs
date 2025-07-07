use chrono::NaiveTime;
use ratatui::{text::Text, widgets::{Row, TableState}};

pub struct TimeItem {
    pub start_time: NaiveTime,
    pub ticket: String,
    pub text: String,
    pub duration: String,
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
            Text::from(&self.text as &str),
            Text::from(&self.duration as &str),
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
                    text: "daily".into(),
                    duration: "15m".into(),
                },
                TimeItem {
                    start_time: NaiveTime::from_hms_opt(9, 30, 0).unwrap(),
                    ticket: "(W)XAMPL-568".into(),
                    text: "tech analysis".into(),
                    duration: "2h".into(),
                },
            ],
        }
    }
}

impl HomeState {
    pub fn expect_selected_item(&self) -> &TimeItem {
        let idx = self.table.selected().expect("a time item to be selected");
        self.items.get(idx).expect("the selected item to exist")
    }

    pub fn expect_selected_item_mut(&mut self) -> &mut TimeItem {
        let idx = self.table.selected().expect("a time item to be selected");
        self.items.get_mut(idx).expect("the selected item to exist")
    }
}