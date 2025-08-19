use std::str::FromStr;

use color_eyre::{Result, eyre::Context};
use diesel::{RunQueryDsl, SqliteConnection, prelude::*};

use time::{Date, format_description};

use crate::persist::{
    Command, Event, TimeEntry, TimeEntryId, Timesheet,
    schema::{time_entry, timesheet},
};

pub(super) async fn handle(conn: &mut SqliteConnection, cmd: Command) -> Result<Event> {
    match cmd {
        Command::StoreEntry { entry, version } => store_entry(conn, entry, version).await,
        Command::DeleteEntry(id) => delete_entry(conn, id).await,
        Command::LoadTimesheet { day } => load_timesheet(conn, day).await,
        Command::LoadTimesheetsOfMonth { day } => load_timesheets_of_month(conn, day).await,
    }
}

async fn store_entry(conn: &mut SqliteConnection, entry: TimeEntry, version: i32) -> Result<Event> {
    ensure_timesheet_exists(conn, &entry.timesheet_day).await?;

    diesel::insert_into(time_entry::table)
        .values(&entry)
        .on_conflict(time_entry::id)
        .do_update()
        .set(&entry)
        .execute(conn)
        .wrap_err("saving time entry")?;
    Ok(Event::EntryStored {
        id: TimeEntryId::from_str(&entry.id)?,
        version,
    })
}

async fn delete_entry(conn: &mut SqliteConnection, id: TimeEntryId) -> Result<Event> {
    diesel::delete(time_entry::table.filter(time_entry::id.eq(id.to_string())))
        .execute(conn)
        .wrap_err("delete entry")?;
    Ok(Event::Deleted)
}

async fn load_timesheet(conn: &mut SqliteConnection, day: Date) -> Result<Event> {
    let timesheet = load_or_create_timesheet(conn, day).await?;
    let entries = TimeEntry::belonging_to(&timesheet)
        .select(TimeEntry::as_select())
        .order_by(time_entry::start_time)
        .load::<TimeEntry>(conn)
        .wrap_err("loading timesheet entries")?;
    Ok(Event::TimesheetLoaded { timesheet, entries })
}

async fn load_timesheets_of_month(conn: &mut SqliteConnection, day: Date) -> Result<Event> {
    let format = format_description::parse("[year]-[month]-%")?;
    let month_like = day.format(&format)?;
    let timesheets = timesheet::table
        .filter(timesheet::day.like(&month_like))
        .select(Timesheet::as_select())
        .load(conn)
        .wrap_err_with(|| format!("load timesheets of {month_like}"))?;
    Ok(Event::TimesheetsOfMonthLoaded { day, timesheets })
}

async fn ensure_timesheet_exists(conn: &mut SqliteConnection, day: &str) -> Result<()> {
    let sheet = Timesheet {
        day: day.to_string(),
        status: "OPEN".to_string(),
    };
    diesel::insert_into(timesheet::table)
        .values(&sheet)
        .on_conflict(timesheet::day)
        .do_nothing()
        .execute(conn)
        .wrap_err_with(|| format!("ensure timesheet {day} exists"))?;
    Ok(())
}

async fn load_or_create_timesheet(conn: &mut SqliteConnection, day: Date) -> Result<Timesheet> {
    let format = format_description::parse("[year]-[month]-[day]")?;
    let iso_day = day.format(&format)?;
    let loaded = timesheet::table
        .filter(timesheet::day.eq(iso_day))
        .select(Timesheet::as_select())
        .get_result(conn)
        .optional()
        .wrap_err_with(|| format!("load timesheet {day}"))?;
    if let Some(loaded) = loaded {
        return Ok(loaded);
    }
    let created = Timesheet {
        day: day.to_string(),
        status: "OPEN".to_string(),
    };
    diesel::insert_into(timesheet::table)
        .values(&created)
        .execute(conn)
        .wrap_err_with(|| format!("create timesheet {day} since it didn't exist"))?;
    Ok(created)
}
