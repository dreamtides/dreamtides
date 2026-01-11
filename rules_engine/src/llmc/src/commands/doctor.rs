use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, Write};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};

use super::super::config::{self, Config};
use super::super::state::{self, State, WorkerRecord, WorkerStatus};

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

pub fn run_doctor(repair: bool, rebuild: bool) -> Result<()> {
    if rebuild {
        return run_rebuild();
    }

    let report = perform_health_checks(repair)?;
    display_report(&report);

    if !report.errors.is_empty() && !repair {
        println!("\nRepairs needed: {}", report.errors.len());
        println!("Run 'llmc doctor --repair' to attempt fixes.");
    }

    if report.errors.is_empty() {
        Ok(())
    } else {
        bail!("Health checks failed");
    }
}

fn perform_health_checks(repair: bool) -> Result<DoctorReport> {
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
    check_worktrees(&mut report);
    check_sessions(&mut report, repair)?;
    check_git_config(&mut report)?;

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
                worker.status = WorkerStatus::NeedsInput;
                worker.commit_sha = None;
                report.repairs_succeeded.push(format!(
                    "Reset {} from needs_review to needs_input (no commit found)",
                    worker.name
                ));
                repairs += 1;
            }
        }

        if worker.status == WorkerStatus::Working && worker.current_prompt.is_empty() {
            worker.status = WorkerStatus::NeedsInput;
            report
                .repairs_succeeded
                .push(format!("Reset {} from working to needs_input (no prompt)", worker.name));
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

fn check_worktrees(report: &mut DoctorReport) {
    let llmc_root = config::get_llmc_root();
    let worktrees_dir = llmc_root.join(".worktrees");

    if !worktrees_dir.exists() {
        report.warnings.push(DoctorWarning {
            message: "Worktrees directory not found".to_string(),
            details: Some(format!("Expected: {}", worktrees_dir.display())),
        });
        return;
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
                        report.warnings.push(DoctorWarning {
                            message: format!("Orphaned worktree: {}", name),
                            details: Some("Worktree exists but no worker configured".to_string()),
                        });
                    }
                }
            }
        }

        #[expect(clippy::iter_over_hash_type)]
        for worker_name in configured_workers {
            if !found_worktrees.contains(worker_name) {
                report.warnings.push(DoctorWarning {
                    message: format!("Worker '{}' worktree missing", worker_name),
                    details: None,
                });
            }
        }
    }
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
                    report.warnings.push(DoctorWarning {
                        message: format!(
                            "Worker '{}' marked offline but session exists",
                            worker.name
                        ),
                        details: None,
                    });
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
                report.warnings.push(DoctorWarning {
                    message: format!("Orphaned session: {}", session_name),
                    details: Some("Session exists but no worker configured".to_string()),
                });
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

    // Check that origin remote exists and is accessible
    let remote_output = Command::new("git")
        .args(["-C", llmc_root.to_str().unwrap(), "remote", "get-url", "origin"])
        .output();

    match remote_output {
        Ok(output) if output.status.success() => {
            let origin_url = String::from_utf8_lossy(&output.stdout).trim().to_string();
            report.checks_passed.push(format!("Git origin configured: {}", origin_url));

            // Check if origin is accessible
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

    // Check worker branches can rebase
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
                    // Branch exists, check if it can fetch from origin
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
            created_at_unix: now,
            last_activity_unix: now,
            commit_sha: None,
            session_id: session_name,
            crash_count: 0,
            last_crash_unix: None,
        };

        workers.insert(name.clone(), worker);
    }

    print!("\nSave reconstructed state? [y/N]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() == "y" {
        let state = State { workers };
        let state_path = state::get_state_path();
        state.save(&state_path)?;
        println!("State saved to {}", state_path.display());
    } else {
        println!("Rebuild cancelled.");
    }

    Ok(())
}
