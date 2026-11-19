use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use log::LevelFilter;

use crate::{
    cli::{dependency::DependencyArgs, package::PackageArgs},
    workspace::Selection,
};

pub mod dependency;
pub mod package;

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
    /// Print debug output
    #[arg(long, short, global = true, conflicts_with = "quiet")]
    verbose: bool,
    /// Only print warnings and errors
    #[arg(long, short, global = true)]
    quiet: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Set the version of the packages in a crate or workspace
    Package(PackageArgs),
    /// Set the version requirement of a dependency in a crate or workspace
    Dependency(DependencyArgs),
}

/// Options shared by all subcommands.
#[derive(Args, Default)]
pub struct Scope {
    /// Path to the Cargo.toml (or its directory) of the crate or workspace
    #[arg(long, alias = "path", value_name = "PATH")]
    pub manifest_path: Option<PathBuf>,
    /// Only change these workspace members instead of all
    #[arg(long = "package", short = 'p', value_name = "NAME")]
    pub packages: Vec<String>,
    /// Do not change these workspace members
    #[arg(long, value_name = "NAME")]
    pub exclude: Vec<String>,
    /// Show what would be changed without writing any file
    #[arg(long, short)]
    pub dry_run: bool,
}

impl Scope {
    fn selection(&self) -> Selection {
        Selection {
            packages: self.packages.clone(),
            exclude: self.exclude.clone(),
        }
    }
}

impl Cli {
    /// The log level requested on the command line.
    pub const fn log_level(&self) -> LevelFilter {
        if self.verbose {
            LevelFilter::Debug
        } else if self.quiet {
            LevelFilter::Warn
        } else {
            LevelFilter::Info
        }
    }

    /// Runs the appropriate command based on the subcommand provided.
    pub fn run(&self) -> anyhow::Result<()> {
        match &self.command {
            Command::Package(args) => args.run(),
            Command::Dependency(args) => args.run(),
        }
    }
}
