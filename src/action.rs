use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    Error(String),
    Help,
    SetStatusLine(String),
    SetRelevantKeys(Vec<RelevantKey>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
