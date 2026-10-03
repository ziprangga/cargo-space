pub mod dependency_req;

mod add;
mod create;
mod init;
mod new;
mod remove;
mod update;

use clap::{Arg, ArgAction};
use clap::{ArgMatches, Command};
use core_space::CargoResult;

pub fn cli_version() -> Arg {
    Arg::new("version")
        .long("version")
        .short('v')
        .help("Print version information")
        .action(ArgAction::Version)
}

pub fn cli() -> Command {
    Command::new("cargo-space")
        .arg(cli_version())
        .version(env!("CARGO_PKG_VERSION"))
        .disable_version_flag(true)
        .subcommand(new::cli_new())
        .subcommand(init::cli_init())
        .subcommand(create::cli_create())
        .subcommand(add::cli_add())
        .subcommand(remove::cli_remove())
        .subcommand(update::cli_update())
}

pub fn exec(command: &str, args: &ArgMatches) -> CargoResult<()> {
    match command {
        "new" => new::exec_new(args),
        "init" => init::exec_init(args),
        "create" => create::exec_create(args),
        "add" => add::exec_add(args),
        "remove" => remove::exec_remove(args),
        "update" => update::exec_update(args),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_version() {
        let cmd = cli();

        let arg = cmd
            .get_arguments()
            .find(|arg| arg.get_id() == "version")
            .unwrap();

        println!("id: {:?}", arg.get_id());
        println!("short: {:?}", arg.get_short());
        println!("long: {:?}", arg.get_long());

        assert_eq!(arg.get_id().as_str(), "version");
        assert_eq!(arg.get_short(), Some('v'));
        assert_eq!(arg.get_long(), Some("version"));
    }

    #[test]
    fn cli_version_long() {
        let cmd = cli();

        let result = cmd
            .try_get_matches_from(["cargo-space", "--version"])
            .unwrap_err();

        println!("{result}");

        assert_eq!(result.kind(), clap::error::ErrorKind::DisplayVersion);
    }

    #[test]
    fn cli_version_short() {
        let cmd = cli();

        let result = cmd.try_get_matches_from(["cargo-space", "-v"]).unwrap_err();

        println!("{result}");

        assert_eq!(result.kind(), clap::error::ErrorKind::DisplayVersion);
    }
}
