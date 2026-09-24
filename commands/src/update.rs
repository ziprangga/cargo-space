use crate::dependency_req::DepOptions;
use crate::dependency_req::DepReq;
use crate::dependency_req::inherit_split;

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
use core_space::error;
use core_space::manifest::InheritMode;
use core_space::manifest::build_table_dep;
use core_space::manifest::into_inline_table_item_if;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UpdateOptions {
    dep_options: DepOptions,

    package: Option<String>,
    dev: bool,
    build: bool,
    target: Option<String>,
    private: RulesInherit,
    change_inherit: bool,
}

impl UpdateOptions {
    fn run(&self) -> CargoResult<()> {
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

            let pkg_id = ctx.get_pkg_id_mut(pkg)?;
            let pkg_deps = pkg_id.try_get_deps()?;
            let pkg_dep_id = pkg_deps
                .iter()
                .find(|dep| dep.get_name() == crate_name || dep.get_rename() == Some(crate_name))
                .ok_or_else(|| error!("Dependency `{crate_name}` not found"))?;

            let dep_item = pkg_dep_id.get_toml_item()?;
            let target_dep = Dependency::from_toml(crate_name, &dep_item);
            let pkg_manifest_path = pkg_id.get_manifest_path();
            let source_dep = self.dep_options.to_dependency(pkg_manifest_path)?;

            if !self.change_inherit {
                if !(inherit_mode == InheritMode::None) {
                    return Err(error!(
                        "Dependency `{}` is inherit from workspace, if want force change inherit use --change-inherit",
                        crate_name
                    ));
                }

                if let Some(mut tgt_dep) = target_dep {
                    tgt_dep.update(&source_dep);
                    let pkg_item = tgt_dep.to_toml();
                    let pkg_item_inline =
                        into_inline_table_item_if(!self.private.is_all(), pkg_item)?;
                    ctx.add_modifier(
                        Modifier::update_key(
                            Change::new()
                                .with_path(pkg_dep_table.clone())
                                .with_key(crate_name)
                                .with_item(pkg_item_inline),
                            InheritMode::None,
                        ),
                        Target::pkg(pkg),
                    )?;
                }
            } else {
                let update_dep = match inherit_mode {
                    InheritMode::Full => {
                        let root_deps = ctx.get_space_mut().try_get_root_deps()?;
                        let space_dep_id = root_deps
                            .iter()
                            .find(|dep| {
                                dep.get_name() == crate_name || dep.get_rename() == Some(crate_name)
                            })
                            .ok_or_else(|| error!("Dependency `{crate_name}` not found"))?;
                        let space_dep_item = space_dep_id.get_toml_item()?;
                        let space_target_dep = Dependency::from_toml(crate_name, &space_dep_item);
                        let mut tgt_dep = space_target_dep
                            .ok_or_else(|| error!("Dependency `{crate_name}` not found"))?;

                        tgt_dep.update(&source_dep);
                        tgt_dep
                    }

                    InheritMode::Partial => {
                        let root_deps = ctx.get_space_mut().try_get_root_deps()?;
                        let space_dep_id = root_deps
                            .iter()
                            .find(|dep| {
                                dep.get_name() == crate_name || dep.get_rename() == Some(crate_name)
                            })
                            .ok_or_else(|| error!("Dependency `{crate_name}` not found"))?;
                        let space_dep_item = space_dep_id.get_toml_item()?;
                        let space_target_dep = Dependency::from_toml(crate_name, &space_dep_item);
                        let mut tgt_dep = target_dep
                            .ok_or_else(|| error!("Dependency `{crate_name}` not found"))?;

                        let space_dep = space_target_dep
                            .ok_or_else(|| error!("Dependency `{crate_name}` not found"))?;

                        tgt_dep.update(&space_dep);
                        tgt_dep.update(&source_dep);
                        tgt_dep
                    }

                    InheritMode::None => {
                        let mut tgt_dep = target_dep
                            .ok_or_else(|| error!("Dependency `{crate_name}` not found"))?;
                        tgt_dep.update(&source_dep);
                        tgt_dep
                    }
                };

                let (space_dep_item, pkg_dep_item) =
                    inherit_split(self.private.clone(), &update_dep)?;

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
            let space_source_dep = self.dep_options.to_dependency(space_manifest_path)?;

            let root_deps = ctx.get_space_mut().try_get_root_deps()?;
            let space_dep_id = root_deps
                .iter()
                .find(|dep| dep.get_name() == crate_name || dep.get_rename() == Some(crate_name));

            if let Some(dep_id) = space_dep_id {
                let dep_item = dep_id.get_toml_item()?;
                let space_target_dep = Dependency::from_toml(crate_name, &dep_item);

                if let Some(mut tgt_dep) = space_target_dep {
                    tgt_dep.update(&space_source_dep);
                    let space_item = tgt_dep.to_toml();
                    let inline_condition =
                        self.dep_options.features.is_some() || self.dep_options.optional.is_some();
                    let space_item_inline =
                        into_inline_table_item_if(inline_condition, space_item)?;

                    ctx.add_modifier(
                        Modifier::update_key(
                            Change::new()
                                .with_path(space_dep_table.clone())
                                .with_key(crate_name)
                                .with_item(space_item_inline),
                            InheritMode::None,
                        ),
                        Target::space(),
                    )?;
                }
            };
        }

        ctx.apply()?;
        Writer::write(&ctx)?;

        Ok(())
    }
}

pub fn cli_update() -> Command {
    Command::new("update")
        .about("Update a dependency to a Cargo package")
        .arg(
            Arg::new("dep-req")
                .value_name("CRATE")
                .required(true)
                .action(ArgAction::Set),
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
        .arg(
            Arg::new("change-inherit")
                .long("change-inherit")
                .action(ArgAction::SetTrue)
                .requires("package"),
        )
}

pub fn exec_update(args: &ArgMatches) -> CargoResult<()> {
    let dep_req = args
        .get_one::<String>("dep-req")
        .expect("dep-req is required")
        .clone();

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
    let change_inherit = args.get_flag("change-inherit");

    let dep_options = DepOptions {
        dep_req: DepReq::resolve(&dep_req)?,
        path: None,
        git: None,
        tag: None,
        branch: None,
        rev: None,
        registry: None,
        features: None,
        optional: None,
    };

    let opts = UpdateOptions {
        dep_options,

        package,
        dev,
        build,
        target,
        private,
        change_inherit,
    };

    opts.run()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn setup_workspace() -> tempfile::TempDir {
        let dir = tempdir().unwrap();

        fs::create_dir_all(dir.path().join("app/src")).unwrap();

        fs::write(
            dir.path().join("Cargo.toml"),
            r#"[workspace]
members = ["app"]

[workspace.dependencies]
serde = "1"
tokio = "1"
"#,
        )
        .unwrap();

        fs::write(
            dir.path().join("app/Cargo.toml"),
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { workspace = true }
tokio = { workspace = true }
"#,
        )
        .unwrap();

        fs::write(dir.path().join("app/src/main.rs"), "fn main() {}\n").unwrap();

        dir
    }

    #[test]
    fn update_dependency() {
        let dir = setup_workspace();
        let old_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let workspace_before = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();

        println!("=== update_dependency: workspace before ===");
        println!("{workspace_before}");

        UpdateOptions {
            dep_options: DepOptions {
                dep_req: DepReq::resolve("serde@1.0.200").unwrap(),
                path: None,
                git: None,
                tag: None,
                branch: None,
                rev: None,
                registry: None,
                features: None,
                optional: None,
            },
            package: None,
            dev: false,
            build: false,
            target: None,
            private: RulesInherit::None,
            change_inherit: false,
        }
        .run()
        .unwrap();

        std::env::set_current_dir(old_dir).unwrap();

        let workspace_after = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();

        println!("=== update_dependency: workspace after ===");
        println!("{workspace_after}");

        assert!(workspace_after.contains("serde = \"1.0.200\""));
        assert!(workspace_after.contains("tokio = \"1\""));
    }
}
