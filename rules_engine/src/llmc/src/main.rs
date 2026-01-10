mod cli;
mod commands;
mod config;
mod git;
mod patrol;
mod sound;
mod state;
mod tmux;
mod worker;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use tracing_subscriber::fmt;

use crate::commands::{add, down, init, up};

#[tokio::main]
async fn main() -> Result<()> {
    fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init { source, target } => {
            init::run_init(source, target)?;
        }
        Commands::Up { no_patrol } => {
            up::run_up(no_patrol)?;
        }
        Commands::Down { force } => {
            down::run_down(force)?;
        }
        Commands::Add { name, model, role_prompt } => {
            add::run_add(&name, model, role_prompt)?;
        }
        Commands::Nuke { name } => {
            println!("Not implemented: nuke (worker: {})", name);
        }
        Commands::Status => {
            println!("Not implemented: status");
        }
        Commands::Start { worker, task } => {
            println!("Not implemented: start (worker: {}, task: {})", worker, task);
        }
        Commands::Message { worker, message } => {
            println!("Not implemented: message (worker: {}, message: {})", worker, message);
        }
        Commands::Attach { worker } => {
            println!("Not implemented: attach (worker: {})", worker);
        }
        Commands::Review { worker } => {
            println!("Not implemented: review (worker: {})", worker);
        }
        Commands::Reject { worker, reason } => {
            println!("Not implemented: reject (worker: {}, reason: {})", worker, reason);
        }
        Commands::Accept { worker } => {
            println!("Not implemented: accept (worker: {})", worker);
        }
        Commands::Rebase { worker } => {
            println!("Not implemented: rebase (worker: {})", worker);
        }
        Commands::Doctor => {
            println!("Not implemented: doctor");
        }
    }

    Ok(())
}
