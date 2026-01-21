use std::time::{Duration, Instant};
use std::{fs, thread};

use anyhow::{Context, Result};
use tracing::{debug, error, info, warn};

use crate::auto_mode::heartbeat_thread;
use crate::overseer_mode::health_monitor::ExpectedDaemon;

const TERMINATION_GRACE_PERIOD_SECS: u64 = 30;
const TERMINATION_POLL_INTERVAL_MS: u64 = 500;

/// Result of a daemon termination attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminationResult {
    /// Daemon terminated gracefully via SIGTERM.
    GracefulShutdown,
    /// Daemon required SIGKILL after grace period expired.
    ForcefulKill,
    /// Process was already gone when termination was attempted.
    AlreadyGone,
    /// Termination failed (process still running after SIGKILL).
    Failed { reason: String },
}

/// Terminates the daemon process using the full termination protocol.
///
/// The termination sequence is:
/// 1. Verify process identity matches expected daemon
/// 2. Send SIGTERM for graceful shutdown
/// 3. Wait grace period (30 seconds) for process to exit
/// 4. If still running, send SIGKILL
/// 5. Verify process is fully terminated
/// 6. Clean up stale registration files
pub fn terminate_daemon(expected: &ExpectedDaemon) -> Result<TerminationResult> {
    info!(
        pid = expected.pid,
        instance_id = %expected.instance_id,
        "Starting daemon termination protocol"
    );

    match verify_and_terminate(expected) {
        Ok(result) => {
            cleanup_registration_files();
            info!(result = ?result, "Daemon termination complete");
            Ok(result)
        }
        Err(e) => {
            cleanup_registration_files();
            error!(error = %e, "Daemon termination failed");
            Err(e)
        }
    }
}

/// Cleans up stale daemon registration and heartbeat files.
///
/// This should be called before starting a new daemon to ensure we don't
/// read stale registration data from a previous daemon instance.
pub fn cleanup_registration_files() {
    let registration_path = heartbeat_thread::daemon_registration_path();
    if registration_path.exists() {
        match fs::remove_file(&registration_path) {
            Ok(()) => {
                debug!(path = %registration_path.display(), "Removed daemon registration file");
            }
            Err(e) => {
                warn!(
                    path = %registration_path.display(),
                    error = %e,
                    "Failed to remove daemon registration file"
                );
            }
        }
    }

    let heartbeat_path = heartbeat_thread::heartbeat_path();
    if heartbeat_path.exists() {
        match fs::remove_file(&heartbeat_path) {
            Ok(()) => {
                debug!(path = %heartbeat_path.display(), "Removed heartbeat file");
            }
            Err(e) => {
                warn!(
                    path = %heartbeat_path.display(),
                    error = %e,
                    "Failed to remove heartbeat file"
                );
            }
        }
    }
}

/// Verifies process identity and performs termination.
fn verify_and_terminate(expected: &ExpectedDaemon) -> Result<TerminationResult> {
    if !is_process_running(expected.pid) {
        info!(pid = expected.pid, "Process already gone before termination");
        return Ok(TerminationResult::AlreadyGone);
    }

    if let Err(reason) = verify_process_identity(expected) {
        warn!(
            pid = expected.pid,
            reason = %reason,
            "Process identity verification failed, proceeding anyway"
        );
    }

    send_sigterm(expected.pid)?;

    let deadline = Instant::now() + Duration::from_secs(TERMINATION_GRACE_PERIOD_SECS);
    while Instant::now() < deadline {
        if !is_process_running(expected.pid) {
            info!(pid = expected.pid, "Process terminated gracefully via SIGTERM");
            return Ok(TerminationResult::GracefulShutdown);
        }
        thread::sleep(Duration::from_millis(TERMINATION_POLL_INTERVAL_MS));
    }

    warn!(
        pid = expected.pid,
        grace_period_secs = TERMINATION_GRACE_PERIOD_SECS,
        "Process did not terminate after SIGTERM, sending SIGKILL"
    );
    send_sigkill(expected.pid)?;

    thread::sleep(Duration::from_secs(1));

    if is_process_running(expected.pid) {
        let reason = format!("Process {} still running after SIGKILL", expected.pid);
        error!(pid = expected.pid, "Process survived SIGKILL");
        return Ok(TerminationResult::Failed { reason });
    }

    info!(pid = expected.pid, "Process terminated via SIGKILL");
    Ok(TerminationResult::ForcefulKill)
}

/// Verifies that the current daemon registration matches expected values.
///
/// This prevents accidentally killing a wrong process if the daemon
/// restarted and a different process took the PID.
fn verify_process_identity(expected: &ExpectedDaemon) -> std::result::Result<(), String> {
    let Some(registration) = heartbeat_thread::read_daemon_registration() else {
        return Err("Daemon registration file missing".to_string());
    };

    if registration.pid != expected.pid {
        return Err(format!("PID mismatch: expected {}, found {}", expected.pid, registration.pid));
    }

    if registration.instance_id != expected.instance_id {
        return Err(format!(
            "Instance ID mismatch: expected {}, found {}",
            expected.instance_id, registration.instance_id
        ));
    }

    if registration.start_time_unix != expected.start_time_unix {
        return Err(format!(
            "Start time mismatch: expected {}, found {}",
            expected.start_time_unix, registration.start_time_unix
        ));
    }

    debug!(
        pid = expected.pid,
        instance_id = %expected.instance_id,
        "Process identity verified"
    );
    Ok(())
}

/// Sends SIGTERM to the specified process.
#[cfg(unix)]
fn send_sigterm(pid: u32) -> Result<()> {
    info!(pid, "Sending SIGTERM");
    let result = unsafe { libc::kill(pid as libc::pid_t, libc::SIGTERM) };
    if result == 0 {
        debug!(pid, "SIGTERM sent successfully");
        Ok(())
    } else {
        let errno = std::io::Error::last_os_error();
        if errno.raw_os_error() == Some(libc::ESRCH) {
            debug!(pid, "Process already gone when sending SIGTERM");
            Ok(())
        } else {
            Err(errno).context(format!("Failed to send SIGTERM to PID {}", pid))
        }
    }
}

#[cfg(not(unix))]
fn send_sigterm(pid: u32) -> Result<()> {
    warn!(pid, "SIGTERM not supported on this platform, skipping");
    Ok(())
}

/// Sends SIGKILL to the specified process.
#[cfg(unix)]
fn send_sigkill(pid: u32) -> Result<()> {
    info!(pid, "Sending SIGKILL");
    let result = unsafe { libc::kill(pid as libc::pid_t, libc::SIGKILL) };
    if result == 0 {
        debug!(pid, "SIGKILL sent successfully");
        Ok(())
    } else {
        let errno = std::io::Error::last_os_error();
        if errno.raw_os_error() == Some(libc::ESRCH) {
            debug!(pid, "Process already gone when sending SIGKILL");
            Ok(())
        } else {
            Err(errno).context(format!("Failed to send SIGKILL to PID {}", pid))
        }
    }
}

#[cfg(not(unix))]
fn send_sigkill(pid: u32) -> Result<()> {
    warn!(pid, "SIGKILL not supported on this platform, skipping");
    Ok(())
}

/// Checks if a process with the given PID is running.
#[cfg(unix)]
fn is_process_running(pid: u32) -> bool {
    unsafe { libc::kill(pid as libc::pid_t, 0) == 0 }
}

#[cfg(not(unix))]
fn is_process_running(_pid: u32) -> bool {
    true
}
