#[derive(PartialEq, Eq, Hash, Ord, PartialOrd, Clone, Debug, Copy)]
pub enum DepTableKind {
    Normal,
    Development,
    Build,
}

impl DepTableKind {
    pub fn kind_table(&self) -> &'static str {
        match self {
            DepTableKind::Normal => "dependencies",
            DepTableKind::Development => "dev-dependencies",
            DepTableKind::Build => "build-dependencies",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DepTableSection {
    kind: DepTableKind,
    target: Option<String>,
}

impl DepTableSection {
    pub const KINDS: &'static [Self] = &[
        Self::new().with_kind(DepTableKind::Normal),
        Self::new().with_kind(DepTableKind::Development),
        Self::new().with_kind(DepTableKind::Build),
    ];

    pub const fn new() -> Self {
        Self {
            kind: DepTableKind::Normal,
            target: None,
        }
    }

    /// Choose the type of dependency.
    pub const fn with_kind(mut self, kind: DepTableKind) -> Self {
        self.kind = kind;
        self
    }

    /// Choose the platform for the dependency.
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    /// Type of dependency.
    pub fn get_kind(&self) -> &DepTableKind {
        &self.kind
    }

    /// Platform for the dependency.
    pub fn get_target(&self) -> Option<&str> {
        self.target.as_deref()
    }

    /// Keys to the table.
    pub fn to_table(&self) -> Vec<&str> {
        if let Some(target) = &self.target {
            vec!["target", target, self.kind.kind_table()]
        } else {
            vec![self.kind.kind_table()]
        }
    }
}

impl Default for DepTableSection {
    fn default() -> Self {
        Self::new()
    }
}

impl From<DepTableKind> for DepTableSection {
    fn from(kind: DepTableKind) -> Self {
        Self::new().with_kind(kind)
    }
}
