//! A cargo plugin to set package and dependency versions across a whole workspace.

use std::{env, ffi::OsString, process};

use anyhow::Context as _;
use clap::Parser as _;
use cli::Cli;
use log::LevelFilter;
use simplelog::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode};

mod cli;
mod manifest;
mod ops;
#[cfg(test)]
mod test;
mod workspace;

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
    let cli = Cli::parse_from(strip_subcommand(env::args_os().collect()));
    initialize_logger(cli.log_level())?;

    cli.run()
}

/// Removes the `verset` argument cargo inserts when invoked as `cargo verset`.
///
/// Only the first argument is checked, so that values like `--package verset` stay intact.
fn strip_subcommand(mut args: Vec<OsString>) -> Vec<OsString> {
    if args.get(1).is_some_and(|arg| arg == "verset") {
        args.remove(1);
    }
    args
}

fn initialize_logger(level_filter: LevelFilter) -> anyhow::Result<()> {
    TermLogger::init(
        level_filter,
        ConfigBuilder::new()
            .add_filter_allow_str("cargo_verset")
            .build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .context("Failed to initialize logger")
}
