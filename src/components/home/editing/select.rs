use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    style::{Modifier, Style},
    widgets::Table,
};

use super::EditModeBehavior;
use crate::components::home::{action::HomeAction, editing::EditMode, state::HomeState};

#[derive(Default)]
pub struct Select {}

impl EditModeBehavior for Select {
    fn handle_key_event(&mut self, state: &mut HomeState, key: KeyEvent) -> HomeAction {
        match key.code {
            KeyCode::Up | KeyCode::Down => {
                if key.code == KeyCode::Up {
                    state.table.select_previous()
                } else {
                    state.table.select_next()
                }
                if state.table.selected_column().is_none() {
                    state.table.select_first_column();
                }
            }
            KeyCode::Left => state.table.select_previous_column(),
            KeyCode::Right => state.table.select_next_column(),
            KeyCode::Esc => state.table.select(None),
            KeyCode::Char(' ') => {
                let mode_opt = state
                    .table
                    .selected_column()
                    .and_then(|idx| EditMode::from_column_num(idx, state));
                return HomeAction::EnterEditSpecific(mode_opt);
            }
            _ => {}
        }
        HomeAction::None
    }

    fn style_table<'a>(&self, table: Table<'a>) -> Table<'a> {
        table.cell_highlight_style(Style::from(Modifier::BOLD))
    }
}
