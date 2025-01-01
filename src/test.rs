use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use clap::{ColorChoice, Command};
use semver::Version;
use toml::Value;

use super::Cli;

#[test]
fn test_full() -> anyhow::Result<()> {
    let test_dir = "../testdir".to_string();
    if Path::new(&test_dir).exists() {
        fs::remove_dir_all(&test_dir)?;
    }

    fs::create_dir_all(&test_dir)?;

    let status = std::process::Command::new("cargo")
        .arg("init")
        .current_dir(&test_dir)
        .status()?;

    assert!(status.success(), "cargo init did not succeed");

    let cargo_toml = Path::new(&test_dir).join("Cargo.toml");

    assert!(cargo_toml.exists(), "Cargo.toml was not created");

    std::fs::write(
        &cargo_toml,
        r#"
[package]
name = "testdir"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
    )?;

    assert_eq!(
        get_version(&cargo_toml)?,
        "0.1.0",
        "Version in Cargo.toml is not correct"
    );

    let cli = Cli {
        ver: Version::new(0, 3, 1),
        path: Some(PathBuf::from(&test_dir)),
        dry_run: false,
    };

    cli.run()?;

    assert_eq!(
        get_version(&cargo_toml)?,
        "0.3.1",
        "Version in Cargo.toml is not correct"
    );

    fs::remove_dir_all(&test_dir)?;

    Ok(())
}

fn get_version(cargo_toml: &PathBuf) -> anyhow::Result<String> {
    let toml_content = fs::read_to_string(cargo_toml)?;
    let parsed_toml: Value = toml::de::from_str(&toml_content)?;

    if let Some(package) = parsed_toml.get("package") {
        if let Some(version) = package.get("version") {
            Ok(version
                .as_str()
                .context("context")?
                .trim_matches('"')
                .to_string())
        } else {
            anyhow::bail!("Version field not found in Cargo.toml");
        }
    } else {
        anyhow::bail!("Package section not found in Cargo.toml");
    }
}
/// From <https://github.com/EmbarkStudios/cargo-deny/blob/f6e40d8eff6a507977b20588c842c53bc0bfd427/src/cargo-deny/main.rs#L369>
/// Snapshot tests for the CLI commands
fn snapshot_test_cli_command(app: Command, cmd_name: &str) -> anyhow::Result<()> {
    let mut app = app
        .color(ColorChoice::Never)
        .version("0.0.0")
        .long_version("0.0.0");

    let mut buffer = Vec::new();
    app.write_long_help(&mut buffer)?;
    let help_text = std::str::from_utf8(&buffer)?;

    let snapshot = insta::_macro_support::SnapshotValue::FileText {
        name: Some(cmd_name.into()),
        content: help_text,
    };

    if insta::_macro_support::assert_snapshot(
        snapshot,
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")),
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

    for app in app.get_subcommands() {
        if app.get_name() == "help" {
            continue;
        }

        snapshot_test_cli_command(app.clone(), &format!("{cmd_name}-{}", app.get_name()))?;
    }
    Ok(())
}

#[allow(clippy::expect_used)]
#[test]
fn cli_snapshot() {
    use clap::CommandFactory;

    insta::with_settings!({
        snapshot_path => "../test_snapshots",
    }, {
        snapshot_test_cli_command(
            super::Cli::command().name("cargo_verset"),
            "cargo_wash",
        ).expect("Failed to run snapshot test");
    });
}
