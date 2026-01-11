use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Result, bail};
use serde::Serialize;

use super::super::config;
use super::super::state::{State, WorkerStatus};

/// Runs the status command, displaying the current state of all workers
pub fn run_status(json: bool) -> Result<()> {
    let llmc_root = config::get_llmc_root();
    if !llmc_root.exists() {
        bail!(
            "LLMC workspace not initialized. Run 'llmc init' first.\n\
             Expected workspace at: {}",
            llmc_root.display()
        );
    }

    let (state, _config) = super::super::state::load_state_with_patrol()?;

    if state.workers.is_empty() {
        if json {
            println!("{{\"workers\":[]}}");
        } else {
            println!("No workers configured. Run 'llmc add <name>' to add a worker.");
        }
        return Ok(());
    }

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    if json {
        output_json(&state, now)?;
    } else {
        output_text(&state, now);
    }

    Ok(())
}

#[derive(Serialize)]
struct StatusOutput {
    workers: Vec<WorkerStatusOutput>,
}

#[derive(Serialize)]
struct WorkerStatusOutput {
    name: String,
    status: String,
    branch: String,
    time_in_state_secs: u64,
    commit_sha: Option<String>,
    prompt_excerpt: Option<String>,
}

fn output_json(state: &State, now: u64) -> Result<()> {
    let workers: Vec<WorkerStatusOutput> = state
        .workers
        .values()
        .map(|w| WorkerStatusOutput {
            name: w.name.clone(),
            status: format_status_json(w.status),
            branch: w.branch.clone(),
            time_in_state_secs: now.saturating_sub(w.last_activity_unix),
            commit_sha: w.commit_sha.clone(),
            prompt_excerpt: if w.current_prompt.is_empty() {
                None
            } else {
                Some(truncate_prompt(&w.current_prompt, 50))
            },
        })
        .collect();

    let output = StatusOutput { workers };
    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}

fn output_text(state: &State, now: u64) {
    println!("WORKERS");
    println!("───────");

    let mut workers: Vec<_> = state.workers.values().collect();
    workers.sort_by(|a, b| a.name.cmp(&b.name));

    for worker in workers {
        let status_str = format_status_colored(worker.status);
        let time_str = format_duration(now.saturating_sub(worker.last_activity_unix));

        let mut parts = vec![
            format!("{:<12}", worker.name),
            format!("{:<15}", status_str),
            format!("{:<15}", worker.branch),
            format!("{:>6}", time_str),
        ];

        if let Some(sha) = &worker.commit_sha {
            parts.push(format!("[{}]", &sha[..7.min(sha.len())]));
        }

        if !worker.current_prompt.is_empty() && worker.status != WorkerStatus::Idle {
            let excerpt = truncate_prompt(&worker.current_prompt, 50);
            parts.push(format!("\"{}...\"", excerpt));
        }

        println!("{}", parts.join(" "));
    }

    println!();
    print_summary(state);
}

fn print_summary(state: &State) {
    let mut status_counts: HashMap<WorkerStatus, usize> = HashMap::new();
    let workers: Vec<_> = state.workers.values().collect();
    for worker in workers {
        *status_counts.entry(worker.status).or_insert(0) += 1;
    }

    let total = state.workers.len();
    let status_parts: Vec<String> = status_counts
        .iter()
        .map(|(status, count)| format!("{} {}", count, format_status_json(*status)))
        .collect();

    println!("{} workers: {}", total, status_parts.join(", "));
}

fn format_status_json(status: WorkerStatus) -> String {
    match status {
        WorkerStatus::Idle => "idle".to_string(),
        WorkerStatus::Working => "working".to_string(),
        WorkerStatus::NeedsInput => "needs_input".to_string(),
        WorkerStatus::NeedsReview => "needs_review".to_string(),
        WorkerStatus::Rejected => "rejected".to_string(),
        WorkerStatus::Rebasing => "rebasing".to_string(),
        WorkerStatus::Error => "error".to_string(),
        WorkerStatus::Offline => "offline".to_string(),
    }
}

fn format_status_colored(status: WorkerStatus) -> String {
    if !supports_color() {
        return format_status_json(status);
    }

    let (color_code, text) = match status {
        WorkerStatus::Idle => ("\x1b[32m", "idle"),
        WorkerStatus::Working => ("\x1b[33m", "working"),
        WorkerStatus::NeedsInput => ("\x1b[36m", "needs_input"),
        WorkerStatus::NeedsReview => ("\x1b[34m", "needs_review"),
        WorkerStatus::Rejected => ("\x1b[31m", "rejected"),
        WorkerStatus::Rebasing => ("\x1b[35m", "rebasing"),
        WorkerStatus::Error => ("\x1b[1;31m", "error"),
        WorkerStatus::Offline => ("\x1b[90m", "offline"),
    };

    format!("{}{}\x1b[0m", color_code, text)
}

fn supports_color() -> bool {
    std::env::var("TERM").map(|term| term != "dumb" && !term.is_empty()).unwrap_or(false)
        && std::io::IsTerminal::is_terminal(&std::io::stdout())
}

#[expect(clippy::integer_division)]
fn format_duration(secs: u64) -> String {
    if secs < 60 {
        return format!("{}s", secs);
    }

    let mins = secs / 60;
    if mins < 60 {
        return format!("{}m", mins);
    }

    let hours = mins / 60;
    let remaining_mins = mins % 60;
    if remaining_mins > 0 {
        format!("{}h{}m", hours, remaining_mins)
    } else {
        format!("{}h", hours)
    }
}

fn truncate_prompt(prompt: &str, max_len: usize) -> String {
    let trimmed = prompt.trim();
    if trimmed.len() <= max_len {
        return trimmed.to_string();
    }

    let truncated = &trimmed[..max_len];
    let last_space = truncated.rfind(' ').unwrap_or(max_len);
    trimmed[..last_space].trim().to_string()
}
