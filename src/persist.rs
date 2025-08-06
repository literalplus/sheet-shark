use std::str::FromStr;

use color_eyre::{
    Result,
    eyre::{Context, eyre},
};
use diesel::{Connection, RunQueryDsl, SqliteConnection, prelude::*};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use tokio::{
    runtime::Builder,
    select,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::LocalSet,
};
use tracing::{debug, error, info, warn};

pub mod model;
mod schema;
pub use model::*;

use crate::{
    config::get_data_dir,
    persist::schema::{
        time_entry::{self},
        timesheet,
    },
};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn start_async(
    cmd_rx: UnboundedReceiver<Command>,
    evt_tx: UnboundedSender<Event>,
) -> Result<std::thread::JoinHandle<()>> {
    let handler = PersistHandler {
        conn: prepare_connection()?,
        cmd_rx,
        evt_tx,
    };
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime to build in persist thread");
    let handle = std::thread::Builder::new()
        .name("persist".into())
        .spawn(move || {
            let local = LocalSet::new();
            local.spawn_local(handler.run());
            runtime.block_on(local);
        })?;
    Ok(handle)
}

fn prepare_connection() -> Result<SqliteConnection> {
    let mut db_url = get_data_dir();
    db_url.push("sharkdb.sqlite");
    let db_url = db_url.to_str().expect("path to convert to string");
    let mut conn = SqliteConnection::establish(db_url)
        .wrap_err_with(|| format!("connecting to sqlite {db_url}"))?;

    debug!("Running any pending migrations now.");
    match conn.run_pending_migrations(MIGRATIONS) {
        Ok(migrations_run) => {
            for migration in migrations_run {
                info!("Schema migration run: {}", migration);
            }
        }
        Err(e) => Err(eyre!(e)).wrap_err_with(|| "running sqlite migrations")?,
    }
    Ok(conn)
}

struct PersistHandler {
    conn: SqliteConnection,
    cmd_rx: UnboundedReceiver<model::Command>,
    evt_tx: UnboundedSender<model::Event>,
}

impl PersistHandler {
    async fn run(mut self) -> Result<()> {
        loop {
            select! {
                biased; // Stop should take prio
                _ = self.evt_tx.closed() => {
                    debug!("Persistence events channel closed, shutting down persist handler...");
                    self.cmd_rx.close();
                    while let Ok(leftover_cmd) = self.cmd_rx.try_recv() {
                        warn!("Still handling leftover command {leftover_cmd:?}");
                        self.try_handle(leftover_cmd).await;
                    }
                    return Ok(());
                },
                work_opt = self.cmd_rx.recv() => {
                    let work = work_opt.expect("nobody else to close the cmd_rx");
                    info!("Persistence command: {work:?}");
                    self.try_handle(work).await;
                }
            }
        }
    }

    async fn try_handle(&mut self, cmd: model::Command) {
        match self.handle(cmd).await {
            Ok(event) => {
                if let Err(err) = self.evt_tx.send(event) {
                    debug!("Unable to send persistence event: {err:?}");
                }
            }
            Err(err) => {
                error!("Error handling persistence command: {err:?}");
                let event = model::Event::Failure(format!("{err:?}"));
                if let Err(err) = self.evt_tx.send(event) {
                    debug!("Unable to send persistence error: {err:?}");
                }
            }
        }
    }

    async fn handle(&mut self, cmd: model::Command) -> Result<model::Event> {
        match cmd {
            model::Command::StoreEntry { entry, version } => {
                self.ensure_timesheet_exists(&entry.timesheet_day).await?;

                diesel::insert_into(time_entry::table)
                    .values(&entry)
                    .on_conflict(time_entry::id)
                    .do_update()
                    .set(&entry)
                    .execute(&mut self.conn)
                    .wrap_err("saving time entry")?;
                Ok(model::Event::EntryStored {
                    id: TimeEntryId::from_str(&entry.id)?,
                    version,
                })
            }
            model::Command::DeleteEntry(id) => {
                diesel::delete(time_entry::table.filter(time_entry::id.eq(id.to_string())))
                    .execute(&mut self.conn)
                    .wrap_err("delete entry")?;
                Ok(model::Event::Deleted)
            }
            model::Command::LoadTimesheet { day } => {
                let timesheet = self.load_or_create_timesheet(&day).await?;
                let entries = TimeEntry::belonging_to(&timesheet)
                    .select(TimeEntry::as_select())
                    .load::<TimeEntry>(&mut self.conn)
                    .wrap_err("loading timesheet entries")?;
                Ok(model::Event::TimesheetLoaded { timesheet, entries })
            }
        }
    }

    async fn ensure_timesheet_exists(&mut self, day: &str) -> Result<()> {
        let sheet = Timesheet {
            day: day.to_string(),
            status: "OPEN".to_string(),
        };
        diesel::insert_into(timesheet::table)
            .values(&sheet)
            .on_conflict(timesheet::day)
            .do_nothing()
            .execute(&mut self.conn)
            .wrap_err_with(|| format!("ensure timesheet {day} exists"))?;
        Ok(())
    }

    async fn load_or_create_timesheet(&mut self, day: &str) -> Result<Timesheet> {
        let loaded = timesheet::table
            .filter(timesheet::day.eq(day))
            .select(Timesheet::as_select())
            .get_result(&mut self.conn)
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
            .execute(&mut self.conn)
            .wrap_err_with(|| format!("create timesheet {day} since it didn't exist"))?;
        Ok(created)
    }
}
