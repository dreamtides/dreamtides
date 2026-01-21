use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Result, bail};
use serde::Serialize;

use crate::auto_mode::heartbeat_thread;
use crate::commands::console::CONSOLE_PREFIX;
use crate::commands::{console, overseer};
use crate::config::{self, Config};
use crate::state::{self, State, WorkerRecord, WorkerStatus};
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
    let (state, config) = match state::load_state_with_patrol() {
        Ok(result) => result,
        Err(e) => {
            eprintln!("\x1b[33mWarning: Patrol failed, showing raw state: {}\x1b[0m", e);
            let state = State::load(&state::get_state_path())?;
            let config = Config::load(&config::get_config_path())?;
            (state, config)
        }
    };
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
        output_json(&state, &config, now)?;
    } else {
        output_text(&state, &config, now);
    }
    Ok(())
}
/// Returns the effective status for display purposes.
/// Workers in `NeedsReview` that haven't received their self-review prompt yet
/// are shown as `Reviewing` since they're still in the self-review phase.
fn get_effective_status(worker: &WorkerRecord, config: &Config) -> WorkerStatus {
    if worker.status == WorkerStatus::NeedsReview && !state::is_truly_needs_review(worker, config) {
        WorkerStatus::Reviewing
    } else {
        worker.status
    }
}
#[derive(Serialize)]
struct StatusOutput {
    workers: Vec<WorkerStatusOutput>,
    consoles: Vec<ConsoleStatusOutput>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    auto_workers: Vec<WorkerStatusOutput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    auto_mode_summary: Option<AutoModeSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    overseer: Option<OverseerStatus>,
}

#[derive(Serialize)]
struct AutoModeSummary {
    active: bool,
    worker_count: usize,
    tasks_completed: u64,
}

#[derive(Serialize)]
struct OverseerStatus {
    active: bool,
    pid: u32,
    uptime_secs: u64,
}

#[derive(Serialize)]
struct DaemonStatus {
    active: bool,
    pid: u32,
    uptime_secs: u64,
}
#[derive(Serialize)]
struct ConsoleStatusOutput {
    name: String,
    session_id: String,
}
#[derive(Serialize)]
struct WorkerStatusOutput {
    name: String,
    status: String,
    branch: String,
    time_in_state_secs: u64,
    commit_sha: Option<String>,
    prompt_cmd: Option<String>,
    prompt_excerpt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_reason: Option<String>,
}
fn output_json(state: &State, config: &Config, now: u64) -> Result<()> {
    let auto_worker_names: std::collections::HashSet<&String> = state.auto_workers.iter().collect();
    let (auto_workers, regular_workers): (Vec<_>, Vec<_>) =
        state.workers.values().partition(|w| auto_worker_names.contains(&w.name));
    let workers: Vec<WorkerStatusOutput> =
        regular_workers.into_iter().map(|w| worker_to_output(w, config, now)).collect();
    let auto_workers: Vec<WorkerStatusOutput> =
        auto_workers.into_iter().map(|w| worker_to_output(w, config, now)).collect();
    let consoles: Vec<ConsoleStatusOutput> = console::list_console_sessions()
        .unwrap_or_default()
        .into_iter()
        .map(|session_id| {
            let name = session_id.strip_prefix(CONSOLE_PREFIX).unwrap_or(&session_id).to_string();
            ConsoleStatusOutput { name, session_id }
        })
        .collect();
    let auto_mode_summary = if state.auto_mode {
        Some(AutoModeSummary {
            active: true,
            worker_count: auto_workers.len(),
            tasks_completed: state.last_task_completion_unix.map(|_| 0).unwrap_or(0),
        })
    } else {
        None
    };
    let overseer = get_overseer_status(now);
    let output = StatusOutput { workers, consoles, auto_workers, auto_mode_summary, overseer };
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn worker_to_output(w: &WorkerRecord, config: &Config, now: u64) -> WorkerStatusOutput {
    let effective_status = get_effective_status(w, config);
    WorkerStatusOutput {
        name: w.name.clone(),
        status: format_status_json(effective_status),
        branch: w.branch.clone(),
        time_in_state_secs: now.saturating_sub(w.last_activity_unix),
        commit_sha: w.commit_sha.clone(),
        prompt_cmd: w.prompt_cmd.clone(),
        prompt_excerpt: if w.current_prompt.is_empty() {
            None
        } else {
            Some(truncate_prompt(&w.current_prompt, 50))
        },
        error_reason: w.error_reason.clone(),
    }
}

fn get_overseer_status(now: u64) -> Option<OverseerStatus> {
    let registration = overseer::read_overseer_registration().ok()??;
    if !is_process_alive(registration.pid) {
        return None;
    }
    Some(OverseerStatus {
        active: true,
        pid: registration.pid,
        uptime_secs: now.saturating_sub(registration.start_time_unix),
    })
}

fn get_daemon_status(now: u64) -> Option<DaemonStatus> {
    let registration = heartbeat_thread::read_daemon_registration()?;
    if !is_process_alive(registration.pid) {
        return None;
    }
    Some(DaemonStatus {
        active: true,
        pid: registration.pid,
        uptime_secs: now.saturating_sub(registration.start_time_unix),
    })
}

fn is_process_alive(pid: u32) -> bool {
    std::process::Command::new("kill")
        .args(["-0", &pid.to_string()])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
fn output_text(state: &State, config: &Config, now: u64) {
    let auto_worker_names: std::collections::HashSet<&String> = state.auto_workers.iter().collect();
    let (auto_workers, regular_workers): (Vec<_>, Vec<_>) =
        state.workers.values().partition(|w| auto_worker_names.contains(&w.name));
    print_overseer_section(now);
    print_daemon_section(now);
    if !regular_workers.is_empty() {
        println!("WORKERS");
        println!("───────");
        let mut workers = regular_workers;
        workers.sort_by(|a, b| a.name.cmp(&b.name));
        for worker in workers {
            print_worker_line(worker, config, now);
        }
    }
    if state.auto_mode && !auto_workers.is_empty() {
        println!();
        println!("AUTO WORKERS");
        println!("────────────");
        let mut workers = auto_workers;
        workers.sort_by(|a, b| a.name.cmp(&b.name));
        for worker in workers {
            print_worker_line(worker, config, now);
        }
    }
    if let Ok(consoles) = console::list_console_sessions()
        && !consoles.is_empty()
    {
        println!();
        println!("CONSOLES");
        println!("────────");
        let mut consoles = consoles;
        consoles.sort();
        for session_id in consoles {
            let name = session_id.strip_prefix(CONSOLE_PREFIX).unwrap_or(&session_id);
            println!("console{:<12} active", name);
        }
    }
    println!();
    print_summary(state, config, &auto_worker_names);
}

fn print_overseer_section(now: u64) {
    if let Some(status) = get_overseer_status(now) {
        println!("OVERSEER");
        println!("────────");
        println!(
            "  Status: {}  PID: {}  Uptime: {}",
            format_active_colored(),
            status.pid,
            format_duration(status.uptime_secs)
        );
        println!();
    }
}

fn print_daemon_section(now: u64) {
    if let Some(status) = get_daemon_status(now) {
        println!("DAEMON");
        println!("──────");
        println!(
            "  Status: {}  PID: {}  Uptime: {}",
            format_active_colored(),
            status.pid,
            format_duration(status.uptime_secs)
        );
        println!();
    }
}

fn print_worker_line(worker: &WorkerRecord, config: &Config, now: u64) {
    let effective_status = get_effective_status(worker, config);
    let status_str = format_status_colored(effective_status);
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
    if let Some(cmd) = &worker.prompt_cmd {
        parts.push(format!("({})", cmd));
    }
    if effective_status == WorkerStatus::Error
        && let Some(reason) = &worker.error_reason
    {
        parts.push(format!("({})", reason));
    }
    if !worker.current_prompt.is_empty() && effective_status != WorkerStatus::Idle {
        let excerpt = truncate_prompt(&worker.current_prompt, 50);
        parts.push(format!("\"{}...\"", excerpt));
    }
    println!("{}", parts.join(" "));
}

fn format_active_colored() -> String {
    if supports_color() { "\x1b[32mrunning\x1b[0m".to_string() } else { "running".to_string() }
}
fn print_summary(
    state: &State,
    config: &Config,
    auto_worker_names: &std::collections::HashSet<&String>,
) {
    let (auto_workers, regular_workers): (Vec<_>, Vec<_>) =
        state.workers.values().partition(|w| auto_worker_names.contains(&w.name));
    let mut status_counts: HashMap<WorkerStatus, usize> = HashMap::new();
    for worker in &regular_workers {
        let effective_status = get_effective_status(worker, config);
        *status_counts.entry(effective_status).or_insert(0) += 1;
    }
    let status_parts: Vec<String> = status_counts
        .iter()
        .map(|(status, count)| format!("{} {}", count, format_status_json(*status)))
        .collect();
    println!("{} workers: {}", regular_workers.len(), status_parts.join(", "));
    if state.auto_mode {
        let mut auto_status_counts: HashMap<WorkerStatus, usize> = HashMap::new();
        for worker in &auto_workers {
            let effective_status = get_effective_status(worker, config);
            *auto_status_counts.entry(effective_status).or_insert(0) += 1;
        }
        let auto_status_parts: Vec<String> = auto_status_counts
            .iter()
            .map(|(status, count)| format!("{} {}", count, format_status_json(*status)))
            .collect();
        println!("Auto Mode: {} workers ({})", auto_workers.len(), auto_status_parts.join(", "));
    }
}
fn format_status_json(status: WorkerStatus) -> String {
    match status {
        WorkerStatus::Idle => "idle".to_string(),
        WorkerStatus::Working => "working".to_string(),
        WorkerStatus::NeedsReview => "needs_review".to_string(),
        WorkerStatus::Reviewing => "reviewing".to_string(),
        WorkerStatus::Rejected => "rejected".to_string(),
        WorkerStatus::Rebasing => "rebasing".to_string(),
        WorkerStatus::Error => "error".to_string(),
        WorkerStatus::Offline => "offline".to_string(),
        WorkerStatus::NoChanges => "no_changes".to_string(),
    }
}
fn format_status_colored(status: WorkerStatus) -> String {
    if !supports_color() {
        return format_status_json(status);
    }
    let (color_code, text) = match status {
        WorkerStatus::Idle => ("\x1b[32m", "idle"),
        WorkerStatus::Working => ("\x1b[33m", "working"),
        WorkerStatus::NeedsReview => ("\x1b[34m", "needs_review"),
        WorkerStatus::Reviewing => ("\x1b[36m", "reviewing"),
        WorkerStatus::Rejected => ("\x1b[31m", "rejected"),
        WorkerStatus::Rebasing => ("\x1b[35m", "rebasing"),
        WorkerStatus::Error => ("\x1b[1;31m", "error"),
        WorkerStatus::Offline => ("\x1b[90m", "offline"),
        WorkerStatus::NoChanges => ("\x1b[93m", "no_changes"),
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
