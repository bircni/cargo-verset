use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::Context as _;
use cargo_metadata::MetadataCommand;

use crate::manifest::{Change, Manifest};

/// Which workspace members a command applies to.
#[derive(Debug, Default)]
pub struct Selection {
    /// Only these members, all members if empty
    pub packages: Vec<String>,
    /// Members to skip
    pub exclude: Vec<String>,
}

impl Selection {
    /// Whether the member with this package name is selected.
    pub fn selects(&self, name: &str) -> bool {
        (self.packages.is_empty() || self.packages.iter().any(|p| p == name))
            && !self.exclude.iter().any(|e| e == name)
    }

    /// Whether the selection is narrower than the full workspace.
    pub const fn is_filtered(&self) -> bool {
        !self.packages.is_empty() || !self.exclude.is_empty()
    }
}

/// The root manifest and all member manifests of a workspace (or a single crate).
pub struct Workspace {
    root: PathBuf,
    manifests: Vec<Manifest>,
}

impl Workspace {
    /// Loads the workspace that `path` (a `Cargo.toml` or its directory) belongs to.
    ///
    /// Defaults to the current directory.
    pub fn load(path: Option<&Path>) -> anyhow::Result<Self> {
        let mut manifest_path = match path {
            Some(path) => path.to_path_buf(),
            None => env::current_dir().context("Failed to get the current directory")?,
        };
        if manifest_path.is_dir() {
            manifest_path.push("Cargo.toml");
        }
        if !manifest_path.is_file() {
            anyhow::bail!("Could not find {}", manifest_path.display());
        }

        // `--no-deps` resolves workspace members without touching the network or the lockfile
        let metadata = MetadataCommand::new()
            .manifest_path(&manifest_path)
            .no_deps()
            .exec()
            .with_context(|| {
                format!(
                    "Failed to load the workspace of {}",
                    manifest_path.display()
                )
            })?;

        let root = metadata.workspace_root.clone().into_std_path_buf();
        let root_manifest = root.join("Cargo.toml");
        let mut members: Vec<PathBuf> = metadata
            .workspace_packages()
            .into_iter()
            .map(|package| package.manifest_path.clone().into_std_path_buf())
            .filter(|path| *path != root_manifest)
            .collect();
        members.sort();

        let manifests = std::iter::once(root_manifest)
            .chain(members)
            .map(|path| Manifest::load(&path))
            .collect::<anyhow::Result<_>>()?;

        Ok(Self { root, manifests })
    }

    pub fn manifests_mut(&mut self) -> &mut [Manifest] {
        &mut self.manifests
    }

    /// Fails if the selection names a package that is not a workspace member.
    pub fn validate(&self, selection: &Selection) -> anyhow::Result<()> {
        for name in selection.packages.iter().chain(&selection.exclude) {
            if !self
                .manifests
                .iter()
                .any(|m| m.package_name() == Some(name.as_str()))
            {
                anyhow::bail!("Package '{name}' is not a member of the workspace");
            }
        }
        Ok(())
    }

    /// Reports the changes and, unless `dry_run` is set, writes the edited manifests.
    pub fn finish(&self, changes: &[Change], dry_run: bool) -> anyhow::Result<()> {
        for change in changes {
            let manifest = change
                .manifest
                .strip_prefix(&self.root)
                .unwrap_or(&change.manifest)
                .display();
            let key = &change.key;
            let new = &change.new;
            match &change.old {
                Some(_) if change.is_noop() => log::info!("{manifest}: {key} is already {new}"),
                Some(old) => log::info!("{manifest}: {key} {old} -> {new}"),
                None => log::info!("{manifest}: {key} set to {new}"),
            }
        }

        if dry_run {
            log::info!("Dry run: no files were changed");
            return Ok(());
        }

        let mut written = 0_usize;
        for manifest in &self.manifests {
            if manifest.save()? {
                written += 1;
            }
        }
        log::info!("Updated {written} manifest(s)");
        Ok(())
    }
}
