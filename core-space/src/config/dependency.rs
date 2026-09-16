use super::GitSource;
use super::PathSource;
use super::RegistrySource;
use super::Source;
use crate::IndexSet;
use crate::manifest::Array;
use crate::manifest::Item;
use crate::manifest::Table;

use std::path::Path;

pub trait RulesInheritExt {
    fn is_pkg_only(&self) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum DataInherit {
    Features,
    Optional,
    Registry,
}

impl DataInherit {
    pub const ALL_PKG_ONLY: [DataInherit; 3] = [
        DataInherit::Features,
        DataInherit::Optional,
        DataInherit::Registry,
    ];

    fn is_enabled(&self, conditions: &[Self]) -> bool {
        conditions.contains(self)
    }
}

impl RulesInheritExt for [DataInherit] {
    fn is_pkg_only(&self) -> bool {
        self.len() == DataInherit::ALL_PKG_ONLY.len()
            && DataInherit::ALL_PKG_ONLY
                .iter()
                .all(|condition| self.contains(condition))
    }
}

impl std::fmt::Display for DataInherit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Features => "features",
            Self::Optional => "optional",
            Self::Registry => "registry",
        }
        .fmt(f)
    }
}

impl std::str::FromStr for DataInherit {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "features" => Ok(Self::Features),
            "optional" => Ok(Self::Optional),
            "registry" => Ok(Self::Registry),
            _ => Err(format!(
                "invalid private mode `{value}`; expected features, optional, or registry"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum RulesInherit {
    Choose(Vec<DataInherit>),
    All,

    #[default]
    None,
}

impl RulesInherit {
    fn conditions(&self) -> Option<&[DataInherit]> {
        match self {
            Self::Choose(conditions) => Some(conditions),
            Self::All => Some(&DataInherit::ALL_PKG_ONLY),
            Self::None => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct Dependency {
    name: String,

    source: Option<Source>,
    optional: Option<bool>,
    features: Option<IndexSet<String>>,
    default_features: Option<bool>,
    rename: Option<String>,
    public: Option<bool>,
}

impl Dependency {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(mut self, name: impl AsRef<str>) -> Self {
        self.name = name.as_ref().into();
        self
    }

    pub fn with_source(mut self, source: impl Into<Source>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn with_optional(mut self, opt: bool) -> Self {
        self.optional = Some(opt);
        self
    }

    pub fn with_features(mut self, features: IndexSet<String>) -> Self {
        self.features = Some(features);
        self
    }

    pub fn with_default_features(mut self, default_features: bool) -> Self {
        self.default_features = Some(default_features);
        self
    }

    pub fn with_rename(mut self, rename: &str) -> Self {
        self.rename = Some(rename.into());
        self
    }

    pub fn with_public(mut self, opt: bool) -> Self {
        self.public = Some(opt);
        self
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_source(&self) -> Option<&Source> {
        self.source.as_ref()
    }

    pub fn get_source_mut(&mut self) -> &mut Option<Source> {
        &mut self.source
    }

    pub fn get_optional(&self) -> Option<bool> {
        self.optional
    }

    pub fn get_optional_mut(&mut self) -> &mut Option<bool> {
        &mut self.optional
    }

    pub fn get_features(&self) -> Option<&IndexSet<String>> {
        self.features.as_ref()
    }

    pub fn get_features_mut(&mut self) -> &mut Option<IndexSet<String>> {
        &mut self.features
    }

    pub fn get_default_features(&self) -> Option<bool> {
        self.default_features
    }

    pub fn get_default_features_mut(&mut self) -> &mut Option<bool> {
        &mut self.default_features
    }

    pub fn get_rename(&self) -> Option<&str> {
        self.rename.as_deref()
    }

    pub fn get_rename_mut(&mut self) -> &mut Option<String> {
        &mut self.rename
    }

    pub fn get_public(&self) -> Option<bool> {
        self.public
    }

    pub fn get_public_mut(&mut self) -> &mut Option<bool> {
        &mut self.public
    }

    pub fn disjoint(&self, rules: &RulesInherit) -> (Option<Self>, Option<Self>) {
        let conditions = match rules.conditions() {
            None => return (None, Some(self.clone())),
            Some(conditions) => conditions,
        };

        let mut package = self.clone();
        let mut workspace = self.clone();

        if DataInherit::Registry.is_enabled(conditions) {
            workspace.source = match &self.source {
                Some(Source::Registry(_)) => None,
                source => source.clone(),
            };
        } else {
            package.source = match &self.source {
                Some(Source::Registry(_)) => None,
                source => source.clone(),
            };
        }

        if DataInherit::Features.is_enabled(conditions) {
            workspace.features = None;
        } else {
            package.features = None;
        }

        if DataInherit::Optional.is_enabled(conditions) {
            workspace.optional = None;
        } else {
            package.optional = None;
        }

        if conditions.is_pkg_only() {
            (Some(package), None)
        } else {
            (Some(package), Some(workspace))
        }
    }
}

impl Dependency {
    pub fn from_toml(root_path: &Path, key: &str, item: &Item) -> Option<Self> {
        if let Some(version) = item.as_str() {
            return Some(
                Self::new()
                    .with_name(key)
                    .with_source(RegistrySource::new(version)),
            );
        }

        let table = item.as_table_like()?;

        let (name, rename) = if let Some(value) = table.get("package") {
            (value.as_str()?.to_owned(), Some(key.to_owned()))
        } else {
            (key.to_owned(), None)
        };

        let source = if let Some(value) = table.get("git") {
            let git = value.as_str()?;
            let mut source = GitSource::new(git);

            if let Some(value) = table.get("branch") {
                source = source.with_branch(value.as_str()?);
            }

            if let Some(value) = table.get("tag") {
                source = source.with_tag(value.as_str()?);
            }

            if let Some(value) = table.get("rev") {
                source = source.with_rev(value.as_str()?);
            }

            if let Some(value) = table.get("version") {
                source = source.with_version(value.as_str()?);
            }

            Source::Git(source)
        } else if let Some(value) = table.get("path") {
            let mut source = PathSource::new(root_path.join(value.as_str()?));

            if let Some(value) = table.get("version") {
                source = source.with_version(value.as_str()?);
            }

            Source::Path(source)
        } else if let Some(value) = table.get("version") {
            let mut source = RegistrySource::new(value.as_str()?);

            if let Some(value) = table.get("registry") {
                source = source.with_custom_registry(value.as_str()?);
            }

            Source::Registry(source)
        } else {
            return None;
        };

        let features = table.get("features").and_then(|value| {
            let array = value.as_array()?;

            Some(
                array
                    .iter()
                    .map(|value| value.as_str().map(String::from))
                    .collect::<Option<IndexSet<String>>>()?,
            )
        });

        let default_features = table.get("default-features").and_then(Item::as_bool);

        let optional = table.get("optional").and_then(Item::as_bool);

        Some(Self {
            name,
            source: Some(source),
            optional,
            features,
            default_features,
            rename,
            public: table.get("public").and_then(Item::as_bool),
        })
    }

    pub fn to_toml(&self) -> Item {
        let has_extra_fields = self.features.is_some()
            || self.default_features.is_some()
            || self.optional.is_some()
            || self.rename.is_some()
            || self.public.is_some();

        if let Some(Source::Registry(source)) = self.source.as_ref() {
            if source.get_custom_registry().is_none() && !has_extra_fields {
                return source.get_version().into();
            }
        }

        let mut table = Table::new();

        match self.source.as_ref() {
            Some(Source::Registry(source)) => {
                table.insert("version", source.get_version().into());

                if let Some(registry) = source.get_custom_registry() {
                    table.insert("registry", registry.into());
                }
            }
            Some(Source::Path(source)) => {
                table.insert("path", source.get_path().display().to_string().into());

                if let Some(version) = source.get_version() {
                    table.insert("version", version.into());
                }
            }
            Some(Source::Git(source)) => {
                table.insert("git", source.get_git().into());

                if let Some(branch) = source.get_branch() {
                    table.insert("branch", branch.into());
                }

                if let Some(tag) = source.get_tag() {
                    table.insert("tag", tag.into());
                }

                if let Some(rev) = source.get_rev() {
                    table.insert("rev", rev.into());
                }

                if let Some(version) = source.get_version() {
                    table.insert("version", version.into());
                }
            }
            None => {}
        }

        if let Some(features) = &self.features {
            let mut array = Array::new();

            for feature in features {
                array.push(feature.as_str());
            }

            table.insert("features", array.into());
        }

        if let Some(default_features) = self.default_features {
            table.insert("default-features", default_features.into());
        }

        if let Some(optional) = self.optional {
            table.insert("optional", optional.into());
        }

        if let Some(rename) = &self.rename {
            table.insert("package", rename.as_str().into());
        }

        if let Some(public) = self.public {
            table.insert("public", public.into());
        }

        table.into()
    }
}
