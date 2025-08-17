use crate::cli::pkgoptions::PackageOptions;
use semver::Version;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

// Reads the content of a TOML file as String
fn read_toml(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}

#[test]
fn test_workspace_version_set() {
    let dir = tempdir().unwrap();
    let toml_path = dir.path().join("Cargo.toml");
    let content = r#"
[workspace]
[workspace.package]
name = "myws"
version = "0.1.0"
"#;
    fs::write(&toml_path, content).unwrap();

    let opts = PackageOptions {
        version: Version::new(1, 2, 3),
        path: Some(dir.path().to_path_buf()),
        dry_run: false,
    };
    opts.run().unwrap();
    let toml = read_toml(&toml_path);
    assert!(toml.contains("version = \"1.2.3\""));
}

#[test]
fn test_workspace_no_package_section() {
    let dir = tempdir().unwrap();
    let toml_path = dir.path().join("Cargo.toml");
    let content = r#"
[workspace]
members = ["foo"]
"#;
    fs::write(&toml_path, content).unwrap();

    let opts = PackageOptions {
        version: Version::new(1, 0, 0),
        path: Some(dir.path().to_path_buf()),
        dry_run: false,
    };
    // Should only warn, not panic!
    let _ = opts.run();
    let toml = read_toml(&toml_path);
    assert!(!toml.contains("version = \"1.0.0\""));
}

#[test]
fn test_workspace_no_version_key() {
    let dir = tempdir().unwrap();
    let toml_path = dir.path().join("Cargo.toml");
    let content = r#"
[workspace]
[workspace.package]
name = "myws"
"#;
    fs::write(&toml_path, content).unwrap();

    let opts = PackageOptions {
        version: Version::new(9, 9, 9),
        path: Some(dir.path().to_path_buf()),
        dry_run: false,
    };
    // Should only warn, not panic!
    let _ = opts.run();
    let toml = read_toml(&toml_path);
    assert!(!toml.contains("version = \"9.9.9\""));
}
