use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::commands::review;
use crate::lock::StateLock;
use crate::state::{State, WorkerStatus};
use crate::worker::{self, WorkerTransition};
use crate::{config, git};

/// Runs the accept command, accepting a worker's changes and merging to master
pub fn run_accept(worker: Option<String>, force: bool, json: bool) -> Result<()> {
    let llmc_root = config::get_llmc_root();
    if !llmc_root.exists() {
        bail!(
            "LLMC workspace not initialized. Run 'llmc init' first.\n\
             Expected workspace at: {}",
            llmc_root.display()
        );
    }
    let _lock = StateLock::acquire()?;
    let (mut state, config) = super::super::state::load_state_with_patrol()?;
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
        if force {
            bail!("--force requires specifying a worker name");
        }
        review::load_last_reviewed()?.ok_or_else(|| {
            anyhow::anyhow!("No worker specified and no previously reviewed worker found")
        })?
    };
    let worker_record = state.get_worker(&worker_name).unwrap();
    if !force && worker_record.status != WorkerStatus::NeedsReview {
        bail!(
            "Worker '{}' is in state {:?}, not needs_review\n\
             Use --force to accept regardless of worker state.",
            worker_name,
            worker_record.status
        );
    }
    let worktree_path = PathBuf::from(&worker_record.worktree_path);
    if git::has_uncommitted_changes(&worktree_path)? {
        println!("Worker '{}' has uncommitted changes, amending to commit...", worker_name);
        git::amend_uncommitted_changes(&worktree_path)?;
    }
    let commit_message = git::get_commit_message(&worktree_path, "HEAD")?;
    println!("\n=== Accept Summary ===");
    println!("Worker: {}", worker_name);
    println!("Branch: {}", worker_record.branch);
    println!("Commit message:\n{}", commit_message.trim());
    println!("======================\n");
    println!("Accepting changes from worker '{}'...", worker_name);
    git::fetch_origin(&llmc_root)?;
    println!("Rebasing onto origin/master...");
    let rebase_result = git::rebase_onto(&worktree_path, "origin/master")?;
    if !rebase_result.success {
        let worker_mut = state.get_worker_mut(&worker_name).unwrap();
        let original_task = worker_mut.current_prompt.clone();
        worker::apply_transition(worker_mut, WorkerTransition::ToRebasing)?;
        let conflict_prompt =
            build_conflict_resolution_prompt(&rebase_result.conflicts, &original_task);
        let sender = super::super::tmux::sender::TmuxSender::new();
        sender.send(&worker_mut.session_id, &conflict_prompt)?;
        state.save(&super::super::state::get_state_path())?;
        if json {
            let output = crate::json_output::AcceptOutput {
                worker: worker_name.clone(),
                commit_sha: String::new(),
                commit_message: String::new(),
                status: "rebasing".to_string(),
                needs_conflict_resolution: true,
            };
            crate::json_output::print_json(&output);
        } else {
            println!("\n✓ Agent rebase started");
            println!("  Worker '{}' transitioned to 'rebasing' state", worker_name);
            println!("  The agent will resolve conflicts and continue the rebase");
            println!("  Run 'llmc accept {}' again once complete", worker_name);
        }
        return Ok(());
    }
    let commit_message = git::get_commit_message(&worktree_path, "HEAD")?;
    let cleaned_message = git::strip_agent_attribution(&commit_message);
    println!("Squashing commits...");
    let base_commit = "origin/master";
    git::squash_commits(&worktree_path, base_commit)?;
    if !git::has_staged_changes(&worktree_path)? {
        println!("\n✓ Worker's changes already incorporated into master");
        println!("  No new changes to merge - another worker likely made the same changes");
        println!("Cleaning up worktree and branch...");
        git::remove_worktree(&llmc_root, &worktree_path, true)?;
        git::delete_branch(&llmc_root, &worker_record.branch, true)?;
        println!("Syncing llmc repo with source...");
        git::fetch_origin(&llmc_root)?;
        println!("Recreating worker worktree...");
        let branch_name = format!("llmc/{}", worker_name);
        git::create_branch(&llmc_root, &branch_name, "origin/master")?;
        git::create_worktree(&llmc_root, &branch_name, &worktree_path)?;
        copy_tabula_to_worktree(&llmc_root, &worktree_path)?;
        let worker_mut = state.get_worker_mut(&worker_name).unwrap();
        worker::apply_transition(worker_mut, WorkerTransition::ToIdle)?;
        state.save(&super::super::state::get_state_path())?;
        if json {
            let output = crate::json_output::AcceptOutput {
                worker: worker_name.clone(),
                commit_sha: String::new(),
                commit_message: "No changes (already incorporated)".to_string(),
                status: "idle".to_string(),
                needs_conflict_resolution: false,
            };
            crate::json_output::print_json(&output);
        } else {
            println!("✓ Worker '{}' reset to idle state", worker_name);
        }
        return Ok(());
    }
    println!("Creating squashed commit...");
    create_commit(&worktree_path, &cleaned_message)?;
    let new_commit_sha = git::get_head_commit(&worktree_path)?;
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
    if master_after != new_commit_sha {
        bail!(
            "Master HEAD ({}) does not match worker commit ({}). This should not happen.",
            master_after,
            new_commit_sha
        );
    }
    println!(
        "✓ Master updated: {} -> {}",
        &master_before[..7.min(master_before.len())],
        &master_after[..7.min(master_after.len())]
    );
    verify_commit_exists(&llmc_root, &new_commit_sha)?;
    println!("Fetching commit into source repository...");
    let source_repo = PathBuf::from(&config.repo.source);
    git::fetch_from_local(&source_repo, &llmc_root, &new_commit_sha)?;
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
    git::reset_to_ref(&source_repo, &new_commit_sha)?;
    let source_head = git::get_head_commit(&source_repo)?;
    if source_head != new_commit_sha {
        bail!(
            "Source repository HEAD ({}) does not match new commit ({})",
            source_head,
            new_commit_sha
        );
    }
    println!("✓ Source repository updated to {}", &new_commit_sha[..7.min(new_commit_sha.len())]);
    println!("Cleaning up worktree and branch...");
    git::remove_worktree(&llmc_root, &worktree_path, true)?;
    git::delete_branch(&llmc_root, &worker_record.branch, true)?;
    println!("Syncing llmc repo with source...");
    git::fetch_origin(&llmc_root)?;
    println!("Recreating worker worktree...");
    let branch_name = format!("llmc/{}", worker_name);
    git::create_branch(&llmc_root, &branch_name, "origin/master")?;
    git::create_worktree(&llmc_root, &branch_name, &worktree_path)?;
    copy_tabula_to_worktree(&llmc_root, &worktree_path)?;
    let worker_mut = state.get_worker_mut(&worker_name).unwrap();
    worker::apply_transition(worker_mut, WorkerTransition::ToIdle)?;
    state.save(&super::super::state::get_state_path())?;
    if json {
        let output = crate::json_output::AcceptOutput {
            worker: worker_name.clone(),
            commit_sha: new_commit_sha.clone(),
            commit_message: cleaned_message.clone(),
            status: "idle".to_string(),
            needs_conflict_resolution: false,
        };
        crate::json_output::print_json(&output);
    } else {
        println!("✓ Worker '{}' changes accepted!", worker_name);
        println!("  New commit: {}", &new_commit_sha[..7.min(new_commit_sha.len())]);
        println!("  Worker reset to idle state with fresh worktree");
    }
    Ok(())
}
fn create_commit(worktree_path: &Path, message: &str) -> Result<()> {
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
fn copy_tabula_to_worktree(source_root: &Path, worktree_path: &Path) -> Result<()> {
    let source_file = source_root.join("Tabula.xlsm");
    let dest_file = worktree_path.join("client/Assets/StreamingAssets/Tabula.xlsm");
    if !source_file.exists() {
        return Ok(());
    }
    if dest_file.exists() {
        return Ok(());
    }
    if let Some(parent) = dest_file.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
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
fn verify_commit_exists(repo: &Path, commit_sha: &str) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("cat-file")
        .arg("-t")
        .arg(commit_sha)
        .output()
        .context("Failed to verify commit exists")?;
    if !output.status.success() {
        bail!("Commit {} does not exist in repository at {}", commit_sha, repo.display());
    }
    let obj_type = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if obj_type != "commit" {
        bail!("Object {} is a {}, not a commit", commit_sha, obj_type);
    }
    Ok(())
}
fn format_all_workers(state: &State) -> String {
    if state.workers.is_empty() {
        return "none".to_string();
    }
    state.workers.keys().map(String::as_str).collect::<Vec<_>>().join(", ")
}
fn build_conflict_resolution_prompt(conflicts: &[String], original_task: &str) -> String {
    let mut prompt = String::from(
        "A rebase onto master has encountered conflicts.\n\
         \n",
    );
    prompt
        .push_str(
            &format!(
                "IMPORTANT - Your original task:\n\
         \"{}\"\n\
         \n\
         DO NOT restart your task from scratch. Instead, INCORPORATE your existing changes/intent \n\
         into the new repository state. Your goal is to apply the same logical changes you already \n\
         made, but adapted to work with the new state of the files after master's changes.\n\
         \n",
                original_task.lines().take(3).collect::< Vec < _ >> ().join(" ")
            ),
        );
    prompt.push_str("Conflicting files:\n");
    for file in conflicts {
        let conflict_count = count_conflict_markers(file);
        prompt.push_str(&format!("- {} ({} conflict markers)\n", file, conflict_count));
    }
    prompt
        .push_str(
            "\n\
         Resolution steps:\n\
         1. Examine conflict markers (<<<<<<, =======, >>>>>>>)\n\
         2. Understand what master changed (their version) and what you changed (our version)\n\
         3. Decide how to INCORPORATE YOUR CHANGES into the new state - do NOT just accept theirs\n\
         4. Remove conflict markers and apply your intended changes\n\
         5. Stage resolved files: git add <file>\n\
         6. Continue rebase: git rebase --continue\n\
         7. Run validation: just review\n\
         8. IMPORTANT: If validation modified any files, amend them: git add -A && git commit --amend --no-edit\n\
         \n\
         Notes:\n\
         - View original versions: git show :2:<file> (ours) :3:<file> (theirs)\n\
         - To abort: git rebase --abort\n",
        );
    prompt
}
fn count_conflict_markers(file: &str) -> usize {
    std::fs::read_to_string(file).map(|content| content.matches("<<<<<<<").count()).unwrap_or(0)
}
