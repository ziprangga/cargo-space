pub mod metadata;
mod modifier;
mod write_target;

pub use modifier::Change;
pub use modifier::Modifier;
pub use write_target::TargetToml;
pub use write_target::TargetWriter;

use crate::errors::{CargoResult, error};
use crate::manifest::Manifest;

use metadata::PkgId;
use metadata::Space;

#[derive(Debug, Clone, Default)]
pub struct Context {
    space: Space,
    pkg_id: PkgId,
    target_toml: Vec<TargetToml>,
}

impl Context {
    pub fn discover() -> cargo_metadata::Result<Self> {
        let space = Space::discover()?;

        Ok(Self::new().with_space(space))
    }

    pub fn with_pkg_id_name(mut self, name: &str) -> CargoResult<Self> {
        self.pkg_id = self
            .space
            .get_pkg_id(name)
            .cloned()
            .ok_or_else(|| error!("package `{name}` not found"))?;

        Ok(self)
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

    pub fn with_pkg_id(mut self, pkg_id: PkgId) -> Self {
        self.pkg_id = pkg_id;
        self
    }

    pub fn with_target_toml(mut self, target_toml: TargetToml) -> Self {
        self.target_toml.push(target_toml);
        self
    }

    pub fn get_space(&self) -> &Space {
        &self.space
    }

    pub fn get_pkg_id(&self) -> &PkgId {
        &self.pkg_id
    }

    pub fn get_pkg_id_by_name(&self, name: &str) -> CargoResult<&PkgId> {
        self.space
            .get_pkg_id(name)
            .ok_or_else(|| error!("package `{name}` not found"))
    }

    pub fn get_target_tomls(&self) -> &[TargetToml] {
        &self.target_toml
    }

    pub fn with_modifier(mut self, modifier: Modifier, target: TargetWriter) -> CargoResult<Self> {
        let is_virtual_space = self.space.is_virtual_workspace();

        match target {
            TargetWriter::Space => {
                self.target_toml.push(
                    TargetToml::new()
                        .with_target(TargetWriter::Space)
                        .with_modifier(modifier),
                );
            }

            TargetWriter::Pkg(pkg_name) => {
                let pkg_id = self.get_pkg_id_by_name(&pkg_name)?;
                let is_root_pkg = self.space.is_root_pkg(&pkg_id);

                if is_virtual_space {
                    self.target_toml.push(
                        TargetToml::new()
                            .with_target(TargetWriter::Space)
                            .with_modifier(modifier.clone()),
                    );

                    self.target_toml.push(
                        TargetToml::new()
                            .with_target(TargetWriter::Pkg(pkg_name))
                            .with_modifier(modifier),
                    );
                } else if is_root_pkg {
                    self.target_toml.push(
                        TargetToml::new()
                            .with_target(TargetWriter::Space)
                            .with_modifier(modifier),
                    );
                } else {
                    self.target_toml.push(
                        TargetToml::new()
                            .with_target(TargetWriter::Space)
                            .with_modifier(modifier.clone()),
                    );

                    self.target_toml.push(
                        TargetToml::new()
                            .with_target(TargetWriter::Pkg(pkg_name))
                            .with_modifier(modifier),
                    );
                }
            }

            TargetWriter::None => {}
        }

        Ok(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Writer;

impl Writer {
    pub fn write(ctx: &Context) -> CargoResult<()> {
        for target_toml in ctx.get_target_tomls() {
            let mut manifest = match target_toml.get_target() {
                TargetWriter::Space => {
                    let path = ctx.get_space().get_root_manifest_path();

                    if path.exists() {
                        ctx.get_space().load_space_manifest()?
                    } else {
                        Manifest::new(path)
                    }
                }
                TargetWriter::Pkg(pkg_name) => {
                    let pkg_id = ctx.get_pkg_id_by_name(pkg_name)?;
                    let path = pkg_id.get_manifest_path();

                    if path.exists() {
                        pkg_id.load_pkg_manifest()?
                    } else {
                        Manifest::new(path)
                    }
                }
                TargetWriter::None => continue,
            };

            target_toml.get_modifier().apply(&mut manifest)?;
            manifest.write()?;
        }

        Ok(())
    }
}
