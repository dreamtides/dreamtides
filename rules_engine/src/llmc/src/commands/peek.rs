use anyhow::{Context, Result, bail};

use super::super::config;
use super::super::state::{self, State};
use super::super::tmux::session;

pub fn run_peek(worker: Option<String>, lines: u32) -> Result<()> {
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

    if state.workers.is_empty() {
        bail!("No workers configured. Run 'llmc add <name>' to add a worker.");
    }

    let worker_name = if let Some(name) = worker {
        if !state.workers.contains_key(&name) {
            bail!(
                "Worker '{}' not found\n\
                 Available workers: {}",
                name,
                format_workers(&state)
            );
        }
        name
    } else {
        select_most_recent_worker(&state)?
    };

    let worker_record = state
        .get_worker(&worker_name)
        .ok_or_else(|| anyhow::anyhow!("Worker '{}' not found", worker_name))?;

    let session_id = &worker_record.session_id;

    if !session::session_exists(session_id) {
        bail!(
            "TMUX session '{}' does not exist for worker '{}'\n\
             Run 'llmc up' to start worker sessions",
            session_id,
            worker_name
        );
    }

    let output = session::capture_pane(session_id, lines)
        .with_context(|| format!("Failed to capture pane for worker '{}'", worker_name))?;

    if output.trim().is_empty() {
        println!("(no output captured)");
    } else {
        println!("{}", output);
    }

    Ok(())
}

fn select_most_recent_worker(state: &State) -> Result<String> {
    state
        .workers
        .values()
        .max_by_key(|w| w.last_activity_unix)
        .map(|w| w.name.clone())
        .ok_or_else(|| anyhow::anyhow!("No workers available"))
}

fn format_workers(state: &State) -> String {
    if state.workers.is_empty() {
        return "none".to_string();
    }
    state.workers.keys().map(String::as_str).collect::<Vec<_>>().join(", ")
}
