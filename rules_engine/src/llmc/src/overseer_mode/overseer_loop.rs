use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use std::{fs, thread};

use anyhow::{Context, Result, bail};
use tracing::{debug, error, info};

use crate::auto_mode::heartbeat_thread;
use crate::commands::overseer::OverseerDaemonOptions;
use crate::config::{self, Config};
use crate::overseer_mode::daemon_control::{self, TerminationResult};
use crate::overseer_mode::health_monitor::{ExpectedDaemon, HealthMonitor, HealthStatus};
use crate::overseer_mode::overseer_config::OverseerConfig;
use crate::overseer_mode::{overseer_session, remediation_executor, remediation_prompt};

const HEALTH_CHECK_INTERVAL_SECS: u64 = 5;
const DAEMON_STARTUP_TIMEOUT_SECS: u64 = 60;
const DAEMON_STARTUP_POLL_INTERVAL_MS: u64 = 500;

/// Tracks failure spiral state across daemon restarts.
///
/// A failure spiral is when the daemon fails repeatedly within a short time,
/// indicating that remediation is not working. However, we must attempt
/// remediation at least once before declaring a failure spiral - otherwise
/// we would never try to fix the problem.
///
/// Design invariant: First failure should always attempt remediation.
/// Only after at least one remediation attempt can a quick failure be
/// considered a "spiral".
pub struct FailureSpiralTracker {
    cooldown: Duration,
    remediation_attempted: bool,
}

/// Runs the overseer supervisor loop.
///
/// The overseer:
/// 1. Starts the daemon via shell command (`llmc up --auto`)
/// 2. Monitors daemon health continuously
/// 3. On failure, terminates daemon and runs remediation
/// 4. Detects failure spirals (repeated failures within cooldown)
/// 5. Handles Ctrl-C for graceful shutdown
pub fn run_overseer(config: &Config, daemon_options: &OverseerDaemonOptions) -> Result<()> {
    let overseer_config = validate_overseer_config(config)?;
    info!("Starting overseer supervisor");

    let shutdown_count: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
    let shutdown_clone: Arc<AtomicU32> = Arc::clone(&shutdown_count);
    // Register signal handler for both SIGINT (Ctrl-C) and SIGTERM (kill).
    // The ctrlc crate with "termination" feature handles both signals.
    ctrlc::set_handler(move || {
        let count = shutdown_clone.fetch_add(1, Ordering::SeqCst) + 1;
        // Use write! to stderr for more reliable output in signal context,
        // and flush to ensure the message is visible immediately.
        if count == 1 {
            let _ = writeln!(
                std::io::stderr(),
                "\nReceived shutdown signal, shutting down overseer and daemon gracefully..."
            );
            let _ =
                writeln!(std::io::stderr(), "Send signal again to force immediate termination.");
            let _ = std::io::stderr().flush();
        } else {
            let _ = writeln!(
                std::io::stderr(),
                "\nReceived second shutdown signal, forcing immediate termination..."
            );
            let _ = std::io::stderr().flush();
            std::process::exit(130);
        }
    })
    .context("Failed to set signal handler for SIGINT/SIGTERM")?;
    info!("Signal handler registered for SIGINT and SIGTERM");

    overseer_session::ensure_overseer_session(config)?;
    println!("✓ Overseer Claude Code session ready");

    let mut monitor = HealthMonitor::new(overseer_config.clone());
    let mut spiral_tracker = FailureSpiralTracker::new(overseer_config.get_restart_cooldown());

    loop {
        if shutdown_count.load(Ordering::SeqCst) > 0 {
            info!("Shutdown requested, terminating overseer");
            break;
        }

        let mut daemon_handle =
            start_daemon_and_wait_for_registration(&shutdown_count, daemon_options)?;
        let daemon_start_time = Instant::now();
        println!(
            "✓ Daemon started (PID: {}, instance: {})",
            daemon_handle.expected.pid, daemon_handle.expected.instance_id
        );

        let failure = run_monitor_loop(&mut monitor, &daemon_handle.expected, &shutdown_count);

        if shutdown_count.load(Ordering::SeqCst) > 0 {
            info!("Shutdown requested during monitoring, terminating daemon");
            terminate_daemon_gracefully(&daemon_handle.expected, Some(&mut daemon_handle.child));
            break;
        }

        let Some(failure_status) = failure else {
            continue;
        };

        println!("⚠ Daemon failure detected: {}", failure_status.describe());
        info!(failure = ?failure_status, "Daemon failure detected");

        terminate_daemon_gracefully(&daemon_handle.expected, Some(&mut daemon_handle.child));

        let daemon_runtime = daemon_start_time.elapsed();

        // Check for failure spiral ONLY after remediation has been attempted.
        // This is critical: we must try to fix the problem at least once before
        // declaring that remediation isn't working.
        if spiral_tracker.should_detect_spiral(daemon_runtime) {
            error!(
                "Failure spiral detected - daemon failed within cooldown period after remediation"
            );
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

        // For stalls, skip remediation and just restart the daemon. A stall
        // means a worker is taking too long, not that anything is broken.
        // Remediation would be overkill - the right response is to reset the
        // worker and restart.
        if matches!(failure_status, HealthStatus::Stalled { .. }) {
            info!("Stall detected - skipping remediation, restarting daemon directly");
            println!("\x1b[1;33m⚠ Worker stalled - resetting and restarting daemon...\x1b[0m");
            spiral_tracker.record_stall_handled();
        } else {
            println!("\x1b[1;31m⚠ Entering remediation mode...\x1b[0m");
            run_remediation(&failure_status, config, &shutdown_count)?;

            if shutdown_count.load(Ordering::SeqCst) > 0 {
                info!("Shutdown requested during remediation");
                break;
            }

            if check_manual_intervention_needed()? {
                error!("Manual intervention file created during remediation");
                bail!("Manual intervention required - see .llmc/manual_intervention_needed_*.txt");
            }

            // Record that remediation was attempted - now future quick failures
            // can be detected as failure spirals
            spiral_tracker.record_remediation_attempt();
            println!("\x1b[1;32m✓ Remediation complete. Restarting daemon...\x1b[0m");
        }

        // If daemon ran past cooldown, reset spiral tracking (had healthy period)
        spiral_tracker.record_daemon_stopped(daemon_runtime);

        // Reset log tailer positions to skip errors from the previous daemon's
        // termination and from remediation itself. Without this, the health
        // monitor would detect old WARN/ERROR logs as new failures.
        monitor.reset_log_positions();
    }

    println!("✓ Overseer shutdown complete");
    Ok(())
}

impl FailureSpiralTracker {
    pub fn new(cooldown: Duration) -> Self {
        Self { cooldown, remediation_attempted: false }
    }

    /// Records that remediation was attempted.
    pub fn record_remediation_attempt(&mut self) {
        self.remediation_attempted = true;
    }

    /// Records that a stall was handled (skipped remediation).
    ///
    /// Stalls skip remediation by design, so this does NOT count as a
    /// remediation attempt for spiral detection purposes.
    pub fn record_stall_handled(&mut self) {
        // Intentionally does not set remediation_attempted
    }

    /// Records that the daemon stopped after running for the given duration.
    ///
    /// If the daemon ran longer than the cooldown period, the tracker resets
    /// because the system had a healthy period - subsequent quick failures
    /// should attempt remediation again before being considered a spiral.
    pub fn record_daemon_stopped(&mut self, daemon_runtime: Duration) {
        if daemon_runtime >= self.cooldown {
            debug!(
                runtime_secs = daemon_runtime.as_secs(),
                cooldown_secs = self.cooldown.as_secs(),
                "Daemon ran past cooldown - resetting spiral tracking"
            );
            self.remediation_attempted = false;
        }
    }

    /// Checks if this failure should be detected as a failure spiral.
    ///
    /// Returns true only if:
    /// 1. At least one remediation has been attempted
    /// 2. The daemon failed within the cooldown period
    pub fn should_detect_spiral(&self, daemon_runtime: Duration) -> bool {
        if !self.remediation_attempted {
            debug!("No remediation attempted yet - cannot be a failure spiral");
            return false;
        }

        if daemon_runtime < self.cooldown {
            error!(
                elapsed_secs = daemon_runtime.as_secs(),
                cooldown_secs = self.cooldown.as_secs(),
                "Daemon failed within restart cooldown after remediation - failure spiral detected"
            );
            true
        } else {
            debug!(
                elapsed_secs = daemon_runtime.as_secs(),
                cooldown_secs = self.cooldown.as_secs(),
                "Daemon ran longer than cooldown, not a failure spiral"
            );
            false
        }
    }
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
fn start_daemon_and_wait_for_registration(
    shutdown_count: &Arc<AtomicU32>,
    daemon_options: &OverseerDaemonOptions,
) -> Result<DaemonHandle> {
    info!("Starting daemon");
    println!("Starting daemon...");

    // Clean up any existing LLMC sessions from a previous daemon instance.
    // This is necessary because the overseer may have crashed while sessions
    // were still running.
    daemon_control::cleanup_existing_sessions()?;

    // Clean up stale registration files from any previous daemon instance.
    // This ensures we wait for the new daemon's registration rather than
    // reading a stale file from a crashed daemon.
    daemon_control::cleanup_registration_files();

    // Use --force to clean up any stale sessions from previous runs
    let mut args = vec!["up".to_string(), "--auto".to_string(), "--force".to_string()];
    if let Some(n) = daemon_options.concurrency {
        args.push("--concurrency".to_string());
        args.push(n.to_string());
    }
    if let Some(ref cmd) = daemon_options.post_accept_command {
        args.push("--post-accept-command".to_string());
        args.push(cmd.clone());
    }

    debug!(args = ?args, "Starting daemon with arguments");

    let mut child = Command::new("llmc")
        .args(&args)
        .env("FORCE_COLOR", "1")
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
        if shutdown_count.load(Ordering::SeqCst) > 0 {
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
         Check logs at {}/logs/ for details.",
        DAEMON_STARTUP_TIMEOUT_SECS,
        config::get_llmc_root().display()
    );
}

/// Runs the health monitoring loop until a failure is detected or shutdown.
///
/// The loop checks for shutdown every 100ms to ensure responsive Ctrl+C
/// handling, while only performing expensive health checks every
/// HEALTH_CHECK_INTERVAL_SECS.
fn run_monitor_loop(
    monitor: &mut HealthMonitor,
    expected: &ExpectedDaemon,
    shutdown_count: &Arc<AtomicU32>,
) -> Option<HealthStatus> {
    info!(pid = expected.pid, "Entering health monitoring loop");
    println!("Monitoring daemon health (Ctrl-C to stop)...");

    let health_check_interval = Duration::from_secs(HEALTH_CHECK_INTERVAL_SECS);
    let shutdown_poll_interval = Duration::from_millis(100);
    let mut last_health_check = Instant::now();

    loop {
        if shutdown_count.load(Ordering::SeqCst) > 0 {
            return None;
        }

        thread::sleep(shutdown_poll_interval);

        if last_health_check.elapsed() >= health_check_interval {
            last_health_check = Instant::now();
            let status = monitor.check_daemon_health(expected);
            if !status.is_healthy() {
                return Some(status);
            }
            debug!("Health check passed");
        }
    }
}

/// Terminates the daemon gracefully with full termination protocol.
///
/// If `child` is provided, it will be used to reap the zombie process after
/// SIGKILL. This is necessary when the overseer is the parent of the daemon,
/// because `kill(pid, 0)` returns 0 for zombie processes until they are reaped.
fn terminate_daemon_gracefully(expected: &ExpectedDaemon, child: Option<&mut Child>) {
    info!(pid = expected.pid, "Initiating daemon termination");
    println!("Terminating daemon...");

    match daemon_control::terminate_daemon(expected, child) {
        Ok(TerminationResult::GracefulShutdown) => {
            info!("Daemon terminated gracefully");
            println!("✓ Daemon terminated gracefully");
        }
        Ok(TerminationResult::ForcefulKill) => {
            info!("Daemon required forceful kill");
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
                    error!(path = %path.display(), error = %e, "Failed to read intervention file");
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
    shutdown_count: &Arc<AtomicU32>,
) -> Result<()> {
    info!(failure = ?failure, "Starting remediation");

    let prompt = remediation_prompt::build_remediation_prompt(failure, config);
    debug!(prompt_length = prompt.len(), "Built remediation prompt");

    remediation_executor::execute_remediation(&prompt, config, shutdown_count)?;

    info!("Remediation completed");
    Ok(())
}
