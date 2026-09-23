mod add;
mod create;
mod init;
mod new;
mod remove;

use clap::{ArgMatches, Command};
use core_space::CargoResult;

pub fn cli() -> Command {
    Command::new("cargo-space")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(new::cli_new())
        .subcommand(init::cli_init())
        .subcommand(create::cli_create())
        .subcommand(add::cli_add())
        .subcommand(remove::cli_remove())
}

pub fn exec(command: &str, args: &ArgMatches) -> CargoResult<()> {
    match command {
        "new" => new::exec_new(args),
        "init" => init::exec_init(args),
        "create" => create::exec_create(args),
        "add" => add::exec_add(args),
        "remove" => remove::exec_remove(args),
        _ => unreachable!(),
    }
}
