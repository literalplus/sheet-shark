use crossterm::event::KeyEvent;
use enum_dispatch::enum_dispatch;
use ratatui::prelude::Constraint;
use ratatui::widgets::{Row, Table, TableState};

use crate::components::home::{
    action::HomeAction,
    editing::project::Project,
    state::{HomeState, TimeItem},
};
use crate::persist;
use crate::widgets::table_popup::TablePopup;

mod shared;
pub(super) use shared::EditModeBehavior;

mod description;
mod duration;
mod project;
mod ticket;
mod time;

use self::{description::Description, duration::Duration, ticket::Ticket, time::Time};

#[derive(PartialEq, Eq)]
#[enum_dispatch(EditModeBehavior)]
pub enum EditMode {
    Time,
    Project,
    Ticket,
    Description,
    Duration,
}

impl EditMode {
    pub fn of_time() -> Self {
        Time::default().into()
    }

    pub fn of_project(state: &HomeState) -> Self {
        Project::new(state).into()
    }

    pub fn of_ticket(state: &HomeState) -> Self {
        Ticket::new(state).into()
    }

    pub fn of_description(state: &HomeState) -> Self {
        Description::new(state).into()
    }

    pub fn of_duration() -> Self {
        Duration::default().into()
    }

    pub fn from_column_num(idx: usize, state: &HomeState) -> Option<Self> {
        Some(match idx {
            0 => Self::of_time(),
            1 => Self::of_project(state),
            2 => Self::of_ticket(state),
            3 => Self::of_description(state),
            4 | usize::MAX => Self::of_duration(), // MAX is set by select_last_column()
            _ => return None,
        })
    }

    pub fn get_column_num(&self) -> usize {
        match self {
            EditMode::Time(_) => 0,
            EditMode::Project(_) => 1,
            EditMode::Ticket(_) => 2,
            EditMode::Description(_) => 3,
            EditMode::Duration(_) => 4,
        }
    }
}
