use anyhow::{Context, Result, bail};

use crate::llmc::config;
use crate::llmc::state::{self, State};
use crate::llmc::tmux::session;
pub fn run_peek(worker: Option<String>, lines: u32, json: bool) -> Result<()> {
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
    if json {
        let lines: Vec<String> = if output.trim().is_empty() {
            Vec::new()
        } else {
            output.lines().map(std::string::ToString::to_string).collect()
        };
        let json_output = crate::json_output::PeekOutput { worker: worker_name, lines };
        crate::json_output::print_json(&json_output);
    } else if output.trim().is_empty() {
        println!("(no output captured)");
    } else {
        let trimmed_output = trim_trailing_blank_lines(&output);
        println!("{}", trimmed_output);
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
/// Removes trailing blank lines from output while preserving content
fn trim_trailing_blank_lines(output: &str) -> &str {
    let mut end = output.len();
    let mut chars = output.char_indices().rev().peekable();
    while let Some((idx, ch)) = chars.next() {
        if ch == '\n' {
            if let Some(&(_, next_ch)) = chars.peek()
                && (next_ch == '\n' || next_ch == '\r')
            {
                end = idx;
                continue;
            }
            break;
        } else if !ch.is_whitespace() {
            break;
        }
        end = idx;
    }
    &output[..end]
}
