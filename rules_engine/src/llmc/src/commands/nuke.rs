use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};

use super::super::config::{self as config, Config};
use super::super::git;
use super::super::lock::StateLock;
use super::super::state::{self, State, WorkerStatus};
use super::super::tmux::session;
use super::add;
/// Runs the nuke command, permanently removing a worker or resetting it to idle
/// state
pub fn run_nuke(worker: Option<&str>, all: bool, reset: bool) -> Result<()> {
    let llmc_root = config::get_llmc_root();
    if !llmc_root.exists() {
        bail!(
            "LLMC workspace not initialized. Run 'llmc init' first.\n\
             Expected workspace at: {}",
            llmc_root.display()
        );
    }
    if reset && all {
        bail!("Cannot specify both --reset and --all");
    }
    let _lock = StateLock::acquire()?;
    let state_path = state::get_state_path();
    let mut state = State::load(&state_path)?;
    if all {
        if worker.is_some() {
            bail!("Cannot specify both --all and a worker name");
        }
        let worker_names: Vec<_> = state.workers.keys().cloned().collect();
        if worker_names.is_empty() {
            println!("No workers to nuke.");
            return Ok(());
        }
        println!("This will permanently delete {} workers:", worker_names.len());
        for name in &worker_names {
            println!("  - {}", name);
        }
        println!("\nProceed? [y/N] ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
        let total_count = worker_names.len();
        let mut nuked_count = 0;
        for name in worker_names {
            if nuke_worker(&mut state, &llmc_root, &name)? {
                nuked_count += 1;
            }
        }
        if nuked_count > 0 {
            state.save(&state_path)?;
            println!("✓ {} worker(s) have been nuked", nuked_count);
            if nuked_count < total_count {
                tracing::info!(
                    "Nuked {} out of {} workers (some operations were cancelled)",
                    nuked_count,
                    total_count
                );
            }
        } else {
            tracing::info!("All nuke operations were cancelled");
        }
    } else {
        let Some(worker) = worker else {
            bail!("Worker name required (or use --all to nuke all workers)");
        };
        if reset {
            if reset_worker(&mut state, &llmc_root, worker)? {
                state.save(&state_path)?;
                println!("✓ Worker '{}' has been reset to idle state", worker);
            }
        } else if nuke_worker(&mut state, &llmc_root, worker)? {
            state.save(&state_path)?;
            println!("✓ Worker '{}' has been nuked", worker);
        }
    }
    Ok(())
}
fn nuke_worker(state: &mut State, llmc_root: &Path, worker: &str) -> Result<bool> {
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
    if !confirm_nuke(worker, &session_id, &worktree_path, &branch)? {
        tracing::info!("User cancelled nuke operation for worker '{}'", worker);
        println!("Cancelled nuking '{}'.", worker);
        return Ok(false);
    }
    println!("Nuking worker '{}'...", worker);
    if let Err(e) = session::kill_session(&session_id) {
        eprintln!("  ⚠ Failed to kill TMUX session {}: {}", session_id, e);
    } else {
        println!("  ✓ Killed TMUX session: {}", session_id);
    }
    if worktree_path.exists() {
        if let Err(e) = git::remove_worktree(llmc_root, &worktree_path, true) {
            eprintln!("  ⚠ Failed to remove worktree: {}", e);
        } else {
            println!("  ✓ Removed worktree: {}", worktree_path.display());
        }
    }
    if let Err(e) = git::delete_branch(llmc_root, &branch, true) {
        eprintln!("  ⚠ Failed to delete branch {}: {}", branch, e);
    } else {
        println!("  ✓ Deleted branch: {}", branch);
    }
    state.remove_worker(worker);
    println!("  ✓ Removed from state.json");
    if let Err(e) = remove_worker_from_config(worker) {
        eprintln!("  ⚠ Failed to remove worker from config.toml: {}", e);
    } else {
        println!("  ✓ Removed from config.toml (if present)");
    }
    tracing::info!("Successfully nuked worker '{}'", worker);
    Ok(true)
}
fn reset_worker(state: &mut State, llmc_root: &Path, worker: &str) -> Result<bool> {
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
    if !confirm_reset(worker, &session_id, &worktree_path, &branch)? {
        tracing::info!("User cancelled reset operation for worker '{}'", worker);
        println!("Cancelled resetting '{}'.", worker);
        return Ok(false);
    }
    println!("Resetting worker '{}' to idle state...", worker);
    if let Err(e) = session::kill_session(&session_id) {
        eprintln!("  ⚠ Failed to kill TMUX session {}: {}", session_id, e);
    } else {
        println!("  ✓ Killed TMUX session: {}", session_id);
    }
    if worktree_path.exists() {
        if let Err(e) = git::remove_worktree(llmc_root, &worktree_path, true) {
            eprintln!("  ⚠ Failed to remove worktree: {}", e);
        } else {
            println!("  ✓ Removed worktree: {}", worktree_path.display());
        }
    }
    if let Err(e) = git::delete_branch(llmc_root, &branch, true) {
        eprintln!("  ⚠ Failed to delete branch {}: {}", branch, e);
    } else {
        println!("  ✓ Deleted branch: {}", branch);
    }
    println!("  Fetching latest master...");
    git::fetch_origin(llmc_root)?;
    println!("  Creating new branch {}...", branch);
    if git::branch_exists(llmc_root, &branch) {
        println!("    Branch already exists (reusing)");
    } else {
        git::create_branch(llmc_root, &branch, "origin/master")?;
    }
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
    add::copy_tabula_to_worktree(llmc_root, &worktree_path)?;
    let worker_mut = state.get_worker_mut(worker).unwrap();
    worker_mut.status = WorkerStatus::Idle;
    worker_mut.current_prompt.clear();
    worker_mut.commit_sha = None;
    worker_mut.last_activity_unix = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    worker_mut.crash_count = 0;
    worker_mut.last_crash_unix = None;
    println!("  ✓ Reset state to Idle");
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
fn confirm_nuke(
    worker: &str,
    session_id: &str,
    worktree_path: &Path,
    branch: &str,
) -> Result<bool> {
    println!(
        "This will permanently remove worker '{}':\n\
         \n\
         Removes:\n\
         - TMUX session: {}\n\
         - Worktree: {}\n\
         - Branch: {}\n\
         - Any uncommitted work will be LOST\n\
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
fn remove_worker_from_config(worker: &str) -> Result<()> {
    let config_path = config::get_config_path();
    let config_content = fs::read_to_string(&config_path).context("Failed to read config.toml")?;
    let section_header = format!("[workers.{}]", worker);
    let lines: Vec<&str> = config_content.lines().collect();
    let mut new_lines = Vec::new();
    let mut skip_section = false;
    for line in lines {
        let trimmed = line.trim();
        if trimmed == section_header {
            skip_section = true;
            continue;
        }
        if skip_section {
            if trimmed.starts_with('[') {
                skip_section = false;
            } else {
                continue;
            }
        }
        new_lines.push(line);
    }
    let new_content = new_lines.join("\n");
    if !new_content.ends_with('\n') && !new_content.is_empty() {
        fs::write(&config_path, format!("{}\n", new_content))
            .context("Failed to write config.toml")?;
    } else {
        fs::write(&config_path, new_content).context("Failed to write config.toml")?;
    }
    Ok(())
}
