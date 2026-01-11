use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};

use super::super::config;
use super::super::state::{State, WorkerStatus};
use super::super::tmux::sender::TmuxSender;

/// Runs the message command, sending a follow-up message to a worker
pub fn run_message(worker: &str, message: &str) -> Result<()> {
    let llmc_root = config::get_llmc_root();
    if !llmc_root.exists() {
        bail!(
            "LLMC workspace not initialized. Run 'llmc init' first.\n\
             Expected workspace at: {}",
            llmc_root.display()
        );
    }

    let (mut state, _config) = super::super::state::load_state_with_patrol()?;

    let worker_record = state.get_worker(worker).ok_or_else(|| {
        anyhow::anyhow!(
            "Worker '{}' not found\n\
             Available workers: {}",
            worker,
            format_all_workers(&state)
        )
    })?;

    verify_valid_state_for_message(worker, worker_record.status)?;

    println!("Sending message to worker '{}'...", worker);

    let tmux_sender = TmuxSender::new();
    tmux_sender
        .send(&worker_record.session_id, message)
        .with_context(|| format!("Failed to send message to worker '{}'", worker))?;

    let worker_mut = state.get_worker_mut(worker).unwrap();
    worker_mut.last_activity_unix = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    state.save(&super::super::state::get_state_path())?;

    println!("✓ Message sent to worker '{}'", worker);

    Ok(())
}

fn verify_valid_state_for_message(worker: &str, status: WorkerStatus) -> Result<()> {
    match status {
        WorkerStatus::Working
        | WorkerStatus::NeedsReview
        | WorkerStatus::Rejected
        | WorkerStatus::Rebasing
        | WorkerStatus::Error => Ok(()),
        _ => {
            bail!(
                "Worker '{}' is in state {:?}, which cannot receive messages\n\
                 Valid states for messaging: Working, NeedsReview, Rejected, Rebasing, Error",
                worker,
                status
            )
        }
    }
}

fn format_all_workers(state: &State) -> String {
    if state.workers.is_empty() {
        return "none".to_string();
    }
    state.workers.keys().map(String::as_str).collect::<Vec<_>>().join(", ")
}
