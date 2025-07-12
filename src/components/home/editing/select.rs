use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    style::{Modifier, Style, Stylize, palette::tailwind},
    widgets::Table,
};

use super::EditModeBehavior;
use crate::components::home::{
    action::HomeAction,
    editing::{shared::handle_movement, EditMode},
    state::HomeState,
};

#[derive(Default)]
pub struct Select {}

fn handle_jump_key(state: &mut HomeState, key: KeyEvent) -> Option<EditMode> {
    let edit_creator: Box<dyn for<'a> Fn(&'a HomeState) -> EditMode> = match key.code {
        KeyCode::Char('#') => Box::new(EditMode::of_time),
        KeyCode::Char('t') => Box::new(EditMode::of_ticket),
        KeyCode::Char('x') => Box::new(EditMode::of_description),
        KeyCode::Char('d') => Box::new(|_| EditMode::of_duration()),
        _ => return None,
    };
    state.ensure_row_selected();
    Some(edit_creator(state))
}

impl EditModeBehavior for Select {
    fn handle_key_event(&mut self, state: &mut HomeState, key: KeyEvent) -> HomeAction {
        if let Some(next_mode) = handle_jump_key(state, key) {
            return HomeAction::EnterEditSpecific(Some(next_mode));
        }
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
