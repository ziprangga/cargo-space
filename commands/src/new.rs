use clap::{Arg, ArgAction, ArgMatches, Command};
use std::path::Path;
use std::path::PathBuf;

use core_space::CargoResult;
use core_space::bail_out;
use core_space::config::Edition;
use core_space::config::PackageItems;
use core_space::config::Workspace;
use core_space::context::Change;
use core_space::context::Context;
use core_space::context::Modifier;
use core_space::context::TargetWriter;
use core_space::context::Writer;
use core_space::manifest::InheritMode;
use core_space::manifest::TablePath;
use core_space::vcs::GitRepo;
use core_space::vcs::IgnoreList;

#[derive(Debug)]
pub struct NewOptions {
    name: String,
    edition: Option<String>,
    registry: Option<String>,

    non_vcs: bool,
    non_virtual: bool,
}

impl NewOptions {
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

        let space_item = workspace.to_toml();

        let mut ctx = Context::new().with_new_space_from(&path);

        ctx = ctx.add_modifier(
            Modifier::insert_items(
                Change::new()
                    .with_path(TablePath::new().push("workspace"))
                    .with_item(space_item),
                InheritMode::None,
            ),
            TargetWriter::Space,
        );

        if self.non_virtual {
            let pkg_items = package_items.to_toml();
            build_non_virtual_src(&path)?;

            ctx = ctx.add_modifier(
                Modifier::add_key(
                    Change::new()
                        .with_path(TablePath::new().push("package"))
                        .with_key("name")
                        .with_item(self.name.as_str().into()),
                    InheritMode::None,
                ),
                TargetWriter::space(),
            );

            ctx = ctx.add_modifier(
                Modifier::insert_items(
                    Change::new()
                        .with_path(TablePath::new().push("package"))
                        .with_item(pkg_items),
                    InheritMode::Full,
                ),
                TargetWriter::Space,
            )
        }

        Writer::write(&ctx)?;

        version_control(&path, self.non_vcs)?;

        Ok(())
    }
}

fn version_control(dir: &Path, non_vcs: bool) -> CargoResult<()> {
    if non_vcs {
        return Ok(());
    }

    GitRepo::init(dir)?;
    GitRepo::write_gitignore_file(dir, &IgnoreList::default())
}

fn build_pkg_items(edition: Option<String>, registry: Option<String>) -> PackageItems {
    PackageItems::new()
        .with_version("0.1.0")
        .with_edition(edition.as_deref().unwrap_or("2024"))
        .with_publish(registry.as_ref().map(|registry| vec![registry.clone()]))
}

fn build_non_virtual_src(path: &Path) -> CargoResult<()> {
    let src_path = path.join("src");
    std::fs::create_dir_all(&src_path)?;
    std::fs::write(
        src_path.join("main.rs"),
        "fn main() {\n    println!(\"Hello, world!\");\n}\n",
    )?;

    Ok(())
}

fn resolver_default(edition: Option<String>) -> CargoResult<String> {
    let edition = edition.as_deref().unwrap_or("2024").parse::<Edition>()?;
    let value = edition.default_resolver().to_manifest();

    Ok(value)
}

pub fn cli_new() -> Command {
    Command::new("new")
        .about("Create a new cargo package at <path>")
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

    let opts = NewOptions {
        non_vcs,
        name,
        edition,
        registry,
        non_virtual,
    };

    opts.run()
}
