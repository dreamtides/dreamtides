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
pub fn run_pick(worker: &str, json: bool) -> Result<()> {
    let llmc_root = config::get_llmc_root();
    if !llmc_root.exists() {
        bail!(
            "LLMC workspace not initialized. Run 'llmc init' first.\n\
             Expected workspace at: {}",
            llmc_root.display()
        );
    }

    let (state, config) = state::load_state_with_patrol()?;

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

    // Get commit message and strip agent attribution
    let commit_message = git::get_commit_message(&worktree_path, "HEAD")?;
    let cleaned_message = git::strip_agent_attribution(&commit_message);

    println!("Squashing commits...");
    let base_commit = "origin/master";
    git::squash_commits(&worktree_path, base_commit)?;

    // Check if there are any changes to commit after the squash
    if !git::has_staged_changes(&worktree_path)? {
        println!("\n✓ Worker's changes already incorporated into master");
        println!("  No new changes to pick - another worker likely made the same changes");
        return Ok(());
    }

    println!("Creating squashed commit...");
    create_commit(&worktree_path, &cleaned_message)?;

    let final_commit = git::get_head_commit(&worktree_path)?;

    // Merge the rebased changes into llmc master
    println!("Syncing local master with origin/master...");
    git::checkout_branch(&llmc_root, "master")?;
    git::reset_to_ref(&llmc_root, "origin/master")?;
    let master_before = git::get_head_commit(&llmc_root)?;

    println!("Merging to master...");
    git::fast_forward_merge(&llmc_root, &worker_record.branch)?;
    let master_after = git::get_head_commit(&llmc_root)?;

    if master_before == master_after {
        bail!(
            "Master branch was not updated after merge. This should not happen.\n\
             Before: {}\n\
             After: {}",
            master_before,
            master_after
        );
    }

    if master_after != final_commit {
        bail!(
            "Master HEAD ({}) does not match worker commit ({}). This should not happen.",
            master_after,
            final_commit
        );
    }

    println!(
        "✓ Master updated: {} -> {}",
        &master_before[..7.min(master_before.len())],
        &master_after[..7.min(master_after.len())]
    );

    // Fetch the commit into the source repository
    println!("Fetching commit into source repository...");
    let source_repo = PathBuf::from(&config.repo.source);
    git::fetch_from_local(&source_repo, &llmc_root, &final_commit)?;

    println!("Updating source repository...");
    git::checkout_branch(&source_repo, "master")?;

    if git::has_uncommitted_changes(&source_repo)? {
        bail!(
            "The master branch in source repository has uncommitted changes.\n\
             This would result in data loss. Please commit or stash your changes first.\n\
             Repository: {}",
            source_repo.display()
        );
    }

    git::reset_to_ref(&source_repo, &final_commit)?;

    let source_head = git::get_head_commit(&source_repo)?;
    if source_head != final_commit {
        bail!(
            "Source repository HEAD ({}) does not match new commit ({})",
            source_head,
            final_commit
        );
    }

    println!("✓ Source repository updated to {}", &final_commit[..7.min(final_commit.len())]);

    if json {
        let output = crate::json_output::PickOutput {
            worker: worker.to_string(),
            success: true,
            commit_sha: Some(final_commit.clone()),
        };
        crate::json_output::print_json(&output);
    } else {
        println!("\n✓ Successfully picked changes from worker '{}'", worker);
        println!("  Final commit: {}", &final_commit[..7.min(final_commit.len())]);
        println!("  Branch rebased onto origin/master and merged to source repository");
    }

    Ok(())
}

fn format_all_workers(state: &State) -> String {
    if state.workers.is_empty() {
        return "none".to_string();
    }
    state.workers.keys().map(String::as_str).collect::<Vec<_>>().join(", ")
}

fn create_commit(worktree_path: &PathBuf, message: &str) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(worktree_path)
        .arg("commit")
        .arg("-m")
        .arg(message)
        .output()
        .context("Failed to execute git commit")?;

    if !output.status.success() {
        bail!("Failed to create commit: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}
