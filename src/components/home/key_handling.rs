use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use super::Home;
use crate::components::home::{
    action::HomeAction,
    editing::{EditMode, EditModeBehavior},
    movement::handle_movement,
};

pub fn handle(home: &mut Home, key: KeyEvent) -> HomeAction {
    if key.kind != KeyEventKind::Press {
        return HomeAction::None;
    }

    match &mut home.edit_mode {
        Some(mode) => mode.handle_key_event(&mut home.state, key),
        None => handle_outside_edit(home, key),
    }
}

fn handle_outside_edit(home: &mut Home, key: KeyEvent) -> HomeAction {
    let state = &mut home.state;
    if state.timesheet.is_none() {
        // Loading...
        return HomeAction::None;
    }

    let already_selecting = state.table.selected().is_some();
    if handle_movement(state, key) && !already_selecting {
        return HomeAction::EnterSelect;
    }
    match key.code {
        KeyCode::End => {
            state.table.select_last();
            state.table.select_last_column();
        }
        KeyCode::Esc => {
            return HomeAction::ExitToCalendar;
        }
        KeyCode::Char(' ') => {
            let mode_opt = state
                .table
                .selected_column()
                .and_then(|idx| EditMode::from_column_num(idx, state));
            return HomeAction::EnterEditSpecific(mode_opt);
        }
        KeyCode::Char('s') => {
            if let Some(idx) = state.table.selected() {
                return HomeAction::SplitItemDown(idx);
            }
        }
        KeyCode::Char('S') => {
            if let Some(idx) = state.table.selected() {
                return HomeAction::MergeItemDown(idx);
            }
        }
        KeyCode::Char('e') => {
            return HomeAction::Export;
        }
        KeyCode::Char('x') => {
            return HomeAction::ToggleBreak;
        }
        _ => {}
    }
    HomeAction::None
}
