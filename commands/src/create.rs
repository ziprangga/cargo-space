use clap::{Arg, ArgAction, ArgMatches, Command};
use std::path::Path;
use std::path::PathBuf;

use core_space::CargoResult;
use core_space::config::Dependency;
use core_space::config::PackageItems;
use core_space::config::Source;
use core_space::context::Change;
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
pub struct CreatePackageOption {
    path: PathBuf,
    name: Option<String>,
    kind: Option<NewPackageKind>,
    edition: Option<String>,
    registry: Option<String>,

    dep: bool,
    non_inherit: bool,
}

impl CreatePackageOption {
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

        println!("path: {}", path.display());

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
                Change::new()
                    .with_path(TablePath::new().push("package"))
                    .with_key("name")
                    .with_item(pkg_name.clone().into()),
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
                Modifier::insert_items_at(
                    Change::new()
                        .with_path(TablePath::new().push("package"))
                        .with_item(pkg_items),
                    InheritMode::None,
                ),
                Target::pkg(pkg_name.clone()),
            )?
        } else {
            let space_manifest = ctx.get_space_mut().try_get_manifest()?;
            let table_path = TablePath::new().push("workspace").push("package");
            let pkg_items = space_manifest.get_item_of_table(&table_path)?.clone();

            ctx.add_modifier(
                Modifier::insert_items_at(
                    Change::new()
                        .with_path(TablePath::new().push("package"))
                        .with_item(pkg_items),
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
                    Change::new()
                        .with_path(TablePath::new().push("workspace").push("dependencies"))
                        .with_key(pkg_name.clone())
                        .with_item(item_inline),
                    InheritMode::None,
                ),
                Target::space(),
            )?;
        }

        ctx.add_modifier(
            Modifier::update_key(
                Change::new()
                    .with_path(TablePath::new().push("workspace"))
                    .with_key("members")
                    .with_item(member_path.into()),
                InheritMode::None,
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

    let opts = CreatePackageOption {
        path,
        name,
        kind,
        edition,
        registry,

        dep,
        non_inherit,
    };

    opts.run()?;

    Ok(())
}
