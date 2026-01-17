use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, Write};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};

use super::super::config::{self, Config};
use super::super::state::{self, State, WorkerRecord, WorkerStatus};
use super::super::{git, worker};
/// Result of a doctor check
#[derive(Debug, Clone)]
pub struct DoctorReport {
    pub checks_passed: Vec<String>,
    pub warnings: Vec<DoctorWarning>,
    pub errors: Vec<DoctorError>,
    pub repairs_succeeded: Vec<String>,
    pub repairs_failed: Vec<String>,
}
#[derive(Debug, Clone)]
pub struct DoctorWarning {
    pub message: String,
    pub details: Option<String>,
}
#[derive(Debug, Clone)]
pub struct DoctorError {
    pub message: String,
    pub details: Option<String>,
}
#[expect(clippy::fn_params_excessive_bools)]
pub fn run_doctor(repair: bool, yes: bool, rebuild: bool, json: bool) -> Result<()> {
    if rebuild {
        return run_rebuild();
    }
    let report = perform_health_checks(repair, yes || json)?;

    if json {
        let mut issues = Vec::new();
        for warning in &report.warnings {
            issues.push(crate::json_output::DoctorIssue {
                category: "warning".to_string(),
                description: warning.message.clone(),
                severity: "warning".to_string(),
            });
        }
        for error in &report.errors {
            issues.push(crate::json_output::DoctorIssue {
                category: "error".to_string(),
                description: error.message.clone(),
                severity: "error".to_string(),
            });
        }
        let output = crate::json_output::DoctorOutput {
            healthy: report.errors.is_empty(),
            issues,
            repairs: report.repairs_succeeded.clone(),
        };
        crate::json_output::print_json(&output);
    } else {
        display_report(&report);
        if !report.errors.is_empty() && !repair {
            println!("\nRepairs needed: {}", report.errors.len());
            println!("Run 'llmc doctor --repair' to attempt fixes.");
        }
    }

    if report.errors.is_empty() {
        Ok(())
    } else {
        bail!("Health checks failed");
    }
}
fn perform_health_checks(repair: bool, yes: bool) -> Result<DoctorReport> {
    let mut report = DoctorReport {
        checks_passed: Vec::new(),
        warnings: Vec::new(),
        errors: Vec::new(),
        repairs_succeeded: Vec::new(),
        repairs_failed: Vec::new(),
    };
    check_binaries(&mut report);
    check_config(&mut report)?;
    check_state(&mut report, repair)?;
    check_worktrees(&mut report, repair)?;
    check_sessions(&mut report, repair)?;
    check_git_config(&mut report)?;
    check_crash_flag(&mut report)?;
    check_idle_workers_clean(&mut report)?;
    check_orphaned_git_state(&mut report)?;
    check_error_workers(&mut report)?;
    if repair {
        attempt_worker_repairs(&mut report, yes)?;
    }
    Ok(report)
}
fn check_binaries(report: &mut DoctorReport) {
    let binaries = vec![("tmux", "tmux"), ("git", "git"), ("claude", "claude")];
    for (name, binary) in binaries {
        if Command::new("which").arg(binary).output().map(|o| o.status.success()).unwrap_or(false) {
            report.checks_passed.push(format!("{} binary found", name));
        } else {
            report.errors.push(DoctorError {
                message: format!("{} binary not found in PATH", name),
                details: Some(format!("Install {} to use LLMC", name)),
            });
        }
    }
}
fn check_config(report: &mut DoctorReport) -> Result<()> {
    let config_path = config::get_config_path();
    if !config_path.exists() {
        report.errors.push(DoctorError {
            message: "config.toml not found".to_string(),
            details: Some(format!("Expected at: {}", config_path.display())),
        });
        return Ok(());
    }
    match Config::load(&config_path) {
        Ok(_) => {
            report.checks_passed.push("config.toml valid".to_string());
        }
        Err(e) => {
            report.errors.push(DoctorError {
                message: "config.toml parse error".to_string(),
                details: Some(format!("{}", e)),
            });
        }
    }
    Ok(())
}
fn check_state(report: &mut DoctorReport, repair: bool) -> Result<()> {
    let state_path = state::get_state_path();
    if !state_path.exists() {
        report.errors.push(DoctorError {
            message: "state.json not found".to_string(),
            details: Some(format!("Expected at: {}", state_path.display())),
        });
        return Ok(());
    }
    let content = match fs::read_to_string(&state_path) {
        Ok(c) => c,
        Err(e) => {
            report.errors.push(DoctorError {
                message: "Failed to read state.json".to_string(),
                details: Some(format!("{}", e)),
            });
            return Ok(());
        }
    };
    let mut state: State = match serde_json::from_str(&content) {
        Ok(s) => s,
        Err(e) => {
            report.errors.push(DoctorError {
                message: "state.json parse error".to_string(),
                details: Some(format!("{}", e)),
            });
            return Ok(());
        }
    };
    if let Err(e) = state::validate_state(&state) {
        if repair {
            let repairs = attempt_state_repairs(&mut state, report);
            if repairs > 0 {
                match state.save(&state_path) {
                    Ok(_) => {
                        report.checks_passed.push("state.json valid (after repairs)".to_string());
                    }
                    Err(e) => {
                        report.errors.push(DoctorError {
                            message: "Failed to save repaired state".to_string(),
                            details: Some(format!("{}", e)),
                        });
                    }
                }
            }
        } else {
            report.errors.push(DoctorError {
                message: "state.json validation failed".to_string(),
                details: Some(format!("{}", e)),
            });
        }
    } else {
        report.checks_passed.push("state.json valid".to_string());
    }
    let worker_count = state.workers.len();
    report.checks_passed.push(format!("{} workers configured", worker_count));
    Ok(())
}
fn attempt_state_repairs(state: &mut State, report: &mut DoctorReport) -> usize {
    let mut repairs = 0;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    #[expect(clippy::iter_over_hash_type)]
    for worker in state.workers.values_mut() {
        if worker.status == WorkerStatus::NeedsReview && worker.commit_sha.is_none() {
            if let Ok(sha) = get_head_commit(&worker.worktree_path) {
                worker.commit_sha = Some(sha.clone());
                report.repairs_succeeded.push(format!(
                    "Set commit_sha for {} to {}",
                    worker.name,
                    &sha[..7.min(sha.len())]
                ));
                repairs += 1;
            } else {
                worker.status = WorkerStatus::Working;
                worker.commit_sha = None;
                report.repairs_succeeded.push(format!(
                    "Reset {} from needs_review to working (no commit found)",
                    worker.name
                ));
                repairs += 1;
            }
        }
        if worker.status == WorkerStatus::Working && worker.current_prompt.is_empty() {
            worker.status = WorkerStatus::Idle;
            report
                .repairs_succeeded
                .push(format!("Reset {} from working to idle (no prompt)", worker.name));
            repairs += 1;
        }
        if worker.created_at_unix > now {
            worker.created_at_unix = now;
            report.repairs_succeeded.push(format!("Fixed future created_at for {}", worker.name));
            repairs += 1;
        }
        if worker.last_activity_unix > now {
            worker.last_activity_unix = now;
            report
                .repairs_succeeded
                .push(format!("Fixed future last_activity for {}", worker.name));
            repairs += 1;
        }
    }
    repairs
}
fn get_head_commit(worktree_path: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["log", "-1", "--format=%H", "HEAD"])
        .current_dir(worktree_path)
        .output()
        .context("Failed to get HEAD commit")?;
    if !output.status.success() {
        bail!("git log failed");
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
fn check_worktrees(report: &mut DoctorReport, repair: bool) -> Result<()> {
    let llmc_root = config::get_llmc_root();
    let worktrees_dir = llmc_root.join(".worktrees");
    if !worktrees_dir.exists() {
        report.warnings.push(DoctorWarning {
            message: "Worktrees directory not found".to_string(),
            details: Some(format!("Expected: {}", worktrees_dir.display())),
        });
        return Ok(());
    }
    let state_path = state::get_state_path();
    if let Ok(state) = State::load(&state_path) {
        let configured_workers: HashSet<_> = state.workers.keys().collect();
        let mut found_worktrees = HashSet::new();
        if let Ok(entries) = fs::read_dir(&worktrees_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir()
                    && let Some(name) = entry.file_name().to_str()
                {
                    found_worktrees.insert(name.to_string());
                    if !configured_workers.contains(&name.to_string()) {
                        if repair {
                            match git::remove_worktree(&llmc_root, &entry.path(), true) {
                                Ok(()) => {
                                    report
                                        .repairs_succeeded
                                        .push(format!("Removed orphaned worktree: {}", name));
                                }
                                Err(e) => {
                                    report.repairs_failed.push(format!(
                                        "Failed to remove orphaned worktree '{}': {}",
                                        name, e
                                    ));
                                }
                            }
                        } else {
                            report.warnings.push(DoctorWarning {
                                message: format!("Orphaned worktree: {}", name),
                                details: Some(
                                    "Worktree exists but no worker configured".to_string(),
                                ),
                            });
                        }
                    }
                }
            }
        }
        #[expect(clippy::iter_over_hash_type)]
        for worker_name in configured_workers {
            if !found_worktrees.contains(worker_name) {
                if repair {
                    let worker = state.workers.get(worker_name).unwrap();
                    let worktree_path = std::path::Path::new(&worker.worktree_path);
                    match super::add::recreate_missing_worktree(
                        worker_name,
                        &worker.branch,
                        worktree_path,
                    ) {
                        Ok(()) => {
                            report.repairs_succeeded.push(format!(
                                "Recreated missing worktree for worker '{}'",
                                worker_name
                            ));
                        }
                        Err(e) => {
                            report.repairs_failed.push(format!(
                                "Failed to recreate worktree for worker '{}': {}",
                                worker_name, e
                            ));
                        }
                    }
                } else {
                    report.warnings.push(DoctorWarning {
                        message: format!("Worker '{}' worktree missing", worker_name),
                        details: None,
                    });
                }
            }
        }
    }
    Ok(())
}
fn check_sessions(report: &mut DoctorReport, repair: bool) -> Result<()> {
    let output = Command::new("tmux").args(["list-sessions", "-F", "#{session_name}"]).output();
    let sessions: HashSet<String> = match output {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|line| line.starts_with("llmc-"))
            .map(std::string::ToString::to_string)
            .collect(),
        _ => HashSet::new(),
    };
    let state_path = state::get_state_path();
    if let Ok(mut state) = State::load(&state_path) {
        let mut needs_save = false;
        #[expect(clippy::iter_over_hash_type)]
        for worker in state.workers.values_mut() {
            let expected_session = format!("llmc-{}", worker.name);
            if sessions.contains(&expected_session) {
                if worker.status == WorkerStatus::Offline {
                    if repair {
                        match Command::new("tmux")
                            .args(["kill-session", "-t", &expected_session])
                            .output()
                        {
                            Ok(output) if output.status.success() => {
                                report.repairs_succeeded.push(format!(
                                    "Killed orphaned session for offline worker '{}'",
                                    worker.name
                                ));
                            }
                            Ok(output) => {
                                report.repairs_failed.push(format!(
                                    "Failed to kill session {}: {}",
                                    expected_session,
                                    String::from_utf8_lossy(&output.stderr)
                                ));
                            }
                            Err(e) => {
                                report.repairs_failed.push(format!(
                                    "Failed to kill session {}: {}",
                                    expected_session, e
                                ));
                            }
                        }
                    } else {
                        report.warnings.push(DoctorWarning {
                            message: format!(
                                "Worker '{}' marked offline but session exists",
                                worker.name
                            ),
                            details: None,
                        });
                    }
                }
            } else if worker.status != WorkerStatus::Offline {
                if repair {
                    worker.status = WorkerStatus::Offline;
                    report.repairs_succeeded.push(format!(
                        "Marked worker '{}' as offline (session missing)",
                        worker.name
                    ));
                    needs_save = true;
                } else {
                    report.warnings.push(DoctorWarning {
                        message: format!("Worker '{}' session missing", worker.name),
                        details: Some(format!("Expected session: {}", expected_session)),
                    });
                }
            }
        }
        if needs_save {
            state.save(&state_path)?;
        }
        #[expect(clippy::iter_over_hash_type)]
        for session_name in &sessions {
            if let Some(worker_name) = session_name.strip_prefix("llmc-")
                && !state.workers.contains_key(worker_name)
            {
                if repair {
                    match Command::new("tmux").args(["kill-session", "-t", session_name]).output() {
                        Ok(output) if output.status.success() => {
                            report
                                .repairs_succeeded
                                .push(format!("Killed orphaned session: {}", session_name));
                        }
                        Ok(output) => {
                            report.repairs_failed.push(format!(
                                "Failed to kill session {}: {}",
                                session_name,
                                String::from_utf8_lossy(&output.stderr)
                            ));
                        }
                        Err(e) => {
                            report
                                .repairs_failed
                                .push(format!("Failed to kill session {}: {}", session_name, e));
                        }
                    }
                } else {
                    report.warnings.push(DoctorWarning {
                        message: format!("Orphaned session: {}", session_name),
                        details: Some("Session exists but no worker configured".to_string()),
                    });
                }
            }
        }
    }
    Ok(())
}
fn check_git_config(report: &mut DoctorReport) -> Result<()> {
    let llmc_root = config::get_llmc_root();
    let git_dir = llmc_root.join(".git");
    if !git_dir.exists() {
        report.errors.push(DoctorError {
            message: "Git repository not initialized".to_string(),
            details: Some(format!("Expected .git at: {}", git_dir.display())),
        });
        return Ok(());
    }
    report.checks_passed.push("Git repository exists".to_string());
    let remote_output = Command::new("git")
        .args(["-C", llmc_root.to_str().unwrap(), "remote", "get-url", "origin"])
        .output();
    match remote_output {
        Ok(output) if output.status.success() => {
            let origin_url = String::from_utf8_lossy(&output.stdout).trim().to_string();
            report.checks_passed.push(format!("Git origin configured: {}", origin_url));
            let fetch_check = Command::new("git")
                .args(["-C", llmc_root.to_str().unwrap(), "fetch", "--dry-run", "origin"])
                .output();
            if let Ok(fetch_out) = fetch_check {
                if fetch_out.status.success() {
                    report.checks_passed.push("Git origin is accessible".to_string());
                } else {
                    report.warnings.push(DoctorWarning {
                        message: "Cannot fetch from origin".to_string(),
                        details: Some(String::from_utf8_lossy(&fetch_out.stderr).to_string()),
                    });
                }
            }
        }
        Ok(_) | Err(_) => {
            report.errors.push(DoctorError {
                message: "Git origin remote not configured".to_string(),
                details: Some(
                    "Worker branches need origin to fetch updates. \
                     Run 'git remote add origin <path>' in ~/llmc"
                        .to_string(),
                ),
            });
        }
    }
    check_worker_branches(report)?;
    Ok(())
}
fn check_worker_branches(report: &mut DoctorReport) -> Result<()> {
    let state_path = state::get_state_path();
    if let Ok(state) = State::load(&state_path) {
        let llmc_root = config::get_llmc_root();
        #[expect(clippy::iter_over_hash_type)]
        for worker in state.workers.values() {
            let branch_check = Command::new("git")
                .args(["-C", llmc_root.to_str().unwrap(), "rev-parse", "--verify", &worker.branch])
                .output();
            match branch_check {
                Ok(output) if output.status.success() => {
                    let fetch_check = Command::new("git")
                        .args([
                            "-C",
                            &worker.worktree_path,
                            "fetch",
                            "--dry-run",
                            "origin",
                            "master",
                        ])
                        .output();
                    if let Ok(fetch_out) = fetch_check
                        && !fetch_out.status.success()
                    {
                        report.warnings.push(DoctorWarning {
                            message: format!(
                                "Worker '{}' cannot fetch from origin/master",
                                worker.name
                            ),
                            details: Some(String::from_utf8_lossy(&fetch_out.stderr).to_string()),
                        });
                    }
                }
                _ => {
                    report.warnings.push(DoctorWarning {
                        message: format!(
                            "Worker '{}' branch '{}' not found",
                            worker.name, worker.branch
                        ),
                        details: Some("Branch may have been deleted manually".to_string()),
                    });
                }
            }
        }
    }
    Ok(())
}
fn check_crash_flag(report: &mut DoctorReport) -> Result<()> {
    let state_path = state::get_state_path();
    if !state_path.exists() {
        return Ok(());
    }
    let state = State::load(&state_path)?;
    if state.daemon_running {
        report.warnings.push(DoctorWarning {
            message: "Previous daemon crash detected".to_string(),
            details: Some(
                "State file indicates previous daemon crash. \
                 Run 'llmc doctor --repair' to reset crash flag."
                    .to_string(),
            ),
        });
    } else {
        report.checks_passed.push("No daemon crash detected".to_string());
    }
    Ok(())
}
fn check_idle_workers_clean(report: &mut DoctorReport) -> Result<()> {
    let state_path = state::get_state_path();
    if !state_path.exists() {
        return Ok(());
    }
    let state = State::load(&state_path)?;
    let mut all_clean = true;
    let workers: Vec<_> = state.workers.values().collect();
    for worker in workers {
        if worker.status == WorkerStatus::Idle {
            let worktree_path = std::path::Path::new(&worker.worktree_path);
            if !worktree_path.exists() {
                continue;
            }
            match git::is_worktree_clean(worktree_path) {
                Ok(true) => {}
                Ok(false) => {
                    all_clean = false;
                    report.errors.push(DoctorError {
                        message: format!(
                            "Worker '{}' is marked idle but has uncommitted changes",
                            worker.name
                        ),
                        details: Some(format!(
                            "Run 'llmc reset {}' or 'llmc doctor --repair' to clean the worktree",
                            worker.name
                        )),
                    });
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to check worktree cleanliness for '{}': {}",
                        worker.name,
                        e
                    );
                }
            }
        }
    }
    if all_clean {
        report.checks_passed.push("All idle workers have clean worktrees".to_string());
    }
    Ok(())
}
fn check_orphaned_git_state(report: &mut DoctorReport) -> Result<()> {
    let state_path = state::get_state_path();
    if !state_path.exists() {
        return Ok(());
    }
    let state = State::load(&state_path)?;
    let mut all_clean = true;
    let workers: Vec<_> = state.workers.values().collect();
    for worker in workers {
        let worktree_path = std::path::Path::new(&worker.worktree_path);
        if !worktree_path.exists() {
            continue;
        }
        let has_rebase = git::is_rebase_in_progress(worktree_path);
        if has_rebase && worker.status != WorkerStatus::Rebasing {
            all_clean = false;
            report.warnings.push(DoctorWarning {
                message: format!(
                    "Worker '{}' has orphaned git rebase state (not marked as rebasing)",
                    worker.name
                ),
                details: Some(format!(
                    "Run 'llmc reset {}' or 'llmc doctor --repair' to clean up",
                    worker.name
                )),
            });
        }
    }
    if all_clean {
        report.checks_passed.push("No orphaned git rebase state found".to_string());
    }
    Ok(())
}
fn check_error_workers(report: &mut DoctorReport) -> Result<()> {
    let state_path = state::get_state_path();
    if !state_path.exists() {
        return Ok(());
    }
    let state = State::load(&state_path)?;
    let error_workers: Vec<&WorkerRecord> =
        state.workers.values().filter(|w| w.status == WorkerStatus::Error).collect();
    if error_workers.is_empty() {
        report.checks_passed.push("No workers in error state".to_string());
    } else {
        for worker in &error_workers {
            report.warnings.push(DoctorWarning {
                message: format!("Worker '{}' is in error state", worker.name),
                details: Some(format!(
                    "Run 'llmc reset {}' or 'llmc doctor --repair' to recover. \
                     Worker may have crashed or encountered inconsistent state.",
                    worker.name
                )),
            });
        }
    }
    Ok(())
}
fn attempt_worker_repairs(report: &mut DoctorReport, yes: bool) -> Result<()> {
    let state_path = state::get_state_path();
    let config_path = config::get_config_path();
    let mut state = State::load(&state_path)?;
    let config = Config::load(&config_path)?;
    if state.daemon_running {
        state.daemon_running = false;
        state.save(&state_path)?;
        report.repairs_succeeded.push("Reset daemon_running crash flag".to_string());
    }
    let mut workers_to_reset: Vec<String> = Vec::new();
    let workers: Vec<_> = state.workers.values().collect();
    for worker in workers {
        let should_reset = worker.status == WorkerStatus::Error
            || (worker.status == WorkerStatus::Idle
                && std::path::Path::new(&worker.worktree_path).exists()
                && !git::is_worktree_clean(std::path::Path::new(&worker.worktree_path))
                    .unwrap_or(true));
        if should_reset {
            workers_to_reset.push(worker.name.clone());
        }
    }
    if workers_to_reset.is_empty() {
        return Ok(());
    }
    if !yes {
        println!("\nThe following repairs will be performed:");
        for worker_name in &workers_to_reset {
            println!(
                "  - Reset worker '{}' to clean state (DISCARDS uncommitted changes)",
                worker_name
            );
        }
        print!("\nContinue with repairs? (y/N): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            println!("Repairs cancelled.");
            return Ok(());
        }
    }
    for worker_name in &workers_to_reset {
        match worker::reset_worker_to_clean_state(worker_name, &mut state, &config) {
            Ok(actions) => {
                for action in actions {
                    report.repairs_succeeded.push(action);
                }
            }
            Err(e) => {
                report
                    .repairs_failed
                    .push(format!("Failed to reset worker '{}': {}", worker_name, e));
            }
        }
    }
    state.save(&state_path)?;
    Ok(())
}
fn display_report(report: &DoctorReport) {
    println!("LLMC Doctor");
    println!("───────────\n");
    for check in &report.checks_passed {
        println!("✓ {}", check);
    }
    if !report.warnings.is_empty() {
        println!();
        for warning in &report.warnings {
            if let Some(details) = &warning.details {
                println!("⚠ {} ({})", warning.message, details);
            } else {
                println!("⚠ {}", warning.message);
            }
        }
    }
    if !report.errors.is_empty() {
        println!();
        for error in &report.errors {
            if let Some(details) = &error.details {
                println!("✗ {} ({})", error.message, details);
            } else {
                println!("✗ {}", error.message);
            }
        }
    }
    if !report.repairs_succeeded.is_empty() {
        println!("\nRepairs completed:");
        for repair in &report.repairs_succeeded {
            println!("  ✓ {}", repair);
        }
    }
    if !report.repairs_failed.is_empty() {
        println!("\nRepairs failed:");
        for repair in &report.repairs_failed {
            println!("  ✗ {}", repair);
        }
    }
}
fn run_rebuild() -> Result<()> {
    println!("Rebuilding state from filesystem...\n");
    let llmc_root = config::get_llmc_root();
    let worktrees_dir = llmc_root.join(".worktrees");
    let mut worktrees = Vec::new();
    if worktrees_dir.exists()
        && let Ok(entries) = fs::read_dir(&worktrees_dir)
    {
        for entry in entries.flatten() {
            if entry.path().is_dir()
                && let Some(name) = entry.file_name().to_str()
            {
                worktrees.push(name.to_string());
            }
        }
    }
    worktrees.sort();
    let output = Command::new("tmux").args(["list-sessions", "-F", "#{session_name}"]).output();
    let sessions: HashSet<String> = match output {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|line| line.starts_with("llmc-"))
            .map(std::string::ToString::to_string)
            .collect(),
        _ => HashSet::new(),
    };
    println!("Found worktrees: {}", worktrees.join(", "));
    let session_names: Vec<_> = sessions.iter().map(std::string::String::as_str).collect();
    println!("Found TMUX sessions: {}", session_names.join(", "));
    if worktrees.is_empty() {
        println!("\nNo worktrees found. Nothing to rebuild.");
        return Ok(());
    }
    let config_path = config::get_config_path();
    let _config = Config::load(&config_path)?;
    println!("\nReconstructed state:");
    let mut workers = HashMap::new();
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    for name in &worktrees {
        let worktree_path = worktrees_dir.join(name);
        let session_name = format!("llmc-{}", name);
        let has_session = sessions.contains(&session_name);
        let status = if has_session { WorkerStatus::Idle } else { WorkerStatus::Offline };
        let worktree_check = if worktree_path.exists() { "✓" } else { "✗" };
        let session_check = if has_session { "✓" } else { "✗" };
        let status_str = if has_session { "idle" } else { "offline" };
        println!(
            "  {}: worktree {}, session {} -> {}",
            name, worktree_check, session_check, status_str
        );
        let branch = format!("llmc/{}", name);
        let worker = WorkerRecord {
            name: name.clone(),
            worktree_path: worktree_path.to_string_lossy().to_string(),
            branch,
            status,
            current_prompt: String::new(),
            prompt_cmd: None,
            created_at_unix: now,
            last_activity_unix: now,
            commit_sha: None,
            session_id: session_name,
            crash_count: 0,
            last_crash_unix: None,
            on_complete_sent_unix: None,
            self_review: false,
        };
        workers.insert(name.clone(), worker);
    }
    print!("\nSave reconstructed state? [y/N]: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    if input.trim().to_lowercase() == "y" {
        let state = State { workers, daemon_running: false };
        let state_path = state::get_state_path();
        state.save(&state_path)?;
        println!("State saved to {}", state_path.display());
    } else {
        println!("Rebuild cancelled.");
    }
    Ok(())
}
