use ratatui::widgets::TableState;

pub struct TimeItem {
    pub start_time: String,
    pub ticket: String,
    pub text: String,
    pub duration: String,
}

impl TimeItem {
    pub const fn ref_array(&self) -> [&String; 4] {
        [&self.start_time, &self.ticket, &self.text, &self.duration]
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
                    start_time: "0915".into(),
                    ticket: "(W)SCRUM-17".into(),
                    text: "daily".into(),
                    duration: "15m".into(),
                },
                TimeItem {
                    start_time: "0930".into(),
                    ticket: "(W)XAMPL-568".into(),
                    text: "tech analysis".into(),
                    duration: "2h".into(),
                },
            ],
        }
    }
}
