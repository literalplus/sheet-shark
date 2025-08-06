use std::time::Duration;

use clap::Parser;
use cli::Cli;
use color_eyre::eyre::{Result, WrapErr, eyre};
use futures::executor;
use tokio::sync::mpsc;

use crate::app::App;

mod action;
mod app;
mod cli;
mod components;
mod config;
mod errors;
mod layout;
mod logging;
mod persist;
mod shared;
mod tui;

fn main() -> Result<()> {
    bootstrap(|| {
        let args = Cli::parse();

        let (persist_tx, persist_rx) = mpsc::unbounded_channel();
        let (persisted_tx, persisted_rx) = mpsc::unbounded_channel();
        let persist_handle = persist::start_async(persist_rx, persisted_tx)?;

        let app = App::new(args.tick_rate, args.frame_rate, persist_tx, persisted_rx)?;
        executor::block_on(app.run())?;

        // Allow remaining actions on the persist thread to complete; App closes channel to initiate shutdown
        persist_handle
            .join()
            .map_err(|err| eyre!("Persist thread panicked: {err:?}"))?;
        Ok(())
    })
}

fn bootstrap(fn_do_run: fn() -> Result<()>) -> Result<()> {
    crate::errors::init()?;
    crate::logging::init()?;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .wrap_err_with(|| "Failed to start Tokio runtime")?;
    let _guard = runtime.enter();

    let result = fn_do_run();
    runtime.shutdown_timeout(Duration::from_secs(5));

    result
}
