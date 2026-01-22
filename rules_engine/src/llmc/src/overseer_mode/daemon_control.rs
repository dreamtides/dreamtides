use std::process::Child;
use std::time::{Duration, Instant};
use std::{fs, thread};

use anyhow::{Context, Result};
use tracing::{debug, error, info};

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
///
/// If `child` is provided, it will be used to reap the zombie process after
/// SIGKILL, ensuring proper cleanup when the overseer is the parent process.
pub fn terminate_daemon(
    expected: &ExpectedDaemon,
    child: Option<&mut Child>,
) -> Result<TerminationResult> {
    info!(
        pid = expected.pid,
        instance_id = %expected.instance_id,
        "Starting daemon termination protocol"
    );

    match verify_and_terminate(expected, child) {
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
                info!(
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
                info!(
                    path = %heartbeat_path.display(),
                    error = %e,
                    "Failed to remove heartbeat file"
                );
            }
        }
    }
}

/// Cleans up any existing LLMC sessions before starting a new daemon.
///
/// This runs `llmc down --force` to ensure any stale sessions from a
/// previous daemon instance are terminated. This is necessary because
/// the overseer may have crashed while sessions were still running.
pub fn cleanup_existing_sessions() -> Result<()> {
    info!("Cleaning up any existing LLMC sessions before daemon startup");

    let output = std::process::Command::new("llmc")
        .args(["down", "--force"])
        .output()
        .context("Failed to run 'llmc down --force'")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        info!(
            exit_code = output.status.code(),
            stderr = %stderr,
            "llmc down --force returned non-zero exit code (continuing anyway)"
        );
    } else {
        debug!("Successfully cleaned up existing sessions");
    }

    Ok(())
}

/// Verifies process identity and performs termination.
///
/// If `child` is provided, it will be used to reap the zombie process after
/// SIGKILL. This is necessary when the calling process is the parent of the
/// daemon, because `kill(pid, 0)` returns 0 for zombie processes until they
/// are reaped via `wait()`.
fn verify_and_terminate(
    expected: &ExpectedDaemon,
    mut child: Option<&mut Child>,
) -> Result<TerminationResult> {
    if !is_process_running(expected.pid) {
        info!(pid = expected.pid, "Process already gone before termination");
        return Ok(TerminationResult::AlreadyGone);
    }

    if let Err(reason) = verify_process_identity(expected) {
        info!(
            pid = expected.pid,
            reason = %reason,
            "Process identity verification failed, proceeding anyway"
        );
    }

    send_sigterm(expected.pid)?;

    let deadline = Instant::now() + Duration::from_secs(TERMINATION_GRACE_PERIOD_SECS);
    while Instant::now() < deadline {
        // Try to reap the child if it has exited (avoids zombie state)
        if let Some(ref mut c) = child {
            match c.try_wait() {
                Ok(Some(status)) => {
                    info!(
                        pid = expected.pid,
                        exit_status = ?status,
                        "Process terminated gracefully via SIGTERM (reaped child)"
                    );
                    return Ok(TerminationResult::GracefulShutdown);
                }
                Ok(None) => {
                    // Child still running, continue waiting
                }
                Err(e) => {
                    debug!(
                        pid = expected.pid,
                        error = %e,
                        "Failed to check child status"
                    );
                }
            }
        } else if !is_process_running(expected.pid) {
            info!(pid = expected.pid, "Process terminated gracefully via SIGTERM");
            return Ok(TerminationResult::GracefulShutdown);
        }
        thread::sleep(Duration::from_millis(TERMINATION_POLL_INTERVAL_MS));
    }

    info!(
        pid = expected.pid,
        grace_period_secs = TERMINATION_GRACE_PERIOD_SECS,
        "Process did not terminate after SIGTERM, sending SIGKILL"
    );
    send_sigkill(expected.pid)?;

    // Allow a moment for the kernel to process the signal
    thread::sleep(Duration::from_millis(100));

    // If we have the child handle, use wait() to reap the zombie.
    // This is critical because kill(pid, 0) returns 0 for zombie processes,
    // making them appear "still running" until they are reaped by their parent.
    if let Some(ref mut c) = child {
        match c.wait() {
            Ok(status) => {
                info!(
                    pid = expected.pid,
                    exit_status = ?status,
                    "Process terminated via SIGKILL (reaped child)"
                );
                return Ok(TerminationResult::ForcefulKill);
            }
            Err(e) => {
                // wait() failed, fall back to kill(pid, 0) check
                debug!(
                    pid = expected.pid,
                    error = %e,
                    "Failed to wait on child, falling back to process check"
                );
            }
        }
    }

    // Fallback: wait a bit more and check via kill(pid, 0)
    thread::sleep(Duration::from_millis(900));

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
    info!(pid, "SIGTERM not supported on this platform, skipping");
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
    info!(pid, "SIGKILL not supported on this platform, skipping");
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
