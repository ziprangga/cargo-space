mod metadata;
mod modifier;
mod target;

pub use metadata::MANIFEST_FILENAME;
pub use metadata::PkgId;
pub use metadata::Space;
pub use modifier::Change;
pub use modifier::Modifier;
pub use target::Target;
pub use target::TargetConfig;

use crate::errors::CargoResult;
use crate::manifest::Manifest;
use crate::utility::IndexMap;
use crate::utility::index_map;

use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct Context {
    space: Space,
    target_configs: Vec<TargetConfig>,
}

impl Context {
    pub fn discover() -> cargo_metadata::Result<Self> {
        let space = Space::discover()?;

        Ok(Self::new().with_space(space))
    }

    pub fn with_new_space_from(mut self, path: impl Into<PathBuf>) -> Self {
        let root_path = path.into();
        let manifest_path = root_path.join(MANIFEST_FILENAME);
        let space = Space::new()
            .with_root_path(root_path)
            .with_root_manifest_path(manifest_path.clone());
        self.space = space;
        self
    }

    pub fn with_new_pkg_id_from(
        mut self,
        name: impl Into<String>,
        path: impl Into<PathBuf>,
    ) -> Self {
        let pkg_path = path.into();
        let name = name.into();
        let manifest_path = pkg_path.join(MANIFEST_FILENAME);
        let pkg_id = PkgId::new()
            .with_name(name)
            .with_path(pkg_path)
            .with_manifest_path(manifest_path);
        self.space.add_member(pkg_id);
        self
    }
}

impl Context {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_space(mut self, space: Space) -> Self {
        self.space = space;
        self
    }

    pub fn with_target_configs(mut self, target_config: TargetConfig) -> Self {
        self.target_configs.push(target_config);
        self
    }

    pub fn get_space(&self) -> &Space {
        &self.space
    }

    pub fn get_pkg_id_by_name(&self, name: &str) -> &PkgId {
        self.space.get_pkg_id(name).expect("package not found")
    }

    pub fn get_target_configs(&self) -> &[TargetConfig] {
        &self.target_configs
    }

    pub fn add_modifier(mut self, modifier: Modifier, target: Target) -> Self {
        let is_virtual_space = self.space.is_virtual_workspace();

        match target {
            Target::Space => {
                let mut target_config = TargetConfig::new().with_target(Target::Space);
                target_config.add_modifier(&modifier);
                self.target_configs.push(target_config);
            }

            Target::Pkg(pkg_name) => {
                let pkg_id = self.get_pkg_id_by_name(&pkg_name);
                let is_root_pkg = self.space.is_root_pkg(&pkg_id);

                if is_virtual_space {
                    let mut pkg = TargetConfig::new().with_target(Target::Pkg(pkg_name));
                    pkg.add_modifier(&modifier);
                    self.target_configs.push(pkg);
                } else {
                    if is_root_pkg {
                        let mut space = TargetConfig::new().with_target(Target::Space);
                        space.add_modifier(&modifier);
                        self.target_configs.push(space);
                    } else {
                        let mut pkg = TargetConfig::new().with_target(Target::Pkg(pkg_name));
                        pkg.add_modifier(&modifier);
                        self.target_configs.push(pkg);
                    }
                }
            }

            Target::None => {}
        }

        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Writer;

impl Writer {
    pub fn write(ctx: &Context) -> CargoResult<()> {
        let mut manifests: IndexMap<PathBuf, Manifest> = index_map();

        for target_config in ctx.get_target_configs() {
            let path = match target_config.get_target() {
                Target::Space => ctx.get_space().get_root_manifest_path().to_path_buf(),

                Target::Pkg(pkg_name) => ctx
                    .get_pkg_id_by_name(pkg_name)
                    .get_manifest_path()
                    .to_path_buf(),

                Target::None => continue,
            };

            if !manifests.contains_key(&path) {
                let manifest = match target_config.get_target() {
                    Target::Space => {
                        if path.exists() {
                            ctx.get_space().load_space_manifest()?
                        } else {
                            Manifest::new(&path)
                        }
                    }
                    Target::Pkg(pkg_name) => {
                        let pkg_id = ctx.get_pkg_id_by_name(pkg_name);

                        if path.exists() {
                            pkg_id.load_pkg_manifest()?
                        } else {
                            Manifest::new(&path)
                        }
                    }
                    Target::None => continue,
                };

                manifests.insert(path.clone(), manifest);
            }

            for modifier in target_config.get_modifiers() {
                modifier.apply(manifests.get_mut(&path).expect("manifest must exist"))?;
            }
        }

        for (_, manifest) in manifests {
            manifest.write()?;
        }

        Ok(())
    }
}
