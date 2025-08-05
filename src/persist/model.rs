use super::schema::*;
use diesel::prelude::*;
use type_safe_id::{StaticType, TypeSafeId};

#[derive(Debug, Clone)]
pub enum Command {
    StoreEntry(TimeEntry),
    LoadTimesheet { day: String },
}

#[derive(Debug, Clone)]
pub enum Event {
    Failure(String),
    EntryStored(TimeEntryId),
    TimesheetLoaded {
        timesheet: Timesheet,
        entries: Vec<TimeEntry>,
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
    pub duration_mins: i32,
    pub description: String,
    pub start_time: String,
}

#[derive(Default, Clone)]
pub struct TimeEntryMarker;
pub type TimeEntryId = TypeSafeId<TimeEntryMarker>;

impl StaticType for TimeEntryMarker {
    const TYPE: &'static str = "tent";
}
