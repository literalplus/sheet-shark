use super::schema::*;
use diesel::prelude::*;
use type_safe_id::{StaticType, TypeSafeId};

#[derive(Debug, Clone)]
pub enum Command {
    StoreEntry(TimeEntry),
}

#[derive(Debug, Clone)]
pub enum Event {
    EntryStored(TimeEntryId),
}

#[derive(Insertable)]
#[diesel(table_name = timesheet)]
pub struct Timesheet<'a> {
    pub day: &'a str,
    pub status: &'a str,
}

#[derive(Insertable, AsChangeset, Debug, Clone, Default)]
#[diesel(table_name = time_entry)]
pub struct TimeEntry {
    pub id: String,
    pub timesheet_day: String,
    pub duration_mins: i32,
    pub description: String,
}

impl StaticType for TimeEntry {
    const TYPE: &'static str = "tent";
}

pub type TimeEntryId = TypeSafeId<TimeEntry>;
