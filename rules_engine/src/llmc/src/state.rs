use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::config::{self, Config};
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
}
/// State file tracking all workers and their status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    /// All workers indexed by name
    #[serde(default)]
    pub workers: HashMap<String, WorkerRecord>,
    /// Whether the daemon is currently running (for crash detection)
    #[serde(default)]
    pub daemon_running: bool,
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
/// Runs patrol to update worker states, then returns the updated state
pub fn load_state_with_patrol() -> Result<(State, super::config::Config)> {
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
        State { workers: HashMap::new(), daemon_running: false }
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

    /// Gets all workers with NeedsReview status (regardless of on_complete
    /// state). Use `get_workers_truly_needing_review()` to get only workers
    /// ready for human review.
    #[cfg(test)]
    pub fn get_workers_needing_review(&self) -> Vec<&WorkerRecord> {
        self.workers.values().filter(|w| w.status == WorkerStatus::NeedsReview).collect()
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
#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::state::*;
    fn create_test_worker(name: &str) -> WorkerRecord {
        WorkerRecord {
            name: name.to_string(),
            worktree_path: format!("/tmp/llmc/.worktrees/{}", name),
            branch: format!("llmc/{}", name),
            status: WorkerStatus::Idle,
            current_prompt: String::new(),
            prompt_cmd: None,
            created_at_unix: 1000000000,
            last_activity_unix: 1000000000,
            commit_sha: None,
            session_id: format!("llmc-{}", name),
            crash_count: 0,
            last_crash_unix: None,
            on_complete_sent_unix: None,
            self_review: false,
            pending_self_review: false,
        }
    }
    #[test]
    fn test_new_state() {
        let state = State::new();
        assert!(state.workers.is_empty());
    }
    #[test]
    fn test_add_and_get_worker() {
        let mut state = State::new();
        let worker = create_test_worker("adam");
        state.add_worker(worker.clone());
        assert_eq!(state.workers.len(), 1);
        let retrieved = state.get_worker("adam").unwrap();
        assert_eq!(retrieved.name, "adam");
        assert_eq!(retrieved.status, WorkerStatus::Idle);
    }
    #[test]
    fn test_get_worker_mut() {
        let mut state = State::new();
        state.add_worker(create_test_worker("adam"));
        if let Some(worker) = state.get_worker_mut("adam") {
            worker.status = WorkerStatus::Working;
        }
        assert_eq!(state.get_worker("adam").unwrap().status, WorkerStatus::Working);
    }
    #[test]
    fn test_remove_worker() {
        let mut state = State::new();
        state.add_worker(create_test_worker("adam"));
        assert_eq!(state.workers.len(), 1);
        state.remove_worker("adam");
        assert_eq!(state.workers.len(), 0);
        assert!(state.get_worker("adam").is_none());
    }
    #[test]
    fn test_get_idle_workers() {
        let mut state = State::new();
        let mut worker1 = create_test_worker("adam");
        worker1.status = WorkerStatus::Idle;
        state.add_worker(worker1);
        let mut worker2 = create_test_worker("baker");
        worker2.status = WorkerStatus::Working;
        state.add_worker(worker2);
        let mut worker3 = create_test_worker("charlie");
        worker3.status = WorkerStatus::Idle;
        state.add_worker(worker3);
        let idle = state.get_idle_workers();
        assert_eq!(idle.len(), 2);
        let names: Vec<&str> = idle.iter().map(|w| w.name.as_str()).collect();
        assert!(names.contains(&"adam"));
        assert!(names.contains(&"charlie"));
    }
    #[test]
    fn test_get_workers_needing_review() {
        let mut state = State::new();
        let mut worker1 = create_test_worker("adam");
        worker1.status = WorkerStatus::NeedsReview;
        worker1.commit_sha = Some("abc123".to_string());
        state.add_worker(worker1);
        let mut worker2 = create_test_worker("baker");
        worker2.status = WorkerStatus::Working;
        state.add_worker(worker2);
        let mut worker3 = create_test_worker("charlie");
        worker3.status = WorkerStatus::NeedsReview;
        worker3.commit_sha = Some("def456".to_string());
        state.add_worker(worker3);
        let needs_review = state.get_workers_needing_review();
        assert_eq!(needs_review.len(), 2);
        let names: Vec<&str> = needs_review.iter().map(|w| w.name.as_str()).collect();
        assert!(names.contains(&"adam"));
        assert!(names.contains(&"charlie"));
    }
    #[test]
    fn test_save_and_load() {
        let dir = TempDir::new().unwrap();
        let state_path = dir.path().join("state.json");
        let mut state = State::new();
        state.add_worker(create_test_worker("adam"));
        state.save(&state_path).unwrap();
        assert!(state_path.exists());
        let loaded = State::load(&state_path).unwrap();
        assert_eq!(loaded.workers.len(), 1);
        assert!(loaded.get_worker("adam").is_some());
    }
    #[test]
    fn test_load_nonexistent_file() {
        let dir = TempDir::new().unwrap();
        let state_path = dir.path().join("state.json");
        let state = State::load(&state_path).unwrap();
        assert!(state.workers.is_empty());
    }
    #[test]
    fn test_atomic_write_creates_backup() {
        let dir = TempDir::new().unwrap();
        let state_path = dir.path().join("state.json");
        let backup_path = dir.path().join("state.json.bak");
        let mut state = State::new();
        state.add_worker(create_test_worker("adam"));
        state.save(&state_path).unwrap();
        assert!(state_path.exists());
        assert!(!backup_path.exists());
        let mut state2 = State::new();
        state2.add_worker(create_test_worker("baker"));
        state2.save(&state_path).unwrap();
        assert!(state_path.exists());
        assert!(backup_path.exists());
        let backup_state = State::load(&backup_path).unwrap();
        assert!(backup_state.get_worker("adam").is_some());
        assert!(backup_state.get_worker("baker").is_none());
    }
    #[test]
    fn test_validate_duplicate_names() {
        let mut state = State::new();
        let worker = create_test_worker("adam");
        state.workers.insert("adam".to_string(), worker.clone());
        state.workers.insert("adam_duplicate".to_string(), worker);
        let result = validate_state(&state);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate worker name"));
    }
    #[test]
    fn test_validate_needs_review_requires_commit_sha() {
        let mut state = State::new();
        let mut worker = create_test_worker("adam");
        worker.status = WorkerStatus::NeedsReview;
        worker.commit_sha = None;
        state.add_worker(worker);
        let result = validate_state(&state);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("needs_review but no commit_sha"));
    }
    #[test]
    fn test_validate_future_timestamps() {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let future = now + 10000;
        let mut state = State::new();
        let mut worker = create_test_worker("adam");
        worker.created_at_unix = future;
        state.add_worker(worker);
        let result = validate_state(&state);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timestamp in the future"));
    }
    #[test]
    fn test_worker_status_serialization() {
        let statuses = vec![
            (WorkerStatus::Idle, "\"idle\""),
            (WorkerStatus::Working, "\"working\""),
            (WorkerStatus::NeedsReview, "\"needs_review\""),
            (WorkerStatus::Reviewing, "\"reviewing\""),
            (WorkerStatus::Rejected, "\"rejected\""),
            (WorkerStatus::Rebasing, "\"rebasing\""),
            (WorkerStatus::Error, "\"error\""),
            (WorkerStatus::Offline, "\"offline\""),
        ];
        for (status, expected_json) in statuses {
            let json = serde_json::to_string(&status).unwrap();
            assert_eq!(json, expected_json);
            let deserialized: WorkerStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, status);
        }
    }
    #[test]
    fn test_worker_record_serialization() {
        let worker = create_test_worker("adam");
        let json = serde_json::to_string(&worker).unwrap();
        let deserialized: WorkerRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, worker.name);
        assert_eq!(deserialized.worktree_path, worker.worktree_path);
        assert_eq!(deserialized.branch, worker.branch);
        assert_eq!(deserialized.status, worker.status);
    }
    #[test]
    fn test_get_state_path() {
        let path = get_state_path();
        assert!(path.ends_with("llmc/state.json"));
    }
    #[test]
    fn test_key_name_mismatch() {
        let mut state = State::new();
        let worker = create_test_worker("adam");
        state.workers.insert("wrong_key".to_string(), worker);
        let result = validate_state(&state);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("doesn't match worker name"));
    }
}
