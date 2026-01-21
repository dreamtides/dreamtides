#![allow(dead_code)]
use std::fs::{self, File, Metadata};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use tracing::{debug, info, warn};

use crate::auto_mode::heartbeat_thread::DaemonRegistration;
use crate::auto_mode::{auto_logging, heartbeat_thread};
use crate::overseer_mode::overseer_config::OverseerConfig;
use crate::state::{self, State};
/// Result of a daemon health check.
///
/// Returned by `check_daemon_health()` to indicate the daemon's current health
/// status. The overseer uses this to determine when to terminate and remediate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    /// Daemon is running normally with no detected issues.
    Healthy,
    /// Daemon process is no longer running (PID doesn't exist).
    ProcessGone,
    /// Heartbeat file is missing or timestamp is stale.
    HeartbeatStale { age_secs: u64 },
    /// Error or warning detected in daemon logs.
    LogError { message: String },
    /// No task completions for longer than the stall timeout.
    Stalled { stall_secs: u64 },
    /// Daemon registration doesn't match expected values (unexpected restart).
    IdentityMismatch { reason: String },
}
/// Expected daemon identity for verification.
///
/// When the overseer starts a daemon, it records these expected values
/// to detect if the daemon unexpectedly restarts or if a different process
/// takes its PID.
#[derive(Debug, Clone)]
pub struct ExpectedDaemon {
    /// Expected process ID.
    pub pid: u32,
    /// Expected start time (unix timestamp).
    pub start_time_unix: u64,
    /// Expected instance ID.
    pub instance_id: String,
}
/// Tracks position in a log file for incremental reading.
///
/// Allows the health monitor to read only new log entries since the last check,
/// avoiding re-reading old entries on each health check cycle.
pub struct LogTailer {
    path: PathBuf,
    last_position: u64,
    last_inode: Option<u64>,
}
/// Health monitor for the auto mode daemon.
///
/// Performs periodic health checks to detect daemon failures. The overseer
/// uses this to decide when to terminate and remediate.
pub struct HealthMonitor {
    config: OverseerConfig,
    log_tailer: LogTailer,
}
/// Reads the daemon registration and returns expected daemon info.
///
/// Returns None if the daemon registration file doesn't exist.
pub fn read_expected_daemon() -> Option<ExpectedDaemon> {
    heartbeat_thread::read_daemon_registration().map(|r| ExpectedDaemon::from_registration(&r))
}
/// Performs a quick health check without maintaining log tailer state.
///
/// This is a convenience function for one-off checks. For continuous
/// monitoring, use `HealthMonitor` to properly track log file position.
pub fn quick_health_check(config: &OverseerConfig, expected: &ExpectedDaemon) -> HealthStatus {
    let mut monitor = HealthMonitor::new(config.clone());
    monitor.check_daemon_health(expected)
}
impl HealthStatus {
    /// Returns true if the daemon is healthy.
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }

    /// Returns a human-readable description of the health status.
    pub fn describe(&self) -> String {
        match self {
            HealthStatus::Healthy => "Daemon is healthy".to_string(),
            HealthStatus::ProcessGone => "Daemon process is no longer running".to_string(),
            HealthStatus::HeartbeatStale { age_secs } => {
                format!("Heartbeat is stale ({}s old)", age_secs)
            }
            HealthStatus::LogError { message } => {
                format!("Error/warning in logs: {}", message)
            }
            HealthStatus::Stalled { stall_secs } => {
                format!("No task completions for {}s", stall_secs)
            }
            HealthStatus::IdentityMismatch { reason } => {
                format!("Daemon identity mismatch: {}", reason)
            }
        }
    }
}
impl ExpectedDaemon {
    /// Creates expected daemon values from a daemon registration.
    pub fn from_registration(reg: &DaemonRegistration) -> Self {
        ExpectedDaemon {
            pid: reg.pid,
            start_time_unix: reg.start_time_unix,
            instance_id: reg.instance_id.clone(),
        }
    }
}
impl LogTailer {
    /// Creates a new log tailer for the given file path.
    ///
    /// The tailer starts at the end of the file, so it only reads entries
    /// added after creation.
    pub fn new(path: PathBuf) -> Self {
        let (last_position, last_inode) = if path.exists() {
            match fs::metadata(&path) {
                Ok(metadata) => (metadata.len(), get_inode(&metadata)),
                Err(_) => (0, None),
            }
        } else {
            (0, None)
        };
        LogTailer { path, last_position, last_inode }
    }

    /// Reads new log entries since the last call.
    ///
    /// Returns a list of new log lines. Handles log rotation by detecting
    /// inode changes or file truncation.
    pub fn read_new_entries(&mut self) -> Vec<String> {
        let mut entries = Vec::new();
        if !self.path.exists() {
            return entries;
        }
        let metadata = match fs::metadata(&self.path) {
            Ok(m) => m,
            Err(e) => {
                warn!("Failed to read log file metadata: {}", e);
                return entries;
            }
        };
        let current_inode = get_inode(&metadata);
        let current_size = metadata.len();
        if current_inode != self.last_inode || current_size < self.last_position {
            debug!(
                "Log file rotated or truncated, resetting position path={} old_pos={} new_size={}",
                self.path.display(),
                self.last_position,
                current_size
            );
            self.last_position = 0;
            self.last_inode = current_inode;
        }
        if current_size == self.last_position {
            return entries;
        }
        let file = match File::open(&self.path) {
            Ok(f) => f,
            Err(e) => {
                warn!("Failed to open log file: {}", e);
                return entries;
            }
        };
        let mut reader = BufReader::new(file);
        if let Err(e) = reader.seek(SeekFrom::Start(self.last_position)) {
            warn!("Failed to seek in log file: {}", e);
            return entries;
        }
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => entries.push(line.trim_end().to_string()),
                Err(e) => {
                    warn!("Error reading log file: {}", e);
                    break;
                }
            }
        }
        self.last_position =
            reader.stream_position().unwrap_or_else(|_| self.last_position + entries.len() as u64);
        entries
    }

    /// Checks if any new entries contain ERROR or WARN level logs.
    ///
    /// Returns the first error/warning message found, or None if all entries
    /// are INFO or below.
    pub fn check_for_errors(&mut self) -> Option<String> {
        for entry in self.read_new_entries() {
            if let Some(message) = parse_log_level_error(&entry) {
                return Some(message);
            }
        }
        None
    }
}
impl HealthMonitor {
    /// Creates a new health monitor with the given configuration.
    pub fn new(config: OverseerConfig) -> Self {
        let log_path = auto_logging::auto_log_path();
        let log_tailer = LogTailer::new(log_path);
        HealthMonitor { config, log_tailer }
    }

    /// Performs a complete health check of the daemon.
    ///
    /// Checks are performed in priority order. Returns the first failure
    /// detected, or `HealthStatus::Healthy` if all checks pass.
    ///
    /// Priority order:
    /// 1. Process identity verification
    /// 2. Heartbeat check
    /// 3. Log monitoring for errors/warnings
    /// 4. Progress tracking (stall detection)
    pub fn check_daemon_health(&mut self, expected: &ExpectedDaemon) -> HealthStatus {
        if let Some(status) = self.check_process_identity(expected) {
            info!(status = ? status, "Health check failed: process identity");
            return status;
        }
        if let Some(status) = self.check_heartbeat(expected) {
            info!(status = ? status, "Health check failed: heartbeat");
            return status;
        }
        if let Some(status) = self.check_logs() {
            info!(status = ? status, "Health check failed: logs");
            return status;
        }
        if let Some(status) = self.check_progress() {
            info!(status = ? status, "Health check failed: progress");
            return status;
        }
        debug!("Health check passed");
        HealthStatus::Healthy
    }

    /// Verifies the daemon process identity matches expected values.
    ///
    /// Detects:
    /// - Daemon process gone (no registration file)
    /// - PID reuse (same PID but different start time or instance ID)
    /// - Unexpected daemon restart (different instance ID)
    fn check_process_identity(&self, expected: &ExpectedDaemon) -> Option<HealthStatus> {
        let registration = heartbeat_thread::read_daemon_registration()?;
        if registration.pid != expected.pid {
            return Some(HealthStatus::IdentityMismatch {
                reason: format!("PID changed from {} to {}", expected.pid, registration.pid),
            });
        }
        if registration.instance_id != expected.instance_id {
            return Some(HealthStatus::IdentityMismatch {
                reason: format!(
                    "Instance ID changed from {} to {}",
                    expected.instance_id, registration.instance_id
                ),
            });
        }
        if registration.start_time_unix != expected.start_time_unix {
            return Some(HealthStatus::IdentityMismatch {
                reason: format!(
                    "Start time changed from {} to {}",
                    expected.start_time_unix, registration.start_time_unix
                ),
            });
        }
        if !is_process_running(expected.pid) {
            return Some(HealthStatus::ProcessGone);
        }
        None
    }

    /// Checks if the heartbeat is fresh.
    fn check_heartbeat(&self, expected: &ExpectedDaemon) -> Option<HealthStatus> {
        let Some(heartbeat) = heartbeat_thread::read_heartbeat() else {
            return Some(HealthStatus::HeartbeatStale { age_secs: u64::MAX });
        };
        if heartbeat.instance_id != expected.instance_id {
            return Some(HealthStatus::IdentityMismatch {
                reason: format!(
                    "Heartbeat instance ID {} doesn't match expected {}",
                    heartbeat.instance_id, expected.instance_id
                ),
            });
        }
        let timeout = self.config.get_heartbeat_timeout();
        if heartbeat_thread::is_heartbeat_stale(&heartbeat, timeout) {
            let now = unix_timestamp_now();
            let age_secs = now.saturating_sub(heartbeat.timestamp_unix);
            return Some(HealthStatus::HeartbeatStale { age_secs });
        }
        None
    }

    /// Checks for errors or warnings in the daemon log.
    fn check_logs(&mut self) -> Option<HealthStatus> {
        self.log_tailer.check_for_errors().map(|message| HealthStatus::LogError { message })
    }

    /// Checks for stalled progress (no task completions within timeout).
    fn check_progress(&self) -> Option<HealthStatus> {
        let state_path = state::get_state_path();
        let state = match State::load(&state_path) {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to load state for progress check: {}", e);
                return None;
            }
        };
        let last_completion = state.last_task_completion_unix?;
        let now = unix_timestamp_now();
        let stall_secs = now.saturating_sub(last_completion);
        let stall_timeout = self.config.get_stall_timeout();
        if Duration::from_secs(stall_secs) > stall_timeout {
            return Some(HealthStatus::Stalled { stall_secs });
        }
        None
    }

    /// Returns a reference to the overseer configuration.
    pub fn config(&self) -> &OverseerConfig {
        &self.config
    }
}
/// Parses a JSON log entry to check if it's an ERROR or WARN level.
///
/// Returns the log message if it's an error/warning, None otherwise.
fn parse_log_level_error(line: &str) -> Option<String> {
    #[derive(Deserialize)]
    struct LogEntry {
        level: String,
        #[serde(default)]
        event: Option<serde_json::Value>,
    }
    let entry: LogEntry = serde_json::from_str(line).ok()?;
    let level = entry.level.to_uppercase();
    if level == "ERROR" || level == "WARN" {
        let message = entry
            .event
            .map(|e| {
                if let Some(obj) = e.as_object()
                    && let Some(error) = obj.get("error")
                {
                    return error.as_str().unwrap_or(line).to_string();
                }
                line.to_string()
            })
            .unwrap_or_else(|| line.to_string());
        return Some(message);
    }
    None
}
/// Returns the current Unix timestamp in seconds.
fn unix_timestamp_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|e| panic!("System time before UNIX epoch: {}", e))
        .as_secs()
}
/// Checks if a process with the given PID is running.
fn is_process_running(pid: u32) -> bool {
    #[cfg(unix)]
    {
        unsafe { libc::kill(pid as libc::pid_t, 0) == 0 }
    }
    #[cfg(not(unix))]
    {
        let _ = pid;
        true
    }
}
/// Gets the inode of a file (Unix only).
#[cfg(unix)]
fn get_inode(metadata: &Metadata) -> Option<u64> {
    Some(metadata.ino())
}
#[cfg(not(unix))]
fn get_inode(_metadata: &Metadata) -> Option<u64> {
    None
}
