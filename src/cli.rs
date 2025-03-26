use std::{env, fs, path::PathBuf};

use clap::Parser;
use semver::Version;
use toml_edit::DocumentMut;

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    /// Version to set in the workspace
    #[clap(long, short)]
    pub ver: Version,
    /// Path to the directory containing the Cargo.toml file
    #[clap(long, short)]
    pub path: Option<PathBuf>,
    /// Run the program without making any changes
    #[clap(long, short)]
    pub dry_run: bool,
}

impl Cli {
    #[expect(
        clippy::option_if_let_else,
        reason = "Cannot borrow `doc` mutably twice"
    )]
    pub fn run(&self) -> anyhow::Result<()> {
        let toml_file = if let Some(path) = self.path.clone() {
            path
        } else {
            env::current_dir()?
        }
        .join("Cargo.toml");
        log::debug!("{}", toml_file.display());

        if fs::exists(&toml_file)? {
            let content = fs::read_to_string(&toml_file)?;
            let mut doc = content.parse::<DocumentMut>()?;

            let entrypoint = match doc.get_mut("workspace") {
                Some(entry) => entry,
                _ => doc.as_item_mut(),
            };
            if let Some(package) = entrypoint.get_mut("package") {
                if let Some(version) = package.get_mut("version") {
                    *version = toml_edit::value(self.ver.to_string());
                    if self.dry_run {
                        log::info!("Dry run: Did not set version to {}!", self.ver);
                    } else {
                        fs::write(&toml_file, doc.to_string())?;
                        log::info!("Successfully set version to {}", self.ver);
                    }
                } else {
                    log::warn!("Version key not found in Cargo.toml");
                }
            } else {
                log::warn!("Package section not found in Cargo.toml");
            }
        } else {
            anyhow::bail!("Could not find a Cargo.toml file");
        }
        Ok(())
    }
}
