use clap::{Arg, ArgAction, ArgMatches, Command};
use std::path::PathBuf;

use core_space::CargoResult;
use core_space::config::PackageItems;
use core_space::config::Workspace;
use core_space::context::Context;
use core_space::context::MANIFEST_FILENAME;
use core_space::context::Modifier;
use core_space::context::Space;
use core_space::context::Target;
use core_space::context::Writer;
use core_space::manifest::InheritMode;
use core_space::manifest::Manifest;
use core_space::manifest::TablePath;

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
                Modifier::add_items_at(
                    TablePath::new().push("workspace"),
                    space_item,
                    InheritMode::None,
                ),
                Target::Space,
            )?;

            ctx
        } else {
            Context::discover()?
        };

        let found_pkg_id = ctx.get_space().get_root_pkg_id().is_some();

        if found_pkg_id {
            let space_manifest = ctx.get_space_mut().try_get_manifest()?;

            let table_path = TablePath::new().push("package");

            let pkg_id_items = space_manifest.get_items_at(&table_path).cloned();

            let pkg_id_dependencies = space_manifest
                .get_items_at(&TablePath::new().push("dependencies"))
                .cloned();

            let pkg_name = ctx
                .get_space()
                .get_root_pkg_id()
                .unwrap()
                .get_name()
                .to_owned();

            let space_item = workspace.to_toml();

            ctx.add_modifier(
                Modifier::add_items_at(
                    TablePath::new().push("workspace"),
                    space_item,
                    InheritMode::None,
                ),
                Target::space(),
            )?;

            if let Some(pkg_items) = pkg_id_items {
                let package_items = PackageItems::from_toml(&pkg_items);

                ctx.add_modifier(
                    Modifier::add_items_at(
                        TablePath::new().push("workspace").push("package"),
                        package_items.to_toml(),
                        InheritMode::None,
                    ),
                    Target::space(),
                )?;

                ctx.add_modifier(
                    Modifier::update_items_at(
                        TablePath::new().push("package"),
                        package_items.to_toml(),
                        InheritMode::Full,
                    ),
                    Target::space(),
                )?;

                ctx.add_modifier(
                    Modifier::update_key(
                        TablePath::new().push("package"),
                        "name",
                        pkg_name.into(),
                        InheritMode::None,
                    ),
                    Target::space(),
                )?;
            }

            if let Some(pkg_dependencies) = pkg_id_dependencies {
                ctx.add_modifier(
                    Modifier::add_items_at(
                        TablePath::new().push("workspace").push("dependencies"),
                        pkg_dependencies.clone(),
                        InheritMode::None,
                    ),
                    Target::space(),
                )?;

                ctx.add_modifier(
                    Modifier::update_items_at(
                        TablePath::new().push("dependencies"),
                        pkg_dependencies,
                        InheritMode::Full,
                    ),
                    Target::space(),
                )?;
            }
        }

        if self.non_virtual && !found_pkg_id {
            let pkg_items = if !space_manifest_path.exists() {
                package_items.to_toml()
            } else {
                let space_manifest = ctx.get_space_mut().try_get_manifest()?;
                let table_path = TablePath::new().push("workspace").push("package");

                if let Some(items) = space_manifest.get_items_at(&table_path).cloned() {
                    items
                } else {
                    package_items.to_toml()
                }
            };

            build_non_virtual_src(&path)?;

            ctx.add_modifier(
                Modifier::add_key(
                    TablePath::new().push("package"),
                    "name",
                    name.into(),
                    InheritMode::None,
                ),
                Target::space(),
            )?;

            ctx.add_modifier(
                Modifier::add_items_at(
                    TablePath::new().push("package"),
                    pkg_items,
                    InheritMode::Full,
                ),
                Target::Space,
            )?;
        }

        ctx.apply()?;
        Writer::write(&ctx)?;

        version_control(&path, self.non_vcs)?;

        Ok(())
    }
}

pub fn cli_init() -> Command {
    Command::new("init")
        .about("Initialize the Cargo Workspace at current directory")
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
    use std::sync::Mutex;
    use tempfile::tempdir;

    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn with_current_dir<T>(path: &std::path::Path, f: impl FnOnce() -> T) -> T {
        let _guard = TEST_MUTEX.lock().unwrap();

        let old_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(path).unwrap();

        let result = f();

        std::env::set_current_dir(old_dir).unwrap();

        result
    }

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

        let result = with_current_dir(dir.path(), || {
            let opts = InitCmd {
                path: None,
                non_vcs: true,
                non_virtual: false,
            };

            opts.run()
        });

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

        let result = with_current_dir(root.path(), || {
            let cmd = InitCmd {
                path: Some(PathBuf::from("workspace")),
                non_vcs: true,
                non_virtual: false,
            };

            cmd.run()
        });

        result.unwrap();

        let manifest_path = path.join(MANIFEST_FILENAME);
        let manifest = fs::read_to_string(&manifest_path).unwrap();

        println!("root: {}", root.path().display());
        println!("path: {}", path.display());
        println!("manifest:\n{manifest}");

        assert!(manifest_path.exists());
    }

    #[test]
    fn init_existing_package_directory() {
        let dir = tempdir().unwrap();

        let manifest_path = dir.path().join(MANIFEST_FILENAME);
        let src_dir = dir.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();

        fs::write(src_dir.join("main.rs"), "fn main() {}\n").unwrap();

        fs::write(
            &manifest_path,
            r#"[package]
    name = "test-pkg"
    version = "0.1.0"
    edition = "2021"

    [dependencies]
    semver = { version = "1.0", features = ["serde"] }
    "#,
        )
        .unwrap();

        let (result, ctx) = with_current_dir(dir.path(), || {
            let result = InitCmd {
                path: None,
                non_vcs: true,
                non_virtual: true,
            }
            .run();

            let ctx = Context::discover();

            (result, ctx)
        });

        println!("result:\n{result:#?}");
        println!("Context:\n{ctx:#?}");

        result.unwrap();

        let manifest = fs::read_to_string(&manifest_path).unwrap();

        println!("manifest:\n{manifest}");

        assert!(manifest.contains("[workspace]"));
        assert!(manifest.contains("resolver = \"3\""));
        assert!(manifest.contains("[workspace.package]"));
        assert!(manifest.contains("[workspace.dependencies]"));
        assert!(manifest.contains("[package]"));
        assert!(manifest.contains("[dependencies]"));
        assert!(manifest.contains("semver.workspace = true"));
        assert!(manifest.contains("name = \"test-pkg\""));
    }
}
