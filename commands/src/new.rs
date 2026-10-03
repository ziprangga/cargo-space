use clap::{Arg, ArgAction, ArgMatches, Command};
use std::path::Path;
use std::path::PathBuf;

use core_space::CargoResult;
use core_space::bail_out;
use core_space::config::Edition;
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
use core_space::vcs::GitRepo;
use core_space::vcs::IgnoreList;

#[derive(Debug)]
pub struct NewCmd {
    path: PathBuf,
    edition: Option<String>,
    registry: Option<String>,

    non_vcs: bool,
    non_virtual: bool,
}

impl NewCmd {
    pub fn run(&self) -> CargoResult<()> {
        let name = self
            .path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();

        if self.path.exists() {
            bail_out!(
                "destination `{}` already exists\n\n\
                     Use `cargo init` to initialize the directory",
                self.path.display()
            );
        }

        std::fs::create_dir_all(&self.path)?;

        let package_items = build_pkg_items(self.edition.clone(), self.registry.clone());
        let default_resolver = resolver_default(self.edition.clone())?;
        let workspace = Workspace::new()
            .with_resolver(default_resolver)
            .with_package_items(&package_items);

        let space_manifest_path = self.path.join(MANIFEST_FILENAME);
        let space_manifest = Manifest::new(space_manifest_path.clone());
        let space = Space::new()
            .with_root_path(&self.path)
            .with_root_manifest_path(space_manifest_path)
            .with_manifest(space_manifest);

        let mut ctx = Context::new().with_space(space);

        let space_item = workspace.to_toml();

        ctx.add_modifier(
            Modifier::add_items_at(
                TablePath::new().push("workspace"),
                space_item,
                InheritMode::None,
            ),
            Target::Space,
        )?;

        if self.non_virtual {
            let pkg_items = package_items.to_toml();
            build_non_virtual_src(&self.path)?;

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

        version_control(&self.path, self.non_vcs)?;

        Ok(())
    }
}

pub fn version_control(dir: &Path, non_vcs: bool) -> CargoResult<()> {
    if non_vcs {
        return Ok(());
    }

    GitRepo::init(dir)?;
    GitRepo::write_gitignore_file(dir, &IgnoreList::default())
}

pub fn build_pkg_items(edition: Option<String>, registry: Option<String>) -> PackageItems {
    PackageItems::new()
        .with_version("0.1.0")
        .with_edition(edition.as_deref().unwrap_or("2024"))
        .with_publish(registry.as_ref().map(|registry| vec![registry.clone()]))
}

pub fn build_non_virtual_src(path: &Path) -> CargoResult<()> {
    let src_path = path.join("src");
    std::fs::create_dir_all(&src_path)?;
    std::fs::write(
        src_path.join("main.rs"),
        "fn main() {\n    println!(\"Hello, world!\");\n}\n",
    )?;

    Ok(())
}

pub fn resolver_default(edition: Option<String>) -> CargoResult<String> {
    let edition = edition.as_deref().unwrap_or("2024").parse::<Edition>()?;
    let value = edition.default_resolver().to_string();

    Ok(value)
}

pub fn cli_new() -> Command {
    Command::new("new")
        .about("Create a new cargo workspace at <path>")
        .arg(
            Arg::new("path")
                .value_name("PATH")
                .required(true)
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("non-vcs")
                .long("non-vcs")
                .action(ArgAction::SetTrue),
        )
        .arg(Arg::new("edition").long("edition").value_name("YEAR"))
        .arg(Arg::new("registry").long("registry").value_name("REGISTRY"))
        .arg(
            Arg::new("non-virtual")
                .long("non-virtual")
                .action(ArgAction::SetTrue),
        )
}

pub fn exec_new(args: &ArgMatches) -> CargoResult<()> {
    let non_vcs = args.get_flag("non-vcs");

    let path = args
        .get_one::<String>("path")
        .map(|p| {
            let new_path = PathBuf::from(p);

            if new_path.is_absolute() {
                new_path
            } else {
                std::env::current_dir()
                    .expect("failed to get current directory")
                    .join(new_path)
            }
        })
        .expect("path is required");

    let edition = args.get_one::<String>("edition").cloned();

    let registry = args.get_one::<String>("registry").cloned();
    let non_virtual = args.get_flag("non-virtual");

    let cmd = NewCmd {
        non_vcs,
        path,
        edition,
        registry,
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
    fn new_virtual_workspace_at_path() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("workspace");

        let opts = NewCmd {
            path,
            edition: None,
            registry: None,
            non_vcs: true,
            non_virtual: false,
        };

        opts.run().unwrap();

        let manifest_path = opts.path.join(MANIFEST_FILENAME);
        let manifest = fs::read_to_string(&manifest_path).unwrap();

        println!("path: {}", opts.path.display());
        println!("manifest:\n{manifest}");

        assert!(manifest_path.exists());
        assert!(manifest.contains("[workspace]"));
        assert!(manifest.contains("resolver = \"3\""));
        assert!(manifest.contains("[workspace.package]"));
        assert!(manifest.contains("version = \"0.1.0\""));
        assert!(manifest.contains("edition = \"2024\""));
        assert!(!manifest.contains("[package]"));
    }

    #[test]
    fn new_non_virtual_workspace_at_path() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("workspace");

        let opts = NewCmd {
            path,
            edition: None,
            registry: None,
            non_vcs: true,
            non_virtual: true,
        };

        opts.run().unwrap();

        let manifest_path = opts.path.join(MANIFEST_FILENAME);
        let manifest = fs::read_to_string(&manifest_path).unwrap();

        println!("path: {}", opts.path.display());
        println!("manifest:\n{manifest}");

        assert!(manifest_path.exists());
        assert!(manifest.contains("[workspace]"));
        assert!(manifest.contains("resolver = \"3\""));
        assert!(manifest.contains("[workspace.package]"));
        assert!(manifest.contains("[package]"));
        assert!(manifest.contains(&format!(
            "name = \"{}\"",
            opts.path.file_name().unwrap().to_string_lossy()
        )));
        assert!(manifest.contains("version = \"0.1.0\""));
        assert!(manifest.contains("edition = \"2024\""));

        assert!(opts.path.join("src").exists());
        assert!(opts.path.join("src/main.rs").exists());
    }

    #[test]
    fn new_with_edition() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("workspace");

        let opts = NewCmd {
            path,
            edition: Some("2021".to_string()),
            registry: None,
            non_vcs: true,
            non_virtual: false,
        };

        opts.run().unwrap();

        let manifest_path = opts.path.join(MANIFEST_FILENAME);
        let manifest = fs::read_to_string(&manifest_path).unwrap();

        println!("path: {}", opts.path.display());
        println!("manifest:\n{manifest}");

        assert!(manifest.contains("[workspace]"));
        assert!(manifest.contains("resolver = \"2\""));
        assert!(manifest.contains("[workspace.package]"));
        assert!(manifest.contains("edition = \"2021\""));
        assert!(!manifest.contains("edition = \"2024\""));
    }

    #[test]
    fn new_with_registry() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("workspace");

        let opts = NewCmd {
            path,
            edition: None,
            registry: Some("my-registry".to_string()),
            non_vcs: true,
            non_virtual: false,
        };

        opts.run().unwrap();

        let manifest_path = opts.path.join(MANIFEST_FILENAME);
        let manifest = fs::read_to_string(&manifest_path).unwrap();

        println!("path: {}", opts.path.display());
        println!("manifest:\n{manifest}");

        assert!(manifest_path.exists());
        assert!(manifest.contains("[workspace]"));
        assert!(manifest.contains("[workspace.package]"));
        assert!(manifest.contains("publish = [\"my-registry\"]"));
    }

    #[test]
    fn new_non_virtual_with_edition_and_registry() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("workspace");

        let opts = NewCmd {
            path,
            edition: Some("2021".to_string()),
            registry: Some("my-registry".to_string()),
            non_vcs: true,
            non_virtual: true,
        };

        opts.run().unwrap();

        let manifest_path = opts.path.join(MANIFEST_FILENAME);
        let manifest = fs::read_to_string(&manifest_path).unwrap();

        println!("path: {}", opts.path.display());
        println!("manifest:\n{manifest}");

        assert!(manifest_path.exists());
        assert!(manifest.contains("[workspace]"));
        assert!(manifest.contains("resolver = \"2\""));
        assert!(manifest.contains("[workspace.package]"));
        assert!(manifest.contains("[package]"));
        assert!(manifest.contains("publish = [\"my-registry\"]"));
        assert!(manifest.contains("edition = \"2021\""));
        assert!(opts.path.join("src/main.rs").exists());
    }

    #[test]
    fn new_existing_path_fails() {
        let dir = tempdir().unwrap();

        let opts = NewCmd {
            path: dir.path().to_path_buf(),
            edition: None,
            registry: None,
            non_vcs: true,
            non_virtual: false,
        };

        let result = opts.run();

        assert!(result.is_err());
    }
}
