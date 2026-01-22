use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use anyhow::{Context, Result, bail};
use tracing::{debug, error, info};

use crate::auto_mode::auto_config::AutoConfig;
use crate::auto_mode::auto_logging::{AutoLogger, CommandResult};
use crate::auto_mode::auto_workers;
use crate::commands::add;
use crate::config::Config;
use crate::state::{State, WorkerStatus};
use crate::tmux::sender::TmuxSender;
use crate::worker::{self, WorkerTransition};
use crate::{config, git, patrol};

#[derive(Debug)]
pub enum AutoAcceptResult {
    /// Worker changes successfully merged to master.
    Accepted { commit_sha: String },
    /// Worker had no changes, reset to idle.
    NoChanges,
    /// Source repository has uncommitted changes, retry later.
    SourceRepoDirty,
    /// Rebase conflict occurred, worker is now in Rebasing state resolving it.
    RebaseConflict { conflicts: Vec<String> },
    /// Worker changes were accepted but cleanup (worktree reset) failed.
    /// The daemon should continue processing other workers; this worker may
    /// need manual cleanup.
    AcceptedWithCleanupFailure { commit_sha: String, cleanup_error: String },
}

/// Error during auto accept that should trigger daemon shutdown.
#[derive(Debug)]
pub struct AutoAcceptError {
    pub worker_name: String,
    pub message: String,
}

/// Accepts a worker's completed changes automatically, without human review.
///
/// This function handles the entire accept workflow for auto mode:
/// 1. Verifies worker is in `needs_review` or `no_changes` state
/// 2. For `no_changes`: resets worker to idle
/// 3. For `needs_review`: rebases, squashes, strips attribution, merges to
///    master
/// 4. Updates stall detection timestamp
///
/// # Design: Separation of Critical and Cleanup Operations
///
/// This function distinguishes between **critical operations** (merging commits
/// to master) and **cleanup operations** (resetting the worker worktree). If
/// the critical operation succeeds but cleanup fails, we return
/// `AcceptedWithCleanupFailure` rather than an error. This ensures the daemon
/// continues processing other workers rather than crashing.
///
/// This design prevents a single cleanup failure from halting the entire auto
/// mode pipeline. The worker may need manual cleanup, but the commit is safely
/// merged.
pub fn auto_accept_worker(
    worker_name: &str,
    state: &mut State,
    config: &Config,
    logger: &AutoLogger,
) -> Result<AutoAcceptResult, AutoAcceptError> {
    let worker = state.get_worker(worker_name).ok_or_else(|| AutoAcceptError {
        worker_name: worker_name.to_string(),
        message: "Worker not found".to_string(),
    })?;

    match worker.status {
        WorkerStatus::NoChanges => {
            info!(worker = %worker_name, "Worker completed with no changes, resetting to idle");
            reset_worker_to_idle(worker_name, state, config, logger)?;
            auto_workers::record_task_completion(state);
            Ok(AutoAcceptResult::NoChanges)
        }
        WorkerStatus::NeedsReview => accept_and_merge(worker_name, state, config, logger),
        status => Err(AutoAcceptError {
            worker_name: worker_name.to_string(),
            message: format!(
                "Worker in unexpected state {:?}, expected needs_review or no_changes",
                status
            ),
        }),
    }
}

/// Executes the post-accept command if configured.
///
/// Returns `Ok(())` if no command is configured or if command succeeds.
/// Returns `Err` if command fails (triggers daemon shutdown).
pub fn execute_post_accept_command(
    worker_name: &str,
    commit_sha: &str,
    auto_config: &AutoConfig,
    logger: &AutoLogger,
) -> Result<(), AutoAcceptError> {
    let Some(command) = &auto_config.post_accept_command else {
        debug!(
            worker = %worker_name,
            commit = %commit_sha,
            "No post_accept_command configured, skipping"
        );
        return Ok(());
    };

    info!(
        worker = %worker_name,
        commit = %commit_sha,
        command = %command,
        "Executing post-accept command"
    );

    let start = Instant::now();
    let output = Command::new("sh").arg("-c").arg(command).output();
    let duration = start.elapsed();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let exit_code = output.status.code().unwrap_or(-1);

            let cmd_result = CommandResult {
                command: command.clone(),
                exit_code,
                duration,
                stdout: stdout.clone(),
                stderr: stderr.clone(),
            };
            logger.log_post_accept(worker_name, commit_sha, &cmd_result);

            if output.status.success() {
                info!(
                    worker = %worker_name,
                    duration_ms = %duration.as_millis(),
                    "Post-accept command completed successfully"
                );
                Ok(())
            } else {
                error!(
                    worker = %worker_name,
                    exit_code = %exit_code,
                    stderr = %stderr,
                    "Post-accept command failed"
                );
                Err(AutoAcceptError {
                    worker_name: worker_name.to_string(),
                    message: format!(
                        "Post-accept command failed with exit code {}: {}",
                        exit_code,
                        stderr.lines().take(5).collect::<Vec<_>>().join(" ")
                    ),
                })
            }
        }
        Err(e) => {
            let error_msg = format!("Failed to execute post-accept command: {}", e);
            error!(worker = %worker_name, error = %e, "Post-accept command execution failed");
            let cmd_result = CommandResult {
                command: command.clone(),
                exit_code: -1,
                duration,
                stdout: String::new(),
                stderr: error_msg.clone(),
            };
            logger.log_post_accept(worker_name, commit_sha, &cmd_result);
            Err(AutoAcceptError { worker_name: worker_name.to_string(), message: error_msg })
        }
    }
}

/// Releases any task pool claims associated with the source repository.
///
/// This is called after a successful accept to release claims made by `lat
/// pop`. If `lat` is not available or the command fails, logs a warning but
/// does not fail the accept operation.
pub fn release_task_pool_claims(source_repo: &Path, logger: &AutoLogger) {
    let command = format!("lat claim --release-worktree {}", source_repo.display());
    info!(command = %command, "Releasing task pool claims");

    let start = Instant::now();
    let output = Command::new("lat")
        .args(["claim", "--release-worktree", &source_repo.display().to_string()])
        .output();
    let duration = start.elapsed();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let exit_code = output.status.code().unwrap_or(-1);

            let cmd_result = CommandResult {
                command: command.clone(),
                exit_code,
                duration,
                stdout: stdout.clone(),
                stderr: stderr.clone(),
            };
            logger.log_task_pool(&cmd_result);

            if output.status.success() {
                info!(
                    duration_ms = %duration.as_millis(),
                    stdout = %stdout.trim(),
                    "Task pool claims released successfully"
                );
            } else {
                debug!(
                    exit_code = %exit_code,
                    stderr = %stderr.trim(),
                    "Task pool claim release command returned non-zero (may not be using lattice)"
                );
            }
        }
        Err(e) => {
            debug!(
                error = %e,
                "lat command not found - skipping claim release (not using lattice task pool)"
            );
        }
    }
}

/// Releases all task pool claims on daemon shutdown.
///
/// This is called during daemon shutdown to ensure no claims are left dangling.
/// Uses `lat claim --release-all` to release all claims regardless of worktree.
pub fn release_all_task_pool_claims(logger: &AutoLogger) {
    let command = "lat claim --release-all";
    info!(command = %command, "Releasing all task pool claims on shutdown");

    let start = Instant::now();
    let output = Command::new("lat").args(["claim", "--release-all"]).output();
    let duration = start.elapsed();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let exit_code = output.status.code().unwrap_or(-1);

            let cmd_result = CommandResult {
                command: command.to_string(),
                exit_code,
                duration,
                stdout: stdout.clone(),
                stderr: stderr.clone(),
            };
            logger.log_task_pool(&cmd_result);

            if output.status.success() {
                info!(
                    duration_ms = %duration.as_millis(),
                    stdout = %stdout.trim(),
                    "All task pool claims released successfully"
                );
            } else {
                debug!(
                    exit_code = %exit_code,
                    stderr = %stderr.trim(),
                    "Task pool claim release-all command returned non-zero"
                );
            }
        }
        Err(e) => {
            debug!(
                error = %e,
                "lat command not found - skipping claim release"
            );
        }
    }
}

impl std::fmt::Display for AutoAcceptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Auto accept failed for worker '{}': {}", self.worker_name, self.message)
    }
}

impl std::error::Error for AutoAcceptError {}

/// Accepts and merges a worker's changes to master.
fn accept_and_merge(
    worker_name: &str,
    state: &mut State,
    config: &Config,
    logger: &AutoLogger,
) -> Result<AutoAcceptResult, AutoAcceptError> {
    let llmc_root = config::get_llmc_root();
    let worker = state.get_worker(worker_name).unwrap();
    let worktree_path = PathBuf::from(&worker.worktree_path);
    let branch = worker.branch.clone();

    // Check source repository for uncommitted changes EARLY, before any work.
    // This allows us to return SourceRepoDirty without modifying any state,
    // enabling clean retry with exponential backoff.
    let source_repo = PathBuf::from(&config.repo.source);
    if git::has_uncommitted_changes(&source_repo).map_err(|e| AutoAcceptError {
        worker_name: worker_name.to_string(),
        message: format!("Failed to check uncommitted changes in source repo: {e}"),
    })? {
        return Ok(AutoAcceptResult::SourceRepoDirty);
    }

    info!(worker = %worker_name, "Starting auto accept workflow");

    // Amend any uncommitted changes before proceeding
    if git::has_uncommitted_changes(&worktree_path).map_err(|e| AutoAcceptError {
        worker_name: worker_name.to_string(),
        message: format!("Failed to check uncommitted changes: {e}"),
    })? {
        info!(worker = %worker_name, "Amending uncommitted changes");
        git::amend_uncommitted_changes(&worktree_path).map_err(|e| AutoAcceptError {
            worker_name: worker_name.to_string(),
            message: format!("Failed to amend uncommitted changes: {e}"),
        })?;
    }

    // Fetch latest origin/master
    git::fetch_origin(&llmc_root).map_err(|e| AutoAcceptError {
        worker_name: worker_name.to_string(),
        message: format!("Failed to fetch origin: {e}"),
    })?;

    // Rebase onto origin/master
    info!(worker = %worker_name, "Rebasing onto origin/master");
    let rebase_result =
        git::rebase_onto(&worktree_path, "origin/master").map_err(|e| AutoAcceptError {
            worker_name: worker_name.to_string(),
            message: format!("Failed to rebase: {e}"),
        })?;

    if !rebase_result.success {
        // Transition worker to Rebasing state and send conflict prompt
        info!(
            worker = %worker_name,
            conflicts = ?rebase_result.conflicts,
            "Rebase conflict detected, transitioning worker to Rebasing state"
        );

        let worker = state.get_worker_mut(worker_name).ok_or_else(|| AutoAcceptError {
            worker_name: worker_name.to_string(),
            message: "Worker not found after rebase conflict".to_string(),
        })?;
        let session_id = worker.session_id.clone();
        let current_prompt = worker.current_prompt.clone();

        worker::apply_transition(worker, WorkerTransition::ToRebasing).map_err(|e| {
            AutoAcceptError {
                worker_name: worker_name.to_string(),
                message: format!("Failed to transition to Rebasing: {e}"),
            }
        })?;

        // Send conflict prompt to worker
        let conflict_prompt =
            patrol::build_conflict_prompt(&rebase_result.conflicts, &current_prompt);
        let sender = TmuxSender::new();
        sender.send(&session_id, &conflict_prompt).map_err(|e| AutoAcceptError {
            worker_name: worker_name.to_string(),
            message: format!("Failed to send conflict prompt: {e}"),
        })?;

        logger.log_accept_failure(
            worker_name,
            &format!("Rebase conflict - worker resolving: {}", rebase_result.conflicts.join(", ")),
        );
        return Ok(AutoAcceptResult::RebaseConflict { conflicts: rebase_result.conflicts });
    }

    // Get commit message and strip agent attribution
    let commit_message =
        git::get_commit_message(&worktree_path, "HEAD").map_err(|e| AutoAcceptError {
            worker_name: worker_name.to_string(),
            message: format!("Failed to get commit message: {e}"),
        })?;
    let cleaned_message = git::strip_agent_attribution(&commit_message);

    // Squash commits
    info!(worker = %worker_name, "Squashing commits");
    git::squash_commits(&worktree_path, "origin/master").map_err(|e| AutoAcceptError {
        worker_name: worker_name.to_string(),
        message: format!("Failed to squash commits: {e}"),
    })?;

    // Check if there are actually changes to merge
    if !git::has_staged_changes(&worktree_path).map_err(|e| AutoAcceptError {
        worker_name: worker_name.to_string(),
        message: format!("Failed to check staged changes: {e}"),
    })? {
        info!(worker = %worker_name, "Worker changes already incorporated into master");
        logger.log_accept_success(worker_name, "no-changes");
        reset_worker_to_idle(worker_name, state, config, logger)?;
        auto_workers::record_task_completion(state);
        return Ok(AutoAcceptResult::NoChanges);
    }

    // Create squashed commit
    create_commit(&worktree_path, &cleaned_message).map_err(|e| AutoAcceptError {
        worker_name: worker_name.to_string(),
        message: format!("Failed to create commit: {e}"),
    })?;

    let new_commit_sha = git::get_head_commit(&worktree_path).map_err(|e| AutoAcceptError {
        worker_name: worker_name.to_string(),
        message: format!("Failed to get HEAD commit: {e}"),
    })?;

    // Sync local master with origin/master
    git::checkout_branch(&llmc_root, "master").map_err(|e| AutoAcceptError {
        worker_name: worker_name.to_string(),
        message: format!("Failed to checkout master: {e}"),
    })?;
    git::reset_to_ref(&llmc_root, "origin/master").map_err(|e| AutoAcceptError {
        worker_name: worker_name.to_string(),
        message: format!("Failed to reset master to origin/master: {e}"),
    })?;

    // Fast-forward merge to master
    info!(worker = %worker_name, commit = %&new_commit_sha[..7.min(new_commit_sha.len())], "Merging to master");
    git::fast_forward_merge(&llmc_root, &branch).map_err(|e| AutoAcceptError {
        worker_name: worker_name.to_string(),
        message: format!("Failed to fast-forward merge: {e}"),
    })?;

    // Verify merge succeeded
    let master_after = git::get_head_commit(&llmc_root).map_err(|e| AutoAcceptError {
        worker_name: worker_name.to_string(),
        message: format!("Failed to get master HEAD after merge: {e}"),
    })?;

    if master_after != new_commit_sha {
        return Err(AutoAcceptError {
            worker_name: worker_name.to_string(),
            message: format!(
                "Master HEAD ({}) does not match worker commit ({})",
                master_after, new_commit_sha
            ),
        });
    }

    // Fetch commit into source repository
    let source_repo = PathBuf::from(&config.repo.source);
    git::fetch_from_local(&source_repo, &llmc_root, &new_commit_sha).map_err(|e| {
        AutoAcceptError {
            worker_name: worker_name.to_string(),
            message: format!("Failed to fetch commit into source repo: {e}"),
        }
    })?;

    // Update source repository
    // Note: We already checked for uncommitted changes at the start of this
    // function, so we can proceed directly to resetting.
    git::checkout_branch(&source_repo, "master").map_err(|e| AutoAcceptError {
        worker_name: worker_name.to_string(),
        message: format!("Failed to checkout master in source repo: {e}"),
    })?;

    git::reset_to_ref(&source_repo, &new_commit_sha).map_err(|e| AutoAcceptError {
        worker_name: worker_name.to_string(),
        message: format!("Failed to reset source repo to new commit: {e}"),
    })?;

    logger.log_accept_success(worker_name, &new_commit_sha);
    info!(
        worker = %worker_name,
        commit = %&new_commit_sha[..7.min(new_commit_sha.len())],
        "Successfully merged to master"
    );

    // Reset worker to idle with fresh worktree.
    // This is cleanup - if it fails, we should NOT crash the daemon since
    // the critical operation (merge to master) already succeeded.
    if let Err(e) = reset_worker_to_idle(worker_name, state, config, logger) {
        error!(
            worker = %worker_name,
            error = %e,
            "Worker cleanup failed after successful accept. \
             The commit was merged successfully but the worker may need manual cleanup."
        );
        auto_workers::record_task_completion(state);
        info!(
            worker = %worker_name,
            commit = %&new_commit_sha[..7.min(new_commit_sha.len())],
            "accept_and_merge returning AcceptedWithCleanupFailure result"
        );
        return Ok(AutoAcceptResult::AcceptedWithCleanupFailure {
            commit_sha: new_commit_sha,
            cleanup_error: e.message,
        });
    }

    auto_workers::record_task_completion(state);

    info!(
        worker = %worker_name,
        commit = %&new_commit_sha[..7.min(new_commit_sha.len())],
        "accept_and_merge returning Accepted result"
    );

    Ok(AutoAcceptResult::Accepted { commit_sha: new_commit_sha })
}

/// Resets a worker to idle state with a fresh worktree.
fn reset_worker_to_idle(
    worker_name: &str,
    state: &mut State,
    _config: &Config,
    logger: &AutoLogger,
) -> Result<(), AutoAcceptError> {
    let llmc_root = config::get_llmc_root();
    let worker = state.get_worker(worker_name).unwrap();
    let worktree_path = PathBuf::from(&worker.worktree_path);
    let branch = worker.branch.clone();
    let old_status = format!("{:?}", worker.status);

    // Remove and recreate worktree
    git::remove_worktree(&llmc_root, &worktree_path, true).map_err(|e| AutoAcceptError {
        worker_name: worker_name.to_string(),
        message: format!("Failed to remove worktree: {e}"),
    })?;
    git::delete_branch(&llmc_root, &branch, true).map_err(|e| AutoAcceptError {
        worker_name: worker_name.to_string(),
        message: format!("Failed to delete branch: {e}"),
    })?;

    // Sync llmc repo with source
    git::fetch_origin(&llmc_root).map_err(|e| AutoAcceptError {
        worker_name: worker_name.to_string(),
        message: format!("Failed to fetch origin: {e}"),
    })?;

    // Recreate worker worktree
    let branch_name = format!("llmc/{}", worker_name);
    git::create_branch(&llmc_root, &branch_name, "origin/master").map_err(|e| AutoAcceptError {
        worker_name: worker_name.to_string(),
        message: format!("Failed to create branch: {e}"),
    })?;
    git::create_worktree(&llmc_root, &branch_name, &worktree_path).map_err(|e| {
        AutoAcceptError {
            worker_name: worker_name.to_string(),
            message: format!("Failed to create worktree: {e}"),
        }
    })?;

    copy_tabula_to_worktree(&llmc_root, &worktree_path).map_err(|e| AutoAcceptError {
        worker_name: worker_name.to_string(),
        message: format!("Failed to copy Tabula: {e}"),
    })?;

    // Recreate Serena project config (ensures MCP tools work on the worktree, not
    // master)
    add::create_serena_project(&worktree_path, worker_name).map_err(|e| AutoAcceptError {
        worker_name: worker_name.to_string(),
        message: format!("Failed to create Serena project: {e}"),
    })?;

    // Recreate Claude hook settings
    add::create_claude_hook_settings_silent(&worktree_path, worker_name).map_err(|e| {
        AutoAcceptError {
            worker_name: worker_name.to_string(),
            message: format!("Failed to create Claude hook settings: {e}"),
        }
    })?;

    // Update worker state
    let worker_mut = state.get_worker_mut(worker_name).unwrap();
    worker::apply_transition(worker_mut, WorkerTransition::ToIdle).map_err(|e| {
        AutoAcceptError {
            worker_name: worker_name.to_string(),
            message: format!("Failed to transition worker to idle: {e}"),
        }
    })?;

    logger.log_worker_state_transition(worker_name, &old_status, "Idle");

    info!(worker = %worker_name, "Worker reset to idle completed successfully");

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
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }
    std::fs::copy(&source_file, &dest_file).with_context(|| {
        format!(
            "Failed to copy Tabula.xlsm from {} to {}",
            source_file.display(),
            dest_file.display()
        )
    })?;
    Ok(())
}
