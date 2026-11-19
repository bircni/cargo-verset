use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context as _;
use toml_edit::{DocumentMut, InlineTable, Item, TableLike, Value};

/// A single edit that was applied to a manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    /// Manifest the edit belongs to
    pub manifest: PathBuf,
    /// Dotted key that was edited, e.g. `package.version`
    pub key: String,
    /// Previous value, `None` if the key was newly inserted
    pub old: Option<String>,
    /// Value after the edit
    pub new: String,
}

impl Change {
    /// Whether the key already had the requested value.
    pub fn is_noop(&self) -> bool {
        self.old.as_deref() == Some(self.new.as_str())
    }
}

/// The kind of a dependency table of a package.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum DepKind {
    /// `[dependencies]`
    Normal,
    /// `[dev-dependencies]`
    Dev,
    /// `[build-dependencies]`
    Build,
}

impl DepKind {
    const ALL: [Self; 3] = [Self::Normal, Self::Dev, Self::Build];

    const fn table_name(self) -> &'static str {
        match self {
            Self::Normal => "dependencies",
            Self::Dev => "dev-dependencies",
            Self::Build => "build-dependencies",
        }
    }
}

/// State of a `version` key in `[package]`.
#[derive(Debug, PartialEq, Eq)]
pub enum VersionField {
    /// `version = "1.2.3"`
    Literal(String),
    /// `version.workspace = true`
    Inherited,
    /// No usable `version` key
    Missing,
}

/// Result of [`set_dep_version`].
#[derive(Debug, PartialEq, Eq)]
pub enum DepEdit {
    /// The requirement was written
    Changed { old: Option<String> },
    /// The entry is `{ workspace = true }` and must be edited in the workspace root
    Inherited,
    /// The entry has no `version` key and none was added
    NoVersion,
}

/// A `Cargo.toml` that is edited in place, preserving formatting and comments.
pub struct Manifest {
    path: PathBuf,
    original: String,
    doc: DocumentMut,
}

impl Manifest {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        Self::parse(path, &content)
    }

    pub fn parse(path: &Path, content: &str) -> anyhow::Result<Self> {
        let doc = content
            .parse::<DocumentMut>()
            .with_context(|| format!("Failed to parse {}", path.display()))?;
        Ok(Self {
            path: path.to_path_buf(),
            original: content.to_owned(),
            doc,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The manifest content including all edits made so far.
    pub fn content(&self) -> String {
        self.doc.to_string()
    }

    /// Writes the manifest back to disk if it was edited. Returns whether it was written.
    pub fn save(&self) -> anyhow::Result<bool> {
        let content = self.content();
        if content == self.original {
            return Ok(false);
        }
        fs::write(&self.path, content)
            .with_context(|| format!("Failed to write {}", self.path.display()))?;
        Ok(true)
    }

    /// Name of the package, `None` for a virtual workspace manifest.
    pub fn package_name(&self) -> Option<&str> {
        self.doc.get("package")?.get("name")?.as_str()
    }

    /// State of `package.version`.
    pub fn package_version(&self) -> VersionField {
        let Some(item) = self.doc.get("package").and_then(|p| p.get("version")) else {
            return VersionField::Missing;
        };
        match item.as_str() {
            Some(version) => VersionField::Literal(version.to_owned()),
            None if is_inherited(item) => VersionField::Inherited,
            None => VersionField::Missing,
        }
    }

    /// Value of `workspace.package.version`, if set.
    pub fn workspace_version(&self) -> Option<&str> {
        self.doc
            .get("workspace")?
            .get("package")?
            .get("version")?
            .as_str()
    }

    /// Replaces the existing literal `package.version`.
    pub fn set_package_version(&mut self, version: &str) {
        if let Some(value) = self
            .doc
            .get_mut("package")
            .and_then(|p| p.get_mut("version"))
            .and_then(Item::as_value_mut)
        {
            replace_value(value, version);
        }
    }

    /// Replaces the existing `workspace.package.version`.
    pub fn set_workspace_version(&mut self, version: &str) {
        if let Some(value) = self
            .doc
            .get_mut("workspace")
            .and_then(|w| w.get_mut("package"))
            .and_then(|p| p.get_mut("version"))
            .and_then(Item::as_value_mut)
        {
            replace_value(value, version);
        }
    }

    /// Calls `visit` for every dependency table in this manifest.
    ///
    /// The arguments are the dotted name of the table, its kind (`None` for
    /// `[workspace.dependencies]`) and the table itself.
    pub fn visit_dep_tables(
        &mut self,
        mut visit: impl FnMut(&str, Option<DepKind>, &mut dyn TableLike),
    ) {
        if let Some(table) = self
            .doc
            .get_mut("workspace")
            .and_then(|w| w.get_mut("dependencies"))
            .and_then(Item::as_table_like_mut)
        {
            visit("workspace.dependencies", None, table);
        }
        for kind in DepKind::ALL {
            if let Some(table) = self
                .doc
                .get_mut(kind.table_name())
                .and_then(Item::as_table_like_mut)
            {
                visit(kind.table_name(), Some(kind), table);
            }
        }
        if let Some(targets) = self.doc.get_mut("target").and_then(Item::as_table_like_mut) {
            for (target, item) in targets.iter_mut() {
                for kind in DepKind::ALL {
                    if let Some(table) = item
                        .get_mut(kind.table_name())
                        .and_then(Item::as_table_like_mut)
                    {
                        let name = format!("target.{}.{}", target.get(), kind.table_name());
                        visit(&name, Some(kind), table);
                    }
                }
            }
        }
    }
}

fn is_inherited(item: &Item) -> bool {
    item.get("workspace").and_then(Item::as_bool) == Some(true)
}

/// Replaces a value but keeps its surrounding whitespace and comments.
fn replace_value(value: &mut Value, new: impl Into<Value>) {
    let decor = value.decor().clone();
    *value = new.into();
    *value.decor_mut() = decor;
}

/// Sets `key` in `table`, returns whether the key had to be inserted.
fn set_key(table: &mut dyn TableLike, key: &str, new: &str) -> bool {
    if let Some(value) = table.get_mut(key).and_then(Item::as_value_mut) {
        replace_value(value, new);
        return false;
    }
    table.insert(key, toml_edit::value(new));
    true
}

/// Name of the package a dependency entry refers to, honoring `package = "..."` renames.
pub fn dep_package_name<'a>(key: &'a str, item: &'a Item) -> &'a str {
    item.get("package").and_then(Item::as_str).unwrap_or(key)
}

/// Whether a dependency entry points to a local crate via `path`.
pub fn is_path_dep(item: &Item) -> bool {
    item.get("path").is_some()
}

/// The version requirement of a dependency entry, if it has one.
pub fn dep_version(item: &Item) -> Option<&str> {
    item.as_str()
        .or_else(|| item.get("version").and_then(Item::as_str))
}

/// Sets the version requirement (and optionally the registry) of a dependency entry in place.
///
/// With `add_missing`, a `version` key is added to entries that only have e.g. `path` or `git`.
pub fn set_dep_version(
    item: &mut Item,
    req: &str,
    registry: Option<&str>,
    add_missing: bool,
) -> DepEdit {
    if let Some(value) = item.as_value_mut().filter(|v| v.is_str()) {
        let old = value.as_str().map(ToOwned::to_owned);
        if let Some(registry) = registry {
            let mut table = InlineTable::new();
            table.insert("version", req.into());
            table.insert("registry", registry.into());
            replace_value(value, table);
        } else {
            replace_value(value, req);
        }
        return DepEdit::Changed { old };
    }

    if is_inherited(item) {
        return DepEdit::Inherited;
    }
    let old = item
        .get("version")
        .and_then(Item::as_str)
        .map(ToOwned::to_owned);
    if old.is_none() && !add_missing {
        return DepEdit::NoVersion;
    }
    let Some(table) = item.as_table_like_mut() else {
        return DepEdit::NoVersion;
    };
    let mut inserted = set_key(table, "version", req);
    if let Some(registry) = registry {
        inserted |= set_key(table, "registry", registry);
    }
    if inserted && let Some(inline) = item.as_inline_table_mut() {
        // newly inserted keys of an inline table need their spacing normalized
        inline.fmt();
    }
    DepEdit::Changed { old }
}
