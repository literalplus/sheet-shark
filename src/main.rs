use std::time::Duration;

use clap::Parser;
use cli::Cli;
use color_eyre::eyre::{Result, WrapErr};
use futures::executor;

use crate::app::App;

mod action;
mod app;
mod cli;
mod components;
mod config;
mod errors;
mod logging;
mod tui;
mod layout;

fn main() -> Result<()> {
    bootstrap(|| {
        let args = Cli::parse();
        let mut app = App::new(args.tick_rate, args.frame_rate)?;

        executor::block_on(app.run())?;

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
