use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::auto_mode::heartbeat_thread;
use crate::config::{self, Config};
use crate::overseer_mode::overseer_loop;

/// Overseer registration record written to `.llmc/overseer.json`.
///
/// Contains metadata about the running overseer for detection by `llmc status`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverseerRegistration {
    /// Process ID of the overseer.
    pub pid: u32,
    /// Unix timestamp when the overseer started.
    pub start_time_unix: u64,
    /// Unique identifier for this overseer instance.
    pub instance_id: String,
}

/// CLI options passed to the overseer that will be forwarded to the daemon.
#[derive(Debug, Clone, Default)]
pub struct OverseerDaemonOptions {
    pub task_pool_command: Option<String>,
    pub concurrency: Option<u32>,
    pub post_accept_command: Option<String>,
}

/// Returns the path to the overseer registration file.
pub fn overseer_registration_path() -> PathBuf {
    config::get_llmc_root().join("overseer.json")
}

/// Reads the overseer registration from disk, if present.
pub fn read_overseer_registration() -> Result<Option<OverseerRegistration>> {
    let path = overseer_registration_path();
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read overseer registration: {}", path.display()))?;
    let registration: OverseerRegistration = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse overseer registration: {}", path.display()))?;
    Ok(Some(registration))
}

/// Runs the overseer command.
///
/// This command starts the overseer supervisor process which manages the
/// auto mode daemon. The overseer monitors the daemon, detects failures,
/// and uses Claude Code to remediate issues.
pub fn run_overseer(
    task_pool_command: Option<String>,
    concurrency: Option<u32>,
    post_accept_command: Option<String>,
) -> Result<()> {
    let config_path = config::get_config_path();
    let config = Config::load(&config_path)?;
    validate_overseer_config(&config)?;
    check_existing_overseer()?;
    let instance_id = heartbeat_thread::generate_instance_id();
    let registration = OverseerRegistration::new(&instance_id);
    registration.write()?;
    info!(
        pid = registration.pid,
        instance_id = %registration.instance_id,
        "Overseer registered"
    );
    let daemon_options =
        OverseerDaemonOptions { task_pool_command, concurrency, post_accept_command };
    let result = overseer_loop::run_overseer(&config, &daemon_options);
    if let Err(e) = OverseerRegistration::remove() {
        tracing::info!(
            error = %e,
            "Failed to remove overseer registration during cleanup (non-fatal, \
             stale registration will be cleaned up on next startup)"
        );
    }
    result
}

pub fn is_process_alive(pid: u32) -> bool {
    std::process::Command::new("kill")
        .args(["-0", &pid.to_string()])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

impl OverseerRegistration {
    fn new(instance_id: &str) -> Self {
        let start_time_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|e| panic!("System time before UNIX epoch: {}", e))
            .as_secs();
        OverseerRegistration {
            pid: std::process::id(),
            start_time_unix,
            instance_id: instance_id.to_string(),
        }
    }

    fn write(&self) -> Result<()> {
        let path = overseer_registration_path();
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
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize overseer registration")?;
        fs::write(&temp_path, &content)
            .with_context(|| format!("Failed to write temp file: {}", temp_path.display()))?;
        fs::rename(&temp_path, &path).with_context(|| {
            format!("Failed to rename {} to {}", temp_path.display(), path.display())
        })?;
        info!(path = %path.display(), "Wrote overseer registration");
        Ok(())
    }

    fn remove() -> Result<()> {
        let path = overseer_registration_path();
        if path.exists() {
            fs::remove_file(&path).with_context(|| {
                format!("Failed to remove overseer registration: {}", path.display())
            })?;
            info!(path = %path.display(), "Removed overseer registration");
        }
        Ok(())
    }
}

fn validate_overseer_config(config: &Config) -> Result<()> {
    let Some(ref overseer_config) = config.overseer else {
        bail!(
            "Overseer requires [overseer] section in config.toml.\n\
             Add:\n\n\
             [overseer]\n\
             remediation_prompt = \"Your instructions for Claude Code remediation\""
        );
    };
    if overseer_config.remediation_prompt.is_none() {
        bail!(
            "Overseer requires 'remediation_prompt' in [overseer] section.\n\
             Add:\n\n\
             [overseer]\n\
             remediation_prompt = \"Your instructions for Claude Code remediation\""
        );
    }
    Ok(())
}

fn check_existing_overseer() -> Result<()> {
    if let Some(registration) = read_overseer_registration()? {
        if is_process_alive(registration.pid) {
            bail!(
                "Another overseer is already running (PID: {}, started at: {}).\n\
                 Only one overseer can run at a time.\n\
                 To stop the existing overseer, send Ctrl-C to that process or:\n\
                   kill {}",
                registration.pid,
                registration.start_time_unix,
                registration.pid
            );
        }
        info!(
            pid = registration.pid,
            instance_id = %registration.instance_id,
            "Found stale overseer registration (process not running), will overwrite"
        );
    }
    Ok(())
}
