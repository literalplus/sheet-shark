use crossterm::event::KeyEvent;
use enum_dispatch::enum_dispatch;
use ratatui::widgets::{Row, Table};

use crate::components::home::{
    action::HomeAction,
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

mod bufedit {
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
}
pub use bufedit::*;

mod movement {
    use crossterm::event::{KeyCode, KeyEvent};

    use crate::components::home::state::HomeState;

    pub fn is_movement(key: KeyEvent) -> bool {
        matches!(
            key.code,
            KeyCode::Left | KeyCode::Right | KeyCode::BackTab | KeyCode::Tab | KeyCode::Down | KeyCode::Up
        )
    }

    /// Returns whether a movement was made - See also [is_movement].
    pub fn handle_movement(state: &mut HomeState, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Up => {
                state.table.select_previous();
                state.ensure_column_selected();
                true
            }
            KeyCode::Down => {
                state.table.select_next();
                state.ensure_column_selected();
                true
            }
            KeyCode::Left | KeyCode::BackTab => select_previous_column(state),
            KeyCode::Right | KeyCode::Tab => select_next_column(state),
            _ => false,
        }
    }

    fn select_previous_column(state: &mut HomeState) -> bool {
        state.ensure_row_selected();
        if state.table.selected_column() == Some(0) {
            if state.table.selected() != Some(0) {
                state.table.select_last_column();
                state.table.select_previous();
            }
        } else {
            state.table.select_previous_column();
        }
        true
    }

    fn select_next_column(state: &mut HomeState) -> bool {
        state.ensure_row_selected();
        if state.is_last_column_selected() && !state.is_last_row_selected() {
            // Do not proceed from last entry, s.t. duration can be entered immediately to create a new row
            if !state.is_last_row_selected() {
                state.table.select_first_column();
                state.table.select_next();
            }
        } else {
            state.table.select_next_column();
        }
        true
    }
}
pub use movement::*;
