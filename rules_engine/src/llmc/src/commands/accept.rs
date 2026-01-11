use std::fs;
use std::io::Write;
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

    let commit_message = git::get_commit_message(&worktree_path, "HEAD")?;
    println!("\n=== Accept Summary ===");
    println!("Worker: {}", worker_name);
    println!("Branch: {}", worker_record.branch);
    println!("Commit message:\n{}", commit_message.trim());
    println!("======================\n");

    print!("Accept these changes and merge to master? [y/N]: ");
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if !matches!(input.trim().to_lowercase().as_str(), "y" | "yes") {
        println!("Accept cancelled.");
        return Ok(());
    }

    println!("\nAccepting changes from worker '{}'...", worker_name);

    git::fetch_origin(&llmc_root)?;

    println!("Rebasing onto origin/master...");
    let rebase_result = git::rebase_onto(&worktree_path, "origin/master")?;

    if !rebase_result.success {
        let worker_mut = state.get_worker_mut(&worker_name).unwrap();
        worker::apply_transition(worker_mut, WorkerTransition::ToRebasing)?;

        let conflict_prompt = build_conflict_resolution_prompt(&rebase_result.conflicts);
        let sender = super::super::tmux::sender::TmuxSender::new();
        sender.send(&worker_mut.session_id, &conflict_prompt)?;

        state.save(&super::super::state::get_state_path())?;

        println!("\n✓ Agent rebase started");
        println!("  Worker '{}' transitioned to 'rebasing' state", worker_name);
        println!("  The agent will resolve conflicts and continue the rebase");
        println!("  Run 'llmc accept {}' again once complete", worker_name);
        return Ok(());
    }

    let commit_message = git::get_commit_message(&worktree_path, "HEAD")?;
    let cleaned_message = git::strip_agent_attribution(&commit_message);

    println!("Squashing commits...");
    let base_commit = "origin/master";
    git::squash_commits(&worktree_path, base_commit)?;

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

    println!("Updating source repository...");
    let source_repo = PathBuf::from(&config.repo.source);
    git::checkout_branch(&source_repo, "master")?;
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
    git::remove_worktree(&llmc_root, &worktree_path, false)?;
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

fn build_conflict_resolution_prompt(conflicts: &[String]) -> String {
    let mut prompt = String::from(
        "A rebase onto master has encountered conflicts.\n\
         \n\
         Conflicting files:\n",
    );

    for file in conflicts {
        let conflict_count = count_conflict_markers(file);
        prompt.push_str(&format!("- {} ({} conflict markers)\n", file, conflict_count));
    }

    prompt.push_str(
        "\n\
         Resolution steps:\n\
         1. Examine conflict markers (<<<<<<, =======, >>>>>>>)\n\
         2. Decide how to resolve each conflict\n\
         3. Remove conflict markers\n\
         4. Stage resolved files: git add <file>\n\
         5. Continue rebase: git rebase --continue\n\
         6. Run validation: just review\n\
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
