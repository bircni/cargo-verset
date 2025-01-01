use anyhow::Context;
use clap::Parser;
use cli::Cli;
use log::LevelFilter;
use simplelog::{ColorChoice, ConfigBuilder, TerminalMode};

mod cli;
#[cfg(test)]
mod test;

fn main() -> anyhow::Result<()> {
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

    Cli::parse().run()?;

    Ok(())
}
