use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use super::Home;
use crate::{
    action::Action,
    components::home::{
        action::HomeAction,
        editing::{EditMode, EditModeBehavior},
        movement::handle_movement,
        state::HomeState,
    },
};

pub fn handle(home: &mut Home, key: KeyEvent) -> Result<Option<Action>> {
    if key.kind != KeyEventKind::Press {
        return Ok(None);
    }

    let action = match &mut home.edit_mode {
        Some(mode) => mode.handle_key_event(&mut home.state, key),
        None => handle_outside_edit(&mut home.state, key),
    };

    match perform_action(home, action) {
        result @ Ok(Some(Action::SetStatusLine(_))) => {
            home.need_status_line_reset = true;
            result
        }
        Ok(None) if home.need_status_line_reset => {
            home.need_status_line_reset = false;
            Ok(Some(Action::SetStatusLine("".into())))
        }
        result => result,
    }
}

fn handle_outside_edit(state: &mut HomeState, key: KeyEvent) -> HomeAction {
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

fn perform_action(home: &mut Home, action: HomeAction) -> Result<Option<Action>> {
    match action {
        HomeAction::EnterEditSpecific(Some(mode)) => {
            home.state.table.select_column(Some(mode.get_column_num()));
            home.edit_mode = Some(mode);
            return Ok(Some(Action::SetStatusLine("".into())));
        }
        HomeAction::EnterEditSpecific(None) => {
            return Ok(Some(Action::SetStatusLine("⛔⛔⛔".into())));
        }
        HomeAction::ExitEdit => home.edit_mode = None,
        HomeAction::SetStatusLine(msg) => return Ok(Some(Action::SetStatusLine(msg))),
        HomeAction::None => {}
    }
    Ok(None)
}
