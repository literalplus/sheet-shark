use strum::Display;

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


#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum Page {
    #[default]
    Home,
}
