use std::collections::BTreeMap;

use anyhow::Context as _;
use semver::{BuildMetadata, Prerelease, Version, VersionReq};

use crate::{
    manifest::{
        Change, DepEdit, Manifest, VersionField, dep_package_name, dep_version, is_path_dep,
        set_dep_version,
    },
    workspace::Selection,
};

/// Which part of a version to increment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Bump {
    Major,
    Minor,
    Patch,
}

/// The version a package should get.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// Set exactly this version
    Set(Version),
    /// Increment the current version
    Bump(Bump),
}

impl Target {
    fn resolve(&self, current: &str) -> anyhow::Result<Version> {
        let bump = match self {
            Self::Set(version) => return Ok(version.clone()),
            Self::Bump(bump) => *bump,
        };
        let mut version = Version::parse(current)
            .with_context(|| format!("Cannot bump invalid version '{current}'"))?;
        match bump {
            Bump::Major => {
                version.major += 1;
                version.minor = 0;
                version.patch = 0;
            }
            Bump::Minor => {
                version.minor += 1;
                version.patch = 0;
            }
            // `1.2.3-rc.1` is released as `1.2.3`
            Bump::Patch if version.pre.is_empty() => version.patch += 1,
            Bump::Patch => {}
        }
        version.pre = Prerelease::EMPTY;
        version.build = BuildMetadata::EMPTY;
        Ok(version)
    }
}

/// Sets the version of all selected packages.
///
/// This updates `workspace.package.version`, every literal `package.version` and the version
/// requirement of path dependencies on the changed packages that no longer match.
pub fn set_version(
    manifests: &mut [Manifest],
    selection: &Selection,
    target: &Target,
) -> anyhow::Result<Vec<Change>> {
    let mut changes = Vec::new();
    // package name -> new version, used to fix up dependents
    let mut versions = BTreeMap::new();

    let inherited_selected = manifests.iter().any(|m| {
        m.package_version() == VersionField::Inherited
            && m.package_name().is_some_and(|name| selection.selects(name))
    });
    let mut workspace_version = None;
    if !selection.is_filtered() || inherited_selected {
        for manifest in manifests.iter_mut() {
            let Some(old) = manifest.workspace_version().map(ToOwned::to_owned) else {
                continue;
            };
            if selection.is_filtered() {
                log::warn!(
                    "workspace.package.version is shared, all members inheriting it are changed"
                );
            }
            let new = target.resolve(&old)?;
            manifest.set_workspace_version(&new.to_string());
            changes.push(Change {
                manifest: manifest.path().to_path_buf(),
                key: String::from("workspace.package.version"),
                old: Some(old),
                new: new.to_string(),
            });
            workspace_version = Some(new);
        }
    }

    for manifest in manifests.iter_mut() {
        let Some(name) = manifest.package_name().map(ToOwned::to_owned) else {
            continue;
        };
        match manifest.package_version() {
            VersionField::Literal(old) if selection.selects(&name) => {
                let new = target.resolve(&old)?;
                manifest.set_package_version(&new.to_string());
                changes.push(Change {
                    manifest: manifest.path().to_path_buf(),
                    key: String::from("package.version"),
                    old: Some(old),
                    new: new.to_string(),
                });
                versions.insert(name, new);
            }
            VersionField::Literal(_) => {}
            VersionField::Inherited => {
                if let Some(version) = &workspace_version {
                    versions.insert(name, version.clone());
                }
            }
            VersionField::Missing => {
                if selection.selects(&name) {
                    log::warn!("Package '{name}' has no version, skipping it");
                }
            }
        }
    }

    for manifest in manifests.iter_mut() {
        let path = manifest.path().to_path_buf();
        manifest.visit_dep_tables(|table_name, _, table| {
            for (key, item) in table.iter_mut() {
                let Some(version) = versions.get(dep_package_name(key.get(), item)) else {
                    continue;
                };
                if !is_path_dep(item) {
                    continue;
                }
                let Some(old) = dep_version(item).map(ToOwned::to_owned) else {
                    continue;
                };
                if VersionReq::parse(&old).is_ok_and(|req| req.matches(version)) {
                    continue;
                }
                let req = if old.trim_start().starts_with('=') {
                    format!("={version}")
                } else {
                    version.to_string()
                };
                if let DepEdit::Changed { old } = set_dep_version(item, &req, None, false) {
                    changes.push(Change {
                        manifest: path.clone(),
                        key: format!("{table_name}.{}.version", key.get()),
                        old,
                        new: req,
                    });
                }
            }
        });
    }

    if changes.is_empty() {
        anyhow::bail!("No package version found that could be set");
    }
    Ok(changes)
}
