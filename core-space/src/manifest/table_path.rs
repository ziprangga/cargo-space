use crate::errors::CargoResult;
use std::fmt;

#[derive(Debug, Default, Clone)]
pub struct TablePath(Vec<String>);

impl TablePath {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn push(mut self, header: impl Into<String>) -> Self {
        self.0.push(header.into());
        self
    }

    pub fn as_slice(&self) -> &[String] {
        &self.0
    }
}

impl fmt::Display for TablePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.join("."))
    }
}

// This file contains code originally derived from Cargo and cargo-edit,
// has been modified and adapted for cargo-space.
#[derive(PartialEq, Eq, Hash, Ord, PartialOrd, Clone, Debug, Copy)]
pub enum TableDepKind {
    Normal,
    Development,
    Build,

    Workspace,

    Unknown,
}

impl TableDepKind {
    fn as_str(&self) -> &'static str {
        match self {
            TableDepKind::Normal => "dependencies",
            TableDepKind::Development => "dev-dependencies",
            TableDepKind::Build => "build-dependencies",
            TableDepKind::Workspace => "workspace dependencies",
            TableDepKind::Unknown => "unknown",
        }
    }

    pub fn dep_table(&self) -> &'static str {
        match self {
            TableDepKind::Normal => "dependencies",
            TableDepKind::Development => "dev-dependencies",
            TableDepKind::Build => "build-dependencies",
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for TableDepKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableDepTarget {
    kind: TableDepKind,
    target: Option<String>,
}

impl TableDepTarget {
    pub const KINDS: &'static [Self] = &[
        Self::new().with_kind(TableDepKind::Normal),
        Self::new().with_kind(TableDepKind::Development),
        Self::new().with_kind(TableDepKind::Build),
    ];

    pub const fn new() -> Self {
        Self {
            kind: TableDepKind::Normal,
            target: None,
        }
    }

    /// Choose the type of dependency.
    pub const fn with_kind(mut self, kind: TableDepKind) -> Self {
        self.kind = kind;
        self
    }

    /// Choose the platform for the dependency.
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    /// Type of dependency.
    pub fn get_kind(&self) -> &TableDepKind {
        &self.kind
    }

    /// Platform for the dependency.
    pub fn get_target(&self) -> Option<&str> {
        self.target.as_deref()
    }

    /// Keys to the table.
    pub fn to_table(&self) -> Vec<&str> {
        if let Some(target) = &self.target {
            vec!["target", target, self.kind.dep_table()]
        } else {
            match self.kind {
                TableDepKind::Workspace => vec!["workspace", "dependencies"],
                _ => vec![self.kind.dep_table()],
            }
        }
    }
}

impl Default for TableDepTarget {
    fn default() -> Self {
        Self::new()
    }
}

impl From<TableDepKind> for TableDepTarget {
    fn from(kind: TableDepKind) -> Self {
        Self::new().with_kind(kind)
    }
}

impl From<String> for TableDepTarget {
    fn from(target: String) -> Self {
        Self::new().with_target(target)
    }
}

impl From<&str> for TableDepTarget {
    fn from(target: &str) -> Self {
        Self::new().with_target(target)
    }
}

pub fn build_table_dep(
    dev: bool,
    build: bool,
    target: Option<String>,
) -> CargoResult<(TablePath, TablePath)> {
    let space_dep_table = TableDepTarget::new().with_kind(TableDepKind::Workspace);

    let mut space_path = TablePath::new();

    for part in space_dep_table.to_table() {
        space_path = space_path.push(part);
    }

    let mut pkg_dep_table = TableDepTarget::new();

    if dev {
        pkg_dep_table = pkg_dep_table.with_kind(TableDepKind::Development);
    } else if build {
        pkg_dep_table = pkg_dep_table.with_kind(TableDepKind::Build);
    }
    if let Some(target) = &target {
        pkg_dep_table = pkg_dep_table.with_target(target);
    }

    let mut pkg_path = TablePath::new();

    for part in pkg_dep_table.to_table() {
        pkg_path = pkg_path.push(part);
    }

    Ok((space_path, pkg_path))
}
