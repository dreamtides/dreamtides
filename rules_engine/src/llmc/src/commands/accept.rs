use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use super::super::state::{State, WorkerStatus};
use super::super::tmux::sender::TmuxSender;
use super::super::worker::{self, WorkerTransition};
use super::super::{config, git};
use super::{rebase, review};

/// Runs the accept command, accepting a worker's changes and merging to master
pub fn run_accept(worker: Option<String>) -> Result<()> {
    let llmc_root = config::get_llmc_root();
    if !llmc_root.exists() {
        bail!(
            "LLMC workspace not initialized. Run 'llmc init' first.\n\
             Expected workspace at: {}",
            llmc_root.display()
        );
    }

    let (mut state, _config) = super::load_state_with_patrol()?;

    let worker_name = if let Some(name) = worker {
        if state.get_worker(&name).is_none() {
            bail!(
                "Worker '{}' not found\n\
                 Available workers: {}",
                name,
                format_all_workers(&state)
            );
        }
        name
    } else {
        review::load_last_reviewed()?.ok_or_else(|| {
            anyhow::anyhow!("No worker specified and no previously reviewed worker found")
        })?
    };

    let worker_record = state.get_worker(&worker_name).unwrap();

    if worker_record.status != WorkerStatus::NeedsReview {
        bail!("Worker '{}' is in state {:?}, not needs_review", worker_name, worker_record.status);
    }

    let worktree_path = PathBuf::from(&worker_record.worktree_path);

    if git::has_uncommitted_changes(&worktree_path)? {
        println!("Worker '{}' has uncommitted changes, amending to commit...", worker_name);
        git::amend_uncommitted_changes(&worktree_path)?;
    }

    println!("Accepting changes from worker '{}'...", worker_name);

    git::fetch_origin(&llmc_root)?;

    println!("Rebasing onto origin/master...");
    let rebase_result = git::rebase_onto(&worktree_path, "origin/master")?;

    if !rebase_result.success {
        bail!(
            "Rebase failed with conflicts. Please resolve manually.\n\
             Conflicted files: {:?}\n\
             To resolve:\n\
             1. cd {}\n\
             2. Resolve conflicts\n\
             3. git rebase --continue\n\
             4. Try 'llmc accept {}' again",
            rebase_result.conflicts,
            worktree_path.display(),
            worker_name
        );
    }

    println!("Squashing commits...");
    let base_commit = "origin/master";
    git::squash_commits(&worktree_path, base_commit)?;

    let commit_message = git::get_commit_message(&worktree_path, "HEAD")?;
    let cleaned_message = git::strip_agent_attribution(&commit_message);

    println!("Amending commit message...");
    amend_commit_message(&worktree_path, &cleaned_message)?;

    let new_commit_sha = git::get_head_commit(&worktree_path)?;

    println!("Merging to master...");
    git::fast_forward_merge(&llmc_root, &worker_record.branch)?;

    println!("Cleaning up worktree and branch...");
    git::remove_worktree(&llmc_root, &worktree_path, false)?;
    git::delete_branch(&llmc_root, &worker_record.branch, true)?;

    println!("Recreating worker worktree...");
    let branch_name = format!("llmc/{}", worker_name);
    git::create_branch(&llmc_root, &branch_name, "origin/master")?;
    git::create_worktree(&llmc_root, &branch_name, &worktree_path)?;
    copy_tabula_to_worktree(&llmc_root, &worktree_path)?;

    let worker_mut = state.get_worker_mut(&worker_name).unwrap();
    worker::apply_transition(worker_mut, WorkerTransition::ToIdle)?;

    state.save(&super::super::state::get_state_path())?;

    println!("✓ Worker '{}' changes accepted!", worker_name);
    println!("  New commit: {}", &new_commit_sha[..7.min(new_commit_sha.len())]);
    println!("  Worker reset to idle state with fresh worktree");

    let other_pending: Vec<_> = state
        .workers
        .iter()
        .filter(|(name, w)| *name != &worker_name && w.status == WorkerStatus::NeedsReview)
        .map(|(name, _)| name.clone())
        .collect();

    if !other_pending.is_empty() {
        tracing::info!("Triggering background rebases for {} pending workers", other_pending.len());

        for pending_worker in other_pending {
            let pending_record = state.get_worker(&pending_worker).unwrap();
            let pending_worktree = PathBuf::from(&pending_record.worktree_path);
            let pending_session = pending_record.session_id.clone();

            if let Ok(true) = git::has_uncommitted_changes(&pending_worktree) {
                tracing::info!(
                    "Worker '{}' has uncommitted changes, amending before rebase",
                    pending_worker
                );
                if let Err(e) = git::amend_uncommitted_changes(&pending_worktree) {
                    tracing::warn!("Failed to amend changes for {}: {}", pending_worker, e);
                    continue;
                }
            }

            match git::rebase_onto(&pending_worktree, "origin/master") {
                Ok(rebase_result) => {
                    if rebase_result.success {
                        tracing::info!("Successfully rebased worker '{}'", pending_worker);
                    } else {
                        tracing::info!(
                            "Worker '{}' has conflicts, marking as rebasing",
                            pending_worker
                        );
                        let worker_mut = state.get_worker_mut(&pending_worker).unwrap();
                        if let Err(e) =
                            worker::apply_transition(worker_mut, WorkerTransition::ToRebasing)
                        {
                            tracing::warn!("Failed to transition worker to rebasing: {}", e);
                            continue;
                        }

                        let conflict_prompt =
                            rebase::build_conflict_prompt(&rebase_result.conflicts);
                        let tmux_sender = TmuxSender::new();
                        if let Err(e) = tmux_sender.send(&pending_session, &conflict_prompt) {
                            tracing::warn!(
                                "Failed to send conflict prompt to worker '{}': {}",
                                pending_worker,
                                e
                            );
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Background rebase failed for {}: {}", pending_worker, e);
                }
            }
        }

        state.save(&super::super::state::get_state_path())?;
    }

    Ok(())
}

fn amend_commit_message(worktree_path: &Path, message: &str) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(worktree_path)
        .arg("commit")
        .arg("--amend")
        .arg("-m")
        .arg(message)
        .output()
        .context("Failed to execute git commit --amend")?;

    if !output.status.success() {
        bail!("Failed to amend commit: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

fn copy_tabula_to_worktree(source_root: &Path, worktree_path: &Path) -> Result<()> {
    let source_file = source_root.join("Tabula.xlsm");
    let dest_file = worktree_path.join("Tabula.xlsm");

    if !source_file.exists() {
        return Ok(());
    }

    fs::copy(&source_file, &dest_file).with_context(|| {
        format!(
            "Failed to copy Tabula.xlsm from {} to {}",
            source_file.display(),
            dest_file.display()
        )
    })?;

    Ok(())
}

fn format_all_workers(state: &State) -> String {
    if state.workers.is_empty() {
        return "none".to_string();
    }
    state.workers.keys().map(String::as_str).collect::<Vec<_>>().join(", ")
}
