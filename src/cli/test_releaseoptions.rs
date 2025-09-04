use std::fs;
use tempfile::TempDir;
use crate::cli::releaseoptions::ReleaseOptions;

#[test]
fn test_get_current_version() {
    let temp_dir = TempDir::new().unwrap();
    let cargo_toml = temp_dir.path().join("Cargo.toml");
    
    fs::write(&cargo_toml, r#"
[package]
name = "test-package"
version = "1.2.3"
edition = "2024"
"#).unwrap();

    let options = ReleaseOptions {
        path: None,
        dry_run: false,
        yes: false,
        publish: false,
        registry: None,
    };

    let version = options.get_current_version(&cargo_toml).unwrap();
    assert_eq!(version.to_string(), "1.2.3");
}

#[test]
fn test_get_current_version_missing_package() {
    let temp_dir = TempDir::new().unwrap();
    let cargo_toml = temp_dir.path().join("Cargo.toml");
    
    fs::write(&cargo_toml, r#"
[workspace]
members = ["crates/*"]
"#).unwrap();

    let options = ReleaseOptions {
        path: None,
        dry_run: false,
        yes: false,
        publish: false,
        registry: None,
    };

    let result = options.get_current_version(&cargo_toml);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Could not find package version"));
}

#[test]
fn test_update_version() {
    let temp_dir = TempDir::new().unwrap();
    let cargo_toml = temp_dir.path().join("Cargo.toml");
    
    fs::write(&cargo_toml, r#"
[package]
name = "test-package"
version = "1.2.3"
edition = "2024"
"#).unwrap();

    let options = ReleaseOptions {
        path: None,
        dry_run: false,
        yes: false,
        publish: false,
        registry: None,
    };

    let new_version = semver::Version::parse("2.0.0").unwrap();
    options.update_version(&cargo_toml, &new_version).unwrap();

    let updated_content = fs::read_to_string(&cargo_toml).unwrap();
    assert!(updated_content.contains(r#"version = "2.0.0""#));
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use git2::Repository;

    fn setup_git_repo(temp_dir: &TempDir) -> Repository {
        let repo = Repository::init(temp_dir.path()).unwrap();
        
        // Configure git user
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@example.com").unwrap();
        
        repo
    }

    fn create_initial_commit(repo: &Repository, temp_dir: &TempDir) {
        let cargo_toml = temp_dir.path().join("Cargo.toml");
        fs::write(&cargo_toml, r#"
[package]
name = "test-package"
version = "0.1.0"
edition = "2024"
"#).unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("Cargo.toml")).unwrap();
        index.write().unwrap();

        let signature = repo.signature().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit",
            &tree,
            &[],
        ).unwrap();
    }

    #[test]
    fn test_release_dry_run() {
        let temp_dir = TempDir::new().unwrap();
        let repo = setup_git_repo(&temp_dir);
        create_initial_commit(&repo, &temp_dir);

        let options = ReleaseOptions {
            path: Some(temp_dir.path().to_path_buf()),
            dry_run: true,
            yes: true,
            publish: false,
            registry: None,
        };

        // This should not fail
        let result = options.run();
        assert!(result.is_ok());

        // Version should still be 0.1.0 since it's a dry run
        let version = options.get_current_version(&temp_dir.path().join("Cargo.toml")).unwrap();
        assert_eq!(version.to_string(), "0.1.0");
    }

    #[test]
    fn test_get_last_release_tag_no_tags() {
        let temp_dir = TempDir::new().unwrap();
        let repo = setup_git_repo(&temp_dir);
        create_initial_commit(&repo, &temp_dir);

        let options = ReleaseOptions {
            path: None,
            dry_run: false,
            yes: false,
            publish: false,
            registry: None,
        };

        let last_tag = options.get_last_release_tag(&repo).unwrap();
        assert!(last_tag.is_none());
    }

    #[test]
    fn test_get_last_release_tag_with_tags() {
        let temp_dir = TempDir::new().unwrap();
        let repo = setup_git_repo(&temp_dir);
        create_initial_commit(&repo, &temp_dir);

        // Create some tags
        let commit = repo.head().unwrap().peel_to_commit().unwrap();
        repo.tag_lightweight("v0.1.0", commit.as_object(), false).unwrap();
        repo.tag_lightweight("v0.2.0", commit.as_object(), false).unwrap();
        repo.tag_lightweight("v0.1.5", commit.as_object(), false).unwrap();
        repo.tag_lightweight("not-a-version", commit.as_object(), false).unwrap();

        let options = ReleaseOptions {
            path: None,
            dry_run: false,
            yes: false,
            publish: false,
            registry: None,
        };

        let last_tag = options.get_last_release_tag(&repo).unwrap();
        assert_eq!(last_tag, Some("v0.2.0".to_string()));
    }
}