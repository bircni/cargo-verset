use insta::assert_snapshot;

use super::{dump, manifest, project, run};
use crate::manifest::DepKind;
use crate::ops::dependency::{Request, set_requirement};
use crate::workspace::Selection;

const CRATE: &str = r#"
[package]
name = "single"
version = "0.1.0"

[dependencies]
serde = "1.0.0" # serde comment
clap = { version = "4.5", features = ["derive"] } # clap comment
local = { path = "../local" }
json = { package = "serde_json", version = "1.0.0" }

[dependencies.tokio]
version = "1.0.0" # tokio comment
features = ["full"]

[dev-dependencies]
serde = { version = "1.0.0", features = ["derive"] }

[build-dependencies]
serde = "1.0.0"

[target.'cfg(windows)'.dependencies]
serde = "1.0.0"
"#;

const WORKSPACE: &[(&str, &str)] = &[
    (
        "Cargo.toml",
        r#"
[workspace]
members = ["a", "b"]
resolver = "3"

[workspace.dependencies]
serde = "1.0.0" # shared
"#,
    ),
    (
        "a/Cargo.toml",
        r#"
[package]
name = "a"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { workspace = true }
log = "0.4.0"
"#,
    ),
    (
        "b/Cargo.toml",
        r#"
[package]
name = "b"
version = "0.1.0"
edition = "2024"

[dependencies]
serde.workspace = true

[dev-dependencies]
log = "0.4.0"
"#,
    ),
];

const WORKSPACE_FILES: &[&str] = &["Cargo.toml", "a/Cargo.toml", "b/Cargo.toml"];

fn set(name: &str, req: &str, registry: Option<&str>, kinds: &[DepKind]) -> anyhow::Result<String> {
    let mut manifests = manifest(CRATE);
    let request = Request {
        name,
        req,
        registry,
        kinds,
    };
    set_requirement(&mut manifests, &Selection::default(), &request)?;
    Ok(manifests[0].content())
}

#[test]
fn test_all_tables_are_updated() {
    assert_snapshot!(set("serde", "2.0.0", None, &[]).unwrap(), @r#"

    [package]
    name = "single"
    version = "0.1.0"

    [dependencies]
    serde = "2.0.0" # serde comment
    clap = { version = "4.5", features = ["derive"] } # clap comment
    local = { path = "../local" }
    json = { package = "serde_json", version = "1.0.0" }

    [dependencies.tokio]
    version = "1.0.0" # tokio comment
    features = ["full"]

    [dev-dependencies]
    serde = { version = "2.0.0", features = ["derive"] }

    [build-dependencies]
    serde = "2.0.0"

    [target.'cfg(windows)'.dependencies]
    serde = "2.0.0"
    "#);
}

#[test]
fn test_kind_filter() {
    assert_snapshot!(set("serde", "2.0.0", None, &[DepKind::Dev, DepKind::Build]).unwrap(), @r#"

    [package]
    name = "single"
    version = "0.1.0"

    [dependencies]
    serde = "1.0.0" # serde comment
    clap = { version = "4.5", features = ["derive"] } # clap comment
    local = { path = "../local" }
    json = { package = "serde_json", version = "1.0.0" }

    [dependencies.tokio]
    version = "1.0.0" # tokio comment
    features = ["full"]

    [dev-dependencies]
    serde = { version = "2.0.0", features = ["derive"] }

    [build-dependencies]
    serde = "2.0.0"

    [target.'cfg(windows)'.dependencies]
    serde = "1.0.0"
    "#);
}

#[test]
fn test_inline_table_keeps_other_keys() {
    assert_snapshot!(set("clap", "~4.6", None, &[]).unwrap(), @r#"

    [package]
    name = "single"
    version = "0.1.0"

    [dependencies]
    serde = "1.0.0" # serde comment
    clap = { version = "~4.6", features = ["derive"] } # clap comment
    local = { path = "../local" }
    json = { package = "serde_json", version = "1.0.0" }

    [dependencies.tokio]
    version = "1.0.0" # tokio comment
    features = ["full"]

    [dev-dependencies]
    serde = { version = "1.0.0", features = ["derive"] }

    [build-dependencies]
    serde = "1.0.0"

    [target.'cfg(windows)'.dependencies]
    serde = "1.0.0"
    "#);
}

#[test]
fn test_table_keeps_other_keys() {
    assert_snapshot!(set("tokio", ">=1.40, <2", None, &[]).unwrap(), @r#"

    [package]
    name = "single"
    version = "0.1.0"

    [dependencies]
    serde = "1.0.0" # serde comment
    clap = { version = "4.5", features = ["derive"] } # clap comment
    local = { path = "../local" }
    json = { package = "serde_json", version = "1.0.0" }

    [dependencies.tokio]
    version = ">=1.40, <2" # tokio comment
    features = ["full"]

    [dev-dependencies]
    serde = { version = "1.0.0", features = ["derive"] }

    [build-dependencies]
    serde = "1.0.0"

    [target.'cfg(windows)'.dependencies]
    serde = "1.0.0"
    "#);
}

#[test]
fn test_renamed_dependency_is_matched_by_package_name() {
    assert_snapshot!(set("serde_json", "1.0.140", None, &[]).unwrap(), @r#"

    [package]
    name = "single"
    version = "0.1.0"

    [dependencies]
    serde = "1.0.0" # serde comment
    clap = { version = "4.5", features = ["derive"] } # clap comment
    local = { path = "../local" }
    json = { package = "serde_json", version = "1.0.140" }

    [dependencies.tokio]
    version = "1.0.0" # tokio comment
    features = ["full"]

    [dev-dependencies]
    serde = { version = "1.0.0", features = ["derive"] }

    [build-dependencies]
    serde = "1.0.0"

    [target.'cfg(windows)'.dependencies]
    serde = "1.0.0"
    "#);
}

#[test]
fn test_path_dependency_gets_a_version_and_keeps_its_path() {
    assert_snapshot!(set("local", "0.3.0", None, &[]).unwrap(), @r#"

    [package]
    name = "single"
    version = "0.1.0"

    [dependencies]
    serde = "1.0.0" # serde comment
    clap = { version = "4.5", features = ["derive"] } # clap comment
    local = { path = "../local", version = "0.3.0" }
    json = { package = "serde_json", version = "1.0.0" }

    [dependencies.tokio]
    version = "1.0.0" # tokio comment
    features = ["full"]

    [dev-dependencies]
    serde = { version = "1.0.0", features = ["derive"] }

    [build-dependencies]
    serde = "1.0.0"

    [target.'cfg(windows)'.dependencies]
    serde = "1.0.0"
    "#);
}

#[test]
fn test_registry() {
    assert_snapshot!(set("serde", "2.0.0", Some("internal"), &[]).unwrap(), @r#"

    [package]
    name = "single"
    version = "0.1.0"

    [dependencies]
    serde = { version = "2.0.0", registry = "internal" } # serde comment
    clap = { version = "4.5", features = ["derive"] } # clap comment
    local = { path = "../local" }
    json = { package = "serde_json", version = "1.0.0" }

    [dependencies.tokio]
    version = "1.0.0" # tokio comment
    features = ["full"]

    [dev-dependencies]
    serde = { version = "2.0.0", features = ["derive"], registry = "internal" }

    [build-dependencies]
    serde = { version = "2.0.0", registry = "internal" }

    [target.'cfg(windows)'.dependencies]
    serde = { version = "2.0.0", registry = "internal" }
    "#);
}

#[test]
fn test_unknown_dependency_is_an_error() {
    let error = set("nope", "1.0.0", None, &[]).unwrap_err();
    assert_eq!(error.to_string(), "Dependency 'nope' not found");
    // matching the key of a renamed dependency would edit the wrong package
    let error = set("json", "1.0.0", None, &[]).unwrap_err();
    assert_eq!(error.to_string(), "Dependency 'json' not found");
}

#[test]
fn test_workspace_dependency() {
    let dir = project(WORKSPACE);
    run(&dir, &["dependency", "serde", "1.0.200"]).unwrap();
    assert_snapshot!(dump(&dir, WORKSPACE_FILES), @r#"
    ### Cargo.toml
    [workspace]
    members = ["a", "b"]
    resolver = "3"

    [workspace.dependencies]
    serde = "1.0.200" # shared
    ### a/Cargo.toml
    [package]
    name = "a"
    version = "0.1.0"
    edition = "2024"

    [dependencies]
    serde = { workspace = true }
    log = "0.4.0"
    ### b/Cargo.toml
    [package]
    name = "b"
    version = "0.1.0"
    edition = "2024"

    [dependencies]
    serde.workspace = true

    [dev-dependencies]
    log = "0.4.0"
    "#);
}

#[test]
fn test_member_dependencies() {
    let dir = project(WORKSPACE);
    run(&dir, &["dependency", "log", "0.4.20"]).unwrap();
    assert_snapshot!(dump(&dir, &["a/Cargo.toml", "b/Cargo.toml"]), @r#"
    ### a/Cargo.toml
    [package]
    name = "a"
    version = "0.1.0"
    edition = "2024"

    [dependencies]
    serde = { workspace = true }
    log = "0.4.20"
    ### b/Cargo.toml
    [package]
    name = "b"
    version = "0.1.0"
    edition = "2024"

    [dependencies]
    serde.workspace = true

    [dev-dependencies]
    log = "0.4.20"
    "#);
}

#[test]
fn test_selected_member_only() {
    let dir = project(WORKSPACE);
    run(&dir, &["dependency", "log", "0.4.20", "-p", "b"]).unwrap();
    assert_snapshot!(dump(&dir, &["a/Cargo.toml", "b/Cargo.toml"]), @r#"
    ### a/Cargo.toml
    [package]
    name = "a"
    version = "0.1.0"
    edition = "2024"

    [dependencies]
    serde = { workspace = true }
    log = "0.4.0"
    ### b/Cargo.toml
    [package]
    name = "b"
    version = "0.1.0"
    edition = "2024"

    [dependencies]
    serde.workspace = true

    [dev-dependencies]
    log = "0.4.20"
    "#);
}

#[test]
fn test_inherited_dependency_of_selected_member_is_an_error() {
    let dir = project(WORKSPACE);
    let before = dump(&dir, WORKSPACE_FILES);
    let error = run(&dir, &["dependency", "serde", "1.0.200", "-p", "a"]).unwrap_err();
    assert_snapshot!(format!("{error:#}"), @"Failed to set the dependency version: Dependency 'serde' is only inherited from [workspace.dependencies] here, run without --package, --exclude and --kind to change it there");
    assert_eq!(dump(&dir, WORKSPACE_FILES), before);
}

#[test]
fn test_missing_dependency_is_not_added() {
    let dir = project(WORKSPACE);
    let before = dump(&dir, WORKSPACE_FILES);
    let error = run(&dir, &["dependency", "anyhow", "1.0.0"]).unwrap_err();
    assert_eq!(
        format!("{error:#}"),
        "Failed to set the dependency version: Dependency 'anyhow' not found"
    );
    assert_eq!(dump(&dir, WORKSPACE_FILES), before);
}

#[test]
fn test_dry_run_changes_nothing() {
    let dir = project(WORKSPACE);
    let before = dump(&dir, WORKSPACE_FILES);
    run(&dir, &["dependency", "serde", "1.0.200", "--dry-run"]).unwrap();
    assert_eq!(dump(&dir, WORKSPACE_FILES), before);
}

#[test]
fn test_invalid_requirement_is_rejected() {
    let dir = project(WORKSPACE);
    assert!(
        run(&dir, &["dependency", "serde", "not-a-version"]).is_err(),
        "invalid requirement was accepted"
    );
}

#[test]
fn test_legacy_flags() {
    let dir = project(WORKSPACE);
    run(&dir, &["dependency", "--name", "log", "--ver", "0.4.20"]).unwrap();
    assert!(
        dump(&dir, &["a/Cargo.toml"]).contains(r#"log = "0.4.20""#),
        "dependency was not updated"
    );
}
