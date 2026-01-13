use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result, bail};

use super::super::state::{self, State};
use super::super::{config, git};

/// Runs the pick command, which grabs all changes from a worker and rebases
/// onto master This is a low-level failure mitigation command that:
/// 1. Adds all untracked files
/// 2. Stages everything
/// 3. Amends to the current commit
/// 4. Rebases onto master
pub fn run_pick(worker: &str) -> Result<()> {
    let llmc_root = config::get_llmc_root();
    if !llmc_root.exists() {
        bail!(
            "LLMC workspace not initialized. Run 'llmc init' first.\n\
             Expected workspace at: {}",
            llmc_root.display()
        );
    }

    let (state, _config) = state::load_state_with_patrol()?;

    let worker_record = state.get_worker(worker).ok_or_else(|| {
        anyhow::anyhow!(
            "Worker '{}' not found\n\
             Available workers: {}",
            worker,
            format_all_workers(&state)
        )
    })?;

    let worktree_path = PathBuf::from(&worker_record.worktree_path);

    if !worktree_path.exists() {
        bail!("Worker '{}' worktree does not exist at: {}", worker, worktree_path.display());
    }

    println!("Picking changes from worker '{}'...", worker);
    println!("Worktree: {}", worktree_path.display());

    // Check if there's a commit to work with
    let has_commit = git::has_commits_ahead_of(&worktree_path, "origin/master").unwrap_or(false);

    if !has_commit {
        println!("Worker has no commits ahead of master, creating initial commit...");

        // Stage all changes
        println!("Staging all changes...");
        let add_output = Command::new("git")
            .arg("-C")
            .arg(&worktree_path)
            .arg("add")
            .arg("-A")
            .output()
            .context("Failed to execute git add -A")?;

        if !add_output.status.success() {
            bail!("Failed to stage changes: {}", String::from_utf8_lossy(&add_output.stderr));
        }

        // Check if there are any changes to commit
        if !git::has_staged_changes(&worktree_path)? {
            bail!("No changes to pick from worker '{}'", worker);
        }

        // Create an initial commit
        println!("Creating initial commit...");
        let commit_output = Command::new("git")
            .arg("-C")
            .arg(&worktree_path)
            .arg("commit")
            .arg("-m")
            .arg("WIP: Changes from worker")
            .output()
            .context("Failed to execute git commit")?;

        if !commit_output.status.success() {
            bail!("Failed to create commit: {}", String::from_utf8_lossy(&commit_output.stderr));
        }

        println!("✓ Initial commit created");
    } else {
        // Add all untracked files
        println!("Adding all untracked files...");
        let add_output = Command::new("git")
            .arg("-C")
            .arg(&worktree_path)
            .arg("add")
            .arg("-A")
            .output()
            .context("Failed to execute git add -A")?;

        if !add_output.status.success() {
            bail!("Failed to add untracked files: {}", String::from_utf8_lossy(&add_output.stderr));
        }

        // Check if there are any changes to amend
        if git::has_staged_changes(&worktree_path)? || git::has_uncommitted_changes(&worktree_path)?
        {
            println!("Amending changes to current commit...");
            git::amend_uncommitted_changes(&worktree_path)?;
            println!("✓ Changes amended to commit");
        } else {
            println!("No uncommitted changes to amend");
        }
    }

    // Fetch latest master
    println!("Fetching latest master...");
    git::fetch_origin(&llmc_root)?;

    // Rebase onto origin/master
    println!("Rebasing onto origin/master...");
    let rebase_result = git::rebase_onto(&worktree_path, "origin/master")?;

    if !rebase_result.success {
        println!("\n⚠ Rebase has conflicts!");
        println!("Conflicting files:");
        for conflict in &rebase_result.conflicts {
            println!("  - {}", conflict);
        }
        println!("\nThe worker is now in a rebasing state.");
        println!("You can:");
        println!("  1. Attach to the worker and resolve conflicts: llmc attach {}", worker);
        println!("  2. Abort the rebase: cd {} && git rebase --abort", worktree_path.display());

        return Ok(());
    }

    let final_commit = git::get_head_commit(&worktree_path)?;
    println!("\n✓ Successfully picked changes from worker '{}'", worker);
    println!("  Final commit: {}", &final_commit[..7.min(final_commit.len())]);
    println!("  Branch rebased onto origin/master");

    Ok(())
}

fn format_all_workers(state: &State) -> String {
    if state.workers.is_empty() {
        return "none".to_string();
    }
    state.workers.keys().map(String::as_str).collect::<Vec<_>>().join(", ")
}
