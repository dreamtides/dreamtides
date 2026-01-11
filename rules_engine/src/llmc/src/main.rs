mod cli;
mod commands;
mod config;
mod git;
mod patrol;
mod recovery;
mod sound;
mod state;
mod tmux;
mod worker;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use tracing_subscriber::{EnvFilter, fmt};

use crate::commands::review::ReviewInterface;
use crate::commands::{
    accept, add, attach, doctor, down, init, message, nuke, rebase, reject, review, start, status,
    up,
};

#[tokio::main]
async fn main() -> Result<()> {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_env("LLMC_LOG").unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init { source, target } => {
            init::run_init(source, target)?;
        }
        Commands::Up { no_patrol, verbose } => {
            up::run_up(no_patrol, verbose)?;
        }
        Commands::Down { force } => {
            down::run_down(force)?;
        }
        Commands::Add { name, model, role_prompt } => {
            add::run_add(&name, model, role_prompt)?;
        }
        Commands::Nuke { name, all } => {
            nuke::run_nuke(name.as_deref(), all)?;
        }
        Commands::Status { json } => {
            status::run_status(json)?;
        }
        Commands::Start { worker, prompt, prompt_file } => {
            start::run_start(worker, prompt, prompt_file)?;
        }
        Commands::Message { worker, message } => {
            message::run_message(&worker, &message)?;
        }
        Commands::Attach { worker } => {
            attach::run_attach(&worker)?;
        }
        Commands::Review { worker, interface } => {
            let interface_enum = match interface.as_str() {
                "difftastic" => ReviewInterface::Difftastic,
                "vscode" => ReviewInterface::VSCode,
                _ => {
                    eprintln!(
                        "Invalid interface: {}. Valid options: difftastic, vscode",
                        interface
                    );
                    std::process::exit(1);
                }
            };
            review::run_review(worker, interface_enum)?;
        }
        Commands::Reject { message } => {
            reject::run_reject(&message)?;
        }
        Commands::Accept { worker } => {
            accept::run_accept(worker)?;
        }
        Commands::Rebase { worker } => {
            rebase::run_rebase(&worker)?;
        }
        Commands::Doctor { repair, rebuild } => {
            doctor::run_doctor(repair, rebuild)?;
        }
    }

    Ok(())
}
