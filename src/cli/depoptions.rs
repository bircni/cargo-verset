use std::{env, fs, path::PathBuf};

use clap::Parser;
use semver::Version;
use toml_edit::DocumentMut;

#[derive(Parser)]
#[command(author, version, about)]
pub struct DependencyOptions {
    /// Version to set for the package
    #[clap(long, short)]
    pub ver: Version,
    /// Name of the package to set the version for
    #[clap(long = "name", short = 'n')]
    pub package_name: String,
    /// Path to the directory containing the Cargo.toml file
    #[clap(long, short)]
    pub path: Option<PathBuf>,
    /// Run the program without making any changes
    #[clap(long, short)]
    pub dry_run: bool,
}

impl DependencyOptions {
    /// Helper to update a dependency item (String or Table) and preserve comment
    fn update_dep_item(item: &mut toml_edit::Item, version: &Version) {
        let comment = item
            .as_value()
            .and_then(|v| v.decor().suffix().map(|s| s.as_str()))
            .flatten();
        if let Some(table) = item.as_table_like() {
            let mut inline = toml_edit::InlineTable::new();
            for (k, v) in table.iter() {
                if k == "version" {
                    if let Some(val) = toml_edit::value(version.to_string()).as_value() {
                        inline.insert(k, val.clone());
                    }
                } else if k == "path" && !table.contains_key("version") {
                    // If there is no version key but there is a path key, remove path and set version
                    // path is skipped
                } else if let Some(val) = v.as_value() {
                    inline.insert(k, val.clone());
                }
            }
            // TODO: add option to set another key, e.g. "git" or "registry"
            // If no version key exists but a path key was removed, set version explicitly
            if !table.contains_key("version") && table.contains_key("path") {
                log::warn!("A path key was found but no version key, setting version explicitly");
                if let Some(val) = toml_edit::value(version.to_string()).as_value() {
                    inline.insert("version", val.clone());
                }
            }
            let mut new_item = toml_edit::Item::Value(toml_edit::Value::InlineTable(inline));
            if let (Some(c), Some(v)) = (comment, new_item.as_value_mut()) {
                v.decor_mut().set_suffix(c);
            }
            *item = new_item;
        } else if item.is_str() {
            let mut new_item = toml_edit::value(version.to_string());
            if let (Some(c), Some(v)) = (comment, new_item.as_value_mut()) {
                v.decor_mut().set_suffix(c);
            }
            *item = new_item;
        } else {
            let mut inline = toml_edit::InlineTable::new();
            if let Some(val) = toml_edit::value(version.to_string()).as_value() {
                inline.insert("version", val.clone());
            }
            let mut new_item = toml_edit::Item::Value(toml_edit::Value::InlineTable(inline));
            if let (Some(c), Some(v)) = (comment, new_item.as_value_mut()) {
                v.decor_mut().set_suffix(c);
            }
            *item = new_item;
        }
    }

    /// Sets the version for a specific dependency in the workspace Cargo.toml.
    pub fn run(&self) -> anyhow::Result<()> {
        let workspace_dir = if let Some(path) = self.path.clone() {
            path
        } else {
            env::current_dir()?
        };
        let workspace_toml = workspace_dir.join("Cargo.toml");
        if fs::metadata(&workspace_toml).is_err() {
            anyhow::bail!("Could not find Cargo.toml in the workspace");
        }
        let content = fs::read_to_string(&workspace_toml)?;
        let mut doc = content.parse::<DocumentMut>()?;
        // 1. [workspace.dependencies]
        if let Some(ws_deps) = doc
            .get_mut("workspace")
            .and_then(|ws| ws.get_mut("dependencies"))
        {
            if let Some(item) = ws_deps.get_mut(self.package_name.as_str()) {
                Self::update_dep_item(item, &self.ver);
                if self.dry_run {
                    log::info!(
                        "Dry run: Did not set version for workspace dependency '{}'!",
                        self.package_name
                    );
                } else {
                    fs::write(&workspace_toml, doc.to_string())?;
                    log::info!(
                        "Successfully set version for workspace dependency '{}' to {}",
                        self.package_name,
                        self.ver
                    );
                }
                return Ok(());
            }
        }
        // 2. [dependencies]
        let deps = doc.entry("dependencies").or_insert(toml_edit::table());
        if let Some(item) = deps.get_mut(self.package_name.as_str()) {
            Self::update_dep_item(item, &self.ver);
        } else {
            deps[self.package_name.as_str()] = toml_edit::value(self.ver.to_string());
        }
        if self.dry_run {
            log::info!(
                "Dry run: Did not set version for root dependency '{}'!",
                self.package_name
            );
        } else {
            fs::write(&workspace_toml, doc.to_string())?;
            log::info!(
                "Successfully set version for root dependency '{}' to {}",
                self.package_name,
                self.ver
            );
        }
        Ok(())
    }
}
