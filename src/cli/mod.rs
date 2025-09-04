use clap::Parser;

use crate::cli::{depoptions::DependencyOptions, pkgoptions::PackageOptions, releaseoptions::ReleaseOptions};

mod depoptions;
mod pkgoptions;
mod releaseoptions;
#[cfg(test)]
mod test_depoptions;
#[cfg(test)]
mod test_pkgoptions;
#[cfg(test)]
mod test_releaseoptions;

#[derive(Parser)]
#[command(author, version, about)]
pub enum Cli {
    /// Set the version of the package in a Cargo.toml file
    Package(PackageOptions),
    /// Set the version of a dependency in a Cargo.toml file
    Dependency(DependencyOptions),
    /// Create a release with changelog generation and version bumping
    Release(ReleaseOptions),
}

impl Cli {
    /// Runs the appropriate command based on the subcommand provided.
    pub fn run(self) -> anyhow::Result<()> {
        match self {
            Self::Package(opts) => opts.run(),
            Self::Dependency(opts) => opts.run(),
            Self::Release(opts) => opts.run(),
        }
    }
}
