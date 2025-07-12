use crossterm::event::KeyEvent;
use enum_dispatch::enum_dispatch;
use ratatui::widgets::{Row, Table};

use crate::components::home::{
    action::HomeAction,
    state::{HomeState, TimeItem},
};

mod shared;
pub(super) use shared::EditModeBehavior;

mod description;
mod duration;
mod select;
mod ticket;
mod time;

use self::{
    description::Description, duration::Duration, select::Select, ticket::Ticket, time::Time,
};

#[derive(PartialEq, Eq)]
#[enum_dispatch(EditModeBehavior)]
pub enum EditMode {
    Select,
    Time,
    Ticket,
    Description,
    Duration,
}

impl Default for EditMode {
    fn default() -> Self {
        EditMode::from(Select::default())
    }
}

impl EditMode {
    pub fn of_time(state: &HomeState) -> Self {
        Time::new(state).into()
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
            0 => Self::of_time(state),
            1 => Self::of_ticket(state),
            2 => Self::of_description(state),
            3 | usize::MAX => Self::of_duration(), // MAX is set by select_last_column()
            _ => return None,
        })
    }

    pub fn get_column_num(&self) -> usize {
        match self {
            EditMode::Select(_) => 0,

            EditMode::Time(_) => 0,
            EditMode::Ticket(_) => 1,
            EditMode::Description(_) => 2,
            EditMode::Duration(_) => 3,
        }
    }
}
