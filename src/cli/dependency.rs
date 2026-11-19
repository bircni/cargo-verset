use anyhow::Context as _;
use clap::Args;
use semver::VersionReq;

use crate::{
    cli::Scope,
    manifest::DepKind,
    ops::dependency::{Request, set_requirement},
    workspace::Workspace,
};

#[derive(Args)]
pub struct DependencyArgs {
    /// Name of the dependency
    #[arg(required_unless_present = "name_flag")]
    pub name: Option<String>,
    /// Version requirement to set, e.g. `1.2.3`, `~1.2` or `>=1, <2`
    #[arg(required_unless_present = "ver", value_parser = parse_requirement)]
    pub req: Option<String>,
    /// Name of the dependency (kept for compatibility with older releases)
    #[arg(long = "name", short = 'n', hide = true, conflicts_with = "name")]
    pub name_flag: Option<String>,
    /// Version requirement to set (kept for compatibility with older releases)
    #[arg(long, hide = true, conflicts_with = "req", value_parser = parse_requirement)]
    pub ver: Option<String>,
    /// Also set a custom registry for the dependency
    #[arg(long, short)]
    pub registry: Option<String>,
    /// Only change these kinds of dependency tables instead of all
    #[arg(long = "kind", short, value_enum, value_name = "KIND")]
    pub kinds: Vec<DepKind>,
    #[command(flatten)]
    pub scope: Scope,
}

/// Validates a version requirement but keeps it as written by the user.
fn parse_requirement(value: &str) -> Result<String, semver::Error> {
    let value = value.trim();
    VersionReq::parse(value)?;
    Ok(value.to_owned())
}

impl DependencyArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        let name = self
            .name
            .as_deref()
            .or(self.name_flag.as_deref())
            .context("A dependency name is required")?;
        let req = self
            .req
            .as_deref()
            .or(self.ver.as_deref())
            .context("A version requirement is required")?;
        let selection = self.scope.selection();
        let mut workspace = Workspace::load(self.scope.manifest_path.as_deref())?;
        workspace.validate(&selection)?;
        let request = Request {
            name,
            req,
            registry: self.registry.as_deref(),
            kinds: &self.kinds,
        };
        let changes = set_requirement(workspace.manifests_mut(), &selection, &request)
            .context("Failed to set the dependency version")?;
        workspace.finish(&changes, self.scope.dry_run)
    }
}
