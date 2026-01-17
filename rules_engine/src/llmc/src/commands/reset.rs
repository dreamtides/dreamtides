use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Result, bail};

use super::super::config::{self, Config};
use super::super::git;
use super::super::lock::StateLock;
use super::super::state::{self, State, WorkerStatus};
use super::super::tmux::session;
use super::add;

/// Runs the reset command, resetting a worker to clean idle state by removing
/// and recreating its worktree and session
pub fn run_reset(worker_name: &str, yes: bool, json: bool) -> Result<()> {
    let llmc_root = config::get_llmc_root();
    if !llmc_root.exists() {
        bail!(
            "LLMC workspace not initialized. Run 'llmc init' first.\n\
             Expected workspace at: {}",
            llmc_root.display()
        );
    }

    let _lock = StateLock::acquire()?;
    let state_path = state::get_state_path();
    let mut state = State::load(&state_path)?;

    let previous_status = state
        .get_worker(worker_name)
        .map(|w| format!("{:?}", w.status).to_lowercase())
        .unwrap_or_else(|| "unknown".to_string());

    if reset_worker(&mut state, &llmc_root, worker_name, yes || json)? {
        state.save(&state_path)?;
        if json {
            let output = crate::json_output::ResetOutput {
                worker: worker_name.to_string(),
                previous_status,
                new_status: "idle".to_string(),
            };
            crate::json_output::print_json(&output);
        } else {
            println!("✓ Worker '{}' has been reset to idle state", worker_name);
        }
    }

    Ok(())
}

fn reset_worker(state: &mut State, llmc_root: &Path, worker: &str, yes: bool) -> Result<bool> {
    let worker_record = state.get_worker(worker).ok_or_else(|| {
        anyhow::anyhow!(
            "Worker '{}' not found\n\
             Available workers: {}",
            worker,
            format_all_workers(state)
        )
    })?;

    let session_id = worker_record.session_id.clone();
    let worktree_path = PathBuf::from(&worker_record.worktree_path);
    let branch = worker_record.branch.clone();

    let config_path = config::get_config_path();
    let cfg = Config::load(&config_path)?;
    let worker_config = cfg.workers.get(worker).cloned();

    if !yes && !confirm_reset(worker, &session_id, &worktree_path, &branch)? {
        tracing::info!("User cancelled reset operation for worker '{}'", worker);
        println!("Cancelled resetting '{}'.", worker);
        return Ok(false);
    }

    println!("Resetting worker '{}' to idle state...", worker);

    // Kill the TMUX session
    if let Err(e) = session::kill_session(&session_id) {
        eprintln!("  ⚠ Failed to kill TMUX session {}: {}", session_id, e);
    } else {
        println!("  ✓ Killed TMUX session: {}", session_id);
    }

    // Remove the worktree
    if worktree_path.exists() {
        if let Err(e) = git::remove_worktree(llmc_root, &worktree_path, true) {
            eprintln!("  ⚠ Failed to remove worktree: {}", e);
        } else {
            println!("  ✓ Removed worktree: {}", worktree_path.display());
        }
    }

    // Delete the branch
    if let Err(e) = git::delete_branch(llmc_root, &branch, true) {
        eprintln!("  ⚠ Failed to delete branch {}: {}", branch, e);
    } else {
        println!("  ✓ Deleted branch: {}", branch);
    }

    // Fetch latest master and create new branch
    println!("  Fetching latest master...");
    git::fetch_origin(llmc_root)?;

    println!("  Creating new branch {}...", branch);
    if git::branch_exists(llmc_root, &branch) {
        println!("    Branch already exists (reusing)");
    } else {
        git::create_branch(llmc_root, &branch, "origin/master")?;
    }

    // Create new worktree
    println!("  Creating new worktree at {}...", worktree_path.display());
    if worktree_path.exists() {
        bail!(
            "Worktree path already exists: {}\n\
             This should not happen. Please remove it manually and try again.",
            worktree_path.display()
        );
    }
    git::create_worktree(llmc_root, &branch, &worktree_path)?;
    println!("  ✓ Created worktree: {}", worktree_path.display());

    // Copy Tabula.xlsm
    add::copy_tabula_to_worktree(llmc_root, &worktree_path)?;

    // Create Serena project config
    add::create_serena_project(&worktree_path, worker)?;

    // Reset worker state
    let worker_mut = state.get_worker_mut(worker).unwrap();
    worker_mut.status = WorkerStatus::Idle;
    worker_mut.current_prompt.clear();
    worker_mut.commit_sha = None;
    worker_mut.last_activity_unix = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    worker_mut.crash_count = 0;
    worker_mut.last_crash_unix = None;
    println!("  ✓ Reset state to Idle");

    // Start new TMUX session if daemon is running
    let daemon_running = add::is_daemon_running();
    if daemon_running {
        println!("  Starting new TMUX session...");
        if let Some(wc) = worker_config {
            match session::start_worker_session(&session_id, &worktree_path, &wc, false) {
                Ok(()) => {
                    println!("  ✓ Started new TMUX session: {}", session_id);
                    println!("  ✓ Worker is ready for tasks");
                }
                Err(e) => {
                    eprintln!("  ⚠ Failed to start TMUX session: {}", e);
                    println!("  The daemon will retry starting this worker automatically.");
                    worker_mut.status = WorkerStatus::Offline;
                }
            }
        } else {
            eprintln!("  ⚠ Worker not found in config.toml, cannot start session");
            worker_mut.status = WorkerStatus::Offline;
        }
    } else {
        println!("  Daemon is not running, worker will start when you run 'llmc up'");
        worker_mut.status = WorkerStatus::Offline;
    }

    tracing::info!("Successfully reset worker '{}' to idle state", worker);
    Ok(true)
}

fn confirm_reset(
    worker: &str,
    session_id: &str,
    worktree_path: &Path,
    branch: &str,
) -> Result<bool> {
    println!(
        "This will reset worker '{}' to a clean idle state:\n\
         \n\
         Will remove:\n\
         - TMUX session: {}\n\
         - Worktree: {}\n\
         - Branch: {}\n\
         - Any uncommitted work will be LOST\n\
         \n\
         Will create:\n\
         - New worktree from origin/master\n\
         - New TMUX session\n\
         - Worker will be in idle state\n\
         \n\
         Configuration in config.toml will be preserved.\n\
         \n\
         Proceed? [y/N] ",
        worker,
        session_id,
        worktree_path.display(),
        branch
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().eq_ignore_ascii_case("y"))
}

fn format_all_workers(state: &State) -> String {
    if state.workers.is_empty() {
        return "none".to_string();
    }
    state.workers.keys().map(String::as_str).collect::<Vec<_>>().join(", ")
}
