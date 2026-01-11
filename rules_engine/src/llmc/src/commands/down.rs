use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail};

use super::super::config::{self, Config};
use super::super::state::{self, State, WorkerStatus};
use super::super::tmux::session;

/// Runs the down command, stopping all worker sessions
pub fn run_down(force: bool) -> Result<()> {
    let llmc_root = config::get_llmc_root();
    if !llmc_root.exists() {
        bail!(
            "LLMC workspace not initialized. Run 'llmc init' first.\n\
             Expected workspace at: {}",
            llmc_root.display()
        );
    }

    let config_path = config::get_config_path();
    let config = Config::load(&config_path)?;

    let state_path = state::get_state_path();
    let mut state = State::load(&state_path)?;

    println!("Stopping LLMC workers...");

    send_shutdown_to_workers(&config, &state, force)?;

    if !force {
        println!("Waiting for graceful exit...");
        thread::sleep(Duration::from_secs(5));
    }

    kill_remaining_sessions(&mut state, force)?;
    state.save(&state_path)?;

    println!("✓ All workers stopped");
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
                    eprintln!("Warning: Worker '{}' not found in config, skipping", worker_name);
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

    cleanup_orphaned_llmc_sessions(state)?;

    Ok(())
}

fn cleanup_orphaned_llmc_sessions(state: &State) -> Result<()> {
    let all_sessions = session::list_sessions()?;
    let tracked_session_ids: Vec<String> =
        state.workers.values().map(|w| w.session_id.clone()).collect();

    let orphaned_sessions: Vec<String> = all_sessions
        .into_iter()
        .filter(|s| s.starts_with("llmc-") && !tracked_session_ids.contains(s))
        .collect();

    if orphaned_sessions.is_empty() {
        return Ok(());
    }

    println!(
        "Found {} orphaned LLMC sessions (not tracked in state file), cleaning up...",
        orphaned_sessions.len()
    );

    for session_name in &orphaned_sessions {
        println!("  Killing orphaned session: {}", session_name);
        if let Err(e) = session::kill_session(session_name) {
            eprintln!("Warning: Failed to kill orphaned session '{}': {}", session_name, e);
        }
    }

    Ok(())
}
