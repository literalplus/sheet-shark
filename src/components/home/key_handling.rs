use color_eyre::Result;
use crossterm::event::{KeyEvent, KeyEventKind};

use super::Home;
use crate::{
    action::Action,
    components::home::{
        action::HomeAction,
        editing::{EditMode, EditModeBehavior},
    },
};

pub fn handle(home: &mut Home, key: KeyEvent) -> Result<Option<Action>> {
    if key.kind != KeyEventKind::Press {
        return Ok(None);
    }

    let action = Some(home.edit_mode.handle_key_event(&mut home.state, key))
        .filter(|it| *it != HomeAction::None)
        .unwrap_or_else(handle_global_key_event);

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

fn handle_global_key_event() -> HomeAction {
    HomeAction::None
}

fn perform_action(home: &mut Home, action: HomeAction) -> Result<Option<Action>> {
    match action {
        HomeAction::EnterEditSpecific(Some(mode)) => {
            home.state.table.select_column(Some(mode.get_column_num()));
            home.edit_mode = mode;
            return Ok(Some(Action::SetStatusLine("".into())));
        }
        HomeAction::EnterEditSpecific(None) => {
            return Ok(Some(Action::SetStatusLine("⛔⛔⛔".into())));
        }
        HomeAction::ExitEdit => home.edit_mode = EditMode::default(),
        HomeAction::SetStatusLine(msg) => return Ok(Some(Action::SetStatusLine(msg))),
        HomeAction::None => {}
    }
    Ok(None)
}
