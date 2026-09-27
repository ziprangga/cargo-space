use clap::{Arg, ArgAction, ArgMatches, Command};
use std::path::Path;
use std::path::PathBuf;

use core_space::CargoResult;
use core_space::config::Dependency;
use core_space::config::PackageItems;
use core_space::config::Source;
use core_space::context::Context;
use core_space::context::MANIFEST_FILENAME;
use core_space::context::Modifier;
use core_space::context::PkgId;
use core_space::context::Target;
use core_space::context::Writer;
use core_space::manifest::InheritMode;
use core_space::manifest::Manifest;
use core_space::manifest::TablePath;
use core_space::manifest::into_inline_table_item;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NewPackageKind {
    Bin,
    Lib,
}

impl NewPackageKind {
    fn is_bin(self) -> bool {
        self == NewPackageKind::Bin
    }
}

impl std::fmt::Display for NewPackageKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            NewPackageKind::Bin => "binary (application)",
            NewPackageKind::Lib => "library",
        }
        .fmt(f)
    }
}

#[derive(Debug)]
pub struct CreatePkgCmd {
    path: PathBuf,
    name: Option<String>,
    kind: Option<NewPackageKind>,
    edition: Option<String>,
    registry: Option<String>,

    dep: bool,
    non_inherit: bool,
}

impl CreatePkgCmd {
    pub fn run(&self) -> CargoResult<()> {
        let path = &self.path;

        if path.exists() {
            anyhow::bail!("destination `{}` already exists", path.display());
        }

        let name = match &self.name {
            Some(name) => name.clone(),
            None => match path.file_name().and_then(|name| name.to_str()) {
                Some(name) => name.to_string(),
                None => {
                    anyhow::bail!(
                        "cannot auto-detect package name from path {:?}; use --name to override",
                        path.as_os_str()
                    );
                }
            },
        };

        let kind = self.kind.unwrap_or(NewPackageKind::Bin);
        let edition = self.edition.as_deref().unwrap_or("2024");
        let registry = self
            .registry
            .as_ref()
            .map(|registry| vec![registry.clone()]);

        std::fs::create_dir_all(path)?;

        let mut ctx = Context::discover()?;

        let pkg_manifest_path = path.join(MANIFEST_FILENAME);
        let pkg_manifest = Manifest::new(pkg_manifest_path.clone());
        let pkg_id = PkgId::new()
            .with_name(name.clone())
            .with_path(&path)
            .with_manifest_path(pkg_manifest_path)
            .with_manifest(pkg_manifest);

        let pkg_name = pkg_id.get_name().to_owned();

        ctx.get_space_mut().add_member(pkg_id);

        ctx.add_modifier(
            Modifier::add_key(
                TablePath::new().push("package"),
                "name",
                pkg_name.clone().into(),
                InheritMode::None,
            ),
            Target::pkg(pkg_name.clone()),
        )?;

        if self.non_inherit {
            let items = PackageItems::new()
                .with_version("0.1.0")
                .with_edition(edition)
                .with_publish(registry);
            let pkg_items = items.to_toml();

            ctx.add_modifier(
                Modifier::add_items_at(
                    TablePath::new().push("package"),
                    pkg_items,
                    InheritMode::None,
                ),
                Target::pkg(pkg_name.clone()),
            )?
        } else {
            let space_manifest = ctx.get_space_mut().try_get_manifest()?;
            let table_path = TablePath::new().push("workspace").push("package");
            let pkg_items = space_manifest.get_items_at(&table_path)?.clone();

            ctx.add_modifier(
                Modifier::add_items_at(
                    TablePath::new().push("package"),
                    pkg_items,
                    InheritMode::Full,
                ),
                Target::pkg(pkg_name.clone()),
            )?;
        }

        let root_path = ctx.get_space().get_root_path();
        let member_path = path
            .strip_prefix(&root_path)
            .ok()
            .map(Path::to_path_buf)
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned())
            .expect("package must be inside workspace");

        if self.dep {
            let dependency = Dependency::new()
                .with_name(pkg_name.clone())
                .with_source(Source::path(&member_path));

            let item = dependency.to_toml();

            let item_inline = into_inline_table_item(item)?;

            ctx.add_modifier(
                Modifier::add_key(
                    TablePath::new().push("workspace").push("dependencies"),
                    pkg_name.clone(),
                    item_inline,
                    InheritMode::None,
                ),
                Target::space(),
            )?;
        }

        ctx.add_modifier(
            Modifier::add_value_to_array(
                TablePath::new().push("workspace"),
                "members",
                member_path.into(),
            ),
            Target::space(),
        )?;

        ctx.apply()?;
        Writer::write(&ctx)?;

        create_source(path, kind)?;

        Ok(())
    }
}

fn create_source(path: &Path, kind: NewPackageKind) -> CargoResult<()> {
    let src_path = path.join("src");
    std::fs::create_dir_all(&src_path)?;

    if kind.is_bin() {
        std::fs::write(
            src_path.join("main.rs"),
            "fn main() {\n    println!(\"Hello, world!\");\n}\n",
        )?;
    } else {
        std::fs::write(
            src_path.join("lib.rs"),
            "pub fn add(left: u64, right: u64) -> u64 {\n left + right\n}\n\n#[cfg(test)]\nmod tests {\n use super::*;\n\n #[test]\n fn it_works() {\n let result = add(2, 2);\n assert_eq!(result, 4);\n }\n}\n",
        )?;
    }

    Ok(())
}

pub fn cli_create() -> Command {
    Command::new("create")
        .about("Create a new package in the workspace")
        .arg(
            Arg::new("path")
                .value_name("PATH")
                .required(true)
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("name")
                .long("name")
                .value_name("NAME")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("lib")
                .long("lib")
                .action(ArgAction::SetTrue)
                .conflicts_with("bin"),
        )
        .arg(
            Arg::new("bin")
                .long("bin")
                .action(ArgAction::SetTrue)
                .conflicts_with("lib"),
        )
        .arg(Arg::new("edition").long("edition").value_name("YEAR"))
        .arg(
            Arg::new("registry")
                .long("registry")
                .value_name("REGISTRY")
                .action(ArgAction::Set),
        )
        .arg(Arg::new("dep").long("dep").action(ArgAction::SetTrue))
        .arg(
            Arg::new("non-inherit")
                .long("non-inherit")
                .action(ArgAction::SetTrue),
        )
}

pub fn exec_create(args: &ArgMatches) -> CargoResult<()> {
    let path_arg = PathBuf::from(args.get_one::<String>("path").expect("path is required"));

    let path = if path_arg.is_absolute() {
        path_arg
    } else {
        std::env::current_dir()?.join(path_arg)
    };

    let name = args.get_one::<String>("name").cloned();

    let kind = if args.get_flag("lib") {
        Some(NewPackageKind::Lib)
    } else {
        Some(NewPackageKind::Bin)
    };

    let edition = args.get_one::<String>("edition").cloned();

    let registry = args.get_one::<String>("registry").cloned();

    let non_inherit = args.get_flag("non-inherit");

    let dep = args.get_flag("dep");

    if !non_inherit && (edition.is_some() || registry.is_some()) {
        anyhow::bail!("the `--edition` option requires `--non-inherit`");
    }

    let cmd = CreatePkgCmd {
        path,
        name,
        kind,
        edition,
        registry,

        dep,
        non_inherit,
    };

    cmd.run()?;

    Ok(())
}

// =========================
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use tempfile::tempdir;

    fn setup_workspace() -> tempfile::TempDir {
        let dir = tempdir().unwrap();

        fs::write(
            dir.path().join("Cargo.toml"),
            r#"
[workspace]
resolver = "3"
members = []

[workspace.package]
version = "0.1.0"
edition = "2024"
"#,
        )
        .unwrap();

        dir
    }

    fn print_workspace_files(workspace: &tempfile::TempDir, label: &str) {
        println!("\n========== {label} ==========");

        let root_manifest = workspace.path().join("Cargo.toml");

        println!("\n--- Cargo.toml ---");
        println!("{}", fs::read_to_string(root_manifest).unwrap());

        if let Ok(entries) = fs::read_dir(workspace.path()) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_dir() {
                    let manifest = path.join("Cargo.toml");

                    if manifest.exists() {
                        println!("\n--- {} ---", manifest.display());
                        println!("{}", fs::read_to_string(manifest).unwrap());

                        let src = path.join("src");

                        if let Ok(source_entries) = fs::read_dir(src) {
                            for source_entry in source_entries.flatten() {
                                let source_path = source_entry.path();

                                if source_path.is_file() {
                                    println!("\n--- {} ---", source_path.display());
                                    println!("{}", fs::read_to_string(source_path).unwrap());
                                }
                            }
                        }
                    }
                }
            }
        }

        println!("\n========== END {label} ==========\n");
    }

    fn run_create(workspace: &tempfile::TempDir, args: &[&str]) -> CargoResult<()> {
        let current_dir = std::env::current_dir()?;
        std::env::set_current_dir(workspace.path())?;

        let result = {
            let matches = cli_create()
                .try_get_matches_from(std::iter::once("create").chain(args.iter().copied()))
                .unwrap();

            exec_create(&matches)
        };

        std::env::set_current_dir(current_dir)?;

        result
    }

    #[test]
    fn create_binary_package() {
        let workspace = setup_workspace();

        print_workspace_files(&workspace, "BEFORE");

        run_create(&workspace, &["app"]).unwrap();

        print_workspace_files(&workspace, "AFTER");

        let manifest = fs::read_to_string(workspace.path().join("app/Cargo.toml")).unwrap();

        assert!(manifest.contains("[package]"));
        assert!(manifest.contains("name = \"app\""));
        assert!(manifest.contains("version = { workspace = true }"));
        assert!(manifest.contains("edition = { workspace = true }"));

        assert!(workspace.path().join("app/src/main.rs").exists());
        assert!(!workspace.path().join("app/src/lib.rs").exists());

        let root_manifest = fs::read_to_string(workspace.path().join("Cargo.toml")).unwrap();

        assert!(root_manifest.contains("members = [\"app\"]"));
    }

    #[test]
    fn create_library_package() {
        let workspace = setup_workspace();

        run_create(&workspace, &["lib", "--lib"]).unwrap();

        assert!(workspace.path().join("lib/src/lib.rs").exists());
        assert!(!workspace.path().join("lib/src/main.rs").exists());

        let manifest = fs::read_to_string(workspace.path().join("lib/Cargo.toml")).unwrap();

        assert!(manifest.contains("[package]"));
        assert!(manifest.contains("name = \"lib\""));
        assert!(manifest.contains("version = { workspace = true }"));
        assert!(manifest.contains("edition = { workspace = true }"));
    }

    #[test]
    fn create_package_with_name() {
        let workspace = setup_workspace();

        run_create(&workspace, &["package-dir", "--name", "my-package"]).unwrap();

        let manifest = fs::read_to_string(workspace.path().join("package-dir/Cargo.toml")).unwrap();

        assert!(manifest.contains("name = \"my-package\""));
    }

    #[test]
    fn create_package_with_non_inherit() {
        let workspace = setup_workspace();

        run_create(&workspace, &["app", "--non-inherit", "--edition", "2021"]).unwrap();

        let manifest = fs::read_to_string(workspace.path().join("app/Cargo.toml")).unwrap();

        assert!(manifest.contains("name = \"app\""));
        assert!(manifest.contains("version = \"0.1.0\""));
        assert!(manifest.contains("edition = \"2021\""));

        assert!(!manifest.contains("version = { workspace = true }"));
        assert!(!manifest.contains("edition = { workspace = true }"));
    }

    #[test]
    fn create_package_with_registry_and_non_inherit() {
        let workspace = setup_workspace();

        run_create(
            &workspace,
            &["app", "--non-inherit", "--registry", "my-registry"],
        )
        .unwrap();

        let manifest = fs::read_to_string(workspace.path().join("app/Cargo.toml")).unwrap();

        assert!(manifest.contains("publish = [\"my-registry\"]"));
    }

    #[test]
    fn create_package_with_dep() {
        let workspace = setup_workspace();

        run_create(&workspace, &["app", "--dep"]).unwrap();

        let root_manifest = fs::read_to_string(workspace.path().join("Cargo.toml")).unwrap();

        assert!(root_manifest.contains("[workspace.dependencies]"));
        assert!(root_manifest.contains("app = { path = \"app\" }"));
    }

    #[test]
    fn create_package_updates_workspace_members() {
        let workspace = setup_workspace();

        print_workspace_files(&workspace, "BEFORE");

        run_create(&workspace, &["app"]).unwrap();

        print_workspace_files(&workspace, "AFTER");

        let root_manifest = fs::read_to_string(workspace.path().join("Cargo.toml")).unwrap();

        assert!(root_manifest.contains("members = [\"app\"]"));
    }

    #[test]
    fn create_package_rejects_existing_path() {
        let workspace = setup_workspace();

        fs::create_dir_all(workspace.path().join("app")).unwrap();

        let result = run_create(&workspace, &["app"]);

        assert!(result.is_err());
    }

    #[test]
    fn create_package_rejects_edition_without_non_inherit() {
        let workspace = setup_workspace();

        let result = run_create(&workspace, &["app", "--edition", "2021"]);

        assert!(result.is_err());
    }

    #[test]
    fn create_package_rejects_registry_without_non_inherit() {
        let workspace = setup_workspace();

        let result = run_create(&workspace, &["app", "--registry", "my-registry"]);

        assert!(result.is_err());
    }

    #[test]
    fn create_package_dep_uses_relative_member_path() {
        let workspace = setup_workspace();

        run_create(&workspace, &["packages/app", "--dep"]).unwrap();

        let root_manifest = fs::read_to_string(workspace.path().join("Cargo.toml")).unwrap();

        assert!(root_manifest.contains("app = { path = \"packages/app\" }"));
    }

    #[test]
    fn create_package_with_default_bin() {
        let workspace = setup_workspace();

        run_create(&workspace, &["app"]).unwrap();

        let source = fs::read_to_string(workspace.path().join("app/src/main.rs")).unwrap();

        assert_eq!(source, "fn main() {\n    println!(\"Hello, world!\");\n}\n");
    }

    #[test]
    fn create_package_with_lib_source() {
        let workspace = setup_workspace();

        run_create(&workspace, &["app", "--lib"]).unwrap();

        let source = fs::read_to_string(workspace.path().join("app/src/lib.rs")).unwrap();

        assert!(source.contains("pub fn add"));
        assert!(source.contains("fn it_works()"));
    }

    #[test]
    fn cli_rejects_bin_and_lib() {
        let result = cli_create().try_get_matches_from(["create", "app", "--bin", "--lib"]);

        assert!(result.is_err());
    }

    #[test]
    fn cli_accepts_non_inherit_edition() {
        let result = cli_create().try_get_matches_from([
            "create",
            "app",
            "--non-inherit",
            "--edition",
            "2021",
        ]);

        assert!(result.is_ok());
    }

    #[test]
    fn new_package_kind_display() {
        assert_eq!(NewPackageKind::Bin.to_string(), "binary (application)");
        assert_eq!(NewPackageKind::Lib.to_string(), "library");
    }

    #[test]
    fn new_package_kind_is_bin() {
        assert!(NewPackageKind::Bin.is_bin());
        assert!(!NewPackageKind::Lib.is_bin());
    }
}
