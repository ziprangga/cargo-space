use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum Source {
    /// Dependency from a registry
    Registry(RegistrySource),
    /// Dependency from a local path
    Path(PathSource),
    /// Dependency from a git repo
    Git(GitSource),
}

impl Source {
    pub fn registry(version: impl AsRef<str>) -> Self {
        Self::Registry(RegistrySource::new(version))
    }

    pub fn path(path: impl Into<PathBuf>) -> Self {
        Self::Path(PathSource::new(path))
    }

    pub fn git(git: impl Into<String>) -> Self {
        Self::Git(GitSource::new(git))
    }

    pub fn as_registry(&self) -> Option<&RegistrySource> {
        match self {
            Self::Registry(source) => Some(source),
            _ => None,
        }
    }

    pub fn as_registry_mut(&mut self) -> Option<&mut RegistrySource> {
        match self {
            Self::Registry(source) => Some(source),
            _ => None,
        }
    }

    pub fn as_path(&self) -> Option<&PathSource> {
        match self {
            Self::Path(source) => Some(source),
            _ => None,
        }
    }

    pub fn as_path_mut(&mut self) -> Option<&mut PathSource> {
        match self {
            Self::Path(source) => Some(source),
            _ => None,
        }
    }

    pub fn as_git(&self) -> Option<&GitSource> {
        match self {
            Self::Git(source) => Some(source),
            _ => None,
        }
    }

    pub fn as_git_mut(&mut self) -> Option<&mut GitSource> {
        match self {
            Self::Git(source) => Some(source),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
#[non_exhaustive]
pub struct RegistrySource {
    /// Version requirement
    version: String,
    /// The registry URL this dependency is from.
    /// If None, then it comes from the default registry (crates.io).
    custom_registry: Option<String>,
}

impl RegistrySource {
    pub fn new(version: impl AsRef<str>) -> Self {
        let version = version.as_ref().split('+').next().unwrap();
        Self {
            version: version.to_owned(),
            custom_registry: None,
        }
    }

    pub fn with_custom_registry(mut self, registry: impl Into<String>) -> Self {
        self.custom_registry = Some(registry.into());
        self
    }

    pub fn get_version(&self) -> &str {
        &self.version
    }

    pub fn get_version_mut(&mut self) -> &mut String {
        &mut self.version
    }

    pub fn get_custom_registry(&self) -> Option<&str> {
        self.custom_registry.as_deref()
    }

    pub fn get_custom_registry_mut(&mut self) -> &mut Option<String> {
        &mut self.custom_registry
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
#[non_exhaustive]
pub struct PathSource {
    /// Local path
    path: PathBuf,
    /// Version requirement for when published
    version: Option<String>,
}

impl PathSource {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            version: None,
        }
    }

    pub fn with_version(mut self, version: impl AsRef<str>) -> Self {
        let version = version.as_ref().split('+').next().unwrap();
        self.version = Some(version.to_owned());
        self
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }

    pub fn get_path_mut(&mut self) -> &mut PathBuf {
        &mut self.path
    }

    pub fn get_version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    pub fn get_version_mut(&mut self) -> &mut Option<String> {
        &mut self.version
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
#[non_exhaustive]
pub struct GitSource {
    /// Repo URL
    git: String,
    /// Select specific branch
    branch: Option<String>,
    /// Select specific tag
    tag: Option<String>,
    /// Select specific rev
    rev: Option<String>,
    /// Version requirement for when published
    version: Option<String>,
}

impl GitSource {
    pub fn new(git: impl Into<String>) -> Self {
        Self {
            git: git.into(),
            branch: None,
            tag: None,
            rev: None,
            version: None,
        }
    }

    pub fn with_branch(mut self, branch: impl Into<String>) -> Self {
        self.branch = Some(branch.into());
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    pub fn with_rev(mut self, rev: impl Into<String>) -> Self {
        self.rev = Some(rev.into());
        self
    }

    pub fn with_version(mut self, version: impl AsRef<str>) -> Self {
        let version = version.as_ref().split('+').next().unwrap();
        self.version = Some(version.to_owned());
        self
    }

    pub fn get_git(&self) -> &str {
        &self.git
    }

    pub fn get_git_mut(&mut self) -> &mut String {
        &mut self.git
    }

    pub fn get_branch(&self) -> Option<&str> {
        self.branch.as_deref()
    }

    pub fn get_branch_mut(&mut self) -> &mut Option<String> {
        &mut self.branch
    }

    pub fn get_tag(&self) -> Option<&str> {
        self.tag.as_deref()
    }

    pub fn get_tag_mut(&mut self) -> &mut Option<String> {
        &mut self.tag
    }

    pub fn get_rev(&self) -> Option<&str> {
        self.rev.as_deref()
    }

    pub fn get_rev_mut(&mut self) -> &mut Option<String> {
        &mut self.rev
    }

    pub fn get_version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    pub fn get_version_mut(&mut self) -> &mut Option<String> {
        &mut self.version
    }
}

impl<'s> From<&'s Source> for Source {
    fn from(inner: &'s Source) -> Self {
        inner.clone()
    }
}

impl From<RegistrySource> for Source {
    fn from(inner: RegistrySource) -> Self {
        Self::Registry(inner)
    }
}

impl From<PathSource> for Source {
    fn from(inner: PathSource) -> Self {
        Self::Path(inner)
    }
}

impl From<GitSource> for Source {
    fn from(inner: GitSource) -> Self {
        Self::Git(inner)
    }
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Registry(src) => src.fmt(f),
            Self::Path(src) => src.fmt(f),
            Self::Git(src) => src.fmt(f),
        }
    }
}

impl std::fmt::Display for RegistrySource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.version.fmt(f)
    }
}

impl std::fmt::Display for PathSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.path.display().fmt(f)
    }
}

impl std::fmt::Display for GitSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.git)?;
        if let Some(branch) = &self.branch {
            write!(f, "?branch={branch}")?;
        } else if let Some(tag) = &self.tag {
            write!(f, "?tag={tag}")?;
        } else if let Some(rev) = &self.rev {
            write!(f, "?rev={rev}")?;
        }
        Ok(())
    }
}
