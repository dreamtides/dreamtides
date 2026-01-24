use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use tracing::trace;

use crate::auto_mode::auto_workers;
use crate::config::{self, Config};
use crate::lock::StateLock;
use crate::patrol::Patrol;
/// Worker state machine states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerStatus {
    /// Worker has no active task, ready to receive work
    Idle,
    /// Worker is actively implementing a task
    Working,
    /// Worker completed work and committed, awaiting human review
    NeedsReview,
    /// Worker received on_complete prompt and is performing self-review
    Reviewing,
    /// Work was rejected with feedback, worker is implementing changes
    Rejected,
    /// Worker is resolving merge conflicts after a rebase
    Rebasing,
    /// Worker is in an error state requiring manual intervention
    Error,
    /// TMUX session is not running
    Offline,
    /// Worker completed task without making any commits
    NoChanges,
}
/// Record for a single worker in the LLMC system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerRecord {
    /// Unique worker identifier
    pub name: String,
    /// Absolute path to git worktree
    pub worktree_path: String,
    /// Git branch name (llmc/<name>)
    pub branch: String,
    /// Current worker state
    pub status: WorkerStatus,
    /// Full prompt text for current task
    pub current_prompt: String,
    /// Command used to generate the prompt (e.g., "bd show dr-abc"), if any
    #[serde(default)]
    pub prompt_cmd: Option<String>,
    /// Unix timestamp of worker creation
    pub created_at_unix: u64,
    /// Unix timestamp of last state change
    pub last_activity_unix: u64,
    /// SHA of commit awaiting review (required when status is needs_review)
    pub commit_sha: Option<String>,
    /// TMUX session identifier
    pub session_id: String,
    /// Number of crashes (for error recovery)
    pub crash_count: u32,
    /// Unix timestamp of last crash
    pub last_crash_unix: Option<u64>,
    /// Unix timestamp when the on_complete prompt was sent (None if not sent)
    #[serde(default)]
    pub on_complete_sent_unix: Option<u64>,
    /// If true, enable the self-review phase for this task. Self-review sends
    /// the self_review prompt to the worker before human review.
    #[serde(default)]
    pub self_review: bool,
    /// If true, a self-review prompt should be sent on the next maintenance
    /// tick. Set by the Stop hook when a worker with self_review enabled
    /// transitions to NeedsReview.
    #[serde(default)]
    pub pending_self_review: bool,
    /// Unix timestamp when commits ahead of master were first detected for a
    /// Working/Rejected worker. Used for fallback recovery timing - we wait
    /// 5 minutes from when commits were first seen, not from when work started.
    /// Reset when worker transitions out of Working/Rejected state.
    #[serde(default)]
    pub commits_first_detected_unix: Option<u64>,
    /// If true, a rebase conflict prompt should be sent on the next patrol run.
    /// Set when a worker is detected in Rebasing state at startup (e.g., after
    /// daemon restart) without having received a conflict prompt.
    #[serde(default)]
    pub pending_rebase_prompt: bool,
    /// Reason the worker entered error state, if any. Used to determine
    /// recovery behavior - some error types (like dirty worktree) should
    /// not auto-recover.
    #[serde(default)]
    pub error_reason: Option<String>,
    /// Number of auto mode retry attempts for transient failures. Reset on
    /// successful task completion. Used by patrol to track recovery attempts
    /// before escalating to hard failure.
    #[serde(default)]
    pub auto_retry_count: u32,
    /// Number of API errors (500s, rate limits) detected via transcript.
    /// Tracked separately from crash_count to enable API-specific backoff.
    #[serde(default)]
    pub api_error_count: u32,
    /// Unix timestamp of last API error. Used for API-specific backoff timing.
    #[serde(default)]
    pub last_api_error_unix: Option<u64>,
    /// Task prompt pending to be sent after session restarts (after /clear).
    /// This field is set when a task is assigned but /clear is still
    /// processing. When SessionStart fires for a worker with a pending
    /// prompt, the prompt is sent and the worker transitions to Working.
    #[serde(default)]
    pub pending_task_prompt: Option<String>,
    /// Unix timestamp when pending_task_prompt was set. Used to detect stale
    /// pending prompts where /clear may have failed to execute (e.g., Claude's
    /// autocomplete menu intercepted the Enter key). If this timestamp is too
    /// old, patrol will resend /clear to retry.
    #[serde(default)]
    pub pending_task_prompt_since_unix: Option<u64>,
    /// Number of times /clear has been retried for this worker. Incremented by
    /// patrol when it detects a stale pending prompt and resends /clear. Reset
    /// to 0 when SessionStart fires (indicating /clear succeeded). If this
    /// exceeds the max retry limit, the daemon shuts down for remediation.
    #[serde(default)]
    pub pending_clear_retry_count: u32,
    /// Command used to generate the pending task prompt (e.g., "bd show
    /// dr-abc"). Stored alongside pending_task_prompt and used when
    /// transitioning to Working.
    #[serde(default)]
    pub pending_prompt_cmd: Option<String>,
    /// Claude session ID for the current task. Captured from SessionStart hook
    /// when a task begins. Used to identify the correct transcript file for
    /// later archival.
    #[serde(default)]
    pub transcript_session_id: Option<String>,
    /// Path to Claude transcript file for the current task. Captured from
    /// SessionStart hook. On task completion or stall, this transcript is
    /// copied to the logs directory for debugging and analysis.
    #[serde(default)]
    pub transcript_path: Option<String>,
    /// ID of the Claude Task currently being worked on by this worker. Set when
    /// a task is claimed from the task pool. Used to release the task back to
    /// pending status if the worker fails, crashes, or the daemon shuts down.
    #[serde(default)]
    pub active_task_id: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    /// All workers indexed by name
    #[serde(default)]
    pub workers: HashMap<String, WorkerRecord>,
    /// Whether the daemon is currently running (for crash detection)
    #[serde(default)]
    pub daemon_running: bool,
    /// Whether the daemon is running in auto mode
    #[serde(default)]
    pub auto_mode: bool,
    /// Names of workers that are auto-managed (auto-1, auto-2, etc.)
    #[serde(default)]
    pub auto_workers: Vec<String>,
    /// Unix timestamp of the last task completion (for stall detection in
    /// overseer)
    #[serde(default)]
    pub last_task_completion_unix: Option<u64>,
    /// Unix timestamp of the last task assignment (for stall detection in
    /// overseer). A stall is only triggered if both last_task_completion_unix
    /// AND last_task_assignment_unix are older than the stall timeout.
    #[serde(default)]
    pub last_task_assignment_unix: Option<u64>,
    /// Unix timestamp after which to retry accepting when source repo is dirty
    #[serde(default)]
    pub source_repo_dirty_retry_after_unix: Option<u64>,
    /// Current backoff in seconds for source repo dirty retry (starts at 60,
    /// doubles)
    #[serde(default)]
    pub source_repo_dirty_backoff_secs: Option<u64>,
    /// Number of times we've retried due to source repo being dirty
    #[serde(default)]
    pub source_repo_dirty_retry_count: Option<u32>,
}
/// Returns true if a worker is truly ready for human review.
///
/// A worker in `NeedsReview` state is NOT ready for human review if:
/// - Self-review is enabled for the worker (`self_review == true`)
/// - A self_review prompt is configured (in defaults)
/// - AND one of these transitional conditions is true:
///   - `pending_self_review` is true (waiting for prompt to be sent)
///   - `on_complete_sent_unix` is None (prompt not yet sent)
///
/// In these cases, the worker is in a transitional state and should be
/// displayed as "reviewing" rather than "needs_review".
pub fn is_truly_needs_review(worker: &WorkerRecord, config: &Config) -> bool {
    if worker.status != WorkerStatus::NeedsReview {
        return false;
    }
    if !worker.self_review {
        return true;
    }
    let has_prompt = config.defaults.self_review.is_some();
    if !has_prompt {
        return true;
    }
    if worker.pending_self_review {
        return false;
    }
    worker.on_complete_sent_unix.is_some()
}
/// Validates state consistency
pub fn validate_state(state: &State) -> Result<()> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let mut seen_names = std::collections::HashSet::new();
    #[expect(clippy::iter_over_hash_type)]
    for worker in state.workers.values() {
        if !seen_names.insert(&worker.name) {
            bail!("Duplicate worker name: {}", worker.name);
        }
    }
    #[expect(clippy::iter_over_hash_type)]
    for (key, worker) in &state.workers {
        if key != &worker.name {
            bail!("Worker key '{}' doesn't match worker name '{}'", key, worker.name);
        }
        if worker.status == WorkerStatus::NeedsReview && worker.commit_sha.is_none() {
            bail!("Worker '{}' has status needs_review but no commit_sha", worker.name);
        }
        if worker.created_at_unix > now {
            bail!("Worker '{}' has created_at_unix timestamp in the future", worker.name);
        }
        if worker.last_activity_unix > now {
            bail!("Worker '{}' has last_activity_unix timestamp in the future", worker.name);
        }
        if let Some(last_crash) = worker.last_crash_unix
            && last_crash > now
        {
            bail!("Worker '{}' has last_crash_unix timestamp in the future", worker.name);
        }
    }
    Ok(())
}
/// Returns the path to the state file (~/llmc/state.json)
pub fn get_state_path() -> PathBuf {
    config::get_llmc_root().join("state.json")
}
/// Runs patrol to update worker states, then returns the updated state.
///
/// This function acquires the state lock to prevent race conditions with the
/// daemon. If the lock cannot be acquired (e.g., daemon is processing), the
/// caller should fall back to loading raw state without patrol.
pub fn load_state_with_patrol() -> Result<(State, super::config::Config)> {
    let _lock = StateLock::acquire()?;
    let state_path = get_state_path();
    let mut state = State::load(&state_path)?;
    let config_path = config::get_config_path();
    let config = Config::load(&config_path)?;
    let patrol = Patrol::new(&config);
    let _report = patrol.run_patrol(&mut state, &config)?;
    state.save(&state_path)?;
    Ok((state, config))
}
impl State {
    /// Creates a new empty state
    pub fn new() -> State {
        State {
            workers: HashMap::new(),
            daemon_running: false,
            auto_mode: false,
            auto_workers: Vec::new(),
            last_task_completion_unix: None,
            last_task_assignment_unix: None,
            source_repo_dirty_retry_after_unix: None,
            source_repo_dirty_backoff_secs: None,
            source_repo_dirty_retry_count: None,
        }
    }

    /// Loads state from the given path
    pub fn load(path: &Path) -> Result<State> {
        if !path.exists() {
            return Ok(State::new());
        }
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read state file: {}", path.display()))?;
        let state: State = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse state file: {}", path.display()))?;
        validate_state(&state)?;
        Ok(state)
    }

    /// Saves state to the given path with atomic writes and backup
    pub fn save(&self, path: &Path) -> Result<()> {
        validate_state(self)?;

        let total_workers = self.workers.len();
        let auto_worker_count =
            self.workers.values().filter(|w| auto_workers::is_auto_worker(&w.name)).count();
        let auto_worker_names: Vec<&str> = self
            .workers
            .values()
            .filter(|w| auto_workers::is_auto_worker(&w.name))
            .map(|w| w.name.as_str())
            .collect();
        trace!(
            path = %path.display(),
            total_workers,
            auto_worker_count,
            ?auto_worker_names,
            auto_mode = self.auto_mode,
            daemon_running = self.daemon_running,
            "Saving state"
        );

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create state directory: {}", parent.display())
            })?;
        }
        let json = serde_json::to_string_pretty(self).context("Failed to serialize state")?;
        let temp_filename = format!("state.{}.tmp", std::process::id());
        let temp_path = path.with_file_name(&temp_filename);
        fs::write(&temp_path, json)
            .with_context(|| format!("Failed to write temp state file: {}", temp_path.display()))?;
        if path.exists() {
            let backup_path = path.with_extension("json.bak");
            fs::copy(path, &backup_path)
                .with_context(|| format!("Failed to create backup: {}", backup_path.display()))?;
        }
        fs::rename(&temp_path, path).with_context(|| {
            format!(
                "Failed to rename temp file {} to state file {}",
                temp_path.display(),
                path.display()
            )
        })?;
        Ok(())
    }

    /// Gets a worker by name (immutable reference)
    pub fn get_worker(&self, name: &str) -> Option<&WorkerRecord> {
        self.workers.get(name)
    }

    /// Gets a worker by name (mutable reference)
    pub fn get_worker_mut(&mut self, name: &str) -> Option<&mut WorkerRecord> {
        self.workers.get_mut(name)
    }

    /// Adds a worker to the state
    pub fn add_worker(&mut self, record: WorkerRecord) {
        self.workers.insert(record.name.clone(), record);
    }

    /// Removes a worker from the state
    pub fn remove_worker(&mut self, name: &str) {
        self.workers.remove(name);
    }

    /// Gets all workers in idle state
    pub fn get_idle_workers(&self) -> Vec<&WorkerRecord> {
        self.workers.values().filter(|w| w.status == WorkerStatus::Idle).collect()
    }

    /// Gets all workers that are truly ready for human review.
    /// This excludes workers in `NeedsReview` state that are still waiting for
    /// the on_complete self-review prompt to be sent.
    pub fn get_workers_truly_needing_review(&self, config: &Config) -> Vec<&WorkerRecord> {
        self.workers.values().filter(|w| is_truly_needs_review(w, config)).collect()
    }
}
impl Default for State {
    fn default() -> Self {
        State::new()
    }
}
