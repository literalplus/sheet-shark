use color_eyre::eyre::{ErrReport, Result};
use std::time::Duration;

use crate::{
    action::{Action, Page},
    components::home::{EDITING_KEYS, Home, SELECTING_KEYS, editing::EditMode, state::TimeItem},
    persist,
};

#[derive(PartialEq, Eq)]
pub enum HomeAction {
    None,

    EnterEditSpecific(Option<EditMode>),
    EnterSelect,
    ExitToCalendar,
    ExitEdit,

    SetStatusLine(String),
    SplitItemDown(usize),
    MergeItemDown(usize),
}

impl From<ErrReport> for HomeAction {
    fn from(value: ErrReport) -> Self {
        Self::SetStatusLine(format!("{value}"))
    }
}

pub fn perform(home: &mut Home, action: HomeAction) -> Result<Option<Action>> {
    let result = match do_perform(home, action) {
        result @ Ok(Some(Action::SetStatusLine(_))) => {
            home.need_status_line_reset = true;
            result
        }
        Ok(None) if home.need_status_line_reset => {
            home.need_status_line_reset = false;
            Ok(Some(Action::SetStatusLine("".into())))
        }
        result => result,
    };
    save_any_dirty_state(home);
    result
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
        HomeAction::EnterSelect => {
            return Ok(Some(Action::SetRelevantKeys(SELECTING_KEYS.to_vec())));
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
            original_item.version.touch();
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
        HomeAction::MergeItemDown(idx) => {
            let items = &mut home.state.items;
            let obsolete_item = items.drain((idx + 1)..(idx + 2)).next();
            let obsolete_item = if let Some(it) = obsolete_item {
                it
            } else {
                return Ok(Some(Action::SetStatusLine("no item to merge with".into())));
            };
            let remaining_item = items.get_mut(idx).expect("item to split to exist");
            remaining_item.version.touch();
            remaining_item.duration += obsolete_item.duration;
            remaining_item.description += &format!(" / {}", obsolete_item.description);
            home.state.items_to_delete.push(obsolete_item);
        }
        HomeAction::ExitToCalendar => {
            return Ok(Some(Action::SetActivePage(Page::Calendar {
                day: home.day,
            })));
        }
        HomeAction::None => {}
    }
    Ok(None)
}

fn save_any_dirty_state(home: &mut Home) {
    let day = home.state.timesheet.clone().map(|it| it.day);
    if day.is_none() {
        return;
    }
    let day = day.unwrap();

    let mut commands_to_send = vec![];
    for item in home.state.items.iter_mut() {
        if item.version.should_save() {
            commands_to_send.push(persist::Command::StoreEntry {
                entry: item.to_persist(&day),
                version: item.version.local,
            });
            item.version.mark_sent();
        }
    }
    for to_delete in home.state.items_to_delete.drain(..) {
        commands_to_send.push(persist::Command::DeleteEntry(to_delete.id));
    }

    for command in commands_to_send {
        home.send_persist(command);
    }
}

fn split_in_half(n: u64) -> (u64, u64) {
    let half_down = n / 2;
    let half_up = n - half_down;
    (half_up, half_down)
}
