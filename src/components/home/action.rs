use color_eyre::eyre::{ErrReport, Result};
use std::{ops::Add, time::Duration};

use crate::{
    action::{Action, Page},
    components::home::{EDITING_KEYS, Home, SELECTING_KEYS, editing::EditMode, state::TimeItem},
    persist::{self, Command},
};

#[derive(PartialEq, Eq)]
pub enum HomeAction {
    None,
    Many(Vec<HomeAction>),

    EnterEditSpecific(Option<EditMode>),
    EnterSelect,
    ExitToCalendar,
    ExitEdit,

    SetStatusLine(String),
    SplitItemDown(usize),
    MergeItemDown(usize),
    SuggestTickets(String),
}

impl From<ErrReport> for HomeAction {
    fn from(value: ErrReport) -> Self {
        Self::SetStatusLine(format!("{value}"))
    }
}

impl Add for HomeAction {
    type Output = HomeAction;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            HomeAction::Many(mut actions) => {
                actions.push(rhs);
                HomeAction::Many(actions)
            }
            HomeAction::None => rhs,
            exactly_one => HomeAction::Many(vec![exactly_one, rhs]),
        }
    }
}

pub fn perform(home: &mut Home, action: HomeAction) -> Result<()> {
    if home.need_status_line_reset {
        home.need_status_line_reset = false;
        home.send_action(Action::SetStatusLine("".into()));
    }
    let actions = do_perform(home, action)?;
    for action in actions {
        if matches!(action, Action::SetStatusLine(_)) {
            home.need_status_line_reset = true;
        }
        home.send_action(action);
    }
    save_any_dirty_state(home);
    Ok(())
}

fn do_perform(home: &mut Home, action: HomeAction) -> Result<Vec<Action>> {
    let out_action = match action {
        HomeAction::Many(actions) => {
            let mut results = vec![];
            for action in actions {
                for result in do_perform(home, action)? {
                    results.push(result);
                }
            }
            return Ok(results);
        }
        HomeAction::EnterEditSpecific(Some(mode)) => {
            home.state.table.select_column(Some(mode.get_column_num()));
            home.edit_mode = Some(mode);
            Action::SetRelevantKeys(EDITING_KEYS.to_vec())
        }
        HomeAction::EnterEditSpecific(None) => Action::SetStatusLine("⛔⛔⛔".into()),
        HomeAction::EnterSelect => Action::SetRelevantKeys(SELECTING_KEYS.to_vec()),
        HomeAction::ExitEdit => {
            home.edit_mode = None;
            Action::SetRelevantKeys(SELECTING_KEYS.to_vec())
        }
        HomeAction::SetStatusLine(msg) => Action::SetStatusLine(msg),
        HomeAction::SplitItemDown(idx) => 'block: {
            let original_item = home
                .state
                .items
                .get_mut(idx)
                .expect("item to split to exist");
            original_item.version.touch();
            let duration_mins = original_item.duration.as_secs().div_ceil(60);
            if duration_mins <= 1 {
                break 'block Action::SetStatusLine("cannot split further!".into());
            }
            let (first_duration, second_duration) = split_in_half(duration_mins);
            original_item.duration = Duration::from_secs(first_duration * 60);
            let new_item = TimeItem::new(
                Duration::from_secs(second_duration * 60),
                original_item.start_time,
            );
            original_item.start_time += new_item.duration;
            home.state.items.insert(idx, new_item);
            return Ok(vec![]);
        }
        HomeAction::MergeItemDown(idx) => 'block: {
            let items = &mut home.state.items;
            let obsolete_item = items.drain((idx + 1)..(idx + 2)).next();
            let obsolete_item = if let Some(it) = obsolete_item {
                it
            } else {
                break 'block Action::SetStatusLine("no item to merge with".into());
            };
            let remaining_item = items.get_mut(idx).expect("item to split to exist");
            remaining_item.version.touch();
            remaining_item.duration += obsolete_item.duration;
            remaining_item.description += &format!(" / {}", obsolete_item.description);
            home.state.items_to_delete.push(obsolete_item);
            return Ok(vec![]);
        }
        HomeAction::ExitToCalendar => Action::SetActivePage(Page::Calendar { day: home.day }),
        HomeAction::SuggestTickets(query) => {
            if !query.is_empty() {
                home.send_persist(Command::SuggestTickets { query });
            }
            return Ok(vec![]);
        }
        HomeAction::None => return Ok(vec![]),
    };
    Ok(vec![out_action])
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
