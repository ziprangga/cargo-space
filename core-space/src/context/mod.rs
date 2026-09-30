mod metadata;
mod modifier;
mod target;
mod writer;

pub use metadata::DepId;
pub use metadata::MANIFEST_FILENAME;
pub use metadata::PkgId;
pub use metadata::Space;
pub use modifier::Modifier;
pub use target::Target;
pub use target::TargetConfig;
pub use writer::Writer;

use crate::errors::CargoResult;
use crate::errors::error;
use crate::utility::IndexMap;
use crate::utility::index_map;

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

    pub fn get_space_mut(&mut self) -> &mut Space {
        &mut self.space
    }

    pub fn get_pkg_id(&self, name: &str) -> CargoResult<&PkgId> {
        self.space
            .get_pkg_id(name)
            .ok_or_else(|| error!("package not found: {name}"))
    }

    pub fn get_pkg_id_mut(&mut self, name: &str) -> CargoResult<&mut PkgId> {
        self.space
            .get_pkg_id_mut(name)
            .ok_or_else(|| error!("package not found: {name}"))
    }

    pub fn get_target_configs(&self) -> &[TargetConfig] {
        &self.target_configs
    }
}

impl Context {
    pub fn add_modifier(&mut self, modifier: Modifier, target: Target) -> CargoResult<()> {
        let is_virtual_space = self.space.is_virtual_workspace();

        match target {
            Target::Space => {
                let mut target_config = TargetConfig::new().with_target(Target::Space);
                target_config.add_modifier(&modifier);
                self.target_configs.push(target_config);
            }

            Target::Pkg(pkg_name) => {
                let pkg_id = self.get_pkg_id(&pkg_name)?;
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

        Ok(())
    }

    pub fn apply(&mut self) -> CargoResult<()> {
        let mut targets: IndexMap<Target, Vec<Modifier>> = index_map();

        for target_config in &self.target_configs {
            let modifiers = targets
                .entry(target_config.get_target().clone())
                .or_default();

            modifiers.extend(target_config.get_modifiers().iter().cloned());
        }

        for (target, modifiers) in targets {
            match target {
                Target::Space => {
                    let manifest = self.space.try_get_manifest_mut()?;

                    for modifier in modifiers {
                        modifier.apply(manifest)?;
                    }
                }

                Target::Pkg(pkg_name) => {
                    let pkg_id = self.get_pkg_id_mut(&pkg_name)?;
                    let manifest = pkg_id.try_get_manifest_mut()?;

                    for modifier in modifiers {
                        modifier.apply(manifest)?;
                    }
                }

                Target::None => {}
            }
        }

        Ok(())
    }
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
// pub struct Writer;

// impl Writer {
//     pub fn write(ctx: &Context) -> CargoResult<()> {
//         let mut paths: IndexSet<PathBuf> = index_set();

//         for target_config in ctx.get_target_configs() {
//             let path = match target_config.get_target() {
//                 Target::Space => ctx.get_space().get_root_manifest_path().to_path_buf(),

//                 Target::Pkg(pkg_name) => {
//                     ctx.get_pkg_id(pkg_name)?.get_manifest_path().to_path_buf()
//                 }

//                 Target::None => continue,
//             };

//             paths.insert(path);
//         }

//         for path in paths {
//             if path == ctx.get_space().get_root_manifest_path() {
//                 let manifest = ctx.get_space().manifest_ref()?;
//                 manifest.write()?;
//             } else {
//                 if let Some(members) = ctx.get_space().get_members() {
//                     for pkg_id in members {
//                         if pkg_id.get_manifest_path() == path {
//                             let manifest = pkg_id.manifest_ref()?;
//                             manifest.write()?;

//                             break;
//                         }
//                     }
//                 }
//             }
//         }

//         Ok(())
//     }
// }
