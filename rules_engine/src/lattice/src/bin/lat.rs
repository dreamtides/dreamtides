use std::process::ExitCode;

use lattice::cli::{argument_parser, command_dispatch};

fn main() -> ExitCode {
    human_panic::setup_panic!();
    let args = argument_parser::parse();
    command_dispatch::run(args)
}
