use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail};

use crate::commands::{console, overseer};
use crate::config::{self, Config};
use crate::overseer_mode::overseer_session;
use crate::state::{self, State, WorkerStatus};
use crate::tmux::session;
/// Runs the down command, stopping all worker sessions
pub fn run_down(
    force: bool,
    kill_consoles: bool,
    allow_overseer_managed: bool,
    json: bool,
) -> Result<()> {
    let llmc_root = config::get_llmc_root();
    if !llmc_root.exists() {
        bail!(
            "LLMC workspace not initialized. Run 'llmc init' first.\n\
             Expected workspace at: {}",
            llmc_root.display()
        );
    }

    // Check if an overseer is managing this instance
    if !allow_overseer_managed {
        if let Ok(Some(registration)) = overseer::read_overseer_registration() {
            if overseer::is_process_alive(registration.pid) {
                bail!(
                    "An overseer is managing this LLMC instance (PID: {}).\n\n\
                     Running 'llmc down' would disrupt autonomous operation.\n\n\
                     If you're running a test scenario that needs to stop workers,\n\
                     make sure LLMC_ROOT is set to your test instance, not the main instance.\n\n\
                     To force shutdown despite active overseer, use:\n\
                       llmc down --allow-overseer-managed [--force]",
                    registration.pid
                );
            }
        }
    }

    let config_path = config::get_config_path();
    let config = Config::load(&config_path)?;
    let state_path = state::get_state_path();
    let mut state = State::load(&state_path)?;
    if !json {
        println!("Stopping LLMC workers...");
    }
    let worker_names: Vec<String> = state.workers.keys().cloned().collect();
    send_shutdown_to_workers(&config, &state, force)?;
    if !force {
        if !json {
            println!("Waiting for graceful exit...");
        }
        thread::sleep(Duration::from_secs(5));
    }
    kill_remaining_sessions(&mut state, force)?;
    cleanup_orphaned_llmc_sessions(&state, kill_consoles, json)?;
    state.save(&state_path)?;
    if json {
        let output = crate::json_output::DownOutput { workers_stopped: worker_names };
        crate::json_output::print_json(&output);
    } else {
        println!("✓ All workers stopped");
    }
    Ok(())
}
fn send_shutdown_to_workers(config: &Config, state: &State, force: bool) -> Result<()> {
    let worker_names: Vec<String> = state.workers.keys().cloned().collect();
    for worker_name in &worker_names {
        if let Some(worker_record) = state.workers.get(worker_name) {
            if worker_record.status == WorkerStatus::Offline {
                continue;
            }
            if !session::session_exists(&worker_record.session_id) {
                continue;
            }
            if force {
                println!("  Force-killing worker '{}'...", worker_name);
                session::kill_session(&worker_record.session_id)?;
            } else {
                println!("  Sending Ctrl-C to worker '{}'...", worker_name);
                if config.get_worker(worker_name).is_none() {
                    tracing::error!(
                        worker = %worker_name,
                        "Worker exists in state but not in config during shutdown - state/config \
                         mismatch. Run 'llmc doctor --repair' after restart to fix."
                    );
                    eprintln!(
                        "Warning: Worker '{}' not found in config.toml, skipping graceful shutdown",
                        worker_name
                    );
                    continue;
                }
                let sender = super::super::tmux::sender::TmuxSender::new();
                if let Err(e) = sender.send_keys_raw(&worker_record.session_id, "C-c") {
                    eprintln!("Warning: Failed to send Ctrl-C to worker '{}': {}", worker_name, e);
                }
            }
        }
    }
    Ok(())
}
fn kill_remaining_sessions(state: &mut State, force: bool) -> Result<()> {
    let worker_names: Vec<String> = state.workers.keys().cloned().collect();
    for worker_name in &worker_names {
        if let Some(worker_record) = state.workers.get_mut(worker_name) {
            if session::session_exists(&worker_record.session_id) {
                if force {
                    session::kill_session(&worker_record.session_id)
                        .context("Failed to kill session")?;
                } else {
                    println!(
                        "  Session '{}' still running after timeout, killing...",
                        worker_record.session_id
                    );
                    session::kill_session(&worker_record.session_id)
                        .context("Failed to kill session")?;
                }
            }
            worker_record.status = WorkerStatus::Offline;
        }
    }
    Ok(())
}
fn cleanup_orphaned_llmc_sessions(state: &State, kill_consoles: bool, json: bool) -> Result<()> {
    let all_sessions = session::list_sessions()?;
    let tracked_session_ids: Vec<String> =
        state.workers.values().map(|w| w.session_id.clone()).collect();
    let orphaned_sessions: Vec<String> = all_sessions
        .into_iter()
        .filter(|s| {
            if !s.starts_with(&config::get_session_prefix_pattern()) {
                return false;
            }
            if tracked_session_ids.contains(s) {
                return false;
            }
            if !kill_consoles && s.starts_with(&console::get_console_prefix()) {
                return false;
            }
            if overseer_session::is_overseer_session(s) {
                tracing::info!(session = %s, "Preserving overseer session during cleanup");
                return false;
            }
            true
        })
        .collect();
    let preserved_consoles: Vec<String> = if !kill_consoles {
        session::list_sessions()?
            .into_iter()
            .filter(|s| s.starts_with(&console::get_console_prefix()))
            .collect()
    } else {
        vec![]
    };
    if orphaned_sessions.is_empty() {
        if !preserved_consoles.is_empty() && !json {
            println!(
                "  {} console session(s) preserved (use --kill-consoles to stop them)",
                preserved_consoles.len()
            );
        }
        return Ok(());
    }
    if !json {
        println!(
            "Found {} orphaned LLMC sessions (not tracked in state file), cleaning up...",
            orphaned_sessions.len()
        );
    }
    for session_name in &orphaned_sessions {
        if !json {
            println!("  Killing orphaned session: {}", session_name);
        }
        if let Err(e) = session::kill_session(session_name) {
            eprintln!("Warning: Failed to kill orphaned session '{}': {}", session_name, e);
        }
    }
    if !preserved_consoles.is_empty() && !json {
        println!(
            "  {} console session(s) preserved (use --kill-consoles to stop them)",
            preserved_consoles.len()
        );
    }
    Ok(())
}
