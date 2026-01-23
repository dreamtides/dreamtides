use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};

use crate::config::Config;
use crate::git;
use crate::state::{State, WorkerRecord, WorkerStatus};
/// Represents a state transition for a worker
#[derive(Debug, Clone, PartialEq)]
pub enum WorkerTransition {
    /// No state change
    None,
    /// Transition to idle state
    ToIdle,
    /// Transition to working state with the given prompt
    ToWorking { prompt: String, prompt_cmd: Option<String> },
    /// Transition to needs review state with the commit SHA
    ToNeedsReview { commit_sha: String },
    /// Transition to reviewing state (on_complete prompt sent)
    ToReviewing,
    /// Transition to rejected state with feedback
    ToRejected { feedback: String },
    /// Transition to rebasing state
    ToRebasing,
    /// Transition to error state with reason
    ToError { reason: String },
    /// Transition to offline state
    ToOffline,
    /// Transition to no_changes state (task completed without commits)
    ToNoChanges,
}
pub fn can_transition(from: &WorkerStatus, to: &WorkerStatus) -> bool {
    matches!(
        (from, to),
        (
            WorkerStatus::Idle | WorkerStatus::Rebasing | WorkerStatus::NoChanges,
            WorkerStatus::Working
        ) | (
            WorkerStatus::Idle
                | WorkerStatus::Working
                | WorkerStatus::Rejected
                | WorkerStatus::Rebasing
                | WorkerStatus::Reviewing,
            WorkerStatus::NeedsReview
        ) | (WorkerStatus::NeedsReview, WorkerStatus::Reviewing)
            | (
                WorkerStatus::Working
                    | WorkerStatus::NeedsReview
                    | WorkerStatus::Reviewing
                    | WorkerStatus::Rejected
                    | WorkerStatus::Offline
                    | WorkerStatus::NoChanges,
                WorkerStatus::Idle
            )
            | (
                WorkerStatus::NeedsReview | WorkerStatus::Reviewing | WorkerStatus::Rebasing,
                WorkerStatus::Rejected
            )
            | (WorkerStatus::Error, WorkerStatus::NeedsReview | WorkerStatus::Idle)
            | (WorkerStatus::Working | WorkerStatus::Rejected, WorkerStatus::NoChanges)
            | (_, WorkerStatus::Rebasing | WorkerStatus::Error | WorkerStatus::Offline)
    )
}
/// Applies a state transition to a worker record
pub fn apply_transition(worker: &mut WorkerRecord, transition: WorkerTransition) -> Result<()> {
    let old_status = worker.status;
    let new_status = match &transition {
        WorkerTransition::None => return Ok(()),
        WorkerTransition::ToIdle => WorkerStatus::Idle,
        WorkerTransition::ToWorking { prompt, prompt_cmd } => {
            worker.current_prompt = prompt.clone();
            worker.prompt_cmd = prompt_cmd.clone();
            WorkerStatus::Working
        }
        WorkerTransition::ToNeedsReview { commit_sha } => {
            worker.commit_sha = Some(commit_sha.clone());
            WorkerStatus::NeedsReview
        }
        WorkerTransition::ToRejected { .. } => WorkerStatus::Rejected,
        WorkerTransition::ToReviewing => WorkerStatus::Reviewing,
        WorkerTransition::ToRebasing => WorkerStatus::Rebasing,
        WorkerTransition::ToError { .. } => WorkerStatus::Error,
        WorkerTransition::ToOffline => WorkerStatus::Offline,
        WorkerTransition::ToNoChanges => WorkerStatus::NoChanges,
    };
    if !can_transition(&old_status, &new_status) {
        bail!(
            "Invalid transition for worker '{}': {:?} -> {:?}",
            worker.name,
            old_status,
            new_status
        );
    }
    match &transition {
        WorkerTransition::ToIdle => {
            let had_stale_state = !worker.current_prompt.is_empty()
                || worker.prompt_cmd.is_some()
                || worker.commit_sha.is_some()
                || worker.self_review;
            if had_stale_state {
                tracing::info!(
                    worker = % worker.name, had_prompt = ! worker.current_prompt
                    .is_empty(), prompt_cmd = ? worker.prompt_cmd, commit_sha = ? worker
                    .commit_sha, self_review = worker.self_review,
                    "Clearing stale worker state during transition to Idle"
                );
            }
            worker.current_prompt.clear();
            worker.prompt_cmd = None;
            worker.commit_sha = None;
            worker.self_review = false;
            worker.commits_first_detected_unix = None;
            worker.error_reason = None;
        }
        WorkerTransition::ToWorking { .. } => {
            worker.on_complete_sent_unix = None;
            worker.commits_first_detected_unix = None;
            worker.error_reason = None;
        }
        WorkerTransition::ToNeedsReview { .. } => {
            worker.error_reason = None;
        }
        WorkerTransition::ToNoChanges => {
            worker.commits_first_detected_unix = None;
            worker.error_reason = None;
        }
        WorkerTransition::ToError { reason } => {
            worker.error_reason = Some(reason.clone());
        }
        _ => {}
    }
    if matches!(
        transition,
        WorkerTransition::ToIdle
            | WorkerTransition::ToNeedsReview { .. }
            | WorkerTransition::ToNoChanges
    ) && (worker.crash_count > 0 || worker.api_error_count > 0)
    {
        tracing::info!(
            "Worker {} completed successfully, resetting error counts (crash={}, api_error={})",
            worker.name,
            worker.crash_count,
            worker.api_error_count
        );
        worker.crash_count = 0;
        worker.last_crash_unix = None;
        worker.api_error_count = 0;
        worker.last_api_error_unix = None;
    }
    worker.status = new_status;
    worker.last_activity_unix = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    tracing::info!(
        operation = "state_transition", worker = & worker.name, from_status = ?
        old_status, to_status = ? new_status, transition_type = ? transition, commit_sha
        = ? worker.commit_sha, "Worker state transition"
    );
    Ok(())
}
/// Resets a worker to clean idle state by discarding changes and resetting to
/// origin branch
pub fn reset_worker_to_clean_state(
    worker_name: &str,
    state: &mut State,
    config: &Config,
) -> Result<Vec<String>> {
    let mut actions = Vec::new();
    let worker = state
        .get_worker(worker_name)
        .with_context(|| format!("Worker '{}' not found", worker_name))?;
    let worktree_path = Path::new(&worker.worktree_path);
    if !worktree_path.exists() {
        bail!("Worktree does not exist for worker '{}'", worker_name);
    }
    if git::is_rebase_in_progress(worktree_path) {
        git::abort_rebase(worktree_path).context("Failed to abort rebase")?;
        actions.push(format!("Aborted in-progress rebase for worker '{}'", worker_name));
    }
    if git::has_uncommitted_changes(worktree_path)? {
        git::reset_to_ref(worktree_path, "HEAD").context("Failed to reset uncommitted changes")?;
        actions.push(format!("Discarded uncommitted changes for worker '{}'", worker_name));
    }
    let origin_branch = config.repo.origin_branch();
    git::reset_to_ref(worktree_path, &origin_branch).context("Failed to reset to origin branch")?;
    actions.push(format!("Reset worker '{}' to {}", worker_name, origin_branch));
    let worker_mut = state.get_worker_mut(worker_name).unwrap();
    worker_mut.current_prompt.clear();
    worker_mut.commit_sha = None;
    let old_status = worker_mut.status;
    if old_status != WorkerStatus::Idle {
        apply_transition(worker_mut, WorkerTransition::ToIdle)?;
        actions
            .push(format!("Transitioned worker '{}' from {:?} to Idle", worker_name, old_status));
    }
    Ok(actions)
}
