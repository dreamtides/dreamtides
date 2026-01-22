use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use lattice::cli::color_theme;
use tokio::sync::mpsc::Receiver;
use tracing::{debug, error, info, warn};

use crate::auto_mode::auto_accept::{self, AutoAcceptResult};
use crate::auto_mode::auto_config::{AutoConfig, ResolvedAutoConfig};
use crate::auto_mode::auto_failure::{self, HardFailure, RecoveryResult, TransientFailure};
use crate::auto_mode::auto_logging::{AutoLogger, TaskResult};
use crate::auto_mode::heartbeat_thread::{DaemonRegistration, HeartbeatThread};
use crate::auto_mode::task_pool::{self, TaskPoolResult};
use crate::auto_mode::{auto_logging, auto_workers, heartbeat_thread};
use crate::config::Config;
use crate::git;
use crate::ipc::messages::HookMessage;
use crate::lock::StateLock;
use crate::patrol::{self, Patrol};
use crate::state::{self, State, WorkerStatus};
use crate::tmux::sender::TmuxSender;
use crate::tmux::session;
use crate::worker::{self, WorkerTransition};

const INITIAL_SOURCE_REPO_DIRTY_BACKOFF_SECS: u64 = 60;
const MAX_SOURCE_REPO_DIRTY_BACKOFF_SECS: u64 = 3600;

/// Runs the auto mode daemon.
///
/// This is the main entry point for autonomous operation. It orchestrates:
/// - Task assignment from the task pool to idle auto workers
/// - Automatic acceptance of completed worker changes
/// - Patrol operations for health monitoring
/// - Graceful shutdown on errors or Ctrl-C
pub fn run_auto_mode(
    llmc_config: &Config,
    auto_config: &ResolvedAutoConfig,
    shutdown: Arc<AtomicBool>,
    ipc_receiver: Option<Receiver<HookMessage>>,
) -> Result<()> {
    let instance_id = heartbeat_thread::generate_instance_id();
    let logger = AutoLogger::new().context("Failed to initialize auto mode logger")?;

    println!("{}", color_theme::dim("Auto mode configuration:"));
    println!(
        "  {}",
        color_theme::muted(format!("Task pool command: {}", auto_config.task_pool_command))
    );
    println!("  {}", color_theme::muted(format!("Concurrency: {}", auto_config.concurrency)));
    if let Some(ref cmd) = auto_config.post_accept_command {
        println!("  {}", color_theme::muted(format!("Post-accept command: {}", cmd)));
    }

    info!(
        instance_id = %instance_id,
        concurrency = auto_config.concurrency,
        task_pool_command = %auto_config.task_pool_command,
        "Starting auto mode daemon"
    );

    // Register daemon
    let log_file = auto_logging::auto_log_path().to_string_lossy().to_string();
    let registration = DaemonRegistration::new(&instance_id, &log_file);
    registration.write().context("Failed to write daemon registration")?;
    logger.log_daemon_startup(&instance_id, auto_config.concurrency);

    // Start heartbeat thread
    let mut heartbeat = HeartbeatThread::start(&instance_id);

    // Run the main orchestration loop
    let result = run_orchestration_loop(
        llmc_config,
        auto_config,
        &instance_id,
        &logger,
        shutdown.clone(),
        ipc_receiver,
    );

    // Cleanup on exit
    heartbeat.stop();
    let shutdown_reason = match &result {
        Ok(()) => "Normal shutdown (Ctrl-C)",
        Err(e) => &format!("Error: {}", e),
    };
    logger.log_daemon_shutdown(&instance_id, shutdown_reason);
    if let Err(e) = DaemonRegistration::remove() {
        warn!("Failed to remove daemon registration: {}", e);
    }

    result
}

fn run_orchestration_loop(
    llmc_config: &Config,
    auto_config: &ResolvedAutoConfig,
    _instance_id: &str,
    logger: &AutoLogger,
    shutdown: Arc<AtomicBool>,
    mut ipc_receiver: Option<Receiver<HookMessage>>,
) -> Result<()> {
    let state_path = state::get_state_path();
    let config_path = crate::config::get_config_path();
    let patrol_interval = Duration::from_secs(llmc_config.defaults.patrol_interval_secs as u64);

    // Load initial state and ensure auto workers exist
    {
        let _lock = StateLock::acquire()?;
        let mut state = State::load(&state_path)?;

        let worker_names = auto_workers::ensure_auto_workers_exist(
            &mut state,
            llmc_config,
            auto_config.concurrency,
        )?;
        auto_workers::set_auto_mode_active(&mut state, worker_names.clone());
        state.daemon_running = true;
        state.save(&state_path)?;

        // Reload config after creating new workers since ensure_auto_workers_exist
        // may have added new workers to config.toml
        let fresh_config =
            Config::load(&config_path).context("Failed to reload config after creating workers")?;

        // Only start sessions for auto workers that don't already have running
        // sessions. Sessions may already exist if they were started by
        // reconcile_and_start_workers (which runs before this function). We
        // don't want to kill and restart those sessions because:
        // 1. The existing sessions may have already fired SessionStart hooks
        // 2. Killing them would create stale hooks that cause race conditions
        // 3. The SessionStart hooks from the existing sessions are the correct ones
        for name in &worker_names {
            if let Some(worker) = state.get_worker(name)
                && !session::session_exists(&worker.session_id)
            {
                println!("{}", color_theme::dim(format!("Starting auto worker '{}'...", name)));
                if let Err(e) = auto_workers::start_auto_worker_session(worker, &fresh_config) {
                    error!(worker = %name, error = %e, "Failed to start auto worker session");
                    return Err(e);
                }
            }
        }

        println!(
            "{}",
            color_theme::success(format!("✓ {} auto worker(s) initialized", worker_names.len()))
        );
        info!(workers = ?worker_names, "Auto workers initialized");
    }

    let patrol = Patrol::new(llmc_config);
    let mut shutdown_error: Option<String> = None;
    let mut iteration_count: u64 = 0;
    let loop_start_time = std::time::Instant::now();
    let mut waiting_for_tasks_printed = false;

    while !shutdown.load(Ordering::SeqCst) && shutdown_error.is_none() {
        thread::sleep(Duration::from_secs(1));
        iteration_count += 1;

        // Collect pending hook events
        let mut pending_hook_events: Vec<HookMessage> = Vec::new();
        if let Some(ref mut rx) = ipc_receiver {
            while let Ok(msg) = rx.try_recv() {
                info!("Received hook event: {:?} (id: {})", msg.event, msg.id);
                pending_hook_events.push(msg);
            }
        }

        let lock_result = StateLock::acquire();
        let Ok(_lock) = lock_result else {
            continue;
        };

        let mut state = match State::load(&state_path) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to load state: {}", e);
                continue;
            }
        };

        // Log worker states at the start of each iteration for diagnostics
        let auto_worker_states: Vec<(&str, WorkerStatus)> = state
            .workers
            .values()
            .filter(|w| auto_workers::is_auto_worker(&w.name))
            .map(|w| (w.name.as_str(), w.status))
            .collect();
        debug!(
            iteration = iteration_count,
            elapsed_secs = loop_start_time.elapsed().as_secs(),
            hook_events_received = pending_hook_events.len(),
            ?auto_worker_states,
            "Auto mode loop iteration"
        );

        // Warn if workers are stuck in Offline state for too long (no SessionStart
        // hook)
        for (name, status) in &auto_worker_states {
            if *status == WorkerStatus::Offline
                && iteration_count > 5
                && pending_hook_events.is_empty()
            {
                warn!(
                    worker = %name,
                    iteration = iteration_count,
                    elapsed_secs = loop_start_time.elapsed().as_secs(),
                    "Worker still in Offline state after {} iterations - \
                     SessionStart hook may not be firing. Check if Claude started correctly \
                     and hook config exists at .worktrees/{}/.claude/settings.json",
                    iteration_count,
                    name
                );
            }
        }

        // Reload config for fresh worker configs (needed after auto workers are
        // created)
        let fresh_config = match Config::load(&config_path) {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to reload config: {}", e);
                continue;
            }
        };

        // Process hook events (transitions workers from Offline -> Idle, etc.)
        for msg in &pending_hook_events {
            if let Err(e) = patrol::handle_hook_event(&msg.event, &mut state, &fresh_config) {
                error!("Error handling hook event {:?}: {}", msg.event, e);
            }
        }
        if !pending_hook_events.is_empty() {
            // Log state after processing hooks for diagnostics
            let states_after: Vec<(&str, WorkerStatus)> = state
                .workers
                .values()
                .filter(|w| auto_workers::is_auto_worker(&w.name))
                .map(|w| (w.name.as_str(), w.status))
                .collect();
            info!(
                hooks_processed = pending_hook_events.len(),
                ?states_after,
                "Processed hook events"
            );
            if let Err(e) = state.save(&state_path) {
                error!("Failed to save state after hook events: {}", e);
            }
        }

        // Process idle auto workers - assign tasks
        let any_task_assigned =
            match process_idle_workers(&mut state, llmc_config, auto_config, logger) {
                Ok(assigned) => assigned,
                Err(e) => {
                    logger.log_error("process_idle_workers", &e.to_string());
                    shutdown_error = Some(e.to_string());
                    break;
                }
            };

        // Track when all workers are idle with no tasks available
        if any_task_assigned {
            waiting_for_tasks_printed = false;
        } else {
            let all_workers_idle = state
                .workers
                .values()
                .filter(|w| auto_workers::is_auto_worker(&w.name))
                .all(|w| w.status == WorkerStatus::Idle);
            if all_workers_idle && !waiting_for_tasks_printed {
                println!("{}", color_theme::muted("Waiting for tasks..."));
                info!("All workers idle, waiting for tasks from task pool");
                waiting_for_tasks_printed = true;
            }
        }

        // Process completed workers - auto accept
        if let Err(e) = process_completed_workers(&mut state, llmc_config, auto_config, logger) {
            logger.log_error("process_completed_workers", &e.to_string());
            shutdown_error = Some(e.to_string());
            break;
        }

        // Save state after processing
        if let Err(e) = state.save(&state_path) {
            error!("Failed to save state: {}", e);
        }

        // Run patrol
        match patrol.run_patrol(&mut state, llmc_config) {
            Ok(report) => {
                if !report.transitions_applied.is_empty() {
                    info!(
                        "Patrol applied {} transitions: {:?}",
                        report.transitions_applied.len(),
                        report.transitions_applied
                    );
                }
                if !report.errors.is_empty() {
                    for err in &report.errors {
                        error!("Patrol error: {}", err);
                    }
                }
            }
            Err(e) => {
                error!("Patrol failed: {}", e);
            }
        }

        // Handle transient failures with recovery attempts
        if let Some(hard_failure) =
            handle_auto_failures(&mut state, llmc_config, logger, &state_path)
        {
            logger.log_error("auto_failure", &hard_failure.to_string());
            shutdown_error = Some(hard_failure.to_string());
            break;
        }

        // Save state after patrol
        if let Err(e) = state.save(&state_path) {
            error!("Failed to save state after patrol: {}", e);
        }

        // Sleep for patrol interval (minus 1 second we already slept)
        let sleep_duration = patrol_interval.saturating_sub(Duration::from_secs(1));
        let sleep_chunks = sleep_duration.as_millis().saturating_div(100);
        for _ in 0..sleep_chunks {
            if shutdown.load(Ordering::SeqCst) || shutdown_error.is_some() {
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
    }

    // Graceful shutdown
    {
        let _lock = StateLock::acquire()?;
        let mut state = State::load(&state_path)?;
        graceful_shutdown(llmc_config, &mut state)?;
        state.daemon_running = false;
        auto_workers::clear_auto_mode_state(&mut state);
        state.save(&state_path)?;
    }

    if let Some(err) = shutdown_error {
        bail!("Auto mode daemon shutdown due to error: {}", err);
    }

    Ok(())
}

/// Processes idle auto workers by assigning tasks from the task pool.
///
/// Returns `true` if any task was assigned to a worker, `false` otherwise.
fn process_idle_workers(
    state: &mut State,
    llmc_config: &Config,
    auto_config: &ResolvedAutoConfig,
    logger: &AutoLogger,
) -> Result<bool> {
    let idle_workers: Vec<String> =
        auto_workers::get_idle_auto_workers(state).iter().map(|w| w.name.clone()).collect();

    let mut any_task_assigned = false;

    for worker_name in idle_workers {
        let task_result =
            task_pool::execute_task_pool_command(&auto_config.task_pool_command, logger);

        match task_result {
            TaskPoolResult::Task(task) => {
                // Skip the first line (task ID like "LDWWQN: task-name") and use the rest as
                // task content
                let task_content =
                    task.lines().skip(1).collect::<Vec<_>>().join("\n").trim().to_string();
                let task_preview: String = if task_content.is_empty() {
                    task.chars().take(60).collect()
                } else {
                    task_content.chars().take(60).collect()
                };
                println!(
                    "{}",
                    color_theme::accent(format!(
                        "[{}] Assigning task: {}...",
                        worker_name, task_preview
                    ))
                );
                info!(worker = %worker_name, task_len = task.len(), "Assigning task to worker");
                assign_task_to_worker(state, llmc_config, &worker_name, &task, logger)?;
                any_task_assigned = true;
            }
            TaskPoolResult::NoTasksAvailable => {
                // No tasks available, skip this worker
            }
            TaskPoolResult::Error(e) => {
                eprintln!(
                    "{}",
                    color_theme::error(format!(
                        "[{}] Task pool command failed: {}",
                        worker_name, e
                    ))
                );
                error!(worker = %worker_name, error = %e, "Task pool command failed");
                return Err(e.into());
            }
        }
    }

    Ok(any_task_assigned)
}

/// Assigns a task to an idle auto worker.
fn assign_task_to_worker(
    state: &mut State,
    _llmc_config: &Config,
    worker_name: &str,
    task: &str,
    logger: &AutoLogger,
) -> Result<()> {
    let worker = state.get_worker(worker_name).context("Worker not found")?;
    let worktree_path = PathBuf::from(&worker.worktree_path);
    let session_id = worker.session_id.clone();

    // Pull latest master
    if git::has_commits_ahead_of(&worktree_path, "origin/master")? {
        info!(worker = %worker_name, "Resetting stale commits before starting task");
        git::reset_to_ref(&worktree_path, "origin/master")?;
    }
    git::pull_rebase(&worktree_path)?;

    // Build prompt
    let full_prompt = build_auto_prompt(&worktree_path, task)?;

    // Send to worker
    let tmux_sender = TmuxSender::new();
    tmux_sender.send(&session_id, "/clear")?;
    tmux_sender.send(&session_id, &full_prompt)?;

    // Update state
    let worker_mut = state.get_worker_mut(worker_name).context("Worker not found")?;
    worker::apply_transition(worker_mut, WorkerTransition::ToWorking {
        prompt: full_prompt,
        prompt_cmd: None,
    })?;

    logger.log_task_assigned(worker_name, task);
    info!(worker = %worker_name, "Task assigned successfully");

    Ok(())
}

/// Builds the full prompt for an auto worker task.
fn build_auto_prompt(worktree_path: &std::path::Path, task: &str) -> Result<String> {
    let repo_root = worktree_path
        .parent()
        .and_then(|p| p.parent())
        .context("Could not determine repository root")?;

    let prompt = format!(
        "You are working in: {}\n\
         Repository root: {}\n\
         \n\
         Follow the conventions in AGENTS.md\n\
         Run validation commands before committing\n\
         Create a single commit with your changes\n\
         Do NOT push to remote\n\
         \n\
         {}",
        worktree_path.display(),
        repo_root.display(),
        task
    );

    Ok(prompt)
}

/// Processes completed auto workers by running auto accept.
fn process_completed_workers(
    state: &mut State,
    llmc_config: &Config,
    auto_config: &ResolvedAutoConfig,
    logger: &AutoLogger,
) -> Result<()> {
    // Check if we're in a backoff period due to source repo being dirty
    if let Some(retry_after) = state.source_repo_dirty_retry_after_unix {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        if now < retry_after {
            let remaining = retry_after - now;
            info!(retry_in_secs = remaining, "Source repo dirty, waiting before retry");
            return Ok(());
        }
    }

    let completed_workers: Vec<String> = state
        .workers
        .values()
        .filter(|w| {
            auto_workers::is_auto_worker(&w.name)
                && (w.status == WorkerStatus::NeedsReview || w.status == WorkerStatus::NoChanges)
        })
        .map(|w| w.name.clone())
        .collect();

    for worker_name in completed_workers {
        println!("{}", color_theme::dim(format!("[{}] Processing completed work...", worker_name)));
        info!(worker = %worker_name, "Processing completed worker");

        // Get auto config for post_accept_command
        let auto_cfg = AutoConfig {
            task_pool_command: Some(auto_config.task_pool_command.clone()),
            concurrency: auto_config.concurrency,
            post_accept_command: auto_config.post_accept_command.clone(),
        };

        match auto_accept::auto_accept_worker(&worker_name, state, llmc_config, logger) {
            Ok(result) => {
                // Reset retry count on successful completion
                auto_failure::reset_retry_count(state, &worker_name);

                match &result {
                    AutoAcceptResult::Accepted { commit_sha } => {
                        // Clear source repo dirty backoff on successful accept
                        state.source_repo_dirty_retry_after_unix = None;
                        state.source_repo_dirty_backoff_secs = None;

                        println!(
                            "{}",
                            color_theme::success(format!(
                                "[{}] ✓ Changes accepted ({})",
                                worker_name,
                                &commit_sha[..8.min(commit_sha.len())]
                            ))
                        );
                        logger.log_task_completed(&worker_name, TaskResult::NeedsReview);
                        info!(worker = %worker_name, commit = %commit_sha, "Worker changes accepted");

                        // Run post-accept command if configured
                        if let Err(e) = auto_accept::execute_post_accept_command(
                            &worker_name,
                            commit_sha,
                            &auto_cfg,
                            logger,
                        ) {
                            eprintln!(
                                "{}",
                                color_theme::error(format!(
                                    "[{}] Post-accept command failed: {}",
                                    worker_name, e
                                ))
                            );
                            error!(worker = %worker_name, error = %e, "Post-accept command failed");
                            return Err(e.into());
                        }
                    }
                    AutoAcceptResult::AcceptedWithCleanupFailure { commit_sha, cleanup_error } => {
                        // Clear source repo dirty backoff - the accept itself succeeded
                        state.source_repo_dirty_retry_after_unix = None;
                        state.source_repo_dirty_backoff_secs = None;

                        // Print success with warning about cleanup failure
                        println!(
                            "{}",
                            color_theme::success(format!(
                                "[{}] ✓ Changes accepted ({})",
                                worker_name,
                                &commit_sha[..8.min(commit_sha.len())]
                            ))
                        );
                        eprintln!(
                            "{}",
                            color_theme::warning(format!(
                                "[{}] ⚠ Worker cleanup failed: {}. Worker may need manual reset.",
                                worker_name, cleanup_error
                            ))
                        );
                        logger.log_task_completed(&worker_name, TaskResult::NeedsReview);
                        warn!(
                            worker = %worker_name,
                            commit = %commit_sha,
                            cleanup_error = %cleanup_error,
                            "Worker changes accepted but cleanup failed - continuing with other workers"
                        );

                        // Still run post-accept command since the accept succeeded
                        if let Err(e) = auto_accept::execute_post_accept_command(
                            &worker_name,
                            commit_sha,
                            &auto_cfg,
                            logger,
                        ) {
                            eprintln!(
                                "{}",
                                color_theme::error(format!(
                                    "[{}] Post-accept command failed: {}",
                                    worker_name, e
                                ))
                            );
                            error!(worker = %worker_name, error = %e, "Post-accept command failed");
                            return Err(e.into());
                        }
                    }
                    AutoAcceptResult::NoChanges => {
                        // Clear source repo dirty backoff on successful accept
                        state.source_repo_dirty_retry_after_unix = None;
                        state.source_repo_dirty_backoff_secs = None;

                        println!(
                            "{}",
                            color_theme::muted(format!("[{}] No changes to accept", worker_name))
                        );
                        logger.log_task_completed(&worker_name, TaskResult::NoChanges);
                        info!(worker = %worker_name, "Worker completed with no changes");
                    }
                    AutoAcceptResult::SourceRepoDirty => {
                        // Calculate exponential backoff
                        let current_backoff = state.source_repo_dirty_backoff_secs.unwrap_or(0);
                        let next_backoff = if current_backoff == 0 {
                            INITIAL_SOURCE_REPO_DIRTY_BACKOFF_SECS
                        } else {
                            (current_backoff * 2).min(MAX_SOURCE_REPO_DIRTY_BACKOFF_SECS)
                        };

                        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                        let retry_after = now + next_backoff;

                        state.source_repo_dirty_backoff_secs = Some(next_backoff);
                        state.source_repo_dirty_retry_after_unix = Some(retry_after);

                        println!(
                            "[{}] Source repository has uncommitted changes. Will retry in {} seconds.",
                            worker_name, next_backoff
                        );
                        warn!(
                            worker = %worker_name,
                            backoff_secs = next_backoff,
                            retry_after_unix = retry_after,
                            "Source repository dirty, scheduling retry with exponential backoff"
                        );
                    }
                    AutoAcceptResult::RebaseConflict { conflicts } => {
                        // Worker is now in Rebasing state, resolving conflicts.
                        // Next iteration will detect completion and retry accept.
                        println!(
                            "[{}] Rebase conflict detected - worker resolving {} conflicting file(s)",
                            worker_name,
                            conflicts.len()
                        );
                        info!(
                            worker = %worker_name,
                            conflict_count = conflicts.len(),
                            ?conflicts,
                            "Worker entered Rebasing state to resolve conflicts"
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "{}",
                    color_theme::error(format!("[{}] Auto accept failed: {}", worker_name, e))
                );
                error!(worker = %worker_name, error = %e, "Auto accept failed");
                return Err(e.into());
            }
        }
    }

    Ok(())
}

/// Performs graceful shutdown of all auto workers.
fn graceful_shutdown(_config: &Config, state: &mut State) -> Result<()> {
    println!("{}", color_theme::dim("Shutting down auto workers..."));
    info!("Initiating graceful shutdown of auto workers");
    let tmux_sender = TmuxSender::new();

    let auto_worker_names: Vec<String> =
        state.workers.keys().filter(|name| auto_workers::is_auto_worker(name)).cloned().collect();

    // Send Ctrl-C to all auto workers
    for worker_name in &auto_worker_names {
        if let Some(worker) = state.get_worker(worker_name) {
            if worker.status == WorkerStatus::Offline {
                continue;
            }
            info!(worker = %worker_name, "Sending Ctrl-C to worker");
            if let Err(e) = tmux_sender.send_keys_raw(&worker.session_id, "C-c") {
                warn!(worker = %worker_name, error = %e, "Failed to send Ctrl-C");
            }
        }
    }

    // Wait for workers to stop gracefully
    thread::sleep(Duration::from_millis(500));

    // Kill remaining sessions
    for worker_name in &auto_worker_names {
        if let Some(worker) = state.get_worker_mut(worker_name)
            && session::session_exists(&worker.session_id)
        {
            if let Err(e) = session::kill_session(&worker.session_id) {
                warn!(worker = %worker_name, error = %e, "Failed to kill session");
            }
            // Preserve rebasing state for recovery
            if worker.status != WorkerStatus::Rebasing {
                worker.status = WorkerStatus::Offline;
            }
        }
    }

    info!("Graceful shutdown complete");
    Ok(())
}

/// Handles auto mode failures by detecting transient failures and attempting
/// recovery.
///
/// Returns `Some(HardFailure)` if a hard failure is detected that requires
/// immediate shutdown. Returns `None` if no hard failures were detected.
fn handle_auto_failures(
    state: &mut State,
    config: &Config,
    _logger: &AutoLogger,
    state_path: &std::path::Path,
) -> Option<HardFailure> {
    // First check for existing hard failures (workers in error state)
    if let Some(hard_failure) = auto_failure::check_for_hard_failures(state) {
        error!(
            failure = %hard_failure,
            "Detected hard failure, triggering shutdown"
        );
        return Some(hard_failure);
    }

    // Detect transient failures
    let transient_failures = auto_failure::detect_transient_failures(state);
    if transient_failures.is_empty() {
        return None;
    }

    // Attempt recovery for each transient failure
    for failure in transient_failures {
        info!(failure = %failure, "Detected transient failure, attempting recovery");

        match auto_failure::attempt_recovery(&failure, state, config) {
            Ok(RecoveryResult::Recovered) => {
                info!(failure = %failure, "Recovery successful");
            }
            Ok(RecoveryResult::RetryNeeded) => {
                warn!(failure = %failure, "Recovery needs retry on next patrol cycle");
            }
            Ok(RecoveryResult::EscalateToHardFailure(hard_failure)) => {
                error!(
                    failure = %failure,
                    hard_failure = %hard_failure,
                    "Transient failure escalated to hard failure"
                );
                return Some(hard_failure);
            }
            Err(e) => {
                error!(
                    failure = %failure,
                    error = %e,
                    "Recovery attempt failed with error"
                );
                return Some(HardFailure::WorkerRetriesExhausted {
                    worker_name: get_worker_name_from_failure(&failure),
                    retry_count: 0,
                });
            }
        }
    }

    // Save state after recovery attempts
    if let Err(e) = state.save(state_path) {
        error!("Failed to save state after recovery attempts: {}", e);
    }

    None
}

/// Extracts the worker name from a transient failure.
fn get_worker_name_from_failure(failure: &TransientFailure) -> String {
    match failure {
        TransientFailure::SessionCrash { worker_name }
        | TransientFailure::TmuxSessionMissing { worker_name } => worker_name.clone(),
    }
}
