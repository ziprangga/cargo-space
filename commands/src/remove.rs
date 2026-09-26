use clap::{Arg, ArgAction, ArgMatches, Command};
use core_space::CargoResult;
use core_space::context::Context;
use core_space::context::Modifier;
use core_space::context::Target;
use core_space::context::Writer;
use core_space::manifest::build_table_dep;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RemoveCmd {
    dependencies: Vec<String>,

    package: Option<String>,
    dev: bool,
    build: bool,
    target: Option<String>,
}

impl RemoveCmd {
    pub fn run(&self) -> CargoResult<()> {
        let (space_table, pkg_table) = build_table_dep(self.dev, self.build, self.target.clone())?;

        let mut ctx = Context::discover()?;
        let mut message = None;

        if let Some(pkg) = &self.package {
            let pkg_id = ctx.get_pkg_id_mut(pkg)?;

            let dep_names = {
                let deps = pkg_id.try_get_deps()?;

                self.dependencies
                    .iter()
                    .filter_map(|dependency| {
                        match deps.iter().find(|dep| {
                            dep.get_name() == dependency
                                || dep.get_rename().is_some_and(|rename| rename == dependency)
                        }) {
                            Some(dep) => Some(dep.get_name().to_owned()),
                            None => {
                                message.get_or_insert(format!("`{dependency}` could not be found"));
                                None
                            }
                        }
                    })
                    .collect::<Vec<_>>()
            };

            for dep_name in dep_names {
                ctx.add_modifier(
                    Modifier::remove_key(pkg_table.clone(), dep_name),
                    Target::pkg(pkg),
                )?;
            }
        } else {
            let members = ctx
                .get_space()
                .get_members()
                .map(|members| members.to_vec());

            if let Some(pkgs) = members {
                for mut pkg_id in pkgs {
                    let dep_names = {
                        let deps = pkg_id.try_get_deps()?;

                        self.dependencies
                            .iter()
                            .filter_map(|dependency| {
                                match deps.iter().find(|dep| {
                                    dep.get_name() == dependency
                                        || dep
                                            .get_rename()
                                            .is_some_and(|rename| rename == dependency)
                                }) {
                                    Some(dep) => Some(dep.get_name().to_owned()),
                                    None => {
                                        message.get_or_insert(format!(
                                            "`{dependency}` could not be found"
                                        ));
                                        None
                                    }
                                }
                            })
                            .collect::<Vec<_>>()
                    };

                    for dep_name in dep_names {
                        ctx.add_modifier(
                            Modifier::remove_key(pkg_table.clone(), dep_name),
                            Target::pkg(pkg_id.get_name()),
                        )?;
                    }
                }
            }

            let dep_names = {
                let deps = ctx.get_space_mut().try_get_root_deps()?;

                self.dependencies
                    .iter()
                    .filter_map(|dependency| {
                        match deps.iter().find(|dep| {
                            dep.get_name() == dependency
                                || dep.get_rename().is_some_and(|rename| rename == dependency)
                        }) {
                            Some(dep) => Some(dep.get_name().to_owned()),
                            None => {
                                message.get_or_insert(format!("`{dependency}` could not be found"));
                                None
                            }
                        }
                    })
                    .collect::<Vec<_>>()
            };

            for dep_name in dep_names {
                ctx.add_modifier(
                    Modifier::remove_key(space_table.clone(), dep_name),
                    Target::space(),
                )?;
            }
        }

        if let Some(msg) = message {
            println!("{msg}");
        }

        ctx.apply()?;
        Writer::write(&ctx)?;

        Ok(())
    }
}

pub fn cli_remove() -> Command {
    Command::new("remove")
        .about("Remove a dependency to a Cargo package")
        .arg(
            Arg::new("dependencies")
                .value_name("CRATE")
                .required(true)
                .num_args(1..)
                .action(ArgAction::Append),
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
}

pub fn exec_remove(args: &ArgMatches) -> CargoResult<()> {
    let dependencies = args
        .get_many::<String>("dependencies")
        .expect("dependencies is required")
        .cloned()
        .collect::<Vec<_>>();

    let package = args.get_one::<String>("package").cloned();

    let dev = args.get_flag("dev");
    let build = args.get_flag("build");

    let target = args.get_one::<String>("target").cloned();

    let cmd = RemoveCmd {
        dependencies,

        package,
        dev,
        build,
        target,
    };

    cmd.run()
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
    fn remove_multiple_dependencies() {
        let dir = setup_workspace();
        let old_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let workspace_before = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
        let package_before = fs::read_to_string(dir.path().join("app/Cargo.toml")).unwrap();

        println!("=== remove_multiple_dependencies: workspace before ===");
        println!("{workspace_before}");

        println!("=== remove_multiple_dependencies: package before ===");
        println!("{package_before}");

        RemoveCmd {
            dependencies: vec!["serde".into(), "tokio".into()],
            package: None,
            dev: false,
            build: false,
            target: None,
        }
        .run()
        .unwrap();

        std::env::set_current_dir(old_dir).unwrap();

        let workspace_after = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
        let package_after = fs::read_to_string(dir.path().join("app/Cargo.toml")).unwrap();

        println!("=== remove_multiple_dependencies: workspace after ===");
        println!("{workspace_after}");

        println!("=== remove_multiple_dependencies: package after ===");
        println!("{package_after}");

        assert!(!workspace_after.contains("serde = \"1\""));
        assert!(!workspace_after.contains("tokio = \"1\""));
        assert!(!package_after.contains("serde = { workspace = true }"));
        assert!(!package_after.contains("tokio = { workspace = true }"));
    }

    #[test]
    fn remove_nonexistent_dependencies() {
        let dir = setup_workspace();
        let old_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let workspace_before = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
        let package_before = fs::read_to_string(dir.path().join("app/Cargo.toml")).unwrap();

        println!("=== remove_nonexistent_dependencies: workspace before ===");
        println!("{workspace_before}");

        println!("=== remove_nonexistent_dependencies: package before ===");
        println!("{package_before}");

        RemoveCmd {
            dependencies: vec!["not-exist".into()],
            package: None,
            dev: false,
            build: false,
            target: None,
        }
        .run()
        .unwrap();

        std::env::set_current_dir(old_dir).unwrap();

        let workspace_after = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
        let package_after = fs::read_to_string(dir.path().join("app/Cargo.toml")).unwrap();

        println!("=== remove_nonexistent_dependencies: workspace after ===");
        println!("{workspace_after}");

        println!("=== remove_nonexistent_dependencies: package after ===");
        println!("{package_after}");

        assert!(workspace_after.contains("serde = \"1\""));
        assert!(workspace_after.contains("tokio = \"1\""));
        assert!(package_after.contains("serde = { workspace = true }"));
        assert!(package_after.contains("tokio = { workspace = true }"));
    }

    #[test]
    fn remove_dependencies_in_package() {
        let dir = setup_workspace();
        let old_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let workspace_before = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
        let package_before = fs::read_to_string(dir.path().join("app/Cargo.toml")).unwrap();

        println!("=== remove_dependencies_in_package: workspace before ===");
        println!("{workspace_before}");

        println!("=== remove_dependencies_in_package: package before ===");
        println!("{package_before}");

        RemoveCmd {
            dependencies: vec!["serde".into()],
            package: Some("app".into()),
            dev: false,
            build: false,
            target: None,
        }
        .run()
        .unwrap();

        std::env::set_current_dir(old_dir).unwrap();

        let workspace_after = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
        let package_after = fs::read_to_string(dir.path().join("app/Cargo.toml")).unwrap();

        println!("=== remove_dependencies_in_package: workspace after ===");
        println!("{workspace_after}");

        println!("=== remove_dependencies_in_package: package after ===");
        println!("{package_after}");

        assert!(workspace_after.contains("serde = \"1\""));
        assert!(workspace_after.contains("tokio = \"1\""));
        assert!(!package_after.contains("serde = { workspace = true }"));
        assert!(package_after.contains("tokio = { workspace = true }"));
    }
}
