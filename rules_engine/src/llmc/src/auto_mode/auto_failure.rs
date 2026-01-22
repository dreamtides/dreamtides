#![allow(dead_code, reason = "Some types and variants are defined for future use by overseer")]

use anyhow::Result;
use tracing::{error, info};

use crate::auto_mode::auto_workers;
use crate::state::{State, WorkerRecord, WorkerStatus};
use crate::tmux::session;

/// Maximum number of retry attempts for transient failures before escalating
/// to a hard failure.
pub const MAX_RETRY_ATTEMPTS: u32 = 2;

/// Categorizes failure types in auto mode.
///
/// The two-tier system distinguishes between transient failures that patrol
/// can attempt to recover from, and hard failures that require immediate
/// shutdown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutoFailureKind {
    /// Transient failures that patrol may be able to recover from with retries.
    Transient(TransientFailure),
    /// Hard failures that require immediate daemon shutdown.
    Hard(HardFailure),
}

/// Transient failures that patrol attempts to recover automatically.
///
/// These failures are typically recoverable by restarting sessions or
/// recreating worktrees. Patrol will retry recovery up to `MAX_RETRY_ATTEMPTS`
/// times before escalating to a hard failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransientFailure {
    /// Worker Claude Code session crashed (process died).
    SessionCrash { worker_name: String },
    /// TMUX session disappeared unexpectedly.
    TmuxSessionMissing { worker_name: String },
}

/// Hard failures that trigger immediate daemon shutdown.
///
/// These failures cannot be recovered by patrol and indicate systemic issues
/// that require human intervention or AI remediation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HardFailure {
    /// `task_pool_command` returned non-zero exit code.
    TaskPoolCommandFailed { message: String },
    /// `post_accept_command` returned non-zero exit code.
    PostAcceptCommandFailed { message: String },
    /// Worker entered error state after patrol retries exhausted.
    WorkerRetriesExhausted { worker_name: String, retry_count: u32 },
    /// Worker is in error state (may not have gone through retries).
    WorkerInErrorState { worker_name: String, error_reason: Option<String> },
    /// Worker not found in state during recovery attempt.
    WorkerNotFound { worker_name: String },
    /// Rebase failed during auto accept workflow.
    RebaseFailure { worker_name: String, message: String },
    /// State file corruption detected.
    StateCorruption { message: String },
    /// Hook IPC communication failure.
    HookIpcFailure { message: String },
    /// Recovery attempt failed with an error.
    RecoveryFailed { worker_name: String, error: String },
}

/// Result of attempting to recover from a transient failure.
#[derive(Debug)]
pub enum RecoveryResult {
    /// Recovery succeeded, worker can continue.
    Recovered,
    /// Recovery failed, should retry (if attempts remain).
    RetryNeeded,
    /// Escalate to hard failure (retries exhausted or unrecoverable).
    EscalateToHardFailure(HardFailure),
}

/// Detects transient failures for auto workers by checking session health.
///
/// Returns a list of detected transient failures that need recovery attempts.
pub fn detect_transient_failures(state: &State) -> Vec<TransientFailure> {
    let mut failures = Vec::new();

    for worker_name in &state.auto_workers {
        if let Some(worker) = state.get_worker(worker_name)
            && should_check_session_health(worker)
            && !session::session_exists(&worker.session_id)
        {
            info!(
                worker = %worker_name,
                status = ?worker.status,
                "Detected missing TMUX session for auto worker"
            );
            failures
                .push(TransientFailure::TmuxSessionMissing { worker_name: worker_name.clone() });
        }
    }

    failures
}

/// Attempts to recover from a transient failure.
///
/// Returns the recovery result indicating success, retry needed, or escalation.
pub fn attempt_recovery(
    failure: &TransientFailure,
    state: &mut State,
    config: &crate::config::Config,
) -> Result<RecoveryResult> {
    let worker_name = match failure {
        TransientFailure::SessionCrash { worker_name }
        | TransientFailure::TmuxSessionMissing { worker_name } => worker_name,
    };

    let Some(worker) = state.get_worker_mut(worker_name) else {
        error!(worker = %worker_name, "Worker not found during recovery attempt");
        return Ok(RecoveryResult::EscalateToHardFailure(HardFailure::WorkerNotFound {
            worker_name: worker_name.clone(),
        }));
    };

    let current_retries = worker.auto_retry_count;
    worker.auto_retry_count += 1;

    if current_retries >= MAX_RETRY_ATTEMPTS {
        error!(
            worker = %worker_name,
            retry_count = current_retries,
            max_retries = MAX_RETRY_ATTEMPTS,
            "Worker retry attempts exhausted, escalating to hard failure"
        );
        return Ok(RecoveryResult::EscalateToHardFailure(HardFailure::WorkerRetriesExhausted {
            worker_name: worker_name.clone(),
            retry_count: current_retries,
        }));
    }

    info!(
        worker = %worker_name,
        attempt = current_retries + 1,
        max_attempts = MAX_RETRY_ATTEMPTS,
        failure = %failure,
        "Attempting recovery from transient failure"
    );

    match failure {
        TransientFailure::SessionCrash { .. } | TransientFailure::TmuxSessionMissing { .. } => {
            recover_missing_session(worker_name, state, config)
        }
    }
}

/// Resets the retry count for a worker after successful task completion.
///
/// Call this after a worker successfully completes a task to clear the retry
/// state and allow fresh retries for future transient failures.
pub fn reset_retry_count(state: &mut State, worker_name: &str) {
    if let Some(worker) = state.get_worker_mut(worker_name)
        && worker.auto_retry_count > 0
    {
        info!(
            worker = %worker_name,
            previous_count = worker.auto_retry_count,
            "Resetting auto retry count after successful completion"
        );
        worker.auto_retry_count = 0;
    }
}

/// Checks if any detected failures should escalate to hard failure.
///
/// This handles the case where patrol detects issues that indicate a hard
/// failure immediately (not transient).
pub fn check_for_hard_failures(state: &State) -> Option<HardFailure> {
    for worker_name in &state.auto_workers {
        if let Some(worker) = state.get_worker(worker_name)
            && worker.status == WorkerStatus::Error
        {
            return Some(HardFailure::WorkerInErrorState {
                worker_name: worker_name.clone(),
                error_reason: worker.error_reason.clone(),
            });
        }
    }
    None
}

impl std::fmt::Display for AutoFailureKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transient(t) => write!(f, "Transient failure: {}", t),
            Self::Hard(h) => write!(f, "Hard failure: {}", h),
        }
    }
}

impl std::fmt::Display for TransientFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SessionCrash { worker_name } => {
                write!(f, "Session crash for worker '{}'", worker_name)
            }
            Self::TmuxSessionMissing { worker_name } => {
                write!(f, "TMUX session missing for worker '{}'", worker_name)
            }
        }
    }
}

impl std::fmt::Display for HardFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TaskPoolCommandFailed { message } => {
                write!(f, "Task pool command failed: {}", message)
            }
            Self::PostAcceptCommandFailed { message } => {
                write!(f, "Post-accept command failed: {}", message)
            }
            Self::WorkerRetriesExhausted { worker_name, retry_count } => {
                write!(
                    f,
                    "Worker '{}' retries exhausted after {} attempts",
                    worker_name, retry_count
                )
            }
            Self::WorkerInErrorState { worker_name, error_reason } => {
                if let Some(reason) = error_reason {
                    write!(f, "Worker '{}' in error state: {}", worker_name, reason)
                } else {
                    write!(f, "Worker '{}' in error state", worker_name)
                }
            }
            Self::WorkerNotFound { worker_name } => {
                write!(f, "Worker '{}' not found in state", worker_name)
            }
            Self::RebaseFailure { worker_name, message } => {
                write!(f, "Rebase failure for worker '{}': {}", worker_name, message)
            }
            Self::StateCorruption { message } => {
                write!(f, "State corruption: {}", message)
            }
            Self::HookIpcFailure { message } => {
                write!(f, "Hook IPC failure: {}", message)
            }
            Self::RecoveryFailed { worker_name, error } => {
                write!(f, "Recovery failed for worker '{}': {}", worker_name, error)
            }
        }
    }
}

/// Returns true if we should check session health for this worker.
///
/// We check session health for workers that are in an active state where
/// we expect a session to be running.
fn should_check_session_health(worker: &WorkerRecord) -> bool {
    matches!(
        worker.status,
        WorkerStatus::Working
            | WorkerStatus::Reviewing
            | WorkerStatus::Rejected
            | WorkerStatus::Rebasing
    )
}

/// Recovers from a missing TMUX session by recreating it.
fn recover_missing_session(
    worker_name: &str,
    state: &mut State,
    config: &crate::config::Config,
) -> Result<RecoveryResult> {
    let Some(worker) = state.get_worker(worker_name) else {
        return Ok(RecoveryResult::RetryNeeded);
    };

    match auto_workers::start_auto_worker_session(worker, config) {
        Ok(()) => {
            info!(worker = %worker_name, "Successfully restarted TMUX session");
            if let Some(w) = state.get_worker_mut(worker_name) {
                w.auto_retry_count = 0;
            }
            Ok(RecoveryResult::Recovered)
        }
        Err(e) => {
            info!(
                worker = %worker_name,
                error = %e,
                "Failed to restart TMUX session, will retry"
            );
            Ok(RecoveryResult::RetryNeeded)
        }
    }
}
