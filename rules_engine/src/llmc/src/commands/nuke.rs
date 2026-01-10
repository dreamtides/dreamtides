use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

use super::super::state::{self, State};
use super::super::tmux::session;
use super::super::{config, git};

/// Runs the nuke command, permanently removing a worker
pub fn run_nuke(worker: &str) -> Result<()> {
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

    let worker_record = state.get_worker(worker).ok_or_else(|| {
        anyhow::anyhow!(
            "Worker '{}' not found\n\
             Available workers: {}",
            worker,
            format_all_workers(&state)
        )
    })?;

    let session_id = worker_record.session_id.clone();
    let worktree_path = PathBuf::from(&worker_record.worktree_path);
    let branch = worker_record.branch.clone();

    if !confirm_nuke(worker, &session_id, &worktree_path, &branch)? {
        println!("Cancelled.");
        return Ok(());
    }

    println!("Nuking worker '{}'...", worker);

    session::kill_session(&session_id).ok();
    println!("  ✓ Killed TMUX session: {}", session_id);

    if worktree_path.exists() {
        git::remove_worktree(&worktree_path).ok();
        println!("  ✓ Removed worktree: {}", worktree_path.display());
    }

    git::delete_branch(&llmc_root, &branch, true).ok();
    println!("  ✓ Deleted branch: {}", branch);

    state.remove_worker(worker);
    state.save(&state_path)?;
    println!("  ✓ Removed from state.json");

    println!("✓ Worker '{}' has been nuked", worker);

    Ok(())
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
