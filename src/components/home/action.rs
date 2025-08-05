use color_eyre::eyre::{ErrReport, Result};
use std::time::Duration;

use crate::{
    action::Action,
    components::home::{EDITING_KEYS, Home, SELECTING_KEYS, editing::EditMode, state::TimeItem},
};

#[derive(PartialEq, Eq)]
pub enum HomeAction {
    None,
    EnterEditSpecific(Option<EditMode>),
    ExitEdit,
    SetStatusLine(String),
    SplitItemDown(usize),
}

impl From<ErrReport> for HomeAction {
    fn from(value: ErrReport) -> Self {
        Self::SetStatusLine(format!("{value}"))
    }
}

pub fn perform(home: &mut Home, action: HomeAction) -> Result<Option<Action>> {
    match do_perform(home, action) {
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

fn do_perform(home: &mut Home, action: HomeAction) -> Result<Option<Action>> {
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
            let new_item = TimeItem::new(
                Duration::from_secs(second_duration * 60),
                original_item.start_time,
            );
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
