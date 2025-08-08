use strum::Display;
use time::{Date, OffsetDateTime};

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    Error(String),
    SetStatusLine(String),
    SetRelevantKeys(Vec<RelevantKey>),
    SetActivePage(Page),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelevantKey {
    pub key: String,
    pub text: String,
}

impl RelevantKey {
    pub fn new(key: &'static str, text: &'static str) -> Self {
        Self {
            key: key.to_owned(),
            text: text.to_owned(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Page {
    Home { day: Date },
    Calendar { day: Date },
}

impl Default for Page {
    fn default() -> Self {
        let today = OffsetDateTime::now_local()
            .expect("find local offset for date")
            .date();
        Page::Home { day: today }
    }
}
