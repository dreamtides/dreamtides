use std::path::Path;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};

use crate::config::{Config, WorkerConfig};
use crate::git;
use crate::state::{State, WorkerRecord, WorkerStatus};
use crate::tmux::sender::TmuxSender;
use crate::tmux::session;
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
}
pub fn can_transition(from: &WorkerStatus, to: &WorkerStatus) -> bool {
    matches!(
        (from, to),
        (WorkerStatus::Idle | WorkerStatus::Rebasing, WorkerStatus::Working)
            | (
                WorkerStatus::Idle
                    | WorkerStatus::Working
                    | WorkerStatus::Rejected
                    | WorkerStatus::Rebasing
                    | WorkerStatus::Reviewing,
                WorkerStatus::NeedsReview
            )
            | (WorkerStatus::NeedsReview, WorkerStatus::Reviewing)
            | (
                WorkerStatus::Working
                    | WorkerStatus::NeedsReview
                    | WorkerStatus::Reviewing
                    | WorkerStatus::Rejected
                    | WorkerStatus::Offline,
                WorkerStatus::Idle
            )
            | (
                WorkerStatus::NeedsReview | WorkerStatus::Reviewing | WorkerStatus::Rebasing,
                WorkerStatus::Rejected
            )
            | (WorkerStatus::Error, WorkerStatus::NeedsReview | WorkerStatus::Idle)
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
        }
        WorkerTransition::ToWorking { .. } => {
            worker.on_complete_sent_unix = None;
        }
        WorkerTransition::ToNeedsReview { .. } => {}
        _ => {}
    }
    if matches!(transition, WorkerTransition::ToIdle | WorkerTransition::ToNeedsReview { .. })
        && worker.crash_count > 0
    {
        tracing::info!(
            "Worker {} completed successfully, resetting crash count from {}",
            worker.name,
            worker.crash_count
        );
        worker.crash_count = 0;
        worker.last_crash_unix = None;
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
/// Starts Claude in a TMUX session with appropriate configuration
pub fn start_claude_in_session(session: &str, config: &WorkerConfig) -> Result<()> {
    let sender = TmuxSender::new();
    let mut claude_cmd = String::from("claude");
    if let Some(model) = &config.model {
        claude_cmd.push_str(&format!(" --model {}", model));
    }
    claude_cmd.push_str(" --dangerously-skip-permissions");
    sender
        .send(session, &claude_cmd)
        .with_context(|| format!("Failed to send Claude command to session '{}'", session))?;
    wait_for_claude_ready(session)?;
    accept_bypass_warning(session, &sender)?;
    sender
        .send(session, "/clear")
        .with_context(|| format!("Failed to send /clear to session '{}'", session))?;
    thread::sleep(Duration::from_millis(500));
    Ok(())
}
/// Resets a worker to clean idle state by discarding changes and resetting to
/// origin/master
pub fn reset_worker_to_clean_state(
    worker_name: &str,
    state: &mut State,
    _config: &Config,
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
    git::reset_to_ref(worktree_path, "origin/master")
        .context("Failed to reset to origin/master")?;
    actions.push(format!("Reset worker '{}' to origin/master", worker_name));
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
/// Waits for Claude to be ready by polling for the ">" prompt
fn wait_for_claude_ready(session: &str) -> Result<()> {
    const MAX_ATTEMPTS: u32 = 60;
    const POLL_INTERVAL_MS: u64 = 500;
    for _ in 0..MAX_ATTEMPTS {
        thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
        let output = session::capture_pane(session, 50)
            .with_context(|| format!("Failed to capture pane for session '{}'", session))?;
        if output.lines().rev().take(5).any(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("> ") || trimmed == ">" || trimmed.starts_with("❯")
        }) {
            return Ok(());
        }
        let lower = output.to_lowercase();
        if lower.contains("bypass") && lower.contains("permissions") {
            return Ok(());
        }
        let command = session::get_pane_command(session)?;
        if !session::is_claude_process(&command) {
            bail!("Claude process not found in session '{}', got command: {}", session, command);
        }
    }
    bail!("Claude did not become ready after 30 seconds");
}
/// Accepts the bypass permissions warning if present
fn accept_bypass_warning(session: &str, sender: &TmuxSender) -> Result<()> {
    thread::sleep(Duration::from_millis(500));
    let output = session::capture_pane(session, 50)
        .with_context(|| format!("Failed to capture pane for session '{}'", session))?;
    let lower = output.to_lowercase();
    let has_bypass_warning = lower.contains("bypass")
        || lower.contains("dangerously")
        || lower.contains("skip-permissions")
        || lower.contains("skip permissions");
    if has_bypass_warning {
        sender.send_keys_raw(session, "Down")?;
        thread::sleep(Duration::from_millis(200));
        sender.send_keys_raw(session, "Enter")?;
        thread::sleep(Duration::from_millis(500));
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use crate::llmc::worker::*;
    #[test]
    fn test_can_transition_idle_to_working() {
        assert!(can_transition(&WorkerStatus::Idle, &WorkerStatus::Working));
    }
    #[test]
    fn test_can_transition_working_to_needs_review() {
        assert!(can_transition(&WorkerStatus::Working, &WorkerStatus::NeedsReview));
    }
    #[test]
    fn test_can_transition_needs_review_to_idle() {
        assert!(can_transition(&WorkerStatus::NeedsReview, &WorkerStatus::Idle));
    }
    #[test]
    fn test_can_transition_needs_review_to_rejected() {
        assert!(can_transition(&WorkerStatus::NeedsReview, &WorkerStatus::Rejected));
    }
    #[test]
    fn test_can_transition_rejected_to_needs_review() {
        assert!(can_transition(&WorkerStatus::Rejected, &WorkerStatus::NeedsReview));
    }
    #[test]
    fn test_can_transition_rejected_to_idle() {
        assert!(can_transition(&WorkerStatus::Rejected, &WorkerStatus::Idle));
    }
    #[test]
    fn test_can_transition_any_to_rebasing() {
        assert!(can_transition(&WorkerStatus::Idle, &WorkerStatus::Rebasing));
        assert!(can_transition(&WorkerStatus::Working, &WorkerStatus::Rebasing));
        assert!(can_transition(&WorkerStatus::NeedsReview, &WorkerStatus::Rebasing));
    }
    #[test]
    fn test_can_transition_any_to_error() {
        assert!(can_transition(&WorkerStatus::Idle, &WorkerStatus::Error));
        assert!(can_transition(&WorkerStatus::Working, &WorkerStatus::Error));
        assert!(can_transition(&WorkerStatus::Rebasing, &WorkerStatus::Error));
    }
    #[test]
    fn test_can_transition_any_to_offline() {
        assert!(can_transition(&WorkerStatus::Idle, &WorkerStatus::Offline));
        assert!(can_transition(&WorkerStatus::Working, &WorkerStatus::Offline));
        assert!(can_transition(&WorkerStatus::Error, &WorkerStatus::Offline));
    }
    #[test]
    fn test_can_transition_idle_to_needs_review() {
        assert!(can_transition(&WorkerStatus::Idle, &WorkerStatus::NeedsReview));
    }
    #[test]
    fn test_cannot_transition_idle_to_rejected() {
        assert!(!can_transition(&WorkerStatus::Idle, &WorkerStatus::Rejected));
    }
    #[test]
    fn test_can_transition_working_to_idle() {
        assert!(can_transition(&WorkerStatus::Working, &WorkerStatus::Idle));
    }
    #[test]
    fn test_apply_transition_to_idle_clears_state() {
        let mut worker = WorkerRecord {
            name: "test".to_string(),
            worktree_path: "/tmp/test".to_string(),
            branch: "llmc/test".to_string(),
            status: WorkerStatus::NeedsReview,
            current_prompt: "Some prompt".to_string(),
            prompt_cmd: None,
            created_at_unix: 1000000000,
            last_activity_unix: 1000000000,
            commit_sha: Some("abc123".to_string()),
            session_id: "llmc-test".to_string(),
            crash_count: 0,
            last_crash_unix: None,
            on_complete_sent_unix: None,
            self_review: false,
        };
        apply_transition(&mut worker, WorkerTransition::ToIdle).unwrap();
        assert_eq!(worker.status, WorkerStatus::Idle);
        assert_eq!(worker.current_prompt, "");
        assert_eq!(worker.commit_sha, None);
    }
    #[test]
    fn test_apply_transition_to_working_sets_prompt() {
        let mut worker = WorkerRecord {
            name: "test".to_string(),
            worktree_path: "/tmp/test".to_string(),
            branch: "llmc/test".to_string(),
            status: WorkerStatus::Idle,
            current_prompt: String::new(),
            prompt_cmd: None,
            created_at_unix: 1000000000,
            last_activity_unix: 1000000000,
            commit_sha: None,
            session_id: "llmc-test".to_string(),
            crash_count: 0,
            last_crash_unix: None,
            on_complete_sent_unix: None,
            self_review: false,
        };
        apply_transition(&mut worker, WorkerTransition::ToWorking {
            prompt: "Test prompt".to_string(),
            prompt_cmd: None,
        })
        .unwrap();
        assert_eq!(worker.status, WorkerStatus::Working);
        assert_eq!(worker.current_prompt, "Test prompt");
    }
    #[test]
    fn test_apply_transition_to_needs_review_sets_commit_sha() {
        let mut worker = WorkerRecord {
            name: "test".to_string(),
            worktree_path: "/tmp/test".to_string(),
            branch: "llmc/test".to_string(),
            status: WorkerStatus::Working,
            current_prompt: "Test prompt".to_string(),
            prompt_cmd: None,
            created_at_unix: 1000000000,
            last_activity_unix: 1000000000,
            commit_sha: None,
            session_id: "llmc-test".to_string(),
            crash_count: 0,
            last_crash_unix: None,
            on_complete_sent_unix: None,
            self_review: false,
        };
        apply_transition(&mut worker, WorkerTransition::ToNeedsReview {
            commit_sha: "abc123".to_string(),
        })
        .unwrap();
        assert_eq!(worker.status, WorkerStatus::NeedsReview);
        assert_eq!(worker.commit_sha, Some("abc123".to_string()));
    }
    #[test]
    fn test_apply_transition_idle_to_needs_review() {
        let mut worker = WorkerRecord {
            name: "test".to_string(),
            worktree_path: "/tmp/test".to_string(),
            branch: "llmc/test".to_string(),
            status: WorkerStatus::Idle,
            current_prompt: String::new(),
            prompt_cmd: None,
            created_at_unix: 1000000000,
            last_activity_unix: 1000000000,
            commit_sha: None,
            session_id: "llmc-test".to_string(),
            crash_count: 0,
            last_crash_unix: None,
            on_complete_sent_unix: None,
            self_review: false,
        };
        apply_transition(&mut worker, WorkerTransition::ToNeedsReview {
            commit_sha: "abc123".to_string(),
        })
        .unwrap();
        assert_eq!(worker.status, WorkerStatus::NeedsReview);
        assert_eq!(worker.commit_sha, Some("abc123".to_string()));
    }
    #[test]
    fn test_apply_transition_none_does_nothing() {
        let mut worker = WorkerRecord {
            name: "test".to_string(),
            worktree_path: "/tmp/test".to_string(),
            branch: "llmc/test".to_string(),
            status: WorkerStatus::Working,
            current_prompt: "Test".to_string(),
            prompt_cmd: None,
            created_at_unix: 1000000000,
            last_activity_unix: 1000000000,
            commit_sha: None,
            session_id: "llmc-test".to_string(),
            crash_count: 0,
            last_crash_unix: None,
            on_complete_sent_unix: None,
            self_review: false,
        };
        let old_status = worker.status;
        apply_transition(&mut worker, WorkerTransition::None).unwrap();
        assert_eq!(worker.status, old_status);
    }
    #[test]
    fn test_can_transition_needs_review_to_reviewing() {
        assert!(can_transition(&WorkerStatus::NeedsReview, &WorkerStatus::Reviewing));
    }
    #[test]
    fn test_can_transition_reviewing_to_needs_review() {
        assert!(can_transition(&WorkerStatus::Reviewing, &WorkerStatus::NeedsReview));
    }
    #[test]
    fn test_can_transition_reviewing_to_idle() {
        assert!(can_transition(&WorkerStatus::Reviewing, &WorkerStatus::Idle));
    }
    #[test]
    fn test_can_transition_reviewing_to_rejected() {
        assert!(can_transition(&WorkerStatus::Reviewing, &WorkerStatus::Rejected));
    }
    #[test]
    fn test_apply_transition_to_reviewing() {
        let mut worker = WorkerRecord {
            name: "test".to_string(),
            worktree_path: "/tmp/test".to_string(),
            branch: "llmc/test".to_string(),
            status: WorkerStatus::NeedsReview,
            current_prompt: "Test prompt".to_string(),
            prompt_cmd: None,
            created_at_unix: 1000000000,
            last_activity_unix: 1000000000,
            commit_sha: Some("abc123".to_string()),
            session_id: "llmc-test".to_string(),
            crash_count: 0,
            last_crash_unix: None,
            on_complete_sent_unix: None,
            self_review: false,
        };
        apply_transition(&mut worker, WorkerTransition::ToReviewing).unwrap();
        assert_eq!(worker.status, WorkerStatus::Reviewing);
        assert_eq!(worker.commit_sha, Some("abc123".to_string()));
    }
}
