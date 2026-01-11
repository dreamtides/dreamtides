#![allow(dead_code)]

use std::path::PathBuf;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};

use crate::config::{Config, WorkerConfig};
use crate::state::{State, WorkerRecord, WorkerStatus};
use crate::tmux::sender::TmuxSender;
use crate::tmux::session;

/// Represents a live worker with its session and sender
pub struct Worker {
    pub name: String,
    pub session_id: String,
    pub worktree_path: PathBuf,
    pub sender: TmuxSender,
}

/// Represents a state transition for a worker
#[derive(Debug, Clone, PartialEq)]
pub enum WorkerTransition {
    /// No state change
    None,
    /// Transition to idle state
    ToIdle,
    /// Transition to working state with the given prompt
    ToWorking { prompt: String },
    /// Transition to needs input state
    ToNeedsInput,
    /// Transition to needs review state with the commit SHA
    ToNeedsReview { commit_sha: String },
    /// Transition to rejected state with feedback
    ToRejected { feedback: String },
    /// Transition to rebasing state
    ToRebasing,
    /// Transition to error state with reason
    ToError { reason: String },
    /// Transition to offline state
    ToOffline,
}

/// Initializes a worker, creating its session and starting Claude
pub fn initialize_worker(name: &str, config: &Config, state: &mut State) -> Result<Worker> {
    let worker_record =
        state.get_worker(name).with_context(|| format!("Worker '{}' not found in state", name))?;
    let worker_config = config
        .get_worker(name)
        .with_context(|| format!("Worker '{}' not found in config", name))?;
    start_claude_in_session(&worker_record.session_id, worker_config)?;
    Ok(Worker::new(worker_config, worker_record))
}

/// Validates whether a state transition is allowed
pub fn can_transition(from: &WorkerStatus, to: &WorkerStatus) -> bool {
    matches!(
        (from, to),
        (
            WorkerStatus::Idle | WorkerStatus::NeedsInput | WorkerStatus::Rebasing,
            WorkerStatus::Working
        ) | (
            WorkerStatus::Working | WorkerStatus::Rejected | WorkerStatus::Rebasing,
            WorkerStatus::NeedsReview
        ) | (WorkerStatus::Working | WorkerStatus::Rebasing, WorkerStatus::NeedsInput)
            | (WorkerStatus::NeedsReview | WorkerStatus::Rejected, WorkerStatus::Idle)
            | (WorkerStatus::NeedsReview | WorkerStatus::Rebasing, WorkerStatus::Rejected)
            | (_, WorkerStatus::Rebasing | WorkerStatus::Error | WorkerStatus::Offline)
    )
}

/// Applies a state transition to a worker record
pub fn apply_transition(worker: &mut WorkerRecord, transition: WorkerTransition) -> Result<()> {
    let old_status = worker.status;
    let new_status = match &transition {
        WorkerTransition::None => return Ok(()),
        WorkerTransition::ToIdle => WorkerStatus::Idle,
        WorkerTransition::ToWorking { prompt } => {
            worker.current_prompt = prompt.clone();
            WorkerStatus::Working
        }
        WorkerTransition::ToNeedsInput => WorkerStatus::NeedsInput,
        WorkerTransition::ToNeedsReview { commit_sha } => {
            worker.commit_sha = Some(commit_sha.clone());
            WorkerStatus::NeedsReview
        }
        WorkerTransition::ToRejected { .. } => WorkerStatus::Rejected,
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
            worker.current_prompt.clear();
            worker.commit_sha = None;
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

/// Shuts down a worker gracefully
pub fn shutdown_worker(worker: &Worker) -> Result<()> {
    sender_ctrl_c(&worker.sender, &worker.session_id)?;
    thread::sleep(Duration::from_millis(1000));
    if session::session_exists(&worker.session_id) {
        session::kill_session(&worker.session_id)?;
    }
    Ok(())
}

/// Builds the prompt preamble with context and instructions
pub fn build_prompt_preamble(worker: &Worker) -> String {
    let repo_root = worker
        .worktree_path
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    format!(
        r#"You are working in a git worktree located at: {}
Repository root: {}

IMPORTANT INSTRUCTIONS:
- Follow all conventions specified in CLAUDE.md and other project documentation
- Run validation commands as specified in the project (e.g., `just fmt`, `just check`, `just clippy`)
- Create a SINGLE commit with your changes when complete
- DO NOT push to remote - your work will be reviewed and merged by the coordinator
- Use the project's code style and patterns

Please implement the requested changes following these guidelines.
"#,
        worker.worktree_path.display(),
        repo_root
    )
}

impl Worker {
    /// Creates a new Worker from config and state
    pub fn new(_config: &WorkerConfig, state: &WorkerRecord) -> Worker {
        Worker {
            name: state.name.clone(),
            session_id: state.session_id.clone(),
            worktree_path: PathBuf::from(&state.worktree_path),
            sender: TmuxSender::new(),
        }
    }
}

/// Waits for Claude to be ready by polling for the ">" prompt
fn wait_for_claude_ready(session: &str) -> Result<()> {
    const MAX_ATTEMPTS: u32 = 60;
    const POLL_INTERVAL_MS: u64 = 500;
    for _ in 0..MAX_ATTEMPTS {
        thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
        let output = session::capture_pane(session, 50)
            .with_context(|| format!("Failed to capture pane for session '{}'", session))?;

        // Check for the '>' prompt (Claude is ready)
        if output.lines().rev().take(5).any(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("> ") || trimmed == ">"
        }) {
            return Ok(());
        }

        // Check for bypass permissions prompt (Claude is waiting for confirmation)
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

/// Sends Ctrl-C to a session
fn sender_ctrl_c(sender: &TmuxSender, session: &str) -> Result<()> {
    sender.send_keys_raw(session, "C-c")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_can_transition_idle_to_working() {
        assert!(can_transition(&WorkerStatus::Idle, &WorkerStatus::Working));
    }
    #[test]
    fn test_can_transition_working_to_needs_review() {
        assert!(can_transition(&WorkerStatus::Working, &WorkerStatus::NeedsReview));
    }
    #[test]
    fn test_can_transition_working_to_needs_input() {
        assert!(can_transition(&WorkerStatus::Working, &WorkerStatus::NeedsInput));
    }
    #[test]
    fn test_can_transition_needs_input_to_working() {
        assert!(can_transition(&WorkerStatus::NeedsInput, &WorkerStatus::Working));
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
    fn test_cannot_transition_idle_to_needs_review() {
        assert!(!can_transition(&WorkerStatus::Idle, &WorkerStatus::NeedsReview));
    }
    #[test]
    fn test_cannot_transition_idle_to_rejected() {
        assert!(!can_transition(&WorkerStatus::Idle, &WorkerStatus::Rejected));
    }
    #[test]
    fn test_apply_transition_to_idle_clears_state() {
        let mut worker = WorkerRecord {
            name: "test".to_string(),
            worktree_path: "/tmp/test".to_string(),
            branch: "llmc/test".to_string(),
            status: WorkerStatus::NeedsReview,
            current_prompt: "Some prompt".to_string(),
            created_at_unix: 1000000000,
            last_activity_unix: 1000000000,
            commit_sha: Some("abc123".to_string()),
            session_id: "llmc-test".to_string(),
            crash_count: 0,
            last_crash_unix: None,
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
            created_at_unix: 1000000000,
            last_activity_unix: 1000000000,
            commit_sha: None,
            session_id: "llmc-test".to_string(),
            crash_count: 0,
            last_crash_unix: None,
        };
        apply_transition(&mut worker, WorkerTransition::ToWorking {
            prompt: "Test prompt".to_string(),
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
            created_at_unix: 1000000000,
            last_activity_unix: 1000000000,
            commit_sha: None,
            session_id: "llmc-test".to_string(),
            crash_count: 0,
            last_crash_unix: None,
        };
        apply_transition(&mut worker, WorkerTransition::ToNeedsReview {
            commit_sha: "abc123".to_string(),
        })
        .unwrap();
        assert_eq!(worker.status, WorkerStatus::NeedsReview);
        assert_eq!(worker.commit_sha, Some("abc123".to_string()));
    }
    #[test]
    fn test_apply_transition_invalid_fails() {
        let mut worker = WorkerRecord {
            name: "test".to_string(),
            worktree_path: "/tmp/test".to_string(),
            branch: "llmc/test".to_string(),
            status: WorkerStatus::Idle,
            current_prompt: String::new(),
            created_at_unix: 1000000000,
            last_activity_unix: 1000000000,
            commit_sha: None,
            session_id: "llmc-test".to_string(),
            crash_count: 0,
            last_crash_unix: None,
        };
        let result = apply_transition(&mut worker, WorkerTransition::ToNeedsReview {
            commit_sha: "abc123".to_string(),
        });
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid transition"));
    }
    #[test]
    fn test_apply_transition_none_does_nothing() {
        let mut worker = WorkerRecord {
            name: "test".to_string(),
            worktree_path: "/tmp/test".to_string(),
            branch: "llmc/test".to_string(),
            status: WorkerStatus::Working,
            current_prompt: "Test".to_string(),
            created_at_unix: 1000000000,
            last_activity_unix: 1000000000,
            commit_sha: None,
            session_id: "llmc-test".to_string(),
            crash_count: 0,
            last_crash_unix: None,
        };
        let old_status = worker.status;
        apply_transition(&mut worker, WorkerTransition::None).unwrap();
        assert_eq!(worker.status, old_status);
    }
    #[test]
    fn test_build_prompt_preamble() {
        let worker = Worker {
            name: "test".to_string(),
            session_id: "llmc-test".to_string(),
            worktree_path: PathBuf::from("/home/user/llmc/.worktrees/test"),
            sender: TmuxSender::new(),
        };
        let preamble = build_prompt_preamble(&worker);
        assert!(preamble.contains("/home/user/llmc/.worktrees/test"));
        assert!(preamble.contains("CLAUDE.md"));
        assert!(preamble.contains("DO NOT push to remote"));
        assert!(preamble.contains("SINGLE commit"));
    }
}
