use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use super::Home;
use crate::{
    action::Action,
    components::home::{
        OUTSIDE_KEYS,
        action::HomeAction,
        editing::{EditMode, EditModeBehavior},
        movement::handle_movement,
        state::HomeState,
    },
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
            if state.table.selected().is_some() {
                state.table.select(None);
                let _ = home
                    .action_tx
                    .as_mut()
                    .unwrap()
                    .send(Action::SetRelevantKeys(OUTSIDE_KEYS.to_vec()));
            } else {
                return HomeAction::ExitToCalendar;
            }
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
