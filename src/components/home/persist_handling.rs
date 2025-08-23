use std::time::Duration;

use crate::{
    components::home::{
        Home,
        action::HomeAction,
        state::{HomeState, TimeItem},
    },
    persist::{self, Event, TimeEntry, Timesheet},
};
use chrono::NaiveTime;
use ratatui::widgets::TableState;
use tracing::error;

pub fn handle(home: &mut Home, event: Event) -> HomeAction {
    match event {
        persist::Event::EntryStored { id, version } => {
            for entry in home.state.items.iter_mut() {
                if entry.id == id {
                    entry.version.notify_saved(version);
                    return HomeAction::SetStatusLine(format!("Stored: {id} v{version}"));
                }
            }
            HomeAction::None
        }
        persist::Event::TimesheetLoaded {
            timesheet,
            entries,
            day: _,
        } if !home.suspended => {
            // prevent creating timesheets when browsing calendar
            let day = timesheet.day.to_string();
            home.state = into_state(timesheet, entries);
            if home.state.items.is_empty() {
                // Without an initial item it's not possible to add one
                home.state
                    .items
                    .push(TimeItem::new(Duration::ZERO, NaiveTime::MIN));
            }
            HomeAction::SetStatusLine(format!("Loaded: {day}"))
        }
        _ => HomeAction::None,
    }
}

fn into_state(timesheet: Timesheet, entries: Vec<TimeEntry>) -> HomeState {
    let items = entries
        .into_iter()
        .filter_map(|entry| match (&entry).try_into() {
            Ok(ok) => Some(ok),
            Err(err) => {
                error!("Failed to load corrupted time entry: {entry:?} due to {err:?}");
                None
            }
        })
        .collect();
    HomeState {
        table: TableState::default(),
        timesheet: Some(timesheet),
        items,
        items_to_delete: vec![],
    }
}
