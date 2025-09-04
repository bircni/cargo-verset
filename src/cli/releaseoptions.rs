use std::{env, fs, path::PathBuf, process::Command};

use anyhow::{Context as _, Result};
use clap::Parser;
use dialoguer::Confirm;
use git2::Repository;
use semver::Version;
use toml_edit::DocumentMut;

#[derive(Parser)]
#[command(author, version, about)]
pub struct ReleaseOptions {
    /// Path to the directory containing the Cargo.toml file
    #[clap(long, short)]
    pub path: Option<PathBuf>,
    /// Run the program without making any changes
    #[clap(long, short)]
    pub dry_run: bool,
    /// Skip confirmation prompts and automatically proceed
    #[clap(long, short)]
    pub yes: bool,
    /// Automatically publish the crate after creating the release
    #[clap(long)]
    pub publish: bool,
    /// Registry to publish to (if --publish is used)
    #[clap(long, short)]
    pub registry: Option<String>,
}

impl ReleaseOptions {
    /// Main entry point for the release command
    pub fn run(&self) -> Result<()> {
        let workspace_dir = if let Some(path) = self.path.clone() {
            path
        } else {
            env::current_dir()?
        };
        
        let cargo_toml = workspace_dir.join("Cargo.toml");
        if !cargo_toml.exists() {
            anyhow::bail!("Could not find Cargo.toml in the workspace");
        }

        // Initialize git repository
        let repo = Repository::open(&workspace_dir)
            .context("Failed to open git repository. Make sure you're in a git repository")?;

        // Get current version from Cargo.toml
        let current_version = self.get_current_version(&cargo_toml)?;
        log::info!("Current version: {current_version}");

        // Get the last release tag
        let last_release_tag = self.get_last_release_tag(&repo)?;
        log::info!("Last release tag: {last_release_tag:?}");

        // Generate changelog and compute next version
        let (changelog, next_version) = self.generate_changelog_and_version(
            &repo, 
            &last_release_tag,
            &current_version
        )?;

        log::info!("Next version: {next_version}");
        log::info!("Generated changelog:\n{changelog}");

        if self.dry_run {
            log::info!("Dry run: No changes were made");
        } else {
            // Update version in Cargo.toml
            self.update_version(&cargo_toml, &next_version)?;

            // Create release commit
            self.create_release_commit(&repo, &next_version, &changelog)?;

            // Ask for push confirmation
            if self.should_push()? {
                self.push_changes(&repo)?;
            }

            // Optionally publish the crate
            if self.publish && self.should_publish()? {
                self.publish_crate(&workspace_dir)?;
            }
        }

        Ok(())
    }

    /// Get the current version from Cargo.toml
    pub fn get_current_version(&self, cargo_toml: &PathBuf) -> Result<Version> {
        let content = fs::read_to_string(cargo_toml)?;
        let doc = content.parse::<DocumentMut>()?;

        let version_str = doc
            .get("package")
            .and_then(|p| p.get("version"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Could not find package version in Cargo.toml"))?;

        Version::parse(version_str).context("Failed to parse current version")
    }

    /// Get the last release tag from git
    pub fn get_last_release_tag(&self, repo: &Repository) -> Result<Option<String>> {
        let tag_names = repo.tag_names(None)?;
        let mut version_tags = Vec::new();

        for tag_name in tag_names.iter().flatten() {
            // Try to parse as version (with or without 'v' prefix)
            let version_str = tag_name.strip_prefix('v').unwrap_or(tag_name);
            if Version::parse(version_str).is_ok() {
                version_tags.push(tag_name.to_owned());
            }
        }

        // Sort by semantic version and get the latest
        version_tags.sort_by(|a, b| {
            let version_a = Version::parse(a.strip_prefix('v').unwrap_or(a)).unwrap();
            let version_b = Version::parse(b.strip_prefix('v').unwrap_or(b)).unwrap();
            version_a.cmp(&version_b)
        });

        Ok(version_tags.last().cloned())
    }

    /// Generate changelog and compute next version using simple commit analysis
    fn generate_changelog_and_version(
        &self,
        repo: &Repository,
        last_release_tag: &Option<String>,
        current_version: &Version,
    ) -> Result<(String, Version)> {
        // Get commits since last release
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;
        
        if let Some(tag) = last_release_tag {
            // Find the commit for this tag
            if let Ok(tag_ref) = repo.find_reference(&format!("refs/tags/{tag}"))
                && let Ok(tag_commit) = tag_ref.peel_to_commit() {
                    revwalk.hide(tag_commit.id())?;
                }
        }

        let mut commits = Vec::new();
        let mut bump_major = false;
        let mut bump_minor = false;
        let mut bump_patch = false;

        for commit_id in revwalk {
            let commit_id = commit_id?;
            let commit = repo.find_commit(commit_id)?;
            
            if let Some(message) = commit.message() {
                commits.push(format!("- {}", message.trim()));
                
                let message_lower = message.to_lowercase();
                
                // Check for breaking changes
                if message_lower.contains("breaking") || message_lower.contains("!:") {
                    bump_major = true;
                }
                // Check for features
                else if message_lower.starts_with("feat") {
                    bump_minor = true;
                }
                // Check for fixes
                else if message_lower.starts_with("fix") {
                    bump_patch = true;
                }
            }
        }

        // Compute next version
        let mut next_version = current_version.clone();
        
        if bump_major {
            next_version.major += 1;
            next_version.minor = 0;
            next_version.patch = 0;
        } else if bump_minor {
            next_version.minor += 1;
            next_version.patch = 0;
        } else if bump_patch {
            next_version.patch += 1;
        } else {
            // Default to patch bump if no conventional commits found
            next_version.patch += 1;
        }

        // Generate changelog
        let changelog = if commits.is_empty() {
            "No changes since last release".to_owned()
        } else {
            format!("Changes in this release:\n\n{}", commits.join("\n"))
        };

        Ok((changelog, next_version))
    }

    /// Update version in Cargo.toml
    pub fn update_version(&self, cargo_toml: &PathBuf, version: &Version) -> Result<()> {
        let content = fs::read_to_string(cargo_toml)?;
        let mut doc = content.parse::<DocumentMut>()?;

        if let Some(package) = doc.get_mut("package") {
            if let Some(version_item) = package.get_mut("version") {
                *version_item = toml_edit::value(version.to_string());
                fs::write(cargo_toml, doc.to_string())?;
                log::info!("Updated version to {version} in Cargo.toml");
            } else {
                anyhow::bail!("Version key not found in package section");
            }
        } else {
            anyhow::bail!("Package section not found in Cargo.toml");
        }

        Ok(())
    }

    /// Create release commit
    fn create_release_commit(&self, repo: &Repository, version: &Version, changelog: &str) -> Result<()> {
        // Stage Cargo.toml
        let mut index = repo.index()?;
        index.add_path(std::path::Path::new("Cargo.toml"))?;
        index.write()?;

        // Create commit
        let signature = repo.signature()?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        let parent_commit = repo.head()?.peel_to_commit()?;

        let commit_message = format!("release: v{version}\n\n{changelog}");
        
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &commit_message,
            &tree,
            &[&parent_commit],
        )?;

        // Create tag
        let tag_name = format!("v{version}");
        repo.tag_lightweight(&tag_name, repo.head()?.peel_to_commit()?.as_object(), false)?;
        
        log::info!("Created release commit and tag: {tag_name}");
        Ok(())
    }

    /// Ask if user wants to push changes
    fn should_push(&self) -> Result<bool> {
        if self.yes {
            return Ok(true);
        }

        Ok(Confirm::new()
            .with_prompt("Push the release commit and tag to remote?")
            .default(true)
            .interact()?)
    }

    /// Push changes to remote
    fn push_changes(&self, repo: &Repository) -> Result<()> {
        // This is a simplified version - in a real implementation you'd want
        // to handle authentication and push both commits and tags
        let output = Command::new("git")
            .args(["push", "origin", "HEAD"])
            .current_dir(repo.workdir().unwrap())
            .output()?;

        if !output.status.success() {
            anyhow::bail!("Failed to push commits: {}", String::from_utf8_lossy(&output.stderr));
        }

        let output = Command::new("git")
            .args(["push", "origin", "--tags"])
            .current_dir(repo.workdir().unwrap())
            .output()?;

        if !output.status.success() {
            anyhow::bail!("Failed to push tags: {}", String::from_utf8_lossy(&output.stderr));
        }

        log::info!("Successfully pushed release commit and tags");
        Ok(())
    }

    /// Ask if user wants to publish the crate
    fn should_publish(&self) -> Result<bool> {
        if self.yes {
            return Ok(true);
        }

        Ok(Confirm::new()
            .with_prompt("Publish the crate to the registry?")
            .default(false)
            .interact()?)
    }

    /// Publish crate using cargo publish
    fn publish_crate(&self, workspace_dir: &PathBuf) -> Result<()> {
        let mut args = vec!["publish"];
        if let Some(registry) = &self.registry {
            args.extend(&["--registry", registry]);
        }

        let output = Command::new("cargo")
            .args(&args)
            .current_dir(workspace_dir)
            .output()?;

        if !output.status.success() {
            anyhow::bail!("Failed to publish crate: {}", String::from_utf8_lossy(&output.stderr));
        }

        log::info!("Successfully published crate");
        Ok(())
    }
}