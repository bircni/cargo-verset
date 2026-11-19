use crate::{
    manifest::{Change, DepEdit, DepKind, Manifest, dep_package_name, set_dep_version},
    workspace::Selection,
};

/// A request to change the version requirement of a dependency.
#[derive(Debug)]
pub struct Request<'a> {
    /// Package name of the dependency
    pub name: &'a str,
    /// Version requirement to set
    pub req: &'a str,
    /// Registry to set alongside the version
    pub registry: Option<&'a str>,
    /// Only edit these kinds of dependency tables, all if empty
    pub kinds: &'a [DepKind],
}

/// Sets the version requirement of a dependency everywhere it is declared.
///
/// `[workspace.dependencies]` is only edited when neither members nor kinds are filtered, entries
/// inheriting from it via `workspace = true` are never touched.
pub fn set_requirement(
    manifests: &mut [Manifest],
    selection: &Selection,
    request: &Request<'_>,
) -> anyhow::Result<Vec<Change>> {
    let mut changes = Vec::new();
    let mut inherited = 0_usize;
    let whole_workspace = !selection.is_filtered() && request.kinds.is_empty();

    for manifest in manifests.iter_mut() {
        let path = manifest.path().to_path_buf();
        let member_selected = manifest
            .package_name()
            .is_some_and(|name| selection.selects(name));
        manifest.visit_dep_tables(|table_name, kind, table| {
            let wanted = kind.map_or(whole_workspace, |kind| {
                member_selected && (request.kinds.is_empty() || request.kinds.contains(&kind))
            });
            if !wanted {
                return;
            }
            for (key, item) in table.iter_mut() {
                if dep_package_name(key.get(), item) != request.name {
                    continue;
                }
                match set_dep_version(item, request.req, request.registry, true) {
                    DepEdit::Changed { old } => changes.push(Change {
                        manifest: path.clone(),
                        key: format!("{table_name}.{}.version", key.get()),
                        old,
                        new: request.req.to_owned(),
                    }),
                    DepEdit::Inherited => inherited += 1,
                    DepEdit::NoVersion => {}
                }
            }
        });
    }

    if changes.is_empty() {
        let name = request.name;
        if inherited > 0 {
            anyhow::bail!(
                "Dependency '{name}' is only inherited from [workspace.dependencies] here, \
                 run without --package, --exclude and --kind to change it there"
            );
        }
        anyhow::bail!("Dependency '{name}' not found");
    }
    Ok(changes)
}
