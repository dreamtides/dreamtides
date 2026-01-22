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

/// Log entry for task pool command executions.
///
/// Logged to `logs/task_pool.log` for each `task_pool_command` invocation.
#[derive(Debug, Clone, Serialize)]
pub struct TaskPoolLogEntry {
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
/// - `task_pool.log`: Task pool command invocations
/// - `post_accept.log`: Post accept command invocations
/// - `auto.log`: High-level auto mode events
pub struct AutoLogger {
    task_pool: Arc<Mutex<LogWriter>>,
    post_accept: Arc<Mutex<LogWriter>>,
    auto: Arc<Mutex<LogWriter>>,
}

/// Returns the path to the logs directory.
pub fn logs_dir() -> PathBuf {
    config::get_llmc_root().join("logs")
}

/// Returns the path to the task pool log file.
pub fn task_pool_log_path() -> PathBuf {
    logs_dir().join("task_pool.log")
}

/// Returns the path to the post accept log file.
pub fn post_accept_log_path() -> PathBuf {
    logs_dir().join("post_accept.log")
}

/// Returns the path to the auto log file.
pub fn auto_log_path() -> PathBuf {
    logs_dir().join("auto.log")
}

/// Returns the current timestamp formatted for log entries.
pub fn timestamp_now() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
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
        let task_pool = LogWriter::new(paths.task_pool_log_path())?;
        let post_accept = LogWriter::new(paths.post_accept_log_path())?;
        let auto = LogWriter::new(paths.auto_log_path())?;
        Ok(Self {
            task_pool: Arc::new(Mutex::new(task_pool)),
            post_accept: Arc::new(Mutex::new(post_accept)),
            auto: Arc::new(Mutex::new(auto)),
        })
    }

    /// Logs a task pool command invocation.
    pub fn log_task_pool(&self, result: &CommandResult) {
        // Exit codes 0, 3 (claim limit), and 4 (no ready tasks) are not errors
        let level = if result.exit_code == 0 || result.exit_code == 3 || result.exit_code == 4 {
            LogLevel::Info
        } else {
            LogLevel::Error
        };
        let entry = TaskPoolLogEntry {
            timestamp: timestamp_now(),
            level,
            command: result.command.clone(),
            exit_code: result.exit_code,
            duration_ms: result.duration.as_millis() as u64,
            stdout: result.stdout.clone(),
            stderr: result.stderr.clone(),
        };
        if let Err(e) = self.write_task_pool_entry(&entry) {
            error!("Failed to write to task_pool.log: {}. Continuing.", e);
        }
    }

    fn write_task_pool_entry(&self, entry: &TaskPoolLogEntry) -> Result<()> {
        let mut writer = self.task_pool.lock().unwrap();
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
        if let Err(e) = self.task_pool.lock().unwrap().flush() {
            info!("Failed to flush task_pool.log: {}", e);
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
            task_pool: Arc::clone(&self.task_pool),
            post_accept: Arc::clone(&self.post_accept),
            auto: Arc::clone(&self.auto),
        }
    }
}
