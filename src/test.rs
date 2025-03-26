#![expect(clippy::unwrap_used, clippy::panic, reason = "Testing module")]
use std::{
    fs,
    path::{Path, PathBuf},
    process,
};

use anyhow::Context as _;
use semver::Version;
use toml::{Value, from_str};

use super::Cli;

#[test]
fn test_initialize_logger() {
    super::initialize_logger().unwrap();
}

#[test]
fn test_full() {
    let test_dir = "../testdir".to_owned();
    if Path::new(&test_dir).exists() {
        fs::remove_dir_all(&test_dir).unwrap();
    }

    fs::create_dir_all(&test_dir).unwrap();

    let status = process::Command::new("cargo")
        .arg("init")
        .current_dir(&test_dir)
        .status()
        .unwrap();

    assert!(status.success(), "cargo init did not succeed");

    let cargo_toml = Path::new(&test_dir).join("Cargo.toml");

    assert!(cargo_toml.exists(), "Cargo.toml was not created");

    fs::write(
        &cargo_toml,
        r#"
[package]
name = "testdir"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
    )
    .unwrap();

    assert_eq!(
        get_version(&cargo_toml),
        "0.1.0",
        "Version in Cargo.toml is not correct"
    );

    let cli = Cli {
        ver: Version::new(0, 3, 1),
        path: Some(PathBuf::from(&test_dir)),
        dry_run: false,
    };

    cli.run().unwrap();

    assert_eq!(
        get_version(&cargo_toml),
        "0.3.1",
        "Version in Cargo.toml is not correct"
    );

    fs::remove_dir_all(&test_dir).unwrap();
}

fn get_version(cargo_toml: &PathBuf) -> String {
    let toml_content = fs::read_to_string(cargo_toml).unwrap();
    let parsed_toml: Value = from_str(&toml_content).unwrap();

    parsed_toml.get("package").map_or_else(
        || {
            panic!("Package section not found in Cargo.toml");
        },
        |package| {
            package.get("version").map_or_else(
                || {
                    panic!("Version field not found in Cargo.toml");
                },
                |version| {
                    version
                        .as_str()
                        .context("context")
                        .unwrap()
                        .trim_matches('"')
                        .to_owned()
                },
            )
        },
    )
}
