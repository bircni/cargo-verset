#![allow(
    clippy::blanket_clippy_restriction_lints,
    reason = "I want it thaaat way"
)]
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
            log::error!("{:#}", e);
            process::exit(1);
        }
    }
}

fn real_main() -> anyhow::Result<()> {
    simplelog::TermLogger::init(
        #[cfg(debug_assertions)]
        LevelFilter::max(),
        #[cfg(not(debug_assertions))]
        LevelFilter::Info,
        ConfigBuilder::new()
            // suppress all logs from dependencies
            .add_filter_allow_str("cargo-verset")
            .build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .context("Failed to initialize logger")?;

    Cli::parse_from(env::args().filter(|a| a != "verset")).run()?;

    Ok(())
}
