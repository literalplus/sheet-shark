use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use enum_dispatch::enum_dispatch;

use crate::action::Action;

use super::Home;

#[enum_dispatch(EditMode)]
pub trait EditModeBehaviour {
    fn key_event_handler(&self) -> fn(home: &mut Home, key: KeyEvent) -> Result<Option<Action>>;
}

#[derive(Default)]
pub struct Select {}

impl Select {
    fn handle_key_event(home: &mut Home, key: KeyEvent) -> Result<Option<Action>> {
        match key.code {
            KeyCode::Up | KeyCode::Down => {
                if key.code == KeyCode::Up {
                    home.table_state.select_previous()
                } else {
                    home.table_state.select_next()
                }
                if home.table_state.selected_column() == None {
                    home.table_state.select_first_column();
                }
            }
            KeyCode::Left => home.table_state.select_previous_column(),
            KeyCode::Right => home.table_state.select_next_column(),
            KeyCode::Esc => home.table_state.select(None),
            KeyCode::Char(' ') => match home.table_state.selected_column() {
                Some(0) => {
                    home.edit_mode = EditMode::from(Time::default());
                    return Ok(Some(Action::SetStatusLine("Here u go ⌚⌚".into())));
                }
                _ => {
                    return Ok(Some(Action::SetStatusLine(
                        "You can't edit this 🔪🔪".into(),
                    )));
                }
            },
            _ => {}
        }
        Ok(None)
    }
}

impl EditModeBehaviour for Select {
    fn key_event_handler(&self) -> fn(home: &mut Home, key: KeyEvent) -> Result<Option<Action>> {
        Self::handle_key_event
    }
}

#[derive(Default)]
pub struct Time {}

impl Time {
    fn handle_key_event(home: &mut Home, key: KeyEvent) -> Result<Option<Action>> {
        match key.code {
            KeyCode::Esc => {
                home.edit_mode = EditMode::from(Select::default());
                return Ok(Some(Action::SetStatusLine("ok then not! 👍👍".into())));
            }
            KeyCode::Enter => {
                home.edit_mode = EditMode::from(Select::default());
                if let Some(idx) = home.table_state.selected() {
                    home.items[idx].start_time = "1456".into();
                }
                return Ok(Some(Action::SetStatusLine("".into())));
            }
            KeyCode::Char(_) => {
                todo!()
            }
            _ => {}
        }
        Ok(None)
    }
}

impl EditModeBehaviour for Time {
    fn key_event_handler(&self) -> fn(home: &mut Home, key: KeyEvent) -> Result<Option<Action>> {
        Self::handle_key_event
    }
}

#[enum_dispatch]
pub enum EditMode {
    Select,
    Time,
}
