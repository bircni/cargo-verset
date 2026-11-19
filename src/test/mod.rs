use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use std::str;

use clap::{ColorChoice, Parser as _};
use insta::_macro_support;
use tempfile::TempDir;

use crate::cli::Cli;
use crate::manifest::Manifest;

mod test_dependency;
mod test_package;

/// Creates a crate or workspace on disk, every package gets an empty lib target.
fn project(files: &[(&str, &str)]) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    for (path, content) in files {
        let path = dir.path().join(path);
        let parent = path.parent().unwrap();
        fs::create_dir_all(parent).unwrap();
        fs::write(&path, content).unwrap();
        if content.contains("[package]") {
            fs::create_dir_all(parent.join("src")).unwrap();
            fs::write(parent.join("src/lib.rs"), "").unwrap();
        }
    }
    dir
}

/// Runs the CLI with the given arguments against the project in `dir`.
fn run(dir: &TempDir, args: &[&str]) -> anyhow::Result<()> {
    let args = std::iter::once("cargo-verset")
        .chain(args.iter().copied())
        .chain(["--manifest-path", dir.path().to_str().unwrap()]);
    Cli::try_parse_from(args)?.run()
}

/// Renders the given files of the project in `dir` for a snapshot.
fn dump(dir: &TempDir, files: &[&str]) -> String {
    let mut out = String::new();
    for file in files {
        let content = fs::read_to_string(dir.path().join(file)).unwrap();
        writeln!(out, "### {file}\n{}", content.trim()).unwrap();
    }
    out
}

/// Parses a single in-memory manifest.
fn manifest(content: &str) -> Vec<Manifest> {
    vec![Manifest::parse(Path::new("Cargo.toml"), content).unwrap()]
}

#[test]
fn test_strip_subcommand() {
    let args = |args: &[&str]| args.iter().map(Into::into).collect::<Vec<_>>();
    assert_eq!(
        crate::strip_subcommand(args(&["cargo-verset", "verset", "package", "-p", "verset"])),
        args(&["cargo-verset", "package", "-p", "verset"]),
        "only the subcommand inserted by cargo is removed"
    );
    assert_eq!(
        crate::strip_subcommand(args(&["cargo-verset", "package", "-p", "verset"])),
        args(&["cargo-verset", "package", "-p", "verset"]),
        "direct invocations are left untouched"
    );
}

/// From <https://github.com/EmbarkStudios/cargo-deny/blob/f6e40d8eff6a507977b20588c842c53bc0bfd427/src/cargo-deny/main.rs#L369>
/// Snapshot tests for the CLI commands
#[expect(clippy::panic, reason = "Snapshot failed")]
fn snapshot_test_cli_command(app: clap::Command, cmd_name: &str) {
    let mut app_cmd = app
        .color(ColorChoice::Never)
        .version("0.0.0")
        .long_version("0.0.0");

    let mut buffer = Vec::new();
    app_cmd.write_long_help(&mut buffer).unwrap();
    let help_text = str::from_utf8(&buffer).unwrap();

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
        panic!("Snapshot failed");
    }

    for cmd in app_cmd.get_subcommands() {
        if cmd.get_name() == "help" {
            continue;
        }

        snapshot_test_cli_command(cmd.clone(), &format!("{cmd_name}-{}", cmd.get_name()));
    }
}

#[test]
fn test_cli_snapshot() {
    use clap::CommandFactory as _;

    insta::with_settings!({
        snapshot_path => "snapshots",
    }, {
        snapshot_test_cli_command(
            super::Cli::command().name("cargo_verset"),
            "cargo_verset",
        );
    });
}
