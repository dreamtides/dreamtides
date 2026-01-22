use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use lattice::cli::color_theme;
use tokio::sync::mpsc::Receiver;

use crate::auto_mode::auto_config::ResolvedAutoConfig;
use crate::auto_mode::auto_orchestrator;
use crate::commands::add;
use crate::config::{self, Config};
use crate::ipc::messages::HookMessage;
use crate::ipc::socket;
use crate::lock::StateLock;
use crate::overseer_mode::overseer_session;
use crate::patrol::Patrol;
use crate::state::{self, State, WorkerStatus};
use crate::tmux::session;
use crate::worker::WorkerTransition;
use crate::{git, patrol, worker};

/// Options for the up command
pub struct UpOptions {
    pub no_patrol: bool,
    pub verbose: bool,
    pub force: bool,
    pub auto: bool,
    pub task_pool_command: Option<String>,
    pub concurrency: Option<u32>,
    pub post_accept_command: Option<String>,
}

/// Runs the up command, starting the LLMC daemon
pub fn run_up(options: UpOptions) -> Result<()> {
    let UpOptions {
        no_patrol,
        verbose,
        force,
        auto,
        task_pool_command,
        concurrency,
        post_accept_command,
    } = options;
    let llmc_root = config::get_llmc_root();
    if !llmc_root.exists() {
        bail!(
            "LLMC workspace not initialized. Run 'llmc init' first.\n\
             Expected workspace at: {}",
            llmc_root.display()
        );
    }
    let config_path = config::get_config_path();
    let config = Config::load(&config_path)?;

    // Validate auto config early if auto mode is requested
    let auto_config = if auto {
        let resolved = ResolvedAutoConfig::resolve(
            config.auto.as_ref(),
            task_pool_command.as_deref(),
            concurrency,
            post_accept_command.as_deref(),
        );
        let Some(cfg) = resolved else {
            bail!(
                "Auto mode requires a task pool command.\n\
                 Either:\n\
                 - Add [auto] section with task_pool_command in config.toml, or\n\
                 - Use --task-pool-command <CMD> flag"
            );
        };
        tracing::info!(
            "Auto mode enabled with task_pool_command={:?}, concurrency={}, post_accept_command={:?}",
            cfg.task_pool_command,
            cfg.concurrency,
            cfg.post_accept_command
        );
        Some(cfg)
    } else {
        None
    };

    // Normal startup sequence (shared by both modes)
    let state_path = state::get_state_path();
    let mut state = State::load(&state_path)?;
    if state.daemon_running {
        println!(
            "{}",
            color_theme::warning(
                "⚠ Previous daemon crash detected. Running enhanced recovery checks..."
            )
        );
        tracing::info!(
            "Daemon crash detected: daemon_running flag was true on startup (recovery in progress)"
        );
    }
    cleanup_orphaned_sessions(&state, force, verbose)?;
    println!("{}", color_theme::dim("Starting LLMC daemon..."));
    state.daemon_running = true;
    state.save(&state_path)?;
    ensure_tmux_running()?;

    // Start IPC listener BEFORE workers so SessionStart hooks can connect
    // immediately when Claude starts. Without this, there's a race condition where
    // hooks fire before the socket exists and are silently dropped.
    let socket_path = socket::get_socket_path();
    let ipc_receiver = match socket::spawn_ipc_listener(socket_path.clone()) {
        Ok(rx) => {
            println!(
                "{}",
                color_theme::success(format!(
                    "✓ IPC listener started at {}",
                    socket_path.display()
                ))
            );
            Some(rx)
        }
        Err(e) => {
            tracing::error!(
                error = %e,
                socket_path = %socket_path.display(),
                "Failed to start IPC listener - hooks will not work. Task completion \
                 detection will rely on fallback polling (5 minute delay). Check if another \
                 llmc instance is running or socket is stale."
            );
            eprintln!(
                "Warning: Failed to start IPC listener: {}\nDaemon will continue without hook support.",
                e
            );
            None
        }
    };

    reconcile_and_start_workers(&config, &mut state, verbose)?;
    state.save(&state_path)?;
    println!("{}", color_theme::success("✓ All workers started"));

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = Arc::clone(&shutdown);
    ctrlc::set_handler(move || {
        println!("\n{}", color_theme::dim("Received Ctrl-C, shutting down gracefully..."));
        shutdown_clone.store(true, Ordering::SeqCst);
    })
    .context("Failed to set Ctrl-C handler")?;

    // Branch to either auto mode or normal mode
    if let Some(ref auto_cfg) = auto_config {
        println!("{}\n", color_theme::dim("Entering auto mode loop (Ctrl-C to stop)..."));
        auto_orchestrator::run_auto_mode(&config, auto_cfg, shutdown, ipc_receiver)?;
        println!("{}", color_theme::success("✓ LLMC auto mode daemon stopped"));
    } else {
        println!("{}\n", color_theme::dim("Entering main loop (Ctrl-C to stop)..."));
        run_main_loop(
            no_patrol,
            verbose,
            shutdown,
            &config,
            &mut state,
            &state_path,
            ipc_receiver,
        )?;
        println!("{}", color_theme::success("✓ LLMC daemon stopped"));
    }
    Ok(())
}
/// Returns the current Unix timestamp in seconds. Never fails - returns 0 if
/// system time is before UNIX_EPOCH (should never happen in practice).
fn unix_timestamp_now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}
/// Cleans up any orphaned LLMC TMUX sessions that don't correspond to workers
/// in the state file
fn cleanup_orphaned_sessions(state: &State, force: bool, verbose: bool) -> Result<()> {
    let all_sessions = session::list_sessions()?;
    let llmc_sessions: Vec<String> = all_sessions
        .into_iter()
        .filter(|s| s.starts_with(&config::get_session_prefix_pattern()))
        .collect();
    if llmc_sessions.is_empty() {
        return Ok(());
    }
    let tracked_session_ids: Vec<String> =
        state.workers.values().map(|w| w.session_id.clone()).collect();
    let orphaned_sessions: Vec<String> = llmc_sessions
        .into_iter()
        .filter(|s| {
            if tracked_session_ids.contains(s) {
                return false;
            }
            if overseer_session::is_overseer_session(s) {
                tracing::info!(session = %s, "Preserving overseer session during cleanup");
                return false;
            }
            true
        })
        .collect();
    let running_tracked_sessions: Vec<&String> =
        tracked_session_ids.iter().filter(|s| session::session_exists(s)).collect();
    if orphaned_sessions.is_empty() {
        if !running_tracked_sessions.is_empty() {
            if force {
                println!(
                    "{}",
                    color_theme::dim(format!(
                        "Found {} tracked LLMC sessions. Cleaning up due to --force...",
                        running_tracked_sessions.len()
                    ))
                );
                for session_id in &running_tracked_sessions {
                    if verbose {
                        println!(
                            "  {}",
                            color_theme::muted(format!("Killing session: {}", session_id))
                        );
                    }
                    session::kill_session(session_id)?;
                }
            } else {
                bail!(
                    "LLMC workers are already running. Found {} tracked TMUX sessions.\n\
                     Run 'llmc down' first if you want to restart the daemon, or use 'llmc up --force' to force cleanup.\n\
                     Use 'tmux list-sessions' to see all active sessions.",
                    running_tracked_sessions.len()
                );
            }
        }
        return Ok(());
    }
    println!(
        "{}",
        color_theme::dim(format!(
            "Found {} orphaned LLMC sessions (not tracked in state file):",
            orphaned_sessions.len()
        ))
    );
    for session_name in &orphaned_sessions {
        println!("  {}", color_theme::muted(format!("- {}", session_name)));
    }
    println!("{}", color_theme::dim("Cleaning up orphaned sessions..."));
    for session_name in &orphaned_sessions {
        if verbose {
            println!(
                "  {}",
                color_theme::muted(format!("Killing orphaned session: {}", session_name))
            );
        }
        if let Err(e) = session::kill_session(session_name) {
            eprintln!(
                "{}",
                color_theme::warning(format!(
                    "Warning: Failed to kill orphaned session '{}': {}",
                    session_name, e
                ))
            );
        }
    }
    println!(
        "{}",
        color_theme::success(format!("✓ Cleaned up {} orphaned sessions", orphaned_sessions.len()))
    );
    Ok(())
}
fn ensure_tmux_running() -> Result<()> {
    if !is_tmux_server_running() {
        println!("{}", color_theme::dim("Starting TMUX server..."));
        start_tmux_server()?;
    }
    Ok(())
}
fn is_tmux_server_running() -> bool {
    std::process::Command::new("tmux")
        .arg("list-sessions")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
fn start_tmux_server() -> Result<()> {
    let output = std::process::Command::new("tmux")
        .arg("start-server")
        .output()
        .context("Failed to execute tmux start-server")?;
    if !output.status.success() {
        bail!("Failed to start TMUX server: {}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}
fn reconcile_and_start_workers(config: &Config, state: &mut State, verbose: bool) -> Result<()> {
    println!("{}", color_theme::dim("Reconciling workers with state..."));
    let worker_names: Vec<String> = state.workers.keys().cloned().collect();
    for worker_name in &worker_names {
        if let Some(worker_record) = state.workers.get_mut(worker_name)
            && !session::session_exists(&worker_record.session_id)
        {
            // Preserve Rebasing state - it indicates important context about a rebase in
            // progress The startup code will detect this and handle it
            // appropriately
            if worker_record.status == WorkerStatus::Rebasing {
                println!(
                    "  {}",
                    color_theme::muted(format!(
                        "Worker '{}' session not found but has Rebasing state, preserving state",
                        worker_name
                    ))
                );
            } else {
                println!(
                    "  {}",
                    color_theme::muted(format!(
                        "Worker '{}' session not found, marking offline",
                        worker_name
                    ))
                );
                worker_record.status = WorkerStatus::Offline;
            }
        }
    }
    let mut failed_workers: Vec<String> = Vec::new();
    for worker_name in &worker_names {
        // Start workers that are Offline OR Rebasing (Rebasing workers need sessions
        // too)
        if let Some(worker_record) = state.workers.get(worker_name)
            && (worker_record.status == WorkerStatus::Offline
                || (worker_record.status == WorkerStatus::Rebasing
                    && !session::session_exists(&worker_record.session_id)))
        {
            println!("  {}", color_theme::dim(format!("Starting worker '{}'...", worker_name)));
            if let Err(e) = start_worker_with_recovery(worker_name, config, state, verbose) {
                tracing::error!(
                    "Failed to start worker '{}' after recovery attempts: {}",
                    worker_name,
                    e
                );
                eprintln!(
                    "  {}",
                    color_theme::warning(format!(
                        "⚠ Failed to start worker '{}': {}",
                        worker_name, e
                    ))
                );
                failed_workers.push(worker_name.clone());
                if let Some(worker_mut) = state.get_worker_mut(worker_name) {
                    worker_mut.status = WorkerStatus::Error;
                    worker_mut.last_activity_unix = unix_timestamp_now();
                    worker_mut.crash_count = worker_mut.crash_count.saturating_add(1);
                    worker_mut.last_crash_unix = Some(unix_timestamp_now());
                }
            }
        }
    }
    if !failed_workers.is_empty() {
        eprintln!(
            "  {}",
            color_theme::warning(format!(
                "⚠ {} worker(s) failed to start: {}",
                failed_workers.len(),
                failed_workers.join(", ")
            ))
        );
        eprintln!(
            "    {}",
            color_theme::muted("The daemon will continue and retry these workers periodically.")
        );
    }
    Ok(())
}
/// Attempts to start a worker with self-healing recovery.
/// If the initial start fails, kills any stale session and retries.
/// On success, resets the crash count to provide positive feedback.
fn start_worker_with_recovery(
    name: &str,
    config: &Config,
    state: &mut State,
    verbose: bool,
) -> Result<()> {
    const MAX_RETRIES: u32 = 2;
    match start_worker(name, config, state, verbose) {
        Ok(()) => {
            if let Some(worker_mut) = state.get_worker_mut(name)
                && worker_mut.crash_count > 0
            {
                tracing::info!(
                    "Worker '{}' started successfully, resetting crash count from {}",
                    name,
                    worker_mut.crash_count
                );
                worker_mut.crash_count = 0;
                worker_mut.last_crash_unix = None;
            }
            return Ok(());
        }
        Err(e) => {
            tracing::info!("Worker '{}' initial start failed: {}. Attempting recovery...", name, e);
            println!(
                "    {}",
                color_theme::muted("Initial start failed, attempting self-healing recovery...")
            );
        }
    }
    for attempt in 1..=MAX_RETRIES {
        let session_id = state
            .get_worker(name)
            .map(|w| w.session_id.clone())
            .unwrap_or_else(|| config::get_worker_session_name(name));
        if session::session_exists(&session_id) {
            println!(
                "    {}",
                color_theme::muted(format!("Killing stale session '{}'...", session_id))
            );
            if let Err(e) = session::kill_session(&session_id) {
                tracing::info!("Failed to kill stale session '{}': {}", session_id, e);
            }
            thread::sleep(Duration::from_millis(500));
        }
        let delay = Duration::from_secs(attempt as u64);
        println!(
            "    {}",
            color_theme::muted(format!(
                "Retry {}/{} after {}s delay...",
                attempt, MAX_RETRIES, attempt
            ))
        );
        thread::sleep(delay);
        match start_worker(name, config, state, verbose) {
            Ok(()) => {
                println!(
                    "    {}",
                    color_theme::success(format!("✓ Recovery successful on retry {}", attempt))
                );
                tracing::info!("Worker '{}' recovered successfully on retry {}", name, attempt);
                if let Some(worker_mut) = state.get_worker_mut(name) {
                    worker_mut.crash_count = 0;
                    worker_mut.last_crash_unix = None;
                }
                return Ok(());
            }
            Err(e) => {
                tracing::info!("Worker '{}' recovery attempt {} failed: {}", name, attempt, e);
                if attempt == MAX_RETRIES {
                    return Err(e);
                }
            }
        }
    }
    unreachable!()
}
fn start_worker(name: &str, config: &Config, state: &mut State, verbose: bool) -> Result<()> {
    let worker_record =
        state.get_worker(name).with_context(|| format!("Worker '{}' not found in state", name))?;
    let worktree_path = PathBuf::from(&worker_record.worktree_path);
    if verbose {
        println!(
            "    {}",
            color_theme::muted(format!("[verbose] Checking worktree: {}", worktree_path.display()))
        );
    }
    if !worktree_path.exists() {
        println!(
            "  {}",
            color_theme::dim(format!("Worker '{}' worktree missing, recreating...", name))
        );
        match add::recreate_missing_worktree(name, &worker_record.branch, &worktree_path) {
            Ok(()) => {
                println!("  {}", color_theme::success("✓ Worktree recreated successfully"));
            }
            Err(e) => {
                bail!(
                    "Failed to recreate worktree for worker '{}': {}\n\
                     Run 'llmc nuke {}' and 'llmc add {}' to manually recreate",
                    name,
                    e,
                    name,
                    name
                );
            }
        }
    }
    ensure_hook_config_exists(&worktree_path, name, verbose)?;
    let Some(worker_config) = config.get_worker(name) else {
        tracing::error!(
            "Worker '{}' exists in state but not in config.toml. This indicates a configuration issue.",
            name
        );
        bail!(
            "Worker '{}' not found in config.toml\n\
             The worker exists in state.json but has no corresponding [workers.{}] section in config.toml.\n\n\
             To fix this:\n\
             1. Run 'llmc doctor --repair' to diagnose and fix the issue, or\n\
             2. Manually add a [workers.{}] section to {}, or\n\
             3. Run 'llmc nuke {}' and 'llmc add {}' to recreate the worker",
            name,
            name,
            name,
            config::get_config_path().display(),
            name,
            name
        );
    };
    if verbose {
        println!(
            "    {}",
            color_theme::muted(format!("[verbose] Session ID: {}", worker_record.session_id))
        );
        println!(
            "    {}",
            color_theme::muted(format!(
                "[verbose] Session exists: {}",
                session::session_exists(&worker_record.session_id)
            ))
        );
    }
    // Always kill and recreate the session to ensure a clean state. If we try to
    // reuse an existing session where Claude is already running, the claude command
    // would be interpreted as user input rather than starting a new Claude
    // instance.
    if session::session_exists(&worker_record.session_id) {
        if verbose {
            println!(
                "    {}",
                color_theme::muted(format!(
                    "[verbose] Killing existing session '{}' to ensure clean state",
                    worker_record.session_id
                ))
            );
        }
        tracing::debug!(
            worker = %name,
            session_id = %worker_record.session_id,
            "Killing existing session to ensure clean state on startup"
        );
        if let Err(e) = session::kill_session(&worker_record.session_id) {
            tracing::info!(
                worker = %name,
                error = %e,
                "Failed to kill existing session, attempting to create new one anyway \
                 (session may have already exited)"
            );
        }
    }
    if verbose {
        println!(
            "    {}",
            color_theme::muted(format!(
                "[verbose] Creating new TMUX session for worker '{}'",
                name
            ))
        );
    }
    tracing::info!(
        worker = %name,
        session_id = %worker_record.session_id,
        worktree = %worktree_path.display(),
        "Creating new TMUX session for worker"
    );
    session::start_worker_session(
        &worker_record.session_id,
        &worktree_path,
        worker_config,
        verbose,
    )?;
    tracing::info!(
        worker = %name,
        session_id = %worker_record.session_id,
        "TMUX session created successfully, waiting for SessionStart hook"
    );
    let is_clean = git::is_worktree_clean(&worktree_path).unwrap_or(false);
    let worker_mut = state.get_worker_mut(name).unwrap();
    if is_clean {
        if let Err(e) = worker::apply_transition(worker_mut, WorkerTransition::ToOffline) {
            tracing::info!(
                worker = %name,
                error = %e,
                "Failed to apply state transition to Offline, setting status directly \
                 (this is a workaround for invalid state machine transitions)"
            );
            worker_mut.status = WorkerStatus::Offline;
        }
        if verbose {
            println!(
                "    {}",
                color_theme::muted(format!(
                    "[verbose] Worker '{}' worktree is clean; waiting for SessionStart hook",
                    name
                ))
            );
        }
    } else if git::is_rebase_in_progress(&worktree_path) {
        // Worker has an active rebase - transition to Rebasing state instead of Error
        // This preserves the rebase state across daemon restarts
        tracing::info!(
            worker = %name,
            "Worker has active rebase in progress at startup, transitioning to Rebasing"
        );
        if let Err(e) = worker::apply_transition(worker_mut, WorkerTransition::ToRebasing) {
            tracing::info!(
                worker = %name,
                error = %e,
                "Failed to apply state transition to Rebasing, setting status directly \
                 (this is a workaround for invalid state machine transitions at startup)"
            );
            worker_mut.status = WorkerStatus::Rebasing;
            worker_mut.last_activity_unix = unix_timestamp_now();
        }
        // Queue a conflict prompt to be sent by the patrol
        worker_mut.pending_rebase_prompt = true;
        println!(
            "  {}",
            color_theme::warning(format!("⚠ Worker '{}' has an active rebase in progress", name))
        );
        println!(
            "    {}",
            color_theme::muted("Claude will receive a rebase conflict prompt once ready.")
        );
        if verbose {
            println!(
                "    {}",
                color_theme::muted(format!(
                    "[verbose] Worker '{}' has active rebase, marked as Rebasing",
                    name
                ))
            );
        }
    } else {
        if let Err(e) = worker::apply_transition(worker_mut, WorkerTransition::ToError {
            reason: "Uncommitted changes at startup".to_string(),
        }) {
            tracing::info!(
                worker = %name,
                error = %e,
                "Failed to apply state transition to Error, setting status directly \
                 (this is a workaround for invalid state machine transitions at startup)"
            );
            worker_mut.status = WorkerStatus::Error;
            worker_mut.last_activity_unix = unix_timestamp_now();
        }
        println!(
            "  {}",
            color_theme::warning(format!("⚠ Worker '{}' has uncommitted changes", name))
        );
        println!(
            "    {}",
            color_theme::muted(format!(
                "Marked as Error. Run 'llmc doctor --fix' or 'llmc reset {}' to recover",
                name
            ))
        );
        if verbose {
            println!(
                "    {}",
                color_theme::muted(format!(
                    "[verbose] Worker '{}' worktree is dirty, marked as Error",
                    name
                ))
            );
        }
    }
    Ok(())
}

fn ensure_hook_config_exists(worktree_path: &Path, worker_name: &str, verbose: bool) -> Result<()> {
    let settings_path = worktree_path.join(".claude").join("settings.json");
    if !settings_path.exists() {
        tracing::debug!(
            worker = %worker_name,
            path = %settings_path.display(),
            "Claude hook settings not found, regenerating (this is normal after Claude Code exits)"
        );
        add::create_claude_hook_settings_silent(worktree_path, worker_name)?;
        if verbose {
            println!(
                "    {}",
                color_theme::muted(format!(
                    "[verbose] Regenerated .claude/settings.json for worker '{}'",
                    worker_name
                ))
            );
        }
    } else if verbose {
        println!(
            "    {}",
            color_theme::muted(format!(
                "[verbose] Worker '{}' has Claude hook config at {}",
                worker_name,
                settings_path.display()
            ))
        );
    }
    Ok(())
}

/// Main daemon loop. Implements a NEVER CRASH philosophy - all errors are
/// logged and the daemon continues running. Only Ctrl-C will stop the daemon.
fn run_main_loop(
    no_patrol: bool,
    verbose: bool,
    shutdown: Arc<AtomicBool>,
    config: &Config,
    state: &mut State,
    state_path: &Path,
    mut ipc_receiver: Option<Receiver<HookMessage>>,
) -> Result<()> {
    let mut config = config.clone();
    let patrol = Patrol::new(&config);
    let patrol_interval = Duration::from_secs(config.defaults.patrol_interval_secs as u64);
    let mut last_patrol = SystemTime::now();
    let mut consecutive_errors: u32 = 0;
    const ERROR_WARNING_THRESHOLD: u32 = 10;
    let mut last_error_warning = SystemTime::UNIX_EPOCH;
    let error_warning_interval = Duration::from_secs(300);
    while !shutdown.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_secs(1));
        let mut pending_hook_events: Vec<HookMessage> = Vec::new();
        if let Some(ref mut rx) = ipc_receiver {
            while let Ok(msg) = rx.try_recv() {
                tracing::info!("Received hook event: {:?} (id: {})", msg.event, msg.id);
                if verbose {
                    println!(
                        "  {}",
                        color_theme::muted(format!("[hook] Received event: {:?}", msg.event))
                    );
                }
                pending_hook_events.push(msg);
            }
        }
        let mut iteration_had_error = false;
        let mut lock_acquired = false;
        match StateLock::acquire() {
            Ok(_lock) => {
                lock_acquired = true;
                match State::load(state_path) {
                    Ok(new_state) => *state = new_state,
                    Err(e) => {
                        tracing::error!("Failed to reload state (continuing with existing): {}", e);
                        iteration_had_error = true;
                    }
                }
                for msg in &pending_hook_events {
                    if let Err(e) = patrol::handle_hook_event(&msg.event, state, &config) {
                        tracing::error!("Error handling hook event {:?}: {}", msg.event, e);
                        iteration_had_error = true;
                    }
                }
                if !pending_hook_events.is_empty()
                    && let Err(e) = state.save(state_path)
                {
                    tracing::error!(
                        "Failed to save state after hook events (daemon continuing): {}",
                        e
                    );
                    iteration_had_error = true;
                }
                poll_worker_states(state);
                if let Err(e) = start_offline_workers(&mut config, state, verbose) {
                    tracing::error!("Error in start_offline_workers (daemon continuing): {}", e);
                    iteration_had_error = true;
                }
                if let Err(e) = state.save(state_path) {
                    tracing::error!("Failed to save state (daemon continuing): {}", e);
                    iteration_had_error = true;
                }
            }
            Err(e) => {
                tracing::debug!("Skipping main loop iteration - failed to acquire lock: {}", e);
            }
        }
        if iteration_had_error {
            consecutive_errors = consecutive_errors.saturating_add(1);
        } else {
            consecutive_errors = 0;
        }
        if consecutive_errors >= ERROR_WARNING_THRESHOLD
            && SystemTime::now().duration_since(last_error_warning).unwrap_or_default()
                >= error_warning_interval
        {
            eprintln!(
                "{}",
                color_theme::warning(format!(
                    "⚠ Warning: {} consecutive errors in daemon main loop. Check logs at {}/logs/ for details.",
                    consecutive_errors,
                    config::get_llmc_root().display()
                ))
            );
            tracing::error!(
                "Daemon has had {} consecutive errors - check logs for details",
                consecutive_errors
            );
            last_error_warning = SystemTime::now();
        }
        if !no_patrol
            && lock_acquired
            && SystemTime::now().duration_since(last_patrol).unwrap_or_default() >= patrol_interval
        {
            if verbose {
                println!("{}", color_theme::muted("Running patrol..."));
            }
            match patrol.run_patrol(state, &config) {
                Ok(report) => {
                    if !report.transitions_applied.is_empty() {
                        tracing::info!(
                            "Patrol applied {} transitions: {:?}",
                            report.transitions_applied.len(),
                            report.transitions_applied
                        );
                    }
                    if !report.rebases_triggered.is_empty() {
                        tracing::info!("Patrol triggered rebases: {:?}", report.rebases_triggered);
                    }
                    if !report.errors.is_empty() {
                        tracing::error!("Patrol encountered errors: {:?}", report.errors);
                    }
                }
                Err(e) => {
                    tracing::error!("Patrol failed (daemon continuing): {}", e);
                }
            }
            last_patrol = SystemTime::now();
        }
    }
    if let Err(e) = graceful_shutdown(&config, state) {
        tracing::error!("Error during graceful shutdown: {}", e);
        eprintln!(
            "{}",
            color_theme::warning(format!("Warning: Error during graceful shutdown: {}", e))
        );
    }
    if let Err(e) = state.save(state_path) {
        tracing::error!("Failed to save final state: {}", e);
        eprintln!(
            "{}",
            color_theme::warning(format!("Warning: Failed to save final state: {}", e))
        );
    }
    Ok(())
}
/// Polls worker states to detect disappeared sessions and retry failed workers.
/// This function is infallible - it will never crash the daemon.
///
/// Self-healing behavior:
/// - Detects workers whose sessions have disappeared and marks them offline
/// - Retries Error workers after a cooldown period (exponential backoff based
///   on crash count)
fn poll_worker_states(state: &mut State) {
    let worker_names: Vec<String> = state.workers.keys().cloned().collect();
    let now = unix_timestamp_now();
    for worker_name in &worker_names {
        if let Some(worker_record) = state.workers.get_mut(worker_name) {
            if worker_record.status == WorkerStatus::Offline {
                continue;
            }
            if worker_record.status != WorkerStatus::Error
                && !session::session_exists(&worker_record.session_id)
            {
                println!(
                    "  {}",
                    color_theme::muted(format!(
                        "Worker '{}' session disappeared, marking offline",
                        worker_record.name
                    ))
                );
                worker_record.status = WorkerStatus::Offline;
                worker_record.last_activity_unix = now;
                continue;
            }
            if worker_record.status == WorkerStatus::Error {
                let base_cooldown_secs = 60u64;
                let backoff_factor = 2u64.pow(worker_record.crash_count.min(5));
                let cooldown_secs = (base_cooldown_secs * backoff_factor).min(30 * 60);
                let time_since_error = now.saturating_sub(worker_record.last_activity_unix);
                if time_since_error >= cooldown_secs {
                    println!(
                        "  {}",
                        color_theme::muted(format!(
                            "Worker '{}' cooldown expired ({}s), marking offline for retry",
                            worker_record.name, cooldown_secs
                        ))
                    );
                    tracing::info!(
                        "Worker '{}' transitioning from Error to Offline for self-healing retry (crash_count={})",
                        worker_record.name,
                        worker_record.crash_count
                    );
                    worker_record.status = WorkerStatus::Offline;
                    worker_record.last_activity_unix = now;
                }
            }
        }
    }
}
/// Attempts to start any offline workers with self-healing recovery.
/// Individual worker failures are logged but don't stop other workers from
/// being started. The daemon continues running and will retry failed workers
/// on subsequent iterations.
fn start_offline_workers(config: &mut Config, state: &mut State, verbose: bool) -> Result<()> {
    let worker_names: Vec<String> = state.workers.keys().cloned().collect();
    let has_offline_workers = worker_names.iter().any(|name| {
        state.workers.get(name).map(|w| w.status == WorkerStatus::Offline).unwrap_or(false)
    });
    if has_offline_workers {
        let config_path = config::get_config_path();
        if verbose {
            println!(
                "  {}",
                color_theme::muted("[verbose] Reloading config before starting offline workers")
            );
        }
        match Config::load(&config_path) {
            Ok(new_config) => *config = new_config,
            Err(e) => {
                tracing::info!(
                    error = %e,
                    config_path = %config_path.display(),
                    "Failed to reload config.toml, using existing config (this is \
                     normal if config file is temporarily locked or has syntax errors)"
                );
                if verbose {
                    println!(
                        "  {}",
                        color_theme::muted("[verbose] Config reload failed, using existing config")
                    );
                }
            }
        }
    }
    let mut failed_workers: Vec<String> = Vec::new();
    for worker_name in &worker_names {
        if let Some(worker_record) = state.workers.get(worker_name)
            && worker_record.status == WorkerStatus::Offline
        {
            let session_id = worker_record.session_id.clone();
            if session::session_exists(&session_id) {
                if verbose {
                    println!(
                        "  {}",
                        color_theme::muted(format!(
                            "[verbose] Worker '{}' session already exists, marking as idle",
                            worker_name
                        ))
                    );
                }
                if let Some(worker_mut) = state.get_worker_mut(worker_name)
                    && let Err(e) = worker::apply_transition(worker_mut, WorkerTransition::ToIdle)
                {
                    tracing::info!(
                        worker = %worker_name,
                        error = %e,
                        "Failed to apply state transition to Idle, setting status directly \
                         (session exists but state machine transition invalid)"
                    );
                    worker_mut.status = WorkerStatus::Idle;
                    worker_mut.current_prompt.clear();
                    worker_mut.prompt_cmd = None;
                    worker_mut.commit_sha = None;
                    worker_mut.self_review = false;
                    worker_mut.pending_self_review = false;
                    worker_mut.last_activity_unix = unix_timestamp_now();
                }
                continue;
            }
            println!(
                "  {}",
                color_theme::dim(format!("Starting offline worker '{}'...", worker_name))
            );
            if let Err(e) = start_worker_with_recovery(worker_name, config, state, verbose) {
                tracing::error!("Failed to start worker '{}' after recovery: {}", worker_name, e);
                eprintln!(
                    "  {}",
                    color_theme::warning(format!(
                        "⚠ Failed to start worker '{}': {}",
                        worker_name, e
                    ))
                );
                failed_workers.push(worker_name.clone());
                if let Some(worker_mut) = state.get_worker_mut(worker_name) {
                    worker_mut.status = WorkerStatus::Error;
                    worker_mut.last_activity_unix = unix_timestamp_now();
                    worker_mut.crash_count = worker_mut.crash_count.saturating_add(1);
                    worker_mut.last_crash_unix = Some(unix_timestamp_now());
                }
            }
        }
    }
    if !failed_workers.is_empty() {
        anyhow::bail!(
            "Failed to start {} worker(s): {}",
            failed_workers.len(),
            failed_workers.join(", ")
        );
    }
    Ok(())
}
fn graceful_shutdown(config: &Config, state: &mut State) -> Result<()> {
    println!("{}", color_theme::dim("Shutting down workers..."));
    state.daemon_running = false;
    let sender = super::super::tmux::sender::TmuxSender::new();
    let worker_names: Vec<String> = state.workers.keys().cloned().collect();
    for worker_name in &worker_names {
        if let Some(worker_record) = state.workers.get(worker_name) {
            if worker_record.status == WorkerStatus::Offline {
                continue;
            }
            println!("  {}", color_theme::muted(format!("Stopping worker '{}'...", worker_name)));
            if config.get_worker(worker_name).is_none() {
                tracing::error!(
                    "Worker '{}' exists in state but not in config during shutdown",
                    worker_name
                );
                eprintln!(
                    "{}",
                    color_theme::warning(format!(
                        "Warning: Worker '{}' not found in config.toml, skipping graceful shutdown",
                        worker_name
                    ))
                );
                continue;
            }
            if let Err(e) = sender.send_keys_raw(&worker_record.session_id, "C-c") {
                eprintln!(
                    "{}",
                    color_theme::warning(format!(
                        "Warning: Failed to send Ctrl-C to worker '{}': {}",
                        worker_name, e
                    ))
                );
            }
        }
    }
    thread::sleep(Duration::from_millis(500));
    for worker_name in &worker_names {
        if let Some(worker_record) = state.workers.get_mut(worker_name) {
            if session::session_exists(&worker_record.session_id)
                && let Err(e) = session::kill_session(&worker_record.session_id)
            {
                eprintln!(
                    "{}",
                    color_theme::warning(format!(
                        "Warning: Failed to kill session '{}': {}",
                        worker_record.session_id, e
                    ))
                );
            }
            // Preserve certain states that represent important context that shouldn't be
            // lost Rebasing state indicates a rebase is in progress with
            // potential conflicts The startup code will detect this and handle
            // it appropriately
            if worker_record.status != WorkerStatus::Rebasing {
                worker_record.status = WorkerStatus::Offline;
            }
        }
    }
    Ok(())
}
