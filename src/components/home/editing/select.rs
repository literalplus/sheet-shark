use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    style::{Modifier, Style},
    widgets::Table,
};

use super::EditModeBehavior;
use crate::components::home::{action::HomeAction, editing::EditMode, state::HomeState};

#[derive(Default)]
pub struct Select {}

fn select_previous_column(state: &mut HomeState) {
    state.ensure_row_selected();
    if state.table.selected_column() == Some(0) {
        if state.table.selected() != Some(0) {
            state.table.select_last_column();
            state.table.select_previous();
        }
    } else {
        state.table.select_previous_column();
    }
}

fn select_next_column(state: &mut HomeState) {
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
}

impl EditModeBehavior for Select {
    fn handle_key_event(&mut self, state: &mut HomeState, key: KeyEvent) -> HomeAction {
        match key.code {
            KeyCode::Up | KeyCode::Down => {
                if key.code == KeyCode::Up {
                    state.table.select_previous()
                } else {
                    state.table.select_next()
                }
                state.ensure_column_selected();
            }
            KeyCode::Left | KeyCode::BackTab => select_previous_column(state),
            KeyCode::Right | KeyCode::Tab => select_next_column(state),
            KeyCode::End => {
                state.table.select_last();
                state.table.select_last_column();
            }
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
