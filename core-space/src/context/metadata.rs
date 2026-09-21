use cargo_metadata::MetadataCommand;
use std::path::Path;
use std::path::PathBuf;

use crate::errors::CargoResult;
use crate::errors::error;
use crate::manifest::Manifest;

pub const MANIFEST_FILENAME: &str = "Cargo.toml";

#[derive(Debug, Clone, Default)]
pub struct Space {
    root_path: PathBuf,
    root_manifest_path: PathBuf,
    root_pkg_id: Option<PkgId>,
    members: Option<Vec<PkgId>>,

    manifest: Option<Manifest>,
}

impl Space {
    pub fn discover() -> cargo_metadata::Result<Space> {
        let metadata = MetadataCommand::new().no_deps().exec()?;

        Ok(Space::from_metadata(&metadata))
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_root_path(mut self, value: impl Into<PathBuf>) -> Self {
        self.root_path = value.into();
        self
    }

    pub fn with_root_manifest_path(mut self, value: impl Into<PathBuf>) -> Self {
        self.root_manifest_path = value.into();
        self
    }

    pub fn with_root_pkg_id(mut self, package: Option<PkgId>) -> Self {
        self.root_pkg_id = package;
        self
    }

    pub fn with_members(mut self, members: Vec<PkgId>) -> Self {
        self.members = Some(members);
        self
    }

    pub fn get_root_path(&self) -> &Path {
        &self.root_path
    }

    pub fn get_root_manifest_path(&self) -> &Path {
        &self.root_manifest_path
    }

    pub fn get_root_pkg_id(&self) -> Option<&PkgId> {
        self.root_pkg_id.as_ref()
    }

    pub fn get_members(&self) -> Option<&[PkgId]> {
        self.members.as_deref()
    }

    pub fn get_pkg_id(&self, name: &str) -> Option<&PkgId> {
        if let Some(package) = &self.root_pkg_id {
            if package.get_name() == name {
                return Some(package);
            }
        }

        self.members
            .as_ref()?
            .iter()
            .find(|package| package.get_name() == name)
    }

    pub fn get_pkg_id_mut(&mut self, name: &str) -> Option<&mut PkgId> {
        if let Some(package) = &mut self.root_pkg_id {
            if package.get_name() == name {
                return Some(package);
            }
        }

        self.members
            .as_mut()?
            .iter_mut()
            .find(|package| package.get_name() == name)
    }

    pub fn add_member(&mut self, pkg_id: PkgId) {
        self.members.get_or_insert_with(Vec::new).push(pkg_id);
    }

    pub fn add_members(&mut self, pkg_ids: Vec<PkgId>) {
        self.members.get_or_insert_with(Vec::new).extend(pkg_ids);
    }

    pub fn is_virtual_workspace(&self) -> bool {
        self.root_pkg_id.is_none()
    }

    pub fn is_root_pkg(&self, pkg_id: &PkgId) -> bool {
        self.root_pkg_id.as_ref() == Some(pkg_id)
    }
}

impl Space {
    pub fn from_metadata(metadata: &cargo_metadata::Metadata) -> Self {
        let root_path = PathBuf::from(metadata.workspace_root.as_std_path());
        let root_manifest_path = root_path.join(MANIFEST_FILENAME);

        let root_pkg_id = metadata
            .root_package()
            .map(|package| PkgId::from_metadata(package, &root_path));

        let members = metadata
            .workspace_packages()
            .into_iter()
            .filter(|package| {
                root_pkg_id
                    .as_ref()
                    .is_none_or(|root| package.name.as_str() != root.get_name())
            })
            .map(|package| PkgId::from_metadata(package, &root_path))
            .collect();

        Self {
            root_path,
            root_manifest_path,
            root_pkg_id,
            members: Some(members),
            manifest: None,
        }
    }
}

impl Space {
    pub fn with_manifest(mut self, manifest: Manifest) -> Self {
        self.manifest = Some(manifest);
        self
    }

    pub fn try_get_manifest(&mut self) -> CargoResult<&Manifest> {
        self.try_load_and_cache_manifest()?;
        self.manifest
            .as_ref()
            .ok_or_else(|| error!("manifest not found"))
    }

    pub fn try_get_manifest_mut(&mut self) -> CargoResult<&mut Manifest> {
        self.try_load_and_cache_manifest()?;
        self.manifest
            .as_mut()
            .ok_or_else(|| error!("manifest not found"))
    }

    pub(crate) fn manifest_ref(&self) -> CargoResult<&Manifest> {
        self.manifest
            .as_ref()
            .ok_or_else(|| error!("manifest not found"))
    }

    fn try_load_and_cache_manifest(&mut self) -> CargoResult<()> {
        if self.manifest.is_some() {
            return Ok(());
        }

        if !self.root_manifest_path.exists() {
            return Err(error!(
                "manifest file not found: {}",
                self.root_manifest_path.display()
            ));
        }

        let manifest = Manifest::from_toml_path(&self.root_manifest_path)?;
        self.manifest = Some(manifest);

        Ok(())
    }
}

impl PartialEq for Space {
    fn eq(&self, other: &Self) -> bool {
        self.root_path == other.root_path
            && self.root_manifest_path == other.root_manifest_path
            && self.root_pkg_id == other.root_pkg_id
            && self.members == other.members
    }
}

impl std::hash::Hash for Space {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.root_path.hash(state);
        self.root_manifest_path.hash(state);
        self.root_pkg_id.hash(state);
        self.members.hash(state);
    }
}

#[derive(Debug, Clone, Default)]
pub struct PkgId {
    name: String,
    path: PathBuf,
    manifest_path: PathBuf,
    workspace_path: Option<PathBuf>,

    manifest: Option<Manifest>,
}

impl PkgId {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(mut self, value: impl Into<String>) -> Self {
        self.name = value.into();
        self
    }

    pub fn with_path(mut self, value: impl Into<PathBuf>) -> Self {
        self.path = value.into();
        self
    }

    pub fn with_manifest_path(mut self, value: impl Into<PathBuf>) -> Self {
        self.manifest_path = value.into();
        self
    }

    pub fn with_workspace_path(mut self, value: Option<impl Into<PathBuf>>) -> Self {
        self.workspace_path = value.map(Into::into);
        self
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_path(&self) -> &Path {
        &self.path
    }

    pub fn get_manifest_path(&self) -> &Path {
        &self.manifest_path
    }

    pub fn get_workspace_path(&self) -> Option<&Path> {
        self.workspace_path.as_deref()
    }
}

impl PkgId {
    pub fn from_metadata(package: &cargo_metadata::Package, workspace_root: &Path) -> Self {
        let manifest_path = PathBuf::from(package.manifest_path.as_std_path());
        let path = manifest_path
            .parent()
            .map(PathBuf::from)
            .unwrap_or_default();

        Self {
            name: package.name.to_string(),
            path,
            manifest_path,
            workspace_path: Some(workspace_root.to_path_buf()),
            manifest: None,
        }
    }
}

impl PkgId {
    pub fn with_manifest(mut self, manifest: Manifest) -> Self {
        self.manifest = Some(manifest);
        self
    }

    pub fn try_get_manifest(&mut self) -> CargoResult<&Manifest> {
        self.try_load_and_cache_manifest()?;
        self.manifest
            .as_ref()
            .ok_or_else(|| error!("manifest not found"))
    }

    pub fn try_get_manifest_mut(&mut self) -> CargoResult<&mut Manifest> {
        self.try_load_and_cache_manifest()?;

        self.manifest
            .as_mut()
            .ok_or_else(|| error!("manifest not found"))
    }

    pub(crate) fn manifest_ref(&self) -> CargoResult<&Manifest> {
        self.manifest
            .as_ref()
            .ok_or_else(|| error!("manifest not found"))
    }

    fn try_load_and_cache_manifest(&mut self) -> CargoResult<()> {
        if self.manifest.is_some() {
            return Ok(());
        }

        if !self.manifest_path.exists() {
            return Err(error!(
                "manifest file not found: {}",
                self.manifest_path.display()
            ));
        }

        let manifest = Manifest::from_toml_path(&self.manifest_path)?;
        self.manifest = Some(manifest);

        Ok(())
    }
}

impl PartialEq for PkgId {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.path == other.path
            && self.manifest_path == other.manifest_path
            && self.workspace_path == other.workspace_path
    }
}

impl std::hash::Hash for PkgId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.path.hash(state);
        self.manifest_path.hash(state);
        self.workspace_path.hash(state);
    }
}
