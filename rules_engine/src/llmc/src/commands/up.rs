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
use super::super::worker;

/// Runs the up command, starting the LLMC daemon
pub fn run_up(no_patrol: bool, verbose: bool) -> Result<()> {
    let llmc_root = config::get_llmc_root();
    if !llmc_root.exists() {
        bail!(
            "LLMC workspace not initialized. Run 'llmc init' first.\n\
             Expected workspace at: {}",
            llmc_root.display()
        );
    }

    println!("Starting LLMC daemon...");

    let config_path = config::get_config_path();
    let config = Config::load(&config_path)?;

    let state_path = state::get_state_path();
    let mut state = State::load(&state_path)?;

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
        bail!(
            "Worktree does not exist for worker '{}': {}\n\
             Run 'llmc nuke {}' and 'llmc add {}' to recreate",
            name,
            worktree_path.display(),
            name,
            name
        );
    }

    let worker_config = config
        .get_worker(name)
        .with_context(|| format!("Worker '{}' not found in config", name))?;

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

    let worker_mut = state.get_worker_mut(name).unwrap();
    worker_mut.status = WorkerStatus::Idle;
    worker_mut.last_activity_unix = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    if verbose {
        println!("    [verbose] Worker '{}' marked as Idle", name);
    }

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
    let patrol = Patrol::new(config);
    let patrol_interval = Duration::from_secs(config.defaults.patrol_interval_secs as u64);
    let mut last_patrol = SystemTime::now();

    while !shutdown.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_secs(1));

        // Reload state to pick up changes from other commands (e.g., llmc start)
        *state = State::load(state_path)?;

        poll_worker_states(state)?;
        start_offline_workers(config, state, verbose)?;
        state.save(state_path)?;

        if !no_patrol
            && SystemTime::now().duration_since(last_patrol).unwrap_or_default() >= patrol_interval
        {
            if verbose {
                println!("Running patrol...");
            }
            match patrol.run_patrol(state, config) {
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

    graceful_shutdown(config, state)?;
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

fn start_offline_workers(config: &Config, state: &mut State, verbose: bool) -> Result<()> {
    let worker_names: Vec<String> = state.workers.keys().cloned().collect();

    for worker_name in &worker_names {
        if let Some(worker_record) = state.workers.get(worker_name)
            && worker_record.status == WorkerStatus::Offline
        {
            println!("  Starting offline worker '{}'...", worker_name);
            start_worker(worker_name, config, state, verbose)?;
        }
    }

    Ok(())
}

fn graceful_shutdown(config: &Config, state: &mut State) -> Result<()> {
    println!("Shutting down workers...");

    let sender = super::super::tmux::sender::TmuxSender::new();

    let worker_names: Vec<String> = state.workers.keys().cloned().collect();

    for worker_name in &worker_names {
        if let Some(worker_record) = state.workers.get(worker_name) {
            if worker_record.status == WorkerStatus::Offline {
                continue;
            }

            println!("  Stopping worker '{}'...", worker_name);

            if config.get_worker(worker_name).is_none() {
                eprintln!("Warning: Worker '{}' not found in config, skipping", worker_name);
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
