use std::ops::Deref;

use crossterm::event::{KeyCode, KeyEvent};
use enum_dispatch::enum_dispatch;
use ratatui::widgets::{Row, Table};

use crate::components::home::{
    action::HomeAction,
    editing::EditMode,
    movement::{handle_movement, is_movement},
    state::{HomeState, TimeItem},
};

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

impl AsRef<String> for BufEditBehavior {
    fn as_ref(&self) -> &String {
        &self.buf
    }
}

impl Deref for BufEditBehavior {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.buf
    }
}

impl PartialEq<String> for BufEditBehavior {
    fn eq(&self, other: &String) -> bool {
        self.buf.eq(other)
    }
}

impl BufEditBehavior {
    pub fn new<T: ToString>(buf: T) -> Self {
        Self {
            buf: buf.to_string(),
        }
    }

    pub fn push(&mut self, chr: char) {
        self.buf.push(chr);
    }

    /// Evaluates whether this [key] should trigger a save before being further handled with
    /// [handle_key_event]. If the save fails, handling should not continue.
    pub fn should_save(&self, key: KeyEvent) -> bool {
        key.code == KeyCode::Enter || is_movement(key)
    }

    /// Handles a key event, after having done a save if [should_save] returned true.
    pub fn handle_key_event(&mut self, state: &mut HomeState, key: KeyEvent) -> HomeAction {
        if handle_movement(state, key) {
            let new_selected_column = state.table.selected_column().unwrap_or(0);
            return HomeAction::EnterEditSpecific(EditMode::from_column_num(
                new_selected_column,
                state,
            ));
        }
        match key.code {
            KeyCode::Enter => return HomeAction::ExitEdit,
            KeyCode::Esc => return HomeAction::ExitEdit,
            KeyCode::Char('^') => {
                self.buf.clear();
            }
            KeyCode::Char(chr) => {
                self.push(chr);
            }
            KeyCode::Backspace => {
                if !self.buf.is_empty() {
                    let (last_char_idx, _) = self.buf.char_indices().last().expect(">0 chars");
                    self.buf.remove(last_char_idx);
                }
            }
            _ => {}
        }
        HomeAction::None
    }
}
