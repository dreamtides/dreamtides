use std::os::unix::process::CommandExt;
use std::process::Command;

use anyhow::{Context, Result, bail};

use super::super::config;
use super::super::state::{self, State};

/// Runs the attach command, connecting to a worker's TMUX session
pub fn run_attach(worker: &str) -> Result<()> {
    let llmc_root = config::get_llmc_root();
    if !llmc_root.exists() {
        bail!(
            "LLMC workspace not initialized. Run 'llmc init' first.\n\
             Expected workspace at: {}",
            llmc_root.display()
        );
    }

    let state_path = state::get_state_path();
    let state = State::load(&state_path)?;

    let worker_record = state.get_worker(worker).ok_or_else(|| {
        anyhow::anyhow!(
            "Worker '{}' not found\n\
             Available workers: {}",
            worker,
            format_workers(&state)
        )
    })?;

    let session_id = &worker_record.session_id;

    if !session_exists(session_id)? {
        bail!(
            "TMUX session '{}' does not exist for worker '{}'\n\
             Run 'llmc up' to start worker sessions",
            session_id,
            worker
        );
    }

    let err = Command::new("tmux").arg("attach-session").arg("-t").arg(session_id).exec();

    Err(anyhow::anyhow!("Failed to exec tmux attach-session: {}", err))
}

fn session_exists(session_id: &str) -> Result<bool> {
    let output = Command::new("tmux")
        .arg("has-session")
        .arg("-t")
        .arg(session_id)
        .output()
        .context("Failed to check TMUX session existence")?;

    Ok(output.status.success())
}

fn format_workers(state: &State) -> String {
    if state.workers.is_empty() {
        return "none".to_string();
    }
    state.workers.keys().map(String::as_str).collect::<Vec<_>>().join(", ")
}
