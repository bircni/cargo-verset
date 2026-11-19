use anyhow::Context as _;
use clap::Args;
use semver::Version;

use crate::{
    cli::Scope,
    ops::package::{Bump, Target, set_version},
    workspace::Workspace,
};

#[derive(Args)]
pub struct PackageArgs {
    /// Version to set
    #[arg(
        value_name = "VERSION",
        required_unless_present_any = ["ver", "bump"],
        conflicts_with_all = ["ver", "bump"]
    )]
    pub new_version: Option<Version>,
    /// Version to set (kept for compatibility with older releases)
    #[arg(long, hide = true, conflicts_with = "bump")]
    pub ver: Option<Version>,
    /// Increment the current version instead of setting an explicit one
    #[arg(long, value_enum, value_name = "LEVEL")]
    pub bump: Option<Bump>,
    #[command(flatten)]
    pub scope: Scope,
}

impl PackageArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        let target = match (self.new_version.as_ref().or(self.ver.as_ref()), self.bump) {
            (Some(version), _) => Target::Set(version.clone()),
            (None, Some(bump)) => Target::Bump(bump),
            (None, None) => anyhow::bail!("Either a version or --bump is required"),
        };
        let selection = self.scope.selection();
        let mut workspace = Workspace::load(self.scope.manifest_path.as_deref())?;
        workspace.validate(&selection)?;
        let changes = set_version(workspace.manifests_mut(), &selection, &target)
            .context("Failed to set the package version")?;
        workspace.finish(&changes, self.scope.dry_run)
    }
}
