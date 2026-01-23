#![allow(dead_code, reason = "Some variants defined for API completeness")]

use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Serialize;
use tracing::{error, info};

use crate::config::{self, LlmcPaths};

const MAX_LOG_SIZE: u64 = 50 * 1024 * 1024;

/// Result of executing a shell command, used for logging.
pub struct CommandResult {
    pub command: String,
    pub exit_code: i32,
    pub duration: Duration,
    pub stdout: String,
    pub stderr: String,
}

/// Log entry for task discovery executions.
///
/// Logged to `logs/task_discovery.log` for each `task_discovery_command`
/// invocation.
#[derive(Debug, Clone, Serialize)]
pub struct TaskDiscoveryLogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub command: String,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub stdout: String,
    pub stderr: String,
}

/// Log entry for post accept command executions.
///
/// Logged to `logs/post_accept.log` for each `post_accept_command` invocation.
#[derive(Debug, Clone, Serialize)]
pub struct PostAcceptLogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub worker_name: String,
    pub commit_sha: String,
    pub command: String,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub stdout: String,
    pub stderr: String,
}

/// Log entry for high-level auto mode events.
///
/// Logged to `logs/auto.log` for daemon lifecycle, task assignments, etc.
#[derive(Debug, Clone, Serialize)]
pub struct AutoLogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub event: AutoEvent,
}

/// Types of high-level auto mode events.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AutoEvent {
    DaemonStartup { instance_id: String, concurrency: u32 },
    DaemonShutdown { instance_id: String, reason: String },
    TaskAssigned { worker_name: String, task_excerpt: String },
    TaskCompleted { worker_name: String, result: TaskResult },
    AcceptSuccess { worker_name: String, commit_sha: String },
    AcceptFailure { worker_name: String, error: String },
    WorkerStateTransition { worker_name: String, from_state: String, to_state: String },
    Error { context: String, error: String },
}

/// Result of a completed task.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskResult {
    NeedsReview,
    NoChanges,
    Error,
}

/// Log levels for structured log entries.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

/// Handle for the auto mode logging system.
///
/// Manages three dedicated log files for auto mode operations:
/// - `task_discovery.log`: Task discovery invocations
/// - `post_accept.log`: Post accept command invocations
/// - `auto.log`: High-level auto mode events
pub struct AutoLogger {
    task_discovery: Arc<Mutex<LogWriter>>,
    post_accept: Arc<Mutex<LogWriter>>,
    auto: Arc<Mutex<LogWriter>>,
}

/// Stored context for task-related errors, used by overseer remediation.
///
/// When a task error causes daemon shutdown, this context is persisted to disk
/// so the overseer can include detailed remediation guidance in its prompt.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct TaskErrorContext {
    pub timestamp: String,
    pub error_type: String,
    pub error_message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_content: Option<String>,
    pub remediation_hint: String,
}

/// Returns the path to the logs directory.
pub fn logs_dir() -> PathBuf {
    config::get_llmc_root().join("logs")
}

/// Returns the path to the task pool log file.
pub fn task_discovery_log_path() -> PathBuf {
    logs_dir().join("task_discovery.log")
}

/// Returns the path to the post accept log file.
pub fn post_accept_log_path() -> PathBuf {
    logs_dir().join("post_accept.log")
}

/// Returns the path to the auto log file.
pub fn auto_log_path() -> PathBuf {
    logs_dir().join("auto.log")
}

/// Returns the path to the task error context file.
pub fn task_error_context_path() -> PathBuf {
    logs_dir().join("last_task_error.json")
}

/// Returns the current timestamp formatted for log entries.
pub fn timestamp_now() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

/// Persists task error context for overseer remediation.
///
/// Writes the error context to a JSON file that the overseer can read when
/// building remediation prompts. This provides structured information about
/// what went wrong and how to fix it.
pub fn persist_task_error_context(context: &TaskErrorContext) {
    let path = task_error_context_path();
    if let Some(parent) = path.parent()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        error!("Failed to create logs directory for task error context: {}", e);
        return;
    }
    let json = match serde_json::to_string_pretty(context) {
        Ok(j) => j,
        Err(e) => {
            error!("Failed to serialize task error context: {}", e);
            return;
        }
    };
    if let Err(e) = std::fs::write(&path, json) {
        error!("Failed to write task error context to {}: {}", path.display(), e);
    } else {
        info!("Persisted task error context to {}", path.display());
    }
}

/// Reads persisted task error context if it exists.
///
/// Returns None if the file doesn't exist or cannot be read.
pub fn read_task_error_context() -> Option<TaskErrorContext> {
    let path = task_error_context_path();
    if !path.exists() {
        return None;
    }
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            info!("Failed to read task error context: {}", e);
            return None;
        }
    };
    match serde_json::from_str(&content) {
        Ok(ctx) => Some(ctx),
        Err(e) => {
            info!("Failed to parse task error context: {}", e);
            None
        }
    }
}

/// Clears the persisted task error context file.
///
/// Called when the daemon starts successfully to clear stale error context.
pub fn clear_task_error_context() {
    let path = task_error_context_path();
    if path.exists() {
        if let Err(e) = std::fs::remove_file(&path) {
            info!("Failed to remove task error context file: {}", e);
        } else {
            info!("Cleared stale task error context file");
        }
    }
}

struct LogWriter {
    path: PathBuf,
    writer: BufWriter<File>,
    current_size: u64,
}

/// Returns the current Unix timestamp in seconds.
fn unix_timestamp_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|e| panic!("System time before UNIX epoch: {}", e))
        .as_secs()
}

impl LogWriter {
    fn new(path: PathBuf) -> Result<Self> {
        let parent = path.parent().unwrap_or_else(|| panic!("Log path has no parent: {:?}", path));
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create log directory: {}", parent.display()))?;
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .with_context(|| format!("Failed to open log file: {}", path.display()))?;
        let current_size = file
            .metadata()
            .with_context(|| format!("Failed to get log file metadata: {}", path.display()))?
            .len();
        Ok(Self { path, writer: BufWriter::new(file), current_size })
    }

    fn write_entry<T: Serialize>(&mut self, entry: &T) -> Result<()> {
        let json = serde_json::to_string(entry).context("Failed to serialize log entry")?;
        let line = format!("{}\n", json);
        let bytes = line.as_bytes();
        self.writer
            .write_all(bytes)
            .with_context(|| format!("Failed to write to log file: {}", self.path.display()))?;
        self.writer
            .flush()
            .with_context(|| format!("Failed to flush log file: {}", self.path.display()))?;
        self.current_size += bytes.len() as u64;
        if self.current_size >= MAX_LOG_SIZE {
            self.rotate()?;
        }
        Ok(())
    }

    fn rotate(&mut self) -> Result<()> {
        self.writer.flush().ok();
        let timestamp = unix_timestamp_now();
        let rotated_name = format!(
            "{}.{}",
            self.path.file_name().unwrap_or_else(|| panic!("No filename")).to_string_lossy(),
            timestamp
        );
        let rotated_path = self.path.with_file_name(rotated_name);
        std::fs::rename(&self.path, &rotated_path)
            .with_context(|| format!("Failed to rotate log file to: {}", rotated_path.display()))?;
        let new_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .with_context(|| format!("Failed to open new log file: {}", self.path.display()))?;
        self.writer = BufWriter::new(new_file);
        self.current_size = 0;
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        self.writer
            .flush()
            .with_context(|| format!("Failed to flush log file: {}", self.path.display()))
    }
}

impl AutoLogger {
    /// Creates a new auto logger and initializes all log files.
    ///
    /// Creates the logs directory if it doesn't exist and opens/creates
    /// the three dedicated log files. Uses LLMC_ROOT from the environment.
    pub fn new() -> Result<Self> {
        Self::new_with_paths(&LlmcPaths::from_env())
    }

    /// Creates a new auto logger with explicit paths.
    ///
    /// Use this in tests to avoid depending on environment variables.
    pub fn new_with_paths(paths: &LlmcPaths) -> Result<Self> {
        let task_discovery = LogWriter::new(paths.task_discovery_log_path())?;
        let post_accept = LogWriter::new(paths.post_accept_log_path())?;
        let auto = LogWriter::new(paths.auto_log_path())?;
        Ok(Self {
            task_discovery: Arc::new(Mutex::new(task_discovery)),
            post_accept: Arc::new(Mutex::new(post_accept)),
            auto: Arc::new(Mutex::new(auto)),
        })
    }

    /// Logs a task discovery invocation.
    pub fn log_task_discovery(&self, result: &CommandResult) {
        // Exit codes 0 and 4 (no ready tasks) are not errors.
        // Exit code 3 (claim limit exceeded) IS an error - indicates claim leak.
        let level = if result.exit_code == 0 || result.exit_code == 4 {
            LogLevel::Info
        } else {
            LogLevel::Error
        };
        let entry = TaskDiscoveryLogEntry {
            timestamp: timestamp_now(),
            level,
            command: result.command.clone(),
            exit_code: result.exit_code,
            duration_ms: result.duration.as_millis() as u64,
            stdout: result.stdout.clone(),
            stderr: result.stderr.clone(),
        };
        if let Err(e) = self.write_task_discovery_entry(&entry) {
            error!("Failed to write to task_discovery.log: {}. Continuing.", e);
        }
    }

    fn write_task_discovery_entry(&self, entry: &TaskDiscoveryLogEntry) -> Result<()> {
        let mut writer = self.task_discovery.lock().unwrap();
        writer.write_entry(entry)
    }

    /// Logs a post accept command invocation.
    pub fn log_post_accept(&self, worker_name: &str, commit_sha: &str, result: &CommandResult) {
        let level = if result.exit_code == 0 { LogLevel::Info } else { LogLevel::Error };
        let entry = PostAcceptLogEntry {
            timestamp: timestamp_now(),
            level,
            worker_name: worker_name.to_string(),
            commit_sha: commit_sha.to_string(),
            command: result.command.clone(),
            exit_code: result.exit_code,
            duration_ms: result.duration.as_millis() as u64,
            stdout: result.stdout.clone(),
            stderr: result.stderr.clone(),
        };
        if let Err(e) = self.write_post_accept_entry(&entry) {
            error!("Failed to write to post_accept.log: {}. Continuing.", e);
        }
    }

    fn write_post_accept_entry(&self, entry: &PostAcceptLogEntry) -> Result<()> {
        let mut writer = self.post_accept.lock().unwrap();
        writer.write_entry(entry)
    }

    /// Logs a high-level auto mode event.
    pub fn log_auto(&self, level: LogLevel, event: AutoEvent) {
        let entry = AutoLogEntry { timestamp: timestamp_now(), level, event };
        if let Err(e) = self.write_auto_entry(&entry) {
            error!("Failed to write to auto.log: {}. Continuing.", e);
        }
    }

    fn write_auto_entry(&self, entry: &AutoLogEntry) -> Result<()> {
        let mut writer = self.auto.lock().unwrap();
        writer.write_entry(entry)
    }

    /// Logs daemon startup event.
    pub fn log_daemon_startup(&self, instance_id: &str, concurrency: u32) {
        self.log_auto(LogLevel::Info, AutoEvent::DaemonStartup {
            instance_id: instance_id.to_string(),
            concurrency,
        });
    }

    /// Logs daemon shutdown event.
    pub fn log_daemon_shutdown(&self, instance_id: &str, reason: &str) {
        self.log_auto(LogLevel::Info, AutoEvent::DaemonShutdown {
            instance_id: instance_id.to_string(),
            reason: reason.to_string(),
        });
    }

    /// Logs a task assignment to a worker.
    ///
    /// Truncates the task to first 100 characters for the excerpt.
    pub fn log_task_assigned(&self, worker_name: &str, task: &str) {
        let task_excerpt =
            if task.len() > 100 { format!("{}...", &task[..100]) } else { task.to_string() };
        self.log_auto(LogLevel::Info, AutoEvent::TaskAssigned {
            worker_name: worker_name.to_string(),
            task_excerpt,
        });
    }

    /// Logs task completion.
    pub fn log_task_completed(&self, worker_name: &str, result: TaskResult) {
        self.log_auto(LogLevel::Info, AutoEvent::TaskCompleted {
            worker_name: worker_name.to_string(),
            result,
        });
    }

    /// Logs a successful accept (merge to master).
    pub fn log_accept_success(&self, worker_name: &str, commit_sha: &str) {
        self.log_auto(LogLevel::Info, AutoEvent::AcceptSuccess {
            worker_name: worker_name.to_string(),
            commit_sha: commit_sha.to_string(),
        });
    }

    /// Logs a failed accept attempt.
    pub fn log_accept_failure(&self, worker_name: &str, error: &str) {
        self.log_auto(LogLevel::Error, AutoEvent::AcceptFailure {
            worker_name: worker_name.to_string(),
            error: error.to_string(),
        });
    }

    /// Logs a worker state transition.
    pub fn log_worker_state_transition(&self, worker_name: &str, from: &str, to: &str) {
        self.log_auto(LogLevel::Info, AutoEvent::WorkerStateTransition {
            worker_name: worker_name.to_string(),
            from_state: from.to_string(),
            to_state: to.to_string(),
        });
    }

    /// Logs an error event with context.
    pub fn log_error(&self, context: &str, error: &str) {
        self.log_auto(LogLevel::Error, AutoEvent::Error {
            context: context.to_string(),
            error: error.to_string(),
        });
    }

    /// Flushes all log files.
    ///
    /// Should be called before shutdown to ensure all entries are written.
    pub fn flush(&self) {
        if let Err(e) = self.task_discovery.lock().unwrap().flush() {
            info!("Failed to flush task_discovery.log: {}", e);
        }
        if let Err(e) = self.post_accept.lock().unwrap().flush() {
            info!("Failed to flush post_accept.log: {}", e);
        }
        if let Err(e) = self.auto.lock().unwrap().flush() {
            info!("Failed to flush auto.log: {}", e);
        }
    }
}

impl Clone for AutoLogger {
    fn clone(&self) -> Self {
        Self {
            task_discovery: Arc::clone(&self.task_discovery),
            post_accept: Arc::clone(&self.post_accept),
            auto: Arc::clone(&self.auto),
        }
    }
}
