use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use std::{fs, thread};

use anyhow::{Context, Result, bail};
use tracing::{debug, error, info, warn};

use crate::auto_mode::heartbeat_thread;
use crate::config::{self, Config};
use crate::overseer_mode::daemon_control::{self, TerminationResult};
use crate::overseer_mode::health_monitor::{ExpectedDaemon, HealthMonitor, HealthStatus};
use crate::overseer_mode::overseer_config::OverseerConfig;
use crate::overseer_mode::{overseer_session, remediation_executor, remediation_prompt};

const HEALTH_CHECK_INTERVAL_SECS: u64 = 5;
const DAEMON_STARTUP_TIMEOUT_SECS: u64 = 60;
const DAEMON_STARTUP_POLL_INTERVAL_MS: u64 = 500;

/// Runs the overseer supervisor loop.
///
/// The overseer:
/// 1. Starts the daemon via shell command (`llmc up --auto`)
/// 2. Monitors daemon health continuously
/// 3. On failure, terminates daemon and runs remediation
/// 4. Detects failure spirals (repeated failures within cooldown)
/// 5. Handles Ctrl-C for graceful shutdown
pub fn run_overseer(config: &Config) -> Result<()> {
    let overseer_config = validate_overseer_config(config)?;
    info!("Starting overseer supervisor");

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = Arc::clone(&shutdown);
    ctrlc::set_handler(move || {
        println!("\nReceived Ctrl-C, shutting down overseer...");
        shutdown_clone.store(true, Ordering::SeqCst);
    })
    .context("Failed to set Ctrl-C handler")?;

    overseer_session::ensure_overseer_session(config)?;
    println!("✓ Overseer Claude Code session ready");

    let mut monitor = HealthMonitor::new(overseer_config.clone());

    loop {
        if shutdown.load(Ordering::SeqCst) {
            info!("Shutdown requested, terminating overseer");
            break;
        }

        let mut daemon_handle = start_daemon_and_wait_for_registration(&shutdown)?;
        let daemon_start_time = Instant::now();
        println!(
            "✓ Daemon started (PID: {}, instance: {})",
            daemon_handle.expected.pid, daemon_handle.expected.instance_id
        );

        let failure = run_monitor_loop(&mut monitor, &daemon_handle.expected, &shutdown);

        if shutdown.load(Ordering::SeqCst) {
            info!("Shutdown requested during monitoring, terminating daemon");
            terminate_daemon_gracefully(&daemon_handle.expected);
            // Also kill the child process directly in case terminate_daemon_gracefully
            // fails
            let _ = daemon_handle.child.kill();
            break;
        }

        let Some(failure_status) = failure else {
            continue;
        };

        println!("⚠ Daemon failure detected: {}", failure_status.describe());
        info!(failure = ?failure_status, "Daemon failure detected");

        terminate_daemon_gracefully(&daemon_handle.expected);
        // Also kill the child process directly to ensure cleanup
        let _ = daemon_handle.child.kill();

        if is_failure_spiral(daemon_start_time, &overseer_config) {
            error!("Failure spiral detected - daemon failed within cooldown period");
            println!(
                "\n❌ FAILURE SPIRAL DETECTED\n\n\
                 The daemon failed within {} seconds of the last restart.\n\
                 This indicates a persistent issue that remediation cannot fix.\n\n\
                 Possible causes:\n\
                 - Configuration error in config.toml\n\
                 - External system failure (network, disk, API limits)\n\
                 - Bug introduced by previous remediation\n\n\
                 Please investigate manually and restart the overseer when ready.",
                overseer_config.restart_cooldown_secs
            );
            bail!("Failure spiral detected - human intervention required");
        }

        if check_manual_intervention_needed()? {
            error!("Manual intervention file found - terminating overseer");
            bail!("Manual intervention required - see .llmc/manual_intervention_needed_*.txt");
        }

        println!("Entering remediation mode...");
        run_remediation(&failure_status, config, &shutdown)?;

        if shutdown.load(Ordering::SeqCst) {
            info!("Shutdown requested during remediation");
            break;
        }

        if check_manual_intervention_needed()? {
            error!("Manual intervention file created during remediation");
            bail!("Manual intervention required - see .llmc/manual_intervention_needed_*.txt");
        }

        println!("Remediation complete. Restarting daemon...");
    }

    println!("✓ Overseer shutdown complete");
    Ok(())
}

/// Handle to a running daemon process with output piping threads.
struct DaemonHandle {
    child: Child,
    expected: ExpectedDaemon,
    /// Handle to the stdout piping thread (if daemon was started with output
    /// capture).
    _stdout_thread: Option<JoinHandle<()>>,
    /// Handle to the stderr piping thread (if daemon was started with output
    /// capture).
    _stderr_thread: Option<JoinHandle<()>>,
}

/// Validates that the overseer configuration is present and complete.
fn validate_overseer_config(config: &Config) -> Result<OverseerConfig> {
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

    info!(
        heartbeat_timeout_secs = overseer_config.heartbeat_timeout_secs,
        stall_timeout_secs = overseer_config.stall_timeout_secs,
        restart_cooldown_secs = overseer_config.restart_cooldown_secs,
        "Overseer configuration validated"
    );

    Ok(overseer_config.clone())
}

/// Starts the daemon and waits for registration.
///
/// Spawns the daemon as a child process with captured stdout/stderr, pipes
/// daemon output to the overseer's stdout, and waits for the daemon to register
/// via heartbeat file.
fn start_daemon_and_wait_for_registration(shutdown: &Arc<AtomicBool>) -> Result<DaemonHandle> {
    info!("Starting daemon");
    println!("Starting daemon...");

    let mut child = Command::new("llmc")
        .args(["up", "--auto"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn daemon process")?;

    debug!(child_pid = child.id(), "Daemon child process spawned");

    // Spawn thread to pipe daemon stdout to overseer stdout
    let stdout = child.stdout.take();
    let stdout_thread = stdout.map(|stdout| {
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                match line {
                    Ok(line) => println!("{}", line),
                    Err(e) => {
                        debug!("Daemon stdout read error: {}", e);
                        break;
                    }
                }
            }
        })
    });

    // Spawn thread to pipe daemon stderr to overseer stderr
    let stderr = child.stderr.take();
    let stderr_thread = stderr.map(|stderr| {
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                match line {
                    Ok(line) => eprintln!("{}", line),
                    Err(e) => {
                        debug!("Daemon stderr read error: {}", e);
                        break;
                    }
                }
            }
        })
    });

    let deadline = Instant::now() + Duration::from_secs(DAEMON_STARTUP_TIMEOUT_SECS);
    while Instant::now() < deadline {
        if shutdown.load(Ordering::SeqCst) {
            // Clean up child process on shutdown
            let _ = child.kill();
            bail!("Shutdown requested during daemon startup");
        }

        // Check if child process exited unexpectedly
        if let Ok(Some(status)) = child.try_wait() {
            bail!("Daemon process exited unexpectedly during startup with status: {}", status);
        }

        if let Some(registration) = heartbeat_thread::read_daemon_registration() {
            info!(
                pid = registration.pid,
                instance_id = %registration.instance_id,
                start_time = registration.start_time_unix,
                "Daemon registered successfully"
            );
            let expected = ExpectedDaemon::from_registration(&registration);
            return Ok(DaemonHandle {
                child,
                expected,
                _stdout_thread: stdout_thread,
                _stderr_thread: stderr_thread,
            });
        }

        thread::sleep(Duration::from_millis(DAEMON_STARTUP_POLL_INTERVAL_MS));
    }

    // Clean up child process on timeout
    let _ = child.kill();
    bail!(
        "Daemon failed to register within {} seconds.\n\
         Check logs at ~/llmc/logs/ for details.",
        DAEMON_STARTUP_TIMEOUT_SECS
    );
}

/// Runs the health monitoring loop until a failure is detected or shutdown.
fn run_monitor_loop(
    monitor: &mut HealthMonitor,
    expected: &ExpectedDaemon,
    shutdown: &Arc<AtomicBool>,
) -> Option<HealthStatus> {
    info!(pid = expected.pid, "Entering health monitoring loop");
    println!("Monitoring daemon health (Ctrl-C to stop)...");

    loop {
        if shutdown.load(Ordering::SeqCst) {
            return None;
        }

        thread::sleep(Duration::from_secs(HEALTH_CHECK_INTERVAL_SECS));

        let status = monitor.check_daemon_health(expected);
        if !status.is_healthy() {
            return Some(status);
        }

        debug!("Health check passed");
    }
}

/// Terminates the daemon gracefully with full termination protocol.
fn terminate_daemon_gracefully(expected: &ExpectedDaemon) {
    info!(pid = expected.pid, "Initiating daemon termination");
    println!("Terminating daemon...");

    match daemon_control::terminate_daemon(expected) {
        Ok(TerminationResult::GracefulShutdown) => {
            info!("Daemon terminated gracefully");
            println!("✓ Daemon terminated gracefully");
        }
        Ok(TerminationResult::ForcefulKill) => {
            warn!("Daemon required forceful kill");
            println!("✓ Daemon terminated (required SIGKILL)");
        }
        Ok(TerminationResult::AlreadyGone) => {
            info!("Daemon was already gone");
            println!("✓ Daemon was already terminated");
        }
        Ok(TerminationResult::Failed { reason }) => {
            error!(reason = %reason, "Failed to terminate daemon");
            println!("⚠ Failed to terminate daemon: {}", reason);
        }
        Err(e) => {
            error!(error = %e, "Error during daemon termination");
            println!("⚠ Error during termination: {}", e);
        }
    }
}

/// Checks if this is a failure spiral (daemon failed too quickly after
/// restart).
fn is_failure_spiral(start_time: Instant, config: &OverseerConfig) -> bool {
    let elapsed = start_time.elapsed();
    let cooldown = config.get_restart_cooldown();

    if elapsed < cooldown {
        warn!(
            elapsed_secs = elapsed.as_secs(),
            cooldown_secs = cooldown.as_secs(),
            "Daemon failed within restart cooldown - failure spiral detected"
        );
        true
    } else {
        debug!(
            elapsed_secs = elapsed.as_secs(),
            cooldown_secs = cooldown.as_secs(),
            "Daemon ran longer than cooldown, not a failure spiral"
        );
        false
    }
}

/// Checks for manual intervention files.
fn check_manual_intervention_needed() -> Result<bool> {
    let llmc_root = config::get_llmc_root();
    let llmc_dir = llmc_root.join(".llmc");

    if !llmc_dir.exists() {
        return Ok(false);
    }

    let entries = fs::read_dir(&llmc_dir).context("Failed to read .llmc directory")?;

    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        if name.starts_with("manual_intervention_needed_") && name.ends_with(".txt") {
            let path = entry.path();
            match fs::read_to_string(&path) {
                Ok(content) => {
                    error!(
                        path = %path.display(),
                        content = %content,
                        "Manual intervention file found"
                    );
                    println!(
                        "\n❌ MANUAL INTERVENTION REQUIRED\n\n\
                         File: {}\n\n\
                         Content:\n{}\n",
                        path.display(),
                        content
                    );
                }
                Err(e) => {
                    warn!(path = %path.display(), error = %e, "Failed to read intervention file");
                    println!(
                        "\n❌ MANUAL INTERVENTION REQUIRED\n\n\
                         File: {} (could not read: {})\n",
                        path.display(),
                        e
                    );
                }
            }
            return Ok(true);
        }
    }

    Ok(false)
}

/// Runs the remediation process.
fn run_remediation(
    failure: &HealthStatus,
    config: &Config,
    shutdown: &Arc<AtomicBool>,
) -> Result<()> {
    info!(failure = ?failure, "Starting remediation");

    let prompt = remediation_prompt::build_remediation_prompt(failure, config);
    debug!(prompt_length = prompt.len(), "Built remediation prompt");

    remediation_executor::execute_remediation(&prompt, config, shutdown)?;

    info!("Remediation completed");
    Ok(())
}
