use crate::{persist::schema::time_entry::ticket_key, shared::DataVersionNumber};

use super::schema::*;
use chrono::NaiveTime;
use diesel::prelude::*;
use time::Date;
use type_safe_id::{StaticType, TypeSafeId};

#[derive(Debug, Clone)]
pub enum Command {
    StoreEntry {
        entry: TimeEntry,
        version: DataVersionNumber,
    },
    DeleteEntry(TimeEntryId),
    LoadTimesheet {
        day: Date,
    },
    LoadTimesheetsOfMonth {
        day: Date,
    },
    SuggestTickets {
        query: String,
    },
}

#[derive(Debug, Clone)]
pub enum Event {
    Failure(String),
    Deleted,
    EntryStored {
        id: TimeEntryId,
        version: DataVersionNumber,
    },
    TimesheetLoaded {
        day: Date,
        timesheet: Timesheet,
        entries: Vec<TimeEntry>,
    },
    TimesheetsOfMonthLoaded {
        day: Date,
        timesheets: Vec<Timesheet>,
    },
    TicketsSuggested {
        query: String,
        ticket_keys: Vec<String>,
    },
}

#[derive(Insertable, Queryable, Identifiable, Selectable, Debug, Clone)]
#[diesel(primary_key(day))]
#[diesel(table_name = timesheet)]
pub struct Timesheet {
    pub day: String,
    pub status: String,
}

#[derive(
    Queryable, Insertable, AsChangeset, Identifiable, Selectable, Debug, Clone, Associations,
)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(belongs_to(Timesheet, foreign_key = timesheet_day))]
#[diesel(table_name = time_entry)]
pub struct TimeEntry {
    pub id: String,
    pub timesheet_day: String,

    pub project_key: String,
    pub ticket_key: Option<String>,

    pub duration_mins: i32,
    pub description: String,
    pub start_time: String,
}

impl TimeEntry {
    pub fn is_empty_default(&self) -> bool {
        self.ticket_key.is_none()
            && self.duration_mins == 0
            && self.description.is_empty()
            && self.start_time == "00:00"
    }
}

#[derive(Default, Clone, PartialEq, Eq)]
pub struct TimeEntryMarker;
pub type TimeEntryId = TypeSafeId<TimeEntryMarker>;

impl StaticType for TimeEntryMarker {
    const TYPE: &'static str = "tent";
}
