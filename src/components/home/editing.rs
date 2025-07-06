use crossterm::event::{KeyCode, KeyEvent};
use enum_dispatch::enum_dispatch;

use crate::components::home::{action::HomeAction, state::HomeState};

#[enum_dispatch]
pub trait EditModeBehaviour {
    fn handle_key_event(&self, state: &mut HomeState, key: KeyEvent) -> HomeAction;
}

#[derive(Default)]
pub struct Select {}

impl EditModeBehaviour for Select {
    fn handle_key_event(&self, state: &mut HomeState, key: KeyEvent) -> HomeAction {
        match key.code {
            KeyCode::Up | KeyCode::Down => {
                if key.code == KeyCode::Up {
                    state.table.select_previous()
                } else {
                    state.table.select_next()
                }
                if state.table.selected_column() == None {
                    state.table.select_first_column();
                }
            }
            KeyCode::Left => state.table.select_previous_column(),
            KeyCode::Right => state.table.select_next_column(),
            KeyCode::Esc => state.table.select(None),
            KeyCode::Char(' ') => return HomeAction::EnterEdit,
            _ => {}
        }
        HomeAction::None
    }
}

#[derive(Default)]
pub struct Time {}

impl EditModeBehaviour for Time {
    fn handle_key_event(&self, state: &mut HomeState, key: KeyEvent) -> HomeAction {
        match key.code {
            KeyCode::Esc => return HomeAction::ExitEdit,
            KeyCode::Enter => {
                if let Some(idx) = state.table.selected() {
                    state.items[idx].start_time = "1456".into();
                }
                return HomeAction::ExitEdit;
            }
            KeyCode::Char(_) => {
                todo!()
            }
            _ => {}
        }
        HomeAction::None
    }
}

#[enum_dispatch(EditModeBehaviour)]
pub enum EditMode {
    Select,
    Time,
}

impl Default for EditMode {
    fn default() -> Self {
        EditMode::from(Select::default())
    }
}

impl EditMode {
    pub fn from_column_num(idx: usize) -> Option<Self> {
        Some(match idx {
            0 => EditMode::from(Time::default()),
            _ => return None,
        })
    }
}
