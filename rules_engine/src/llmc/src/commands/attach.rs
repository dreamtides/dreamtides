use std::os::unix::process::CommandExt;
use std::process::Command;

use anyhow::{Context, Result, bail};

use super::super::config;
use super::super::state::{self, State};
use super::super::tmux::session;
use super::console;

/// Runs the attach command, connecting to a worker's or console's TMUX session
pub fn run_attach(target: &str) -> Result<()> {
    let llmc_root = config::get_llmc_root();
    if !llmc_root.exists() {
        bail!(
            "LLMC workspace not initialized. Run 'llmc init' first.\n\
             Expected workspace at: {}",
            llmc_root.display()
        );
    }

    // Check if this is a console session
    if console::is_console_name(target) {
        return attach_to_console(target);
    }

    // Otherwise, treat as a worker
    attach_to_worker(target)
}

/// Attaches to a console session
fn attach_to_console(name: &str) -> Result<()> {
    let session_id = console::normalize_console_name(name);

    if !session::session_exists(&session_id) {
        let consoles = console::list_console_sessions()?;
        let available = if consoles.is_empty() {
            "none (run 'llmc console' to create one)".to_string()
        } else {
            consoles.join(", ")
        };
        bail!(
            "Console session '{}' does not exist\n\
             Available consoles: {}",
            session_id,
            available
        );
    }

    let err = Command::new("tmux").arg("attach-session").arg("-t").arg(&session_id).exec();

    Err(anyhow::anyhow!("Failed to exec tmux attach-session: {}", err))
}

/// Attaches to a worker session
fn attach_to_worker(worker: &str) -> Result<()> {
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
