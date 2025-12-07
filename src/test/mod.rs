use std::path::Path;
use std::str;

use clap::ColorChoice;
use insta::_macro_support;

mod test_depoptions;
mod test_pkgoptions;

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
