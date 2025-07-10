use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    style::{Modifier, Style, Stylize, palette::tailwind},
    widgets::Table,
};

use super::EditModeBehavior;
use crate::components::home::{
    action::HomeAction,
    editing::{EditMode, shared::handle_movement},
    state::HomeState,
};

#[derive(Default)]
pub struct Select {}

impl EditModeBehavior for Select {
    fn handle_key_event(&mut self, state: &mut HomeState, key: KeyEvent) -> HomeAction {
        handle_movement(state, key);
        match key.code {
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
        table.cell_highlight_style(
            Style::from(Modifier::BOLD)
                .not_reversed()
                .bg(tailwind::SLATE.c400),
        )
    }
}
