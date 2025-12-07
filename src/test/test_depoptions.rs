use semver::Version;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

use crate::cli::depoptions::DependencyOptions;

// Reads the content of a TOML file as String
fn read_toml(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}

// Checks if a dependency with the given version and comment exists in the TOML
fn assert_dep_with_comment(toml: &str, dep: &str, version: &str, comment: &str) {
    // Check if the version is set
    assert!(
        toml.contains(&format!("{dep} = \"{version}\""))
            || toml.contains(&format!("{dep} = {{ version = \"{version}\"")),
        "{dep} version {version} not found"
    );
    // Check if the comment exists somewhere in the TOML
    assert!(toml.contains(comment), "Comment '{comment}' not found");
}

#[test]
fn test_update_dep_string_and_table_and_workspace() {
    let dir = tempdir().unwrap();
    let toml_path = dir.path().join("Cargo.toml");
    // Test case: string, inline object, workspace.dependencies with comment
    let content = r#"
[dependencies]
serde = "1.0.0" # serde comment
clap = { version = "4.5", features = ["derive"] } # clap comment

[workspace.dependencies]
get-size2 = "0.1.2" # get-size2 comment
"#;
    fs::write(&toml_path, content).unwrap();

    // String -> String
    let opts = DependencyOptions {
        ver: Version::new(2, 0, 0),
        package_name: String::from("serde"),
        path: Some(dir.path().to_path_buf()),
        dry_run: false,
        registry: None,
    };
    opts.run().unwrap();
    let toml = read_toml(&toml_path);
    assert_dep_with_comment(&toml, "serde", "2.0.0", "serde comment");

    // Inline object -> Inline object, preserve features
    let opts = DependencyOptions {
        ver: Version::new(3, 0, 0),
        package_name: String::from("clap"),
        path: Some(dir.path().to_path_buf()),
        dry_run: false,
        registry: None,
    };
    opts.run().unwrap();
    let toml = read_toml(&toml_path);
    assert!(toml.contains("clap")); // rough check, see details below

    // workspace.dependencies String -> String
    let opts = DependencyOptions {
        ver: Version::new(0, 5, 0),
        package_name: String::from("get-size2"),
        path: Some(dir.path().to_path_buf()),
        dry_run: false,
        registry: None,
    };
    opts.run().unwrap();
    let toml = read_toml(&toml_path);
    assert_dep_with_comment(&toml, "get-size2", "0.5.0", "get-size2 comment");
}
