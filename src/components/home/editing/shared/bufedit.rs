use crossterm::event::{KeyCode, KeyEvent};

use crate::components::home::{
    action::HomeAction,
    editing::{
        EditMode,
        shared::{handle_movement, is_movement},
    },
    state::HomeState,
};

use std::ops::Deref;
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
                    self.buf.remove(self.buf.len() - 1);
                }
            }
            _ => {}
        }
        HomeAction::None
    }
}
