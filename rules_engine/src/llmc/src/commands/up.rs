use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};

use super::super::config::{self, Config};
use super::super::patrol::Patrol;
use super::super::state::{self, State, WorkerStatus};
use super::super::tmux::session;
use super::super::{git, worker};
use super::add;

/// Runs the up command, starting the LLMC daemon
pub fn run_up(no_patrol: bool, verbose: bool, force: bool) -> Result<()> {
    let llmc_root = config::get_llmc_root();
    if !llmc_root.exists() {
        bail!(
            "LLMC workspace not initialized. Run 'llmc init' first.\n\
             Expected workspace at: {}",
            llmc_root.display()
        );
    }

    let state_path = state::get_state_path();
    let mut state = State::load(&state_path)?;

    if state.daemon_running {
        println!("⚠ Previous daemon crash detected. Running enhanced recovery checks...");
        tracing::warn!("Daemon crash detected: daemon_running flag was true on startup");
    }

    cleanup_orphaned_sessions(&state, force, verbose)?;

    println!("Starting LLMC daemon...");

    let config_path = config::get_config_path();
    let config = Config::load(&config_path)?;

    state.daemon_running = true;
    state.save(&state_path)?;

    ensure_tmux_running()?;
    reconcile_and_start_workers(&config, &mut state, verbose)?;
    state.save(&state_path)?;

    println!("✓ All workers started");
    println!("Entering main loop (Ctrl-C to stop)...\n");

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = Arc::clone(&shutdown);

    ctrlc::set_handler(move || {
        println!("\nReceived Ctrl-C, shutting down gracefully...");
        shutdown_clone.store(true, Ordering::SeqCst);
    })
    .context("Failed to set Ctrl-C handler")?;

    run_main_loop(no_patrol, verbose, shutdown, &config, &mut state, &state_path)?;

    println!("✓ LLMC daemon stopped");
    Ok(())
}

/// Cleans up any orphaned LLMC TMUX sessions that don't correspond to workers
/// in the state file
fn cleanup_orphaned_sessions(state: &State, force: bool, verbose: bool) -> Result<()> {
    let all_sessions = session::list_sessions()?;
    let llmc_sessions: Vec<String> =
        all_sessions.into_iter().filter(|s| s.starts_with("llmc-")).collect();

    if llmc_sessions.is_empty() {
        return Ok(());
    }

    let tracked_session_ids: Vec<String> =
        state.workers.values().map(|w| w.session_id.clone()).collect();

    let orphaned_sessions: Vec<String> =
        llmc_sessions.into_iter().filter(|s| !tracked_session_ids.contains(s)).collect();

    if orphaned_sessions.is_empty() {
        if !tracked_session_ids.is_empty() {
            if force {
                println!(
                    "Found {} tracked LLMC sessions. Cleaning up due to --force...",
                    tracked_session_ids.len()
                );
                for session_id in &tracked_session_ids {
                    if verbose {
                        println!("  Killing session: {}", session_id);
                    }
                    session::kill_session(session_id)?;
                }
            } else {
                bail!(
                    "LLMC workers are already running. Found {} tracked TMUX sessions.\n\
                     Run 'llmc down' first if you want to restart the daemon, or use 'llmc up --force' to force cleanup.\n\
                     Use 'tmux list-sessions' to see all active sessions.",
                    tracked_session_ids.len()
                );
            }
        }
        return Ok(());
    }

    println!(
        "Found {} orphaned LLMC sessions (not tracked in state file):",
        orphaned_sessions.len()
    );
    for session_name in &orphaned_sessions {
        println!("  - {}", session_name);
    }
    println!("Cleaning up orphaned sessions...");

    for session_name in &orphaned_sessions {
        if verbose {
            println!("  Killing orphaned session: {}", session_name);
        }
        if let Err(e) = session::kill_session(session_name) {
            eprintln!("Warning: Failed to kill orphaned session '{}': {}", session_name, e);
        }
    }

    println!("✓ Cleaned up {} orphaned sessions", orphaned_sessions.len());
    Ok(())
}

fn ensure_tmux_running() -> Result<()> {
    if !is_tmux_server_running() {
        println!("Starting TMUX server...");
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
    println!("Reconciling workers with state...");

    let worker_names: Vec<String> = state.workers.keys().cloned().collect();

    for worker_name in &worker_names {
        if let Some(worker_record) = state.workers.get_mut(worker_name)
            && !session::session_exists(&worker_record.session_id)
        {
            println!("  Worker '{}' session not found, marking offline", worker_name);
            worker_record.status = WorkerStatus::Offline;
        }
    }

    for worker_name in &worker_names {
        if let Some(worker_record) = state.workers.get(worker_name)
            && worker_record.status == WorkerStatus::Offline
        {
            println!("  Starting worker '{}'...", worker_name);
            start_worker(worker_name, config, state, verbose)?;
        }
    }

    Ok(())
}

fn start_worker(name: &str, config: &Config, state: &mut State, verbose: bool) -> Result<()> {
    let worker_record =
        state.get_worker(name).with_context(|| format!("Worker '{}' not found in state", name))?;

    let worktree_path = PathBuf::from(&worker_record.worktree_path);

    if verbose {
        println!("    [verbose] Checking worktree: {}", worktree_path.display());
    }

    if !worktree_path.exists() {
        println!("  Worker '{}' worktree missing, recreating...", name);
        match add::recreate_missing_worktree(name, &worker_record.branch, &worktree_path) {
            Ok(()) => {
                println!("  ✓ Worktree recreated successfully");
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

    let Some(worker_config) = config.get_worker(name) else {
        tracing::warn!(
            "Worker '{}' exists in state but not in config.toml. This indicates a configuration issue.",
            name
        );
        bail!(
            "Worker '{}' not found in config.toml\n\
             The worker exists in state.json but has no corresponding [workers.{}] section in config.toml.\n\n\
             To fix this:\n\
             1. Run 'llmc doctor --repair' to diagnose and fix the issue, or\n\
             2. Manually add a [workers.{}] section to ~/llmc/config.toml, or\n\
             3. Run 'llmc nuke {}' and 'llmc add {}' to recreate the worker",
            name,
            name,
            name,
            name,
            name
        );
    };

    if verbose {
        println!("    [verbose] Session ID: {}", worker_record.session_id);
        println!(
            "    [verbose] Session exists: {}",
            session::session_exists(&worker_record.session_id)
        );
    }

    if !session::session_exists(&worker_record.session_id) {
        if verbose {
            println!("    [verbose] Creating new TMUX session for worker '{}'", name);
        }
        session::start_worker_session(
            &worker_record.session_id,
            &worktree_path,
            worker_config,
            verbose,
        )?;
    } else {
        if verbose {
            println!("    [verbose] Starting Claude in existing session");
        }
        worker::start_claude_in_session(&worker_record.session_id, worker_config)?;
    }

    let is_clean = git::is_worktree_clean(&worktree_path).unwrap_or(false);

    let worker_mut = state.get_worker_mut(name).unwrap();
    if is_clean {
        worker_mut.status = WorkerStatus::Idle;
        if verbose {
            println!("    [verbose] Worker '{}' worktree is clean, marked as Idle", name);
        }
    } else {
        worker_mut.status = WorkerStatus::Error;
        println!("  ⚠ Worker '{}' has uncommitted changes or incomplete rebase", name);
        println!(
            "    Marked as Error. Run 'llmc doctor --fix' or 'llmc reset {}' to recover",
            name
        );
        if verbose {
            println!("    [verbose] Worker '{}' worktree is dirty, marked as Error", name);
        }
    }
    worker_mut.last_activity_unix = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    Ok(())
}

fn run_main_loop(
    no_patrol: bool,
    verbose: bool,
    shutdown: Arc<AtomicBool>,
    config: &Config,
    state: &mut State,
    state_path: &Path,
) -> Result<()> {
    let mut config = config.clone();
    let patrol = Patrol::new(&config);
    let patrol_interval = Duration::from_secs(config.defaults.patrol_interval_secs as u64);
    let mut last_patrol = SystemTime::now();

    while !shutdown.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_secs(1));

        // Reload state to pick up changes from other commands (e.g., llmc start)
        *state = State::load(state_path)?;

        poll_worker_states(state)?;
        start_offline_workers(&mut config, state, verbose)?;
        state.save(state_path)?;

        if !no_patrol
            && SystemTime::now().duration_since(last_patrol).unwrap_or_default() >= patrol_interval
        {
            if verbose {
                println!("Running patrol...");
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
                    if !report.stuck_workers_nudged.is_empty() {
                        tracing::info!(
                            "Patrol nudged stuck workers: {:?}",
                            report.stuck_workers_nudged
                        );
                    }
                    if !report.errors.is_empty() {
                        tracing::error!("Patrol encountered errors: {:?}", report.errors);
                    }
                }
                Err(e) => {
                    tracing::error!("Patrol failed: {}", e);
                }
            }
            last_patrol = SystemTime::now();
        }
    }

    graceful_shutdown(&config, state)?;
    state.save(state_path)?;

    Ok(())
}

fn poll_worker_states(state: &mut State) -> Result<()> {
    let worker_names: Vec<String> = state.workers.keys().cloned().collect();

    for worker_name in &worker_names {
        if let Some(worker_record) = state.workers.get_mut(worker_name) {
            if worker_record.status == WorkerStatus::Offline {
                continue;
            }

            if !session::session_exists(&worker_record.session_id) {
                println!("  Worker '{}' session disappeared, marking offline", worker_record.name);
                worker_record.status = WorkerStatus::Offline;
                worker_record.last_activity_unix =
                    SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            }
        }
    }

    Ok(())
}

fn start_offline_workers(config: &mut Config, state: &mut State, verbose: bool) -> Result<()> {
    let state_path = state::get_state_path();
    let worker_names: Vec<String> = state.workers.keys().cloned().collect();

    let has_offline_workers = worker_names.iter().any(|name| {
        state.workers.get(name).map(|w| w.status == WorkerStatus::Offline).unwrap_or(false)
    });

    if has_offline_workers {
        let config_path = config::get_config_path();
        if verbose {
            println!("  [verbose] Reloading config before starting offline workers");
        }
        *config = Config::load(&config_path).context("Failed to reload config.toml")?;
    }

    for worker_name in &worker_names {
        if let Some(worker_record) = state.workers.get(worker_name)
            && worker_record.status == WorkerStatus::Offline
        {
            println!("  Starting offline worker '{}'...", worker_name);

            let session_id = worker_record.session_id.clone();
            if session::session_exists(&session_id) {
                if verbose {
                    println!(
                        "  [verbose] Session {} already exists, skipping duplicate start",
                        session_id
                    );
                }
                continue;
            }

            if let Some(worker_mut) = state.get_worker_mut(worker_name) {
                worker_mut.status = WorkerStatus::Idle;
                worker_mut.last_activity_unix =
                    SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            }
            state.save(&state_path)?;

            if let Err(e) = start_worker(worker_name, config, state, verbose) {
                eprintln!("  Failed to start worker '{}': {}", worker_name, e);
                if let Some(worker_mut) = state.get_worker_mut(worker_name) {
                    worker_mut.status = WorkerStatus::Error;
                    worker_mut.last_activity_unix =
                        SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
                }
                state.save(&state_path)?;
            }
        }
    }

    Ok(())
}

fn graceful_shutdown(config: &Config, state: &mut State) -> Result<()> {
    println!("Shutting down workers...");

    state.daemon_running = false;

    let sender = super::super::tmux::sender::TmuxSender::new();

    let worker_names: Vec<String> = state.workers.keys().cloned().collect();

    for worker_name in &worker_names {
        if let Some(worker_record) = state.workers.get(worker_name) {
            if worker_record.status == WorkerStatus::Offline {
                continue;
            }

            println!("  Stopping worker '{}'...", worker_name);

            if config.get_worker(worker_name).is_none() {
                tracing::warn!(
                    "Worker '{}' exists in state but not in config during shutdown",
                    worker_name
                );
                eprintln!(
                    "Warning: Worker '{}' not found in config.toml, skipping graceful shutdown",
                    worker_name
                );
                continue;
            }

            if let Err(e) = sender.send_keys_raw(&worker_record.session_id, "C-c") {
                eprintln!("Warning: Failed to send Ctrl-C to worker '{}': {}", worker_name, e);
            }
        }
    }

    thread::sleep(Duration::from_millis(500));

    for worker_name in &worker_names {
        if let Some(worker_record) = state.workers.get_mut(worker_name) {
            if session::session_exists(&worker_record.session_id)
                && let Err(e) = session::kill_session(&worker_record.session_id)
            {
                eprintln!("Warning: Failed to kill session '{}': {}", worker_record.session_id, e);
            }
            worker_record.status = WorkerStatus::Offline;
        }
    }

    Ok(())
}
