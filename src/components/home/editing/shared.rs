use crossterm::event::{KeyCode, KeyEvent};
use enum_dispatch::enum_dispatch;
use ratatui::widgets::{Row, Table};

use crate::components::home::{
    action::HomeAction,
    state::{HomeState, TimeItem},
};

use std::ops::Deref;

#[enum_dispatch]
pub trait EditModeBehavior {
    fn handle_key_event(&mut self, state: &mut HomeState, key: KeyEvent) -> HomeAction;
    fn style_table<'a>(&self, table: Table<'a>) -> Table<'a>;
    fn style_selected_item<'a>(&self, item: &'a TimeItem) -> Row<'a> {
        item.as_row()
    }
}

#[derive(Default)]
pub struct BufEditBehavior {
    buf: String,
}

impl<'a> From<&'a BufEditBehavior> for &'a str {
    fn from(value: &'a BufEditBehavior) -> Self {
        &value.buf
    }
}

impl AsRef<str> for BufEditBehavior {
    fn as_ref(&self) -> &str {
        self.deref()
    }
}

impl Deref for BufEditBehavior {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.buf
    }
}

impl BufEditBehavior {
    pub fn new<T: ToString>(buf: T) -> Self {
        Self {
            buf: buf.to_string(),
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> HomeAction {
        match key.code {
            KeyCode::Esc => return HomeAction::ExitEdit,
            KeyCode::Char(chr) => {
                self.buf.push(chr);
            }
            KeyCode::Backspace => {
                if !self.buf.is_empty() {
                    self.buf.remove(self.buf.len() - 1);
                }
            }
            _ => {}
        }
        HomeAction::None
    }
}
