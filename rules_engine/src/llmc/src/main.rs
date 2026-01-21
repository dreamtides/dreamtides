mod auto_mode;
mod cli;
mod commands;
mod config;
mod editor;
mod git;
mod ipc;
mod lock;
mod logging;
mod overseer_mode;
mod patrol;
mod recovery;
mod sound;
mod state;
mod tmux;
mod worker;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands, ConfigAction, HookAction};
use llmc::json_output;

use crate::commands::review::ReviewInterface;
use crate::commands::{
    accept, add, attach, config as config_cmd, console, doctor, down, hook, init, message, nuke,
    overseer, peek, pick, rebase, reject, reset, review, start, status, up,
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
        Commands::Overseer => "overseer",
        Commands::Down { .. } => "down",
        Commands::Add { .. } => "add",
        Commands::Nuke { .. } => "nuke",
        Commands::Status { .. } => "status",
        Commands::Start { .. } => "start",
        Commands::Message { .. } => "message",
        Commands::Attach { .. } => "attach",
        Commands::Console { .. } => "console",
        Commands::Review { .. } => "review",
        Commands::Reject { .. } => "reject",
        Commands::Accept { .. } => "accept",
        Commands::Rebase { .. } => "rebase",
        Commands::Reset { .. } => "reset",
        Commands::Doctor { .. } => "doctor",
        Commands::Peek { .. } => "peek",
        Commands::Pick { .. } => "pick",
        Commands::Config { .. } => "config",
        Commands::Schema => "schema",
        Commands::Hook { .. } => "hook",
    };

    tracing::info!(operation = "cli_command", command = command_name, "Command started");
    let start_time = std::time::Instant::now();

    let result = match cli.command {
        Commands::Init { source, target, force } => init::run_init(source, target, force),
        Commands::Up {
            no_patrol,
            force,
            auto,
            task_pool_command,
            concurrency,
            post_accept_command,
        } => up::run_up(up::UpOptions {
            no_patrol,
            verbose: cli.verbose,
            force,
            auto,
            task_pool_command,
            concurrency,
            post_accept_command,
        }),
        Commands::Overseer => overseer::run_overseer(),
        Commands::Down { force, kill_consoles, json } => down::run_down(force, kill_consoles, json),
        Commands::Add { name, model, role_prompt, excluded_from_pool, self_review, json } => {
            add::run_add(&name, model, role_prompt, excluded_from_pool, self_review, json)
        }
        Commands::Nuke { name, all, json } => nuke::run_nuke(name.as_deref(), all, json),
        Commands::Status { json } => status::run_status(json),
        Commands::Start { worker, prefix, prompt, prompt_file, prompt_cmd, self_review, json } => {
            start::run_start(worker, prefix, prompt, prompt_file, prompt_cmd, self_review, json)
        }
        Commands::Message { worker, message, json } => message::run_message(&worker, message, json),
        Commands::Attach { target } => attach::run_attach(&target),
        Commands::Console { name } => console::run_console(name.as_deref()),
        Commands::Review { worker, interface, name_only, force, json } => {
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
            review::run_review(worker, interface_enum, name_only, force, json)
        }
        Commands::Reject { message, json } => reject::run_reject(message, json),
        Commands::Accept { worker, force, json } => accept::run_accept(worker, force, json),
        Commands::Rebase { worker, json } => rebase::run_rebase(&worker, json),
        Commands::Reset { worker, all, yes, json } => reset::run_reset(worker, all, yes, json),
        Commands::Doctor { repair, yes, rebuild, json } => {
            doctor::run_doctor(repair, yes, rebuild, json)
        }
        Commands::Peek { worker, lines, json } => peek::run_peek(worker, lines, json),
        Commands::Pick { worker, json } => pick::run_pick(&worker, json),
        Commands::Config { action } => match action {
            ConfigAction::Get { key } => config_cmd::run_config_get(&key),
            ConfigAction::Set { key, value } => config_cmd::run_config_set(&key, &value),
        },
        Commands::Schema => {
            json_output::print_json_schema();
            Ok(())
        }
        Commands::Hook { action } => handle_hook_action(action).await,
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

async fn handle_hook_action(action: HookAction) -> Result<()> {
    match action {
        HookAction::Stop { worker } => hook::run_hook_stop(&worker).await,
        HookAction::SessionStart { worker } => hook::run_hook_session_start(&worker).await,
        HookAction::SessionEnd { worker, reason } => {
            hook::run_hook_session_end(&worker, &reason).await
        }
    }
}

fn display_error(error: &anyhow::Error, verbose: bool) {
    if verbose {
        eprintln!("Error: {error:?}");
    } else {
        eprintln!("Error: {error}");
    }
}
