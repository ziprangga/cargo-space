use crate::context::Context;
use crate::context::Target;
use crate::errors::CargoResult;
use crate::utility::IndexSet;
use crate::utility::index_set;

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Writer;

impl Writer {
    pub fn write(ctx: &Context) -> CargoResult<()> {
        let mut paths: IndexSet<PathBuf> = index_set();

        for target_config in ctx.get_target_configs() {
            let path = match target_config.get_target() {
                Target::Space => ctx.get_space().get_root_manifest_path().to_path_buf(),

                Target::Pkg(pkg_name) => {
                    ctx.get_pkg_id(pkg_name)?.get_manifest_path().to_path_buf()
                }

                Target::None => continue,
            };

            paths.insert(path);
        }

        for path in paths {
            if path == ctx.get_space().get_root_manifest_path() {
                let manifest = ctx.get_space().manifest_ref()?;
                manifest.write()?;
            } else {
                if let Some(members) = ctx.get_space().get_members() {
                    for pkg_id in members {
                        if pkg_id.get_manifest_path() == path {
                            let manifest = pkg_id.manifest_ref()?;
                            manifest.write()?;

                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
