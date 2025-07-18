use std::env;

use color_eyre::{
    Result,
    eyre::{Context, eyre},
};
use diesel::{Connection, ExpressionMethods, RunQueryDsl, SqliteConnection};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use tokio::{
    runtime::Builder,
    select,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::LocalSet,
};
use tracing::{debug, info, warn};

pub mod model;
mod schema;
pub use model::*;

use crate::persist::schema::timesheet;

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
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL in env to connect to sqlite");
    let mut conn = SqliteConnection::establish(&db_url)
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
                    let work = work_opt.expect("nobody else to be able to close the cmd_rx");
                    info!("Persistence command: {work:?}");
                    self.try_handle(work).await;
                }
            }
        }
    }

    async fn try_handle(&mut self, cmd: model::Command) {
        let event = match cmd {
            model::Command::Demo => {
                let sheet = Timesheet {
                    day: "fake",
                    status: "OPEN",
                };
                diesel::insert_into(timesheet::table)
                    .values(&sheet)
                    .on_conflict(timesheet::day)
                    .do_update()
                    .set(timesheet::status.eq("OPEN"))
                    .execute(&mut self.conn)
                    .expect("oida");
                model::Event::Demo
            }
        };
        if let Err(err) = self.evt_tx.send(event) {
            debug!("Unable to send persistence event: {err:?}");
        }
    }
}
