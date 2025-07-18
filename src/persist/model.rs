use super::schema::*;
use diesel::prelude::*;

#[derive(Debug, Clone)]
pub enum Command {
    Demo,
}

#[derive(Debug, Clone)]
pub enum Event {
    Demo,
}

#[derive(Insertable)]
#[diesel(table_name = timesheet)]
pub struct Timesheet<'a> {
    pub day: &'a str,
    pub status: &'a str,
}
