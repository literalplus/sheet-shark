use std::time::Duration;

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use super::Home;
use crate::{
    action::Action,
    components::home::{
        EDITING_KEYS, SELECTING_KEYS,
        action::HomeAction,
        editing::{EditMode, EditModeBehavior},
        movement::handle_movement,
        state::{HomeState, TimeItem},
    },
    persist,
};

pub fn handle(home: &mut Home, key: KeyEvent) -> Result<Option<Action>> {
    if key.kind != KeyEventKind::Press {
        return Ok(None);
    }

    let action = match &mut home.edit_mode {
        Some(mode) => mode.handle_key_event(&mut home.state, key),
        None => handle_outside_edit(home, key),
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

fn handle_outside_edit(home: &mut Home, key: KeyEvent) -> HomeAction {
    let state = &mut home.state;
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
        KeyCode::Char('+') => {
            home.persist_tx
                .as_ref()
                .unwrap()
                .send(persist::Command::Demo)
                .unwrap();
            return HomeAction::SetStatusLine("Saving a demo...".into());
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

fn perform_action(home: &mut Home, action: HomeAction) -> Result<Option<Action>> {
    match action {
        HomeAction::EnterEditSpecific(Some(mode)) => {
            home.state.table.select_column(Some(mode.get_column_num()));
            home.edit_mode = Some(mode);
            return Ok(Some(Action::SetRelevantKeys(EDITING_KEYS.to_vec())));
        }
        HomeAction::EnterEditSpecific(None) => {
            return Ok(Some(Action::SetStatusLine("⛔⛔⛔".into())));
        }
        HomeAction::ExitEdit => {
            home.edit_mode = None;
            return Ok(Some(Action::SetRelevantKeys(SELECTING_KEYS.to_vec())));
        }
        HomeAction::SetStatusLine(msg) => return Ok(Some(Action::SetStatusLine(msg))),
        HomeAction::SplitItemDown(idx) => {
            let original_item = home
                .state
                .items
                .get_mut(idx)
                .expect("item to split to exist");
            let duration_mins = original_item.duration.as_secs().div_ceil(60);
            if duration_mins <= 1 {
                return Ok(Some(Action::SetStatusLine("cannot split further!".into())));
            }
            let (first_duration, second_duration) = split_in_half(duration_mins);
            original_item.duration = Duration::from_secs(first_duration * 60);
            let new_item = TimeItem {
                duration: Duration::from_secs(second_duration * 60),
                start_time: original_item.start_time,
                ..Default::default()
            };
            original_item.start_time += new_item.duration;
            home.state.items.insert(idx, new_item);
        }
        HomeAction::None => {}
    }
    Ok(None)
}

fn split_in_half(n: u64) -> (u64, u64) {
    let half_down = n / 2;
    let half_up = n - half_down;
    (half_up, half_down)
}
