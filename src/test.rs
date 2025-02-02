#![expect(clippy::unwrap_used, reason = "unwrap is used for testing purposes")]
#![expect(clippy::panic, reason = "panic is used for testing purposes")]
use std::{
    fs,
    path::{Path, PathBuf},
    process, str,
};

use anyhow::Context as _;
use clap::{ColorChoice, Command, CommandFactory as _};
use insta::_macro_support;
use semver::Version;
use toml::{from_str, Value};

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
/// From <https://github.com/EmbarkStudios/cargo-deny/blob/f6e40d8eff6a507977b20588c842c53bc0bfd427/src/cargo-deny/main.rs#L369>
/// Snapshot tests for the CLI commands
fn snapshot_test_cli_command(app: Command, cmd_name: &str) -> anyhow::Result<()> {
    let mut app_ex = app
        .color(ColorChoice::Never)
        .version("0.0.0")
        .long_version("0.0.0");

    let mut buffer = Vec::new();
    app_ex.write_long_help(&mut buffer)?;
    let help_text = str::from_utf8(&buffer)?;

    let snapshot = _macro_support::SnapshotValue::FileText {
        name: Some(cmd_name.into()),
        content: help_text,
    };

    if _macro_support::assert_snapshot(
        snapshot,
        Path::new(env!("CARGO_MANIFEST_DIR")),
        "cli-cmd",
        module_path!(),
        file!(),
        line!(),
        "help_text",
    )
    .is_err()
    {
        anyhow::bail!("Snapshot test failed for command: {}", cmd_name);
    }

    for cmd in app_ex.get_subcommands() {
        if cmd.get_name() == "help" {
            continue;
        }

        snapshot_test_cli_command(cmd.clone(), &format!("{cmd_name}-{}", cmd.get_name()))?;
    }
    Ok(())
}

#[test]
fn cli_snapshot() {
    insta::with_settings!({
        snapshot_path => "../test_snapshots",
    }, {
        snapshot_test_cli_command(
            super::Cli::command().name("cargo_verset"),
            "cargo_wash",
        ).unwrap();
    });
}
