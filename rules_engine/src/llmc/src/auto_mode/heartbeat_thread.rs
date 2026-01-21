// TODO: Remove this allow once heartbeat is integrated with auto_orchestrator
#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::config;

/// Interval between heartbeat updates in seconds.
const HEARTBEAT_INTERVAL_SECS: u64 = 5;

/// Heartbeat record written to `.llmc/auto.heartbeat`.
///
/// Used by external processes (like the overseer) to detect if the auto daemon
/// is running and healthy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    /// Unix timestamp of the last heartbeat update.
    pub timestamp_unix: u64,
    /// Unique identifier for this daemon instance.
    pub instance_id: String,
}

/// Daemon registration record written to `.llmc/daemon.json`.
///
/// Contains metadata about the running daemon for identification and debugging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonRegistration {
    /// Process ID of the daemon.
    pub pid: u32,
    /// Unix timestamp when the daemon started.
    pub start_time_unix: u64,
    /// Unique identifier for this daemon instance.
    pub instance_id: String,
    /// Path to the daemon's log file.
    pub log_file: String,
}

/// Handle for managing the heartbeat background thread.
pub struct HeartbeatThread {
    /// Signal to stop the heartbeat thread.
    stop_signal: Arc<AtomicBool>,
    /// Join handle for the background thread.
    join_handle: Option<JoinHandle<()>>,
    /// Instance ID for this daemon.
    instance_id: String,
}

/// Returns the path to the heartbeat file.
pub fn heartbeat_path() -> PathBuf {
    config::get_llmc_root().join("auto.heartbeat")
}

/// Returns the path to the daemon registration file.
pub fn daemon_registration_path() -> PathBuf {
    config::get_llmc_root().join("daemon.json")
}

/// Reads the daemon registration file if it exists.
pub fn read_daemon_registration() -> Option<DaemonRegistration> {
    let path = daemon_registration_path();
    if !path.exists() {
        return None;
    }
    match fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(registration) => Some(registration),
            Err(e) => {
                warn!("Failed to parse daemon registration: {}", e);
                None
            }
        },
        Err(e) => {
            warn!("Failed to read daemon registration: {}", e);
            None
        }
    }
}

/// Reads the heartbeat file if it exists.
pub fn read_heartbeat() -> Option<Heartbeat> {
    let path = heartbeat_path();
    if !path.exists() {
        return None;
    }
    match fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(heartbeat) => Some(heartbeat),
            Err(e) => {
                warn!("Failed to parse heartbeat: {}", e);
                None
            }
        },
        Err(e) => {
            warn!("Failed to read heartbeat: {}", e);
            None
        }
    }
}

/// Returns true if the heartbeat is stale (older than the given timeout).
pub fn is_heartbeat_stale(heartbeat: &Heartbeat, timeout: Duration) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|e| panic!("System time before UNIX epoch: {}", e))
        .as_secs();
    let age_secs = now.saturating_sub(heartbeat.timestamp_unix);
    age_secs > timeout.as_secs()
}

/// Generates a new unique instance ID.
pub fn generate_instance_id() -> String {
    Uuid::new_v4().to_string()
}

/// Writes content to a file atomically using a temp file and rename.
fn atomic_write(path: &PathBuf, content: &str) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| panic!("Path has no parent: {:?}", path));
    fs::create_dir_all(parent)
        .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    let temp_filename = format!(
        "{}.{}.tmp",
        path.file_name()
            .unwrap_or_else(|| panic!("Path has no filename: {:?}", path))
            .to_string_lossy(),
        std::process::id()
    );
    let temp_path = parent.join(&temp_filename);
    fs::write(&temp_path, content)
        .with_context(|| format!("Failed to write temp file: {}", temp_path.display()))?;
    fs::rename(&temp_path, path).with_context(|| {
        format!("Failed to rename {} to {}", temp_path.display(), path.display())
    })?;
    Ok(())
}

impl Heartbeat {
    /// Creates a new heartbeat with the current timestamp.
    pub fn new(instance_id: &str) -> Self {
        let timestamp_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|e| panic!("System time before UNIX epoch: {}", e))
            .as_secs();
        Heartbeat { timestamp_unix, instance_id: instance_id.to_string() }
    }

    /// Writes this heartbeat to the heartbeat file atomically.
    pub fn write(&self) -> Result<()> {
        let path = heartbeat_path();
        let content = serde_json::to_string(self).context("Failed to serialize heartbeat")?;
        atomic_write(&path, &content)
    }
}

impl DaemonRegistration {
    /// Creates a new daemon registration with the current process info.
    pub fn new(instance_id: &str, log_file: &str) -> Self {
        let start_time_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|e| panic!("System time before UNIX epoch: {}", e))
            .as_secs();
        DaemonRegistration {
            pid: std::process::id(),
            start_time_unix,
            instance_id: instance_id.to_string(),
            log_file: log_file.to_string(),
        }
    }

    /// Writes this registration to the daemon registration file atomically.
    pub fn write(&self) -> Result<()> {
        let path = daemon_registration_path();
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize daemon registration")?;
        atomic_write(&path, &content)
    }

    /// Removes the daemon registration file.
    pub fn remove() -> Result<()> {
        let path = daemon_registration_path();
        if path.exists() {
            fs::remove_file(&path).with_context(|| {
                format!("Failed to remove daemon registration: {}", path.display())
            })?;
        }
        Ok(())
    }
}

impl HeartbeatThread {
    /// Creates and starts a new heartbeat background thread.
    ///
    /// The thread updates the heartbeat file every 5 seconds until stopped.
    pub fn start(instance_id: &str) -> Self {
        let stop_signal = Arc::new(AtomicBool::new(false));
        let stop_signal_clone = Arc::clone(&stop_signal);
        let instance_id_owned = instance_id.to_string();
        let instance_id_for_thread = instance_id_owned.clone();
        let join_handle = thread::spawn(move || {
            heartbeat_loop(&instance_id_for_thread, &stop_signal_clone);
        });
        info!("Heartbeat thread started for instance {}", instance_id);
        HeartbeatThread {
            stop_signal,
            join_handle: Some(join_handle),
            instance_id: instance_id_owned,
        }
    }

    /// Stops the heartbeat thread and waits for it to terminate.
    pub fn stop(&mut self) {
        self.stop_signal.store(true, Ordering::SeqCst);
        if let Some(handle) = self.join_handle.take() {
            if let Err(e) = handle.join() {
                error!("Heartbeat thread panicked: {:?}", e);
            } else {
                info!("Heartbeat thread stopped for instance {}", self.instance_id);
            }
        }
    }

    /// Returns the instance ID for this heartbeat thread.
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }
}

impl Drop for HeartbeatThread {
    fn drop(&mut self) {
        self.stop();
    }
}

fn heartbeat_loop(instance_id: &str, stop_signal: &AtomicBool) {
    while !stop_signal.load(Ordering::SeqCst) {
        let heartbeat = Heartbeat::new(instance_id);
        if let Err(e) = heartbeat.write() {
            error!("Failed to write heartbeat: {}. Will retry.", e);
        }
        let mut elapsed = Duration::ZERO;
        let check_interval = Duration::from_millis(100);
        let target_interval = Duration::from_secs(HEARTBEAT_INTERVAL_SECS);
        while elapsed < target_interval && !stop_signal.load(Ordering::SeqCst) {
            thread::sleep(check_interval);
            elapsed += check_interval;
        }
    }
}
