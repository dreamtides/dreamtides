// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    use clap::Parser;
    use tv_lib::cli::{exit_codes, resolve_paths, Args};

    let args = Args::parse();
    match resolve_paths(&args) {
        Ok(paths) => tv_lib::run(paths, args.jsonl),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(exit_codes::INVALID_ARGUMENTS);
        }
    }
}
