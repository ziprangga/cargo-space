mod metadata;
mod modifier;
mod write_target;

pub use metadata::MANIFEST_FILENAME;
pub use metadata::PkgId;
pub use metadata::Space;
pub use modifier::Change;
pub use modifier::Modifier;
pub use write_target::TargetToml;
pub use write_target::TargetWriter;

use crate::errors::CargoResult;
use crate::manifest::Manifest;

use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct Context {
    space: Space,
    target_toml: Vec<TargetToml>,
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

    pub fn with_target_toml(mut self, target_toml: TargetToml) -> Self {
        self.target_toml.push(target_toml);
        self
    }

    pub fn get_space(&self) -> &Space {
        &self.space
    }

    pub fn get_pkg_id_by_name(&self, name: &str) -> &PkgId {
        self.space.get_pkg_id(name).expect("package not found")
    }

    pub fn get_target_tomls(&self) -> &[TargetToml] {
        &self.target_toml
    }

    pub fn add_modifier(mut self, modifier: Modifier, target: TargetWriter) -> Self {
        let is_virtual_space = self.space.is_virtual_workspace();

        match target {
            TargetWriter::Space => {
                let mut target_toml = TargetToml::new().with_target(TargetWriter::Space);
                target_toml.add_modifier(&modifier);
                self.target_toml.push(target_toml);
            }

            TargetWriter::Pkg(pkg_name) => {
                let pkg_id = self.get_pkg_id_by_name(&pkg_name);
                let is_root_pkg = self.space.is_root_pkg(&pkg_id);

                if is_virtual_space {
                    let mut pkg = TargetToml::new().with_target(TargetWriter::Pkg(pkg_name));
                    pkg.add_modifier(&modifier);
                    self.target_toml.push(pkg);
                } else {
                    if is_root_pkg {
                        let mut space = TargetToml::new().with_target(TargetWriter::Space);
                        space.add_modifier(&modifier);
                        self.target_toml.push(space);
                    } else {
                        let mut pkg = TargetToml::new().with_target(TargetWriter::Pkg(pkg_name));
                        pkg.add_modifier(&modifier);
                        self.target_toml.push(pkg);
                    }
                }
            }

            TargetWriter::None => {}
        }

        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Writer;

impl Writer {
    pub fn write(ctx: &Context) -> CargoResult<()> {
        let mut manifests: Vec<(PathBuf, Manifest)> = Vec::new();

        for target_toml in ctx.get_target_tomls() {
            let path = match target_toml.get_target() {
                TargetWriter::Space => ctx.get_space().get_root_manifest_path().to_path_buf(),

                TargetWriter::Pkg(pkg_name) => ctx
                    .get_pkg_id_by_name(pkg_name)
                    .get_manifest_path()
                    .to_path_buf(),

                TargetWriter::None => continue,
            };

            let index = manifests
                .iter()
                .position(|(manifest_path, _)| *manifest_path == path);

            let index = match index {
                Some(index) => index,
                None => {
                    let manifest = match target_toml.get_target() {
                        TargetWriter::Space => {
                            if path.exists() {
                                ctx.get_space().load_space_manifest()?
                            } else {
                                Manifest::new(&path)
                            }
                        }
                        TargetWriter::Pkg(pkg_name) => {
                            let pkg_id = ctx.get_pkg_id_by_name(pkg_name);

                            if path.exists() {
                                pkg_id.load_pkg_manifest()?
                            } else {
                                Manifest::new(&path)
                            }
                        }
                        TargetWriter::None => continue,
                    };

                    manifests.push((path.clone(), manifest));
                    manifests.len() - 1
                }
            };

            for modifier in target_toml.get_modifiers() {
                modifier.apply(&mut manifests[index].1)?;
            }
        }

        for (_, manifest) in manifests {
            manifest.write()?;
        }

        Ok(())
    }
}
