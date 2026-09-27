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
    name: String,
    edition: Option<String>,
    registry: Option<String>,

    non_vcs: bool,
    non_virtual: bool,
}

impl NewCmd {
    pub fn run(&self) -> CargoResult<()> {
        let path = PathBuf::from(&self.name);

        if path.exists() {
            bail_out!(
                "destination `{}` already exists\n\n\
                     Use `cargo init` to initialize the directory",
                path.display()
            );
        }

        std::fs::create_dir_all(&path)?;

        let package_items = build_pkg_items(self.edition.clone(), self.registry.clone());
        let default_resolver = resolver_default(self.edition.clone())?;
        let workspace = Workspace::new()
            .with_resolver(default_resolver)
            .with_package_items(&package_items);

        let space_manifest_path = path.join(MANIFEST_FILENAME);
        let space_manifest = Manifest::new(space_manifest_path.clone());
        let space = Space::new()
            .with_root_path(&path)
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
            build_non_virtual_src(&path)?;

            ctx.add_modifier(
                Modifier::add_key(
                    TablePath::new().push("package"),
                    "name",
                    self.name.as_str().into(),
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
            )?
        }

        ctx.apply()?;
        Writer::write(&ctx)?;

        version_control(&path, self.non_vcs)?;

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

    let name = args
        .get_one::<String>("path")
        .expect("path is required")
        .to_string();

    let edition = args.get_one::<String>("edition").cloned();

    let registry = args.get_one::<String>("registry").cloned();
    let non_virtual = args.get_flag("non-virtual");

    let cmd = NewCmd {
        non_vcs,
        name,
        edition,
        registry,
        non_virtual,
    };

    cmd.run()
}
