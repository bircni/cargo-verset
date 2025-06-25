use clap::Parser;

use crate::cli::{depoptions::DependencyOptions, pkgoptions::PackageOptions};

mod depoptions;
mod pkgoptions;
#[cfg(test)]
mod test_depoptions;
#[cfg(test)]
mod test_pkgoptions;

#[derive(Parser)]
#[command(author, version, about)]
pub enum Cli {
    /// Set the version of the package in a Cargo.toml file
    Package(PackageOptions),
    /// Set the version of a dependency in a Cargo.toml file
    Dependency(DependencyOptions),
}

impl Cli {
    /// Runs the appropriate command based on the subcommand provided.
    pub fn run(self) -> anyhow::Result<()> {
        match self {
            Self::Package(opts) => opts.run(),
            Self::Dependency(opts) => opts.run(),
        }
    }
}
