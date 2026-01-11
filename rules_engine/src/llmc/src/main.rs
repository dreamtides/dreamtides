mod cli;
mod commands;
mod config;
mod git;
mod logging;
mod patrol;
mod recovery;
mod sound;
mod state;
mod tmux;
mod worker;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

use crate::commands::review::ReviewInterface;
use crate::commands::{
    accept, add, attach, doctor, down, init, message, nuke, rebase, reject, review, start, status,
    up,
};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Err(e) = logging::init_logging() {
        eprintln!("Warning: Failed to initialize logging: {e}");
    }

    let command_name = match &cli.command {
        Commands::Init { .. } => "init",
        Commands::Up { .. } => "up",
        Commands::Down { .. } => "down",
        Commands::Add { .. } => "add",
        Commands::Nuke { .. } => "nuke",
        Commands::Status { .. } => "status",
        Commands::Start { .. } => "start",
        Commands::Message { .. } => "message",
        Commands::Attach { .. } => "attach",
        Commands::Review { .. } => "review",
        Commands::Reject { .. } => "reject",
        Commands::Accept { .. } => "accept",
        Commands::Rebase { .. } => "rebase",
        Commands::Doctor { .. } => "doctor",
    };

    tracing::info!(operation = "cli_command", command = command_name, "Command started");
    let start_time = std::time::Instant::now();

    let result = match cli.command {
        Commands::Init { source, target } => init::run_init(source, target),
        Commands::Up { no_patrol } => up::run_up(no_patrol, cli.verbose),
        Commands::Down { force } => down::run_down(force),
        Commands::Add { name, model, role_prompt } => add::run_add(&name, model, role_prompt),
        Commands::Nuke { name, all } => nuke::run_nuke(name.as_deref(), all),
        Commands::Status { json } => status::run_status(json),
        Commands::Start { worker, prompt, prompt_file } => {
            start::run_start(worker, prompt, prompt_file)
        }
        Commands::Message { worker, message } => message::run_message(&worker, &message),
        Commands::Attach { worker } => attach::run_attach(&worker),
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
            review::run_review(worker, interface_enum)
        }
        Commands::Reject { message } => reject::run_reject(&message),
        Commands::Accept { worker } => accept::run_accept(worker),
        Commands::Rebase { worker } => rebase::run_rebase(&worker),
        Commands::Doctor { repair, rebuild } => doctor::run_doctor(repair, rebuild),
    };

    let duration_ms = start_time.elapsed().as_millis() as u64;

    match &result {
        Ok(_) => {
            tracing::info!(
                operation = "cli_command",
                command = command_name,
                duration_ms,
                result = "success",
                "Command completed"
            );
        }
        Err(e) => {
            tracing::error!(
                operation = "cli_command",
                command = command_name,
                duration_ms,
                result = "error",
                error = %e,
                "Command failed"
            );
            display_error(e, cli.verbose);
            std::process::exit(1);
        }
    }

    Ok(())
}

fn display_error(error: &anyhow::Error, verbose: bool) {
    if verbose {
        eprintln!("Error: {error:?}");
    } else {
        eprintln!("Error: {error}");
    }
}
