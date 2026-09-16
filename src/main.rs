use commands::cli;
use commands::exec;
use core_space::CargoResult;

fn run() -> CargoResult<()> {
    let mut args = std::env::args().collect::<Vec<_>>();

    if args.len() > 1 && args[1] == "space" {
        args.remove(1);
    }

    let matches = cli().get_matches_from(args);

    let (command, args) = matches.subcommand().expect("a subcommand is required");

    exec(command, args)
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err:?}");
        std::process::exit(1);
    }
}
