use std::str::FromStr;

use color_eyre::{Result, eyre::Context};
use diesel::{
    RunQueryDsl, SqliteConnection,
    dsl::count,
    expression::{ValidGrouping, is_aggregate},
    prelude::*,
    sql_types::{Nullable, Text},
};

use time::{
    Date, OffsetDateTime, ext::NumericalDuration, format_description::FormatItem,
    macros::format_description,
};
use tracing::warn;

use crate::persist::{
    Command, Event, TimeEntry, TimeEntryId, Timesheet,
    schema::{
        time_entry::{self},
        timesheet,
    },
};

pub(super) async fn handle(conn: &mut SqliteConnection, cmd: Command) -> Result<Event> {
    match cmd {
        Command::StoreEntry { entry, version } => store_entry(conn, entry, version).await,
        Command::DeleteEntry(id) => delete_entry(conn, id).await,
        Command::LoadTimesheet { day } => load_timesheet(conn, day).await,
        Command::LoadTimesheetsOfMonth { day } => load_timesheets_of_month(conn, day).await,
        Command::SuggestTickets { query } => suggest_tickets(conn, query).await,
    }
}

async fn store_entry(conn: &mut SqliteConnection, entry: TimeEntry, version: i32) -> Result<Event> {
    if entry.is_empty_default() {
        return Ok(Event::EntryStored {
            id: TimeEntryId::from_str(&entry.id)?,
            version: -1,
        });
    }
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
    let timesheet = load_timesheet_or_dummy(conn, day).await?;
    let entries = TimeEntry::belonging_to(&timesheet)
        .select(TimeEntry::as_select())
        .order_by(time_entry::start_time)
        .load::<TimeEntry>(conn)
        .wrap_err("loading timesheet entries")?;
    if entries.is_empty() {
        warn!("Noticed empty timesheet while loading, cleaning it up: {day}");
        delete_timesheet(conn, day).await?;
    } else if entries.len() == 1
        && entries[0].start_time == "00:00"
        && entries[0].duration_mins == 0
    {
        warn!("Cleaning up dummy entry: {day}");
        delete_entry(conn, TimeEntryId::from_str(&entries[0].id)?).await?;
    }
    Ok(Event::TimesheetLoaded {
        day,
        timesheet,
        entries,
    })
}

async fn delete_timesheet(conn: &mut SqliteConnection, day: Date) -> Result<()> {
    let iso_day = day.format(ISO_DAY)?;
    diesel::delete(timesheet::table.filter(timesheet::day.eq(iso_day)))
        .execute(conn)
        .wrap_err("delete timesheet")?;
    Ok(())
}

async fn load_timesheets_of_month(conn: &mut SqliteConnection, day: Date) -> Result<Event> {
    let month_like = day.format(ISO_MONTH_WILDCARD)?;
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

async fn load_timesheet_or_dummy(conn: &mut SqliteConnection, day: Date) -> Result<Timesheet> {
    let iso_day = day.format(ISO_DAY)?;
    let loaded = timesheet::table
        .filter(timesheet::day.eq(iso_day))
        .select(Timesheet::as_select())
        .get_result(conn)
        .optional()
        .wrap_err_with(|| format!("load timesheet {day}"))?;
    if let Some(loaded) = loaded {
        return Ok(loaded);
    }
    let dummy = Timesheet {
        day: day.to_string(),
        status: "OPEN".to_string(),
    };
    Ok(dummy)
}

define_sql_function!(fn lower(x: Nullable<Text>) -> Text);
impl ValidGrouping<lower_utils::lower<time_entry::ticket_key>> for lower<time_entry::ticket_key> {
    type IsAggregate = is_aggregate::No;
}

async fn suggest_tickets(conn: &mut SqliteConnection, query: String) -> Result<Event> {
    let query = query.to_lowercase();
    let six_months_ago = OffsetDateTime::now_local()?
        .date()
        .saturating_sub((6 * 30).days());
    let six_months_ago = six_months_ago.format(ISO_DAY)?;
    let filter = time_entry::timesheet_day
        .gt(six_months_ago)
        .and(time_entry::ticket_key.is_not_null());

    let mut select = time_entry::table
        .filter(filter)
        .group_by(lower(time_entry::ticket_key))
        .select(lower(time_entry::ticket_key))
        .order_by(count(time_entry::ticket_key))
        .into_boxed();

    if let Some((jira_project, issue_key)) = query.split_once('-') {
        select = select
            .filter(lower(time_entry::ticket_key).like(format!("{jira_project}%")))
            .filter(lower(time_entry::ticket_key).like(format!("%-{issue_key}%")));
    } else {
        select = select.filter(lower(time_entry::ticket_key).like(format!("{query}%")));
    }

    let ticket_keys = select.get_results(conn)?;
    Ok(Event::TicketsSuggested { ticket_keys, query })
}

const ISO_DAY: &[FormatItem<'static>] = format_description!("[year]-[month]-[day]");
const ISO_MONTH_WILDCARD: &[FormatItem<'static>] = format_description!("[year]-[month]-%");
