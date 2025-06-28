use std::time::Duration;

use anyhow::{Context, Result};
use human_panic::setup_panic;

fn main() -> Result<()> {
    bootstrap(|| {
        println!("Henlo world (:");
        Ok(())
    })
}

fn bootstrap(fn_do_run: fn() -> Result<()>) -> Result<()> {
    setup_panic!();
    if let Err(env_err) = dotenvy::dotenv() {
        if !env_err.not_found() {
            return Err(env_err).with_context(|| "Failed to load `.env` file");
        }
    }

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .with_context(|| "Failed to start Tokio runtime")?;
    let _guard = runtime.enter();

    let result = fn_do_run();
    runtime.shutdown_timeout(Duration::from_secs(5));

    return result;
}
