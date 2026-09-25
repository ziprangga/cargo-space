use clap::{Arg, ArgAction, ArgMatches, Command};
use std::path::PathBuf;

use core_space::CargoResult;
use core_space::config::Workspace;
use core_space::context::Change;
use core_space::context::Context;
use core_space::context::MANIFEST_FILENAME;
use core_space::context::Modifier;
use core_space::context::Space;
use core_space::context::Target;
use core_space::context::Writer;
use core_space::manifest::InheritMode;
use core_space::manifest::Manifest;
use core_space::manifest::TablePath;
use core_space::manifest::into_item;

use crate::new::build_non_virtual_src;
use crate::new::build_pkg_items;
use crate::new::resolver_default;
use crate::new::version_control;

#[derive(Debug)]
pub struct InitCmd {
    path: Option<PathBuf>,
    non_vcs: bool,
    non_virtual: bool,
}

impl InitCmd {
    pub fn run(&self) -> CargoResult<()> {
        let path = self.path.clone().unwrap_or(std::env::current_dir()?);
        let name = path.file_name().unwrap().to_string_lossy().into_owned();

        let space_manifest_path = path.join(MANIFEST_FILENAME);

        let package_items = build_pkg_items(Some("2024".to_string()), None);
        let default_resolver = resolver_default(Some("2024".to_string()))?;
        let workspace = Workspace::new()
            .with_resolver(default_resolver)
            .with_package_items(&package_items);

        let mut ctx = if !space_manifest_path.exists() {
            let space_manifest = Manifest::new(space_manifest_path.clone());

            let space = Space::new()
                .with_root_path(&path)
                .with_root_manifest_path(space_manifest_path.clone())
                .with_manifest(space_manifest);

            let space_item = workspace.to_toml();
            let mut ctx = Context::new().with_space(space);

            ctx.add_modifier(
                Modifier::insert_items_at(
                    Change::new()
                        .with_path(TablePath::new().push("workspace"))
                        .with_item(space_item),
                    InheritMode::None,
                ),
                Target::Space,
            )?;

            ctx
        } else {
            Context::discover()?
        };

        if self.non_virtual {
            let pkg_items = if !space_manifest_path.exists() {
                package_items.to_toml()
            } else {
                let space_manifest = ctx.get_space_mut().try_get_manifest()?;
                let table_path = TablePath::new().push("workspace").push("package");
                space_manifest.get_item_of_table(&table_path)?.clone()
            };

            build_non_virtual_src(&path)?;

            ctx.add_modifier(
                Modifier::add_key(
                    Change::new()
                        .with_path(TablePath::new().push("package"))
                        .with_key("name")
                        .with_item(into_item(name)),
                    InheritMode::None,
                ),
                Target::space(),
            )?;

            ctx.add_modifier(
                Modifier::insert_items_at(
                    Change::new()
                        .with_path(TablePath::new().push("package"))
                        .with_item(pkg_items),
                    InheritMode::Full,
                ),
                Target::Space,
            )?
        }

        ctx.apply()?;
        Writer::write(&ctx)?;

        version_control(&path, self.non_vcs)?;

        Ok(())
    }
}

pub fn cli_init() -> Command {
    Command::new("init")
        .about("Initial the Cargo Workspace at current directory")
        .arg(Arg::new("path").value_name("PATH").action(ArgAction::Set))
        .arg(
            Arg::new("non-vcs")
                .long("non-vcs")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("non-virtual")
                .long("non-virtual")
                .action(ArgAction::SetTrue),
        )
}

pub fn exec_init(args: &ArgMatches) -> CargoResult<()> {
    let path = args.get_one::<String>("path").map(|path| {
        let path = PathBuf::from(path);

        if path.is_absolute() {
            path
        } else {
            std::env::current_dir()
                .expect("failed to get current directory")
                .join(path)
        }
    });

    let non_vcs = args.get_flag("non-vcs");
    let non_virtual = args.get_flag("non-virtual");

    let cmd = InitCmd {
        path,
        non_vcs,
        non_virtual,
    };

    cmd.run()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn init_virtual_workspace_at_path() {
        let dir = tempdir().unwrap();

        let opts = InitCmd {
            path: Some(dir.path().to_path_buf()),
            non_vcs: true,
            non_virtual: false,
        };

        opts.run().unwrap();

        let manifest_path = dir.path().join(MANIFEST_FILENAME);
        let manifest = fs::read_to_string(&manifest_path).unwrap();

        println!("path: {}", dir.path().display());
        println!("manifest:\n{manifest}");

        assert!(manifest_path.exists());
        assert!(manifest.contains("[workspace]"));
        assert!(manifest.contains("resolver = \"3\""));
        assert!(manifest.contains("[workspace.package]"));
        assert!(manifest.contains("edition = \"2024\""));
    }

    #[test]
    fn init_non_virtual_workspace_at_path() {
        let dir = tempdir().unwrap();

        let opts = InitCmd {
            path: Some(dir.path().to_path_buf()),
            non_vcs: true,
            non_virtual: true,
        };

        opts.run().unwrap();

        let manifest_path = dir.path().join(MANIFEST_FILENAME);
        let manifest = fs::read_to_string(&manifest_path).unwrap();

        println!("path: {}", dir.path().display());
        println!("manifest:\n{manifest}");

        assert!(manifest_path.exists());
        assert!(manifest.contains("[workspace]"));
        assert!(manifest.contains("[workspace.package]"));
        assert!(manifest.contains("[package]"));
        assert!(manifest.contains(&format!(
            "name = \"{}\"",
            dir.path().file_name().unwrap().to_string_lossy()
        )));
    }

    #[test]
    fn init_without_path_uses_current_directory() {
        let dir = tempdir().unwrap();

        let old_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let opts = InitCmd {
            path: None,
            non_vcs: true,
            non_virtual: false,
        };

        let result = opts.run();

        std::env::set_current_dir(old_dir).unwrap();

        result.unwrap();

        let manifest_path = dir.path().join(MANIFEST_FILENAME);
        let manifest = fs::read_to_string(&manifest_path).unwrap();

        println!("path: {}", dir.path().display());
        println!("manifest:\n{manifest}");

        assert!(manifest_path.exists());
    }

    #[test]
    fn init_with_relative_path() {
        let root = tempdir().unwrap();
        let path = root.path().join("workspace");
        fs::create_dir_all(&path).unwrap();

        let old_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(root.path()).unwrap();

        let cmd = InitCmd {
            path: Some(PathBuf::from("workspace")),
            non_vcs: true,
            non_virtual: false,
        };

        let result = cmd.run();

        std::env::set_current_dir(old_dir).unwrap();

        result.unwrap();

        let manifest_path = path.join(MANIFEST_FILENAME);
        let manifest = fs::read_to_string(&manifest_path).unwrap();

        println!("root: {}", root.path().display());
        println!("path: {}", path.display());
        println!("manifest:\n{manifest}");

        assert!(manifest_path.exists());
    }
}
