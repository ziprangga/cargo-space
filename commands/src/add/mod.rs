mod dep_req;
use dep_req::DepOptions;
use dep_req::DepReq;

use clap::{Arg, ArgAction, ArgMatches, Command};
use core_space::CargoResult;
use core_space::config::DataInherit;
use core_space::config::Dependency;
use core_space::config::RulesInherit;
use core_space::context::Change;
use core_space::context::Context;
use core_space::context::Modifier;
use core_space::context::Target;
use core_space::context::Writer;
use core_space::manifest::InheritMode;
use core_space::manifest::Item;
use core_space::manifest::TablePath;
use core_space::manifest::build_table_dep;
use core_space::manifest::into_inline_table_item_if;

use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AddOptions {
    dep_options: DepOptions,

    package: Option<String>,
    dev: bool,
    build: bool,
    target: Option<String>,
    private: RulesInherit,
}

impl AddOptions {
    pub fn run(&self) -> CargoResult<()> {
        let mut ctx = Context::discover()?;

        let crate_name = self.dep_options.dep_req.name();

        let (space_dep_table, pkg_dep_table) =
            build_table_dep(self.dev, self.build, self.target.clone())?;

        if let Some(pkg) = &self.package {
            let inherit_mode = match &self.private {
                RulesInherit::Choose(_) => InheritMode::Partial,
                RulesInherit::All => InheritMode::Full,
                RulesInherit::None => InheritMode::None,
            };

            if let Some(item) = dep_from_space(&mut ctx, crate_name, &space_dep_table)? {
                let root_path = ctx.get_space().get_root_path();
                let dependency = Dependency::from_toml(root_path, crate_name, &item);
                if let Some(dep) = dependency {
                    let (space_dep_item, pkg_dep_item) = inherit_split(self.private.clone(), &dep)?;
                    if let Some(pkg_item) = pkg_dep_item {
                        let pkg_item_inline =
                            into_inline_table_item_if(!self.private.is_all(), pkg_item)?;

                        ctx.add_modifier(
                            Modifier::add_key(
                                Change::new()
                                    .with_path(pkg_dep_table.clone())
                                    .with_key(crate_name)
                                    .with_item(pkg_item_inline),
                                inherit_mode,
                            ),
                            Target::pkg(pkg),
                        )?;
                    }

                    if let Some(space_item) = space_dep_item {
                        ctx.add_modifier(
                            Modifier::update_key(
                                Change::new()
                                    .with_path(space_dep_table.clone())
                                    .with_key(crate_name)
                                    .with_item(space_item),
                                InheritMode::None,
                            ),
                            Target::space(),
                        )?;
                    }
                }
            } else {
                let pkg_id = ctx.get_pkg_id_mut(pkg)?;
                let pkg_manifest_path = pkg_id.get_manifest_path();
                let dependency = self.dep_options.to_dependency(pkg_manifest_path)?;
                let (space_dep_item, pkg_dep_item) =
                    inherit_split(self.private.clone(), &dependency)?;

                if let Some(pkg_item) = pkg_dep_item {
                    let pkg_item_inline =
                        into_inline_table_item_if(!self.private.is_all(), pkg_item)?;

                    ctx.add_modifier(
                        Modifier::add_key(
                            Change::new()
                                .with_path(pkg_dep_table.clone())
                                .with_key(crate_name)
                                .with_item(pkg_item_inline),
                            inherit_mode,
                        ),
                        Target::pkg(pkg),
                    )?;
                }

                if let Some(space_item) = space_dep_item {
                    ctx.add_modifier(
                        Modifier::add_key(
                            Change::new()
                                .with_path(space_dep_table.clone())
                                .with_key(crate_name)
                                .with_item(space_item),
                            InheritMode::None,
                        ),
                        Target::space(),
                    )?;
                }
            }
        }

        if self.package.is_none() {
            let space_manifest_path = ctx.get_space().get_root_manifest_path();
            let dependency = self.dep_options.to_dependency(space_manifest_path)?;

            let inline_condition =
                self.dep_options.features.is_some() || self.dep_options.optional.is_some();

            let space_depedency =
                into_inline_table_item_if(inline_condition, dependency.to_toml())?;

            ctx.add_modifier(
                Modifier::add_key(
                    Change::new()
                        .with_path(space_dep_table)
                        .with_key(crate_name)
                        .with_item(space_depedency),
                    InheritMode::None,
                ),
                Target::space(),
            )?;
        }

        ctx.apply()?;
        Writer::write(&ctx)?;

        Ok(())
    }
}

fn dep_from_space(
    ctx: &mut Context,
    dep_name: &str,
    space_dep_table: &TablePath,
) -> CargoResult<Option<Item>> {
    let manifest = ctx.get_space_mut().try_get_manifest()?;
    if !manifest.is_key_exist(&space_dep_table, dep_name) {
        return Ok(None);
    }

    let dep_item = manifest.get_item_of_key(&space_dep_table, dep_name)?;
    Ok(Some(dep_item.clone()))
}

fn inherit_split(
    private: RulesInherit,
    dependency: &Dependency,
) -> CargoResult<(Option<Item>, Option<Item>)> {
    let (dep_item_space, dep_item_pkg) = dependency.disjoint(&private);

    Ok((
        dep_item_space.map(|dependency| dependency.to_toml()),
        dep_item_pkg.map(|dependency| dependency.to_toml()),
    ))
}

pub fn cli_add() -> Command {
    Command::new("add")
        .about("Add a dependency to a Cargo package")
        .arg(
            Arg::new("dep-req")
                .value_name("CRATE")
                .required(true)
                .action(ArgAction::Set),
        )
        .arg(Arg::new("path").long("path").value_name("PATH"))
        .arg(Arg::new("git").long("git").value_name("URL"))
        .arg(Arg::new("tag").long("tag").value_name("TAG"))
        .arg(Arg::new("branch").long("branch").value_name("BRANCH"))
        .arg(Arg::new("rev").long("rev").value_name("REV"))
        .arg(Arg::new("registry").long("registry").value_name("REGISTRY"))
        .arg(
            Arg::new("features")
                .long("features")
                .value_name("FEATURES")
                .value_delimiter(','),
        )
        .arg(
            Arg::new("optional")
                .long("optional")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("package")
                .short('p')
                .long("package")
                .value_name("PACKAGE"),
        )
        .arg(
            Arg::new("dev")
                .long("dev")
                .action(ArgAction::SetTrue)
                .requires("package")
                .conflicts_with("build"),
        )
        .arg(
            Arg::new("build")
                .long("build")
                .action(ArgAction::SetTrue)
                .requires("package")
                .conflicts_with("dev"),
        )
        .arg(
            Arg::new("target")
                .long("target")
                .value_name("TARGET")
                .requires("package"),
        )
        .arg(
            Arg::new("private")
                .long("private")
                .value_name("FIELD")
                .num_args(0..=3)
                .value_parser(clap::value_parser!(DataInherit))
                .requires("package"),
        )
}

pub fn exec_add(args: &ArgMatches) -> CargoResult<()> {
    let dep_req = args
        .get_one::<String>("dep-req")
        .expect("dep-req is required")
        .clone();

    let path = args.get_one::<String>("path").map(PathBuf::from);
    let git = args.get_one::<String>("git").cloned();
    let tag = args.get_one::<String>("tag").cloned();
    let branch = args.get_one::<String>("branch").cloned();
    let rev = args.get_one::<String>("rev").cloned();
    let registry = args.get_one::<String>("registry").cloned();

    let features = args
        .get_many::<String>("features")
        .map(|values| values.cloned().collect());

    let optional = args.get_one::<bool>("optional").copied();

    let package = args.get_one::<String>("package").cloned();

    let dev = args.get_flag("dev");
    let build = args.get_flag("build");

    let target = args.get_one::<String>("target").cloned();

    let private = if args.contains_id("private") {
        match args.get_many::<DataInherit>("private") {
            Some(values) => RulesInherit::Choose(values.copied().collect()),
            None => RulesInherit::All,
        }
    } else {
        RulesInherit::None
    };

    let dep_options = DepOptions {
        dep_req: DepReq::resolve(&dep_req)?,
        path,
        git,
        tag,
        branch,
        rev,
        registry,
        features,
        optional,
    };

    let opts = AddOptions {
        dep_options,

        package,
        dev,
        build,
        target,
        private,
    };

    opts.run()
}
