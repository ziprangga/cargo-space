use core_space::CargoResult;
use core_space::Context;
use core_space::IndexSet;
use core_space::bail_out;
use core_space::config::Dependency;
use core_space::config::GitSource;
use core_space::config::PathSource;
use core_space::config::RegistrySource;
use core_space::config::RulesInherit;
use core_space::config::Source;
use core_space::error;
use core_space::manifest::Item;

use core_space::compatible_version;
use core_space::latest_version;
use core_space::registry_url;

use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct DepReq {
    name: String,
    version: Option<String>,
}

impl DepReq {
    pub fn resolve(name_req: &str) -> CargoResult<Self> {
        let (name, version) = name_req
            .split_once('@')
            .map(|(name, version)| (name, Some(version)))
            .unwrap_or((&name_req, None));

        if name.is_empty() {
            bail_out!("crate name cannot be empty");
        }

        if let Some(version) = version {
            if version.is_empty() {
                bail_out!("version requirement cannot be empty");
            }

            semver::VersionReq::parse(version).with_context(|| {
                if let Some(stripped) = version.strip_prefix('v') {
                    format!(
                        "the version provided, `{version}` is not a \
                             valid SemVer requirement\n\n\
                             help: changing the package to `{name}@{stripped}`"
                    )
                } else {
                    format!("invalid version requirement `{version}`")
                }
            })?;
        }

        let dep_req = Self {
            name: name.into(),
            version: version.map(|s| s.to_owned()),
        };

        Ok(dep_req)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct DepOptions {
    pub dep_req: DepReq,

    pub path: Option<PathBuf>,

    pub git: Option<String>,
    pub tag: Option<String>,
    pub branch: Option<String>,
    pub rev: Option<String>,

    pub registry: Option<String>,

    pub features: Option<IndexSet<String>>,
    pub optional: Option<bool>,
}

impl DepOptions {
    pub fn to_dependency(&self, manifest_path: &Path) -> CargoResult<Dependency> {
        let crate_name = self.dep_req.name();
        let version_req = self.dep_req.version();

        let resolve_version = get_version(
            crate_name,
            version_req,
            manifest_path,
            self.path.as_deref(),
            self.git.as_deref(),
            self.registry.as_deref(),
        )?;

        let source = if let Some(path) = &self.path {
            let mut path_source = PathSource::new(path);
            if let Some(ver) = resolve_version {
                path_source = path_source.with_version(ver)
            }
            Source::Path(path_source)
        } else if let Some(url) = &self.git {
            let mut git_source = GitSource::new(url);
            if let Some(tag) = &self.tag {
                git_source = git_source.with_tag(tag)
            }

            if let Some(branch) = &self.branch {
                git_source = git_source.with_branch(branch)
            }

            if let Some(rev) = &self.rev {
                git_source = git_source.with_rev(rev)
            }

            Source::Git(git_source)
        } else {
            let ver = resolve_version.ok_or_else(|| error!("version not available"))?;

            let mut registry_source = RegistrySource::new(ver);
            if let Some(reg) = &self.registry {
                registry_source = registry_source.with_custom_registry(reg)
            }
            Source::Registry(registry_source)
        };

        let mut dependency = Dependency::new().with_source(source);

        if let Some(features) = &self.features {
            dependency = dependency.with_features(features.clone());
        }

        if let Some(optional) = self.optional {
            dependency = dependency.with_optional(optional);
        }

        Ok(dependency)
    }
}

pub fn inherit_split(
    private: RulesInherit,
    dependency: &Dependency,
) -> CargoResult<(Option<Item>, Option<Item>)> {
    let (dep_item_space, dep_item_pkg) = dependency.disjoint(&private);

    Ok((
        dep_item_space.map(|dependency| dependency.to_toml()),
        dep_item_pkg.map(|dependency| dependency.to_toml()),
    ))
}

fn get_version(
    crate_name: &str,
    version: Option<&str>,
    manifest_path: &Path,

    path: Option<&Path>,
    git: Option<&str>,
    registry: Option<&str>,
) -> CargoResult<Option<String>> {
    match version {
        Some(version) => {
            let registry = registry_url(manifest_path, registry.as_deref())?;
            let version = compatible_version(crate_name, version, &registry)?;
            Ok(Some(version.to_string()))
        }

        None if path.is_some() || git.is_some() => Ok(None),

        None => {
            let registry = registry_url(manifest_path, registry.as_deref())?;
            let version = latest_version(crate_name, &registry)?;

            Ok(Some(version.to_string()))
        }
    }
}
