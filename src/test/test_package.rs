use insta::assert_snapshot;
use semver::Version;

use super::{dump, manifest, project, run};
use crate::ops::package::{Bump, Target, set_version};
use crate::workspace::Selection;

const WORKSPACE: &[(&str, &str)] = &[
    (
        "Cargo.toml",
        r#"
[workspace]
members = ["crates/*"]
resolver = "3"

[workspace.package]
version = "0.1.0" # shared version

[workspace.dependencies]
core = { path = "crates/core", version = "0.1.0" }
"#,
    ),
    (
        "crates/core/Cargo.toml",
        r#"
[package]
name = "core"
version.workspace = true
edition = "2024"
"#,
    ),
    (
        "crates/util/Cargo.toml",
        r#"
[package]
name = "util"
version = "0.4.0"
edition = "2024"

[dependencies]
core = { workspace = true }
"#,
    ),
    (
        "crates/app/Cargo.toml",
        r#"
[package]
name = "app"
version = "0.1.0"
edition = "2024"

[dependencies]
core = { path = "../core", version = "0.1.0", default-features = false } # internal

[dependencies.util]
path = "../util"
version = "=0.4.0"

[dev-dependencies]
core-renamed = { package = "core", path = "../core", version = "0.1" }

[target.'cfg(unix)'.dependencies]
util = { path = "../util" }
"#,
    ),
];

const WORKSPACE_FILES: &[&str] = &[
    "Cargo.toml",
    "crates/core/Cargo.toml",
    "crates/util/Cargo.toml",
    "crates/app/Cargo.toml",
];

fn set(content: &str, target: &Target) -> anyhow::Result<String> {
    let mut manifests = manifest(content);
    set_version(&mut manifests, &Selection::default(), target)?;
    Ok(manifests[0].content())
}

#[test]
fn test_single_crate_keeps_formatting() {
    let content = r#"
[package]
name = "single"
version   =   "0.1.0"   # keep me
"#;
    let result = set(content, &Target::Set(Version::new(1, 2, 3))).unwrap();
    assert_snapshot!(result, @r#"

    [package]
    name = "single"
    version   =   "1.2.3"   # keep me
    "#);
}

#[test]
fn test_root_package_with_workspace_table() {
    let dir = project(&[(
        "Cargo.toml",
        r#"
[package]
name = "root"
version = "0.1.0"
edition = "2024"

[workspace]
"#,
    )]);
    run(&dir, &["package", "1.0.0"]).unwrap();
    assert_snapshot!(dump(&dir, &["Cargo.toml"]), @r#"
    ### Cargo.toml
    [package]
    name = "root"
    version = "1.0.0"
    edition = "2024"

    [workspace]
    "#);
}

#[test]
fn test_full_workspace() {
    let dir = project(WORKSPACE);
    run(&dir, &["package", "2.0.0"]).unwrap();
    assert_snapshot!(dump(&dir, WORKSPACE_FILES), @r#"
    ### Cargo.toml
    [workspace]
    members = ["crates/*"]
    resolver = "3"

    [workspace.package]
    version = "2.0.0" # shared version

    [workspace.dependencies]
    core = { path = "crates/core", version = "2.0.0" }
    ### crates/core/Cargo.toml
    [package]
    name = "core"
    version.workspace = true
    edition = "2024"
    ### crates/util/Cargo.toml
    [package]
    name = "util"
    version = "2.0.0"
    edition = "2024"

    [dependencies]
    core = { workspace = true }
    ### crates/app/Cargo.toml
    [package]
    name = "app"
    version = "2.0.0"
    edition = "2024"

    [dependencies]
    core = { path = "../core", version = "2.0.0", default-features = false } # internal

    [dependencies.util]
    path = "../util"
    version = "=2.0.0"

    [dev-dependencies]
    core-renamed = { package = "core", path = "../core", version = "2.0.0" }

    [target.'cfg(unix)'.dependencies]
    util = { path = "../util" }
    "#);
}

#[test]
fn test_compatible_requirements_are_kept() {
    let dir = project(WORKSPACE);
    run(&dir, &["package", "0.1.5", "--package", "core"]).unwrap();
    assert_snapshot!(dump(&dir, WORKSPACE_FILES), @r#"
    ### Cargo.toml
    [workspace]
    members = ["crates/*"]
    resolver = "3"

    [workspace.package]
    version = "0.1.5" # shared version

    [workspace.dependencies]
    core = { path = "crates/core", version = "0.1.0" }
    ### crates/core/Cargo.toml
    [package]
    name = "core"
    version.workspace = true
    edition = "2024"
    ### crates/util/Cargo.toml
    [package]
    name = "util"
    version = "0.4.0"
    edition = "2024"

    [dependencies]
    core = { workspace = true }
    ### crates/app/Cargo.toml
    [package]
    name = "app"
    version = "0.1.0"
    edition = "2024"

    [dependencies]
    core = { path = "../core", version = "0.1.0", default-features = false } # internal

    [dependencies.util]
    path = "../util"
    version = "=0.4.0"

    [dev-dependencies]
    core-renamed = { package = "core", path = "../core", version = "0.1" }

    [target.'cfg(unix)'.dependencies]
    util = { path = "../util" }
    "#);
}

#[test]
fn test_selected_package_only() {
    let dir = project(WORKSPACE);
    run(&dir, &["package", "0.5.0", "-p", "util"]).unwrap();
    assert_snapshot!(dump(&dir, WORKSPACE_FILES), @r#"
    ### Cargo.toml
    [workspace]
    members = ["crates/*"]
    resolver = "3"

    [workspace.package]
    version = "0.1.0" # shared version

    [workspace.dependencies]
    core = { path = "crates/core", version = "0.1.0" }
    ### crates/core/Cargo.toml
    [package]
    name = "core"
    version.workspace = true
    edition = "2024"
    ### crates/util/Cargo.toml
    [package]
    name = "util"
    version = "0.5.0"
    edition = "2024"

    [dependencies]
    core = { workspace = true }
    ### crates/app/Cargo.toml
    [package]
    name = "app"
    version = "0.1.0"
    edition = "2024"

    [dependencies]
    core = { path = "../core", version = "0.1.0", default-features = false } # internal

    [dependencies.util]
    path = "../util"
    version = "=0.5.0"

    [dev-dependencies]
    core-renamed = { package = "core", path = "../core", version = "0.1" }

    [target.'cfg(unix)'.dependencies]
    util = { path = "../util" }
    "#);
}

#[test]
fn test_excluded_package() {
    let dir = project(WORKSPACE);
    run(&dir, &["package", "--bump", "minor", "--exclude", "util"]).unwrap();
    assert_snapshot!(dump(&dir, WORKSPACE_FILES), @r#"
    ### Cargo.toml
    [workspace]
    members = ["crates/*"]
    resolver = "3"

    [workspace.package]
    version = "0.2.0" # shared version

    [workspace.dependencies]
    core = { path = "crates/core", version = "0.2.0" }
    ### crates/core/Cargo.toml
    [package]
    name = "core"
    version.workspace = true
    edition = "2024"
    ### crates/util/Cargo.toml
    [package]
    name = "util"
    version = "0.4.0"
    edition = "2024"

    [dependencies]
    core = { workspace = true }
    ### crates/app/Cargo.toml
    [package]
    name = "app"
    version = "0.2.0"
    edition = "2024"

    [dependencies]
    core = { path = "../core", version = "0.2.0", default-features = false } # internal

    [dependencies.util]
    path = "../util"
    version = "=0.4.0"

    [dev-dependencies]
    core-renamed = { package = "core", path = "../core", version = "0.2.0" }

    [target.'cfg(unix)'.dependencies]
    util = { path = "../util" }
    "#);
}

#[test]
fn test_bump() {
    let bump = |version: &str, bump| {
        let content = format!("[package]\nname = \"single\"\nversion = \"{version}\"\n");
        let result = set(&content, &Target::Bump(bump)).unwrap();
        result.lines().last().unwrap().to_owned()
    };
    assert_eq!(bump("1.2.3", Bump::Major), r#"version = "2.0.0""#);
    assert_eq!(bump("1.2.3", Bump::Minor), r#"version = "1.3.0""#);
    assert_eq!(bump("1.2.3", Bump::Patch), r#"version = "1.2.4""#);
    assert_eq!(bump("1.2.3-rc.1", Bump::Patch), r#"version = "1.2.3""#);
    assert_eq!(
        bump("1.2.3-rc.1+build", Bump::Minor),
        r#"version = "1.3.0""#
    );
}

#[test]
fn test_bump_invalid_version() {
    let content = "[package]\nname = \"single\"\nversion = \"one\"\n";
    let error = set(content, &Target::Bump(Bump::Patch)).unwrap_err();
    assert_eq!(error.to_string(), "Cannot bump invalid version 'one'");
}

#[test]
fn test_missing_version_is_an_error() {
    for content in [
        "[package]\nname = \"single\"\n",
        "[workspace]\nmembers = []\n",
    ] {
        let error = set(content, &Target::Set(Version::new(1, 0, 0))).unwrap_err();
        assert_eq!(
            error.to_string(),
            "No package version found that could be set"
        );
    }
}

#[test]
fn test_dry_run_changes_nothing() {
    let dir = project(WORKSPACE);
    let before = dump(&dir, WORKSPACE_FILES);
    run(&dir, &["package", "2.0.0", "--dry-run"]).unwrap();
    assert_eq!(dump(&dir, WORKSPACE_FILES), before);
}

#[test]
fn test_unknown_package_is_an_error() {
    let dir = project(WORKSPACE);
    let before = dump(&dir, WORKSPACE_FILES);
    let error = run(&dir, &["package", "2.0.0", "-p", "nope"]).unwrap_err();
    assert_eq!(
        error.to_string(),
        "Package 'nope' is not a member of the workspace"
    );
    assert_eq!(dump(&dir, WORKSPACE_FILES), before);
}

#[test]
fn test_missing_manifest_is_an_error() {
    let dir = project(&[]);
    let error = run(&dir, &["package", "2.0.0"]).unwrap_err();
    assert!(
        error.to_string().starts_with("Could not find "),
        "unexpected error: {error}"
    );
}

#[test]
fn test_legacy_flags() {
    let dir = project(WORKSPACE);
    let path = dir.path().to_str().unwrap();
    let cli = <crate::cli::Cli as clap::Parser>::try_parse_from([
        "cargo-verset",
        "package",
        "--ver",
        "3.0.0",
        "--path",
        path,
    ])
    .unwrap();
    cli.run().unwrap();
    assert!(
        dump(&dir, &["Cargo.toml"]).contains(r#"version = "3.0.0" # shared version"#),
        "workspace version was not set"
    );
}

#[test]
fn test_version_or_bump_is_required() {
    let dir = project(WORKSPACE);
    assert!(run(&dir, &["package"]).is_err(), "no version given");
    assert!(
        run(&dir, &["package", "1.0.0", "--bump", "major"]).is_err(),
        "version and bump given"
    );
}
