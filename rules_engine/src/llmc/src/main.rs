mod cli;
mod commands;
mod config;
mod editor;
mod git;
mod lock;
mod logging;
mod patrol;
mod recovery;
mod sound;
mod state;
mod tmux;
mod worker;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands, ConfigAction};

use crate::commands::review::ReviewInterface;
use crate::commands::{
    accept, add, attach, config as config_cmd, doctor, down, init, message, nuke, peek, pick,
    rebase, reject, reset, review, start, status, up,
};
use crate::logging::config as log_config;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Err(e) = log_config::init_logging(cli.verbose) {
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
        Commands::Reset { .. } => "reset",
        Commands::Doctor { .. } => "doctor",
        Commands::Peek { .. } => "peek",
        Commands::Pick { .. } => "pick",
        Commands::Config { .. } => "config",
    };

    tracing::info!(operation = "cli_command", command = command_name, "Command started");
    let start_time = std::time::Instant::now();

    let result = match cli.command {
        Commands::Init { source, target, force } => init::run_init(source, target, force),
        Commands::Up { no_patrol, force } => up::run_up(no_patrol, cli.verbose, force),
        Commands::Down { force } => down::run_down(force),
        Commands::Add { name, model, role_prompt } => add::run_add(&name, model, role_prompt),
        Commands::Nuke { name, all } => nuke::run_nuke(name.as_deref(), all),
        Commands::Status { json } => status::run_status(json),
        Commands::Start { worker, prefix, prompt, prompt_file, prompt_cmd, skip_self_review } => {
            start::run_start(worker, prefix, prompt, prompt_file, prompt_cmd, skip_self_review)
        }
        Commands::Message { worker, message } => message::run_message(&worker, message),
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
        Commands::Reject { message } => reject::run_reject(message),
        Commands::Accept { worker } => accept::run_accept(worker),
        Commands::Rebase { worker } => rebase::run_rebase(&worker),
        Commands::Reset { worker, yes } => reset::run_reset(&worker, yes),
        Commands::Doctor { repair, yes, rebuild } => doctor::run_doctor(repair, yes, rebuild),
        Commands::Peek { worker, lines } => peek::run_peek(worker, lines),
        Commands::Pick { worker } => pick::run_pick(&worker),
        Commands::Config { action } => match action {
            ConfigAction::Get { key } => config_cmd::run_config_get(&key),
            ConfigAction::Set { key, value } => config_cmd::run_config_set(&key, &value),
        },
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
