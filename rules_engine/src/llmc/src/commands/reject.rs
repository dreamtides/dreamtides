use anyhow::{Context, Result, bail};

use super::super::state::WorkerStatus;
use super::super::tmux::sender::TmuxSender;
use super::super::{config, worker};
use super::review;

/// Runs the reject command, sending feedback to the most recently reviewed
/// worker
pub fn run_reject(message: &str) -> Result<()> {
    let llmc_root = config::get_llmc_root();
    if !llmc_root.exists() {
        bail!(
            "LLMC workspace not initialized. Run 'llmc init' first.\n\
             Expected workspace at: {}",
            llmc_root.display()
        );
    }

    let (mut state, _config) = super::load_state_with_patrol()?;

    let worker_name = review::load_last_reviewed()?.ok_or_else(|| {
        anyhow::anyhow!("No previously reviewed worker found. Use 'llmc review' first.")
    })?;

    let worker_record = state
        .get_worker(&worker_name)
        .ok_or_else(|| anyhow::anyhow!("Worker '{}' not found", worker_name))?;

    if worker_record.status != WorkerStatus::NeedsReview {
        bail!(
            "Worker '{}' is in state {:?}, not needs_review\n\
             Cannot reject a worker that is not awaiting review.",
            worker_name,
            worker_record.status
        );
    }

    let rejection_message = format!(
        "Your changes have been reviewed and require modifications:\n\
         \n\
         {}\n\
         \n\
         Please address these issues and commit the fixes.\n\
         The original diff is still available in your worktree.",
        message
    );

    println!("Sending rejection feedback to worker '{}'...", worker_name);

    let tmux_sender = TmuxSender::new();
    tmux_sender
        .send(&worker_record.session_id, &rejection_message)
        .with_context(|| format!("Failed to send rejection message to worker '{}'", worker_name))?;

    let worker_mut = state.get_worker_mut(&worker_name).unwrap();
    worker::apply_transition(worker_mut, worker::WorkerTransition::ToRejected {
        feedback: message.to_string(),
    })?;

    state.save(&super::super::state::get_state_path())?;

    println!("✓ Rejection sent to worker '{}'", worker_name);
    println!("  Worker transitioned to rejected state");

    Ok(())
}
