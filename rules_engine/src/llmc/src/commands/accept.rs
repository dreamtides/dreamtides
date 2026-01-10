use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use super::super::state::{self, State, WorkerStatus};
use super::super::worker::{self, WorkerTransition};
use super::super::{config, git};
use super::review;

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

    let state_path = state::get_state_path();
    let mut state = State::load(&state_path)?;

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
        bail!(
            "Worker '{}' has uncommitted changes. Cannot accept.\n\
             Please review the worktree and resolve any issues.",
            worker_name
        );
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
    git::remove_worktree(&worktree_path)?;
    git::delete_branch(&llmc_root, &worker_record.branch, true)?;

    println!("Recreating worker worktree...");
    let branch_name = format!("llmc/{}", worker_name);
    git::create_branch(&llmc_root, &branch_name, "origin/master")?;
    git::create_worktree(&llmc_root, &branch_name, &worktree_path)?;
    copy_tabula_to_worktree(&llmc_root, &worktree_path)?;

    let worker_mut = state.get_worker_mut(&worker_name).unwrap();
    worker::apply_transition(worker_mut, WorkerTransition::ToIdle)?;

    state.save(&state_path)?;

    println!("✓ Worker '{}' changes accepted!", worker_name);
    println!("  New commit: {}", &new_commit_sha[..7.min(new_commit_sha.len())]);
    println!("  Worker reset to idle state with fresh worktree");

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
