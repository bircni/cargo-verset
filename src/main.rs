use std::{env, process};

use anyhow::Context as _;
use clap::Parser as _;
use cli::Cli;
use log::LevelFilter;
use simplelog::{ColorChoice, ConfigBuilder, TerminalMode};

mod cli;
#[cfg(test)]
mod test;

fn main() {
    match real_main() {
        Ok(()) => {}
        Err(e) => {
            log::error!("{e:#}");
            process::exit(1);
        }
    }
}

fn real_main() -> anyhow::Result<()> {
    initialize_logger()?;

    Cli::parse_from(env::args().filter(|a| a != "verset")).run()?;

    Ok(())
}

fn initialize_logger() -> anyhow::Result<()> {
    simplelog::TermLogger::init(
        #[cfg(debug_assertions)]
        LevelFilter::max(),
        #[cfg(not(debug_assertions))]
        LevelFilter::Info,
        ConfigBuilder::new()
            // suppress all logs from dependencies
            .add_filter_allow_str("cargo_verset")
            .build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .context("Failed to initialize logger")
}
