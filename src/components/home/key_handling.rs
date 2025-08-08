use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use super::Home;
use crate::components::home::{
    action::HomeAction,
    editing::{EditMode, EditModeBehavior},
    movement::handle_movement,
    state::HomeState,
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
        _ => {}
    }
    HomeAction::None
}

fn handle_jump_key(state: &mut HomeState, key: KeyEvent) -> Option<EditMode> {
    let edit_creator: Box<dyn for<'a> Fn(&'a HomeState) -> EditMode> = match key.code {
        KeyCode::Char('#') => Box::new(|_| EditMode::of_time()),
        KeyCode::Char('t') => Box::new(EditMode::of_ticket),
        KeyCode::Char('e') => Box::new(EditMode::of_description),
        KeyCode::Char('d') => Box::new(|_| EditMode::of_duration()),
        _ => return None,
    };
    state.ensure_row_selected();
    Some(edit_creator(state))
}
