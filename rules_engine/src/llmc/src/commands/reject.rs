use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::commands::review;
use crate::lock::StateLock;
use crate::state::WorkerStatus;
use crate::tmux::sender::TmuxSender;
use crate::{config, editor, git, worker};

/// Runs the reject command, sending feedback to the most recently reviewed
/// worker
pub fn run_reject(message: Option<String>, json: bool) -> Result<()> {
    let llmc_root = config::get_llmc_root();
    if !llmc_root.exists() {
        bail!(
            "LLMC workspace not initialized. Run 'llmc init' first.\n\
             Expected workspace at: {}",
            llmc_root.display()
        );
    }
    let _lock = StateLock::acquire()?;
    let (mut state, _config) = crate::state::load_state_with_patrol()?;
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
    let worktree_path = PathBuf::from(&worker_record.worktree_path);
    let feedback = match message {
        Some(m) if !m.trim().is_empty() => m,
        Some(_) => bail!("Rejection message cannot be empty"),
        None => {
            let diff = get_diff_for_editor(&worktree_path)?;
            let template = format!(
                "# Rejection feedback for worker '{}'\n\
                 # Enter your feedback above the diff. Lines starting with '#' will be ignored.\n\
                 # Save and close the editor to send, or leave empty to abort.\n\
                 #\n\
                 # The diff being reviewed is shown below for reference:\n\
                 #\n\
                 {}\n",
                worker_name,
                prefix_lines_with_hash(&diff)
            );
            editor::open_editor(Some(&template), "reject")?
        }
    };
    let rejection_message = format!(
        "Your changes have been reviewed and require modifications:\n\
         \n\
         {}\n\
         \n\
         Please address these issues and commit the fixes.\n\
         The original diff is still available in your worktree.",
        feedback
    );
    println!("Sending rejection feedback to worker '{}'...", worker_name);
    let tmux_sender = TmuxSender::new();
    tmux_sender
        .send(&worker_record.session_id, &rejection_message)
        .with_context(|| format!("Failed to send rejection message to worker '{}'", worker_name))?;
    let worker_mut = state.get_worker_mut(&worker_name).unwrap();
    worker::apply_transition(worker_mut, worker::WorkerTransition::ToRejected {
        feedback: feedback.clone(),
    })?;
    state.save(&crate::state::get_state_path())?;
    if json {
        let output = crate::json_output::RejectOutput {
            worker: worker_name.clone(),
            previous_status: "needs_review".to_string(),
            new_status: "rejected".to_string(),
        };
        crate::json_output::print_json(&output);
    } else {
        println!("✓ Rejection sent to worker '{}'", worker_name);
        println!("  Worker transitioned to rejected state");
    }
    Ok(())
}
fn get_diff_for_editor(worktree_path: &PathBuf) -> Result<String> {
    let current_branch = git::get_current_branch(worktree_path)?;
    let range = format!("origin/master...{}", current_branch);
    let output = Command::new("git")
        .arg("-C")
        .arg(worktree_path)
        .arg("diff")
        .arg(&range)
        .output()
        .context("Failed to execute git diff")?;
    if !output.status.success() {
        bail!("git diff failed for {}", range);
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
fn prefix_lines_with_hash(text: &str) -> String {
    text.lines().map(|line| format!("# {}", line)).collect::<Vec<_>>().join("\n")
}
