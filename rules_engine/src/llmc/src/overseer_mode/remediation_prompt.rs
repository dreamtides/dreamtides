//! Remediation prompt construction for overseer mode.
//!
//! This module builds the prompt sent to the overseer's Claude Code session
//! when a daemon failure is detected. The prompt includes user-configured
//! instructions, structured error context, and recovery instructions.

#![allow(dead_code)]

use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom};
use std::path::Path;
use std::process::Command;

use chrono::{TimeZone, Utc};
use tracing::{debug, info};

use crate::auto_mode::{auto_logging, heartbeat_thread};
use crate::config::Config;
use crate::overseer_mode::health_monitor::HealthStatus;
use crate::state::{self, State};
/// Default number of log lines to include in remediation prompts.
const DEFAULT_LOG_LINES: usize = 100;
/// Maximum number of characters per log file section to prevent huge prompts.
const MAX_LOG_SECTION_CHARS: usize = 50_000;
/// Builds the remediation prompt for the overseer's Claude Code session.
///
/// The prompt is structured in three sections:
/// 1. User-configured remediation instructions from TOML config
/// 2. Structured error context (failure type, daemon info, logs, state)
/// 3. Recovery instructions
pub fn build_remediation_prompt(failure: &HealthStatus, config: &Config) -> String {
    [
        build_user_instructions_section(config),
        build_error_context_section(failure, config),
        build_recovery_instructions_section(),
    ]
    .join("\n\n")
}
/// Builds the user instructions section from the TOML config.
fn build_user_instructions_section(config: &Config) -> String {
    let mut section = String::from("# Remediation Instructions\n\n");
    if let Some(ref overseer_config) = config.overseer {
        if let Some(ref prompt) = overseer_config.remediation_prompt {
            section.push_str(prompt);
        } else {
            section.push_str("(No custom remediation instructions configured)");
        }
    } else {
        section.push_str("(No overseer configuration found)");
    }
    section
}
/// Builds the structured error context section.
fn build_error_context_section(failure: &HealthStatus, config: &Config) -> String {
    let mut section = String::from("# Error Context\n\n");
    section.push_str(&format_failure_type(failure));
    section.push_str("\n\n");
    section.push_str(&format_task_error_context());
    section.push_str(&format_daemon_registration());
    section.push_str("\n\n");
    section.push_str(&format_worker_states());
    section.push_str("\n\n");
    section.push_str(&format_git_status(config));
    section.push_str("\n\n");
    section.push_str(&format_log_excerpts());
    section
}
/// Formats the failure type with details.
fn format_failure_type(failure: &HealthStatus) -> String {
    let mut result = String::from("## Failure Type\n\n");
    match failure {
        HealthStatus::Healthy => {
            result.push_str("Status: Healthy (unexpected remediation trigger)\n");
        }
        HealthStatus::ProcessGone => {
            result.push_str("Status: **Process Gone**\n\n");
            result.push_str(
                "The daemon process is no longer running. The PID no longer exists.\n\
                 Possible causes:\n\
                 - Daemon crashed due to panic or unhandled error\n\
                 - Process was killed externally (OOM killer, signal)\n\
                 - System resource exhaustion",
            );
        }
        HealthStatus::HeartbeatStale { age_secs } => {
            result.push_str("Status: **Heartbeat Stale**\n\n");
            result.push_str(&format!(
                "The daemon's heartbeat file has not been updated for {} seconds.\n\
                 The heartbeat should update every 5 seconds.\n\n\
                 Possible causes:\n\
                 - Daemon is hung/deadlocked\n\
                 - Daemon crashed but PID file still exists\n\
                 - Heartbeat thread crashed while main thread continues\n\
                 - Filesystem issues preventing heartbeat writes",
                age_secs
            ));
        }
        HealthStatus::LogError { message } => {
            result.push_str("Status: **Error in Logs**\n\n");
            result.push_str(&format!(
                "An ERROR level log entry was detected:\n\n```\n{}\n```\n\n\
                 The daemon shuts down on errors to allow investigation.\n\
                 Review the full logs below for context.",
                message
            ));
        }
        HealthStatus::Stalled { stall_secs } => {
            result.push_str("Status: **Stalled Progress**\n\n");
            result.push_str(&format!(
                "No task completions detected for {} seconds.\n\n\
                 Possible causes:\n\
                 - Worker(s) stuck on a difficult task\n\
                 - Worker(s) in infinite loop\n\
                 - Task pool returning no tasks for extended period\n\
                 - Accept workflow blocked (e.g., merge conflicts)",
                stall_secs
            ));
        }
        HealthStatus::IdentityMismatch { reason } => {
            result.push_str("Status: **Identity Mismatch**\n\n");
            result.push_str(&format!(
                "The daemon's identity does not match expected values:\n{}\n\n\
                 This indicates the daemon restarted unexpectedly or a different\n\
                 process has taken the same PID (PID reuse after crash).",
                reason
            ));
        }
    }
    result
}

/// Formats task error context if available.
///
/// This function reads persisted task error information that was saved when a
/// task-related error caused daemon shutdown. The context includes details
/// about what went wrong and suggested remediation steps.
fn format_task_error_context() -> String {
    let Some(ctx) = auto_logging::read_task_error_context() else {
        return String::new();
    };
    let mut result = String::from("## Task Error Details\n\n");
    result.push_str(&format!("**Error Type:** {}\n\n", ctx.error_type));
    result.push_str(&format!("**Error Message:**\n```\n{}\n```\n\n", ctx.error_message));
    if let Some(ref path) = ctx.file_path {
        result.push_str(&format!("**File Path:** `{}`\n\n", path));
    }
    if let Some(ref task_id) = ctx.task_id {
        result.push_str(&format!("**Task ID:** {}\n\n", task_id));
    }
    if let Some(ref content) = ctx.raw_content {
        result.push_str("**File Content (may be malformed):**\n");
        result.push_str(&format!("```json\n{}\n```\n\n", truncate_string(content, 2000)));
    }
    result.push_str(&format!("**Suggested Remediation:**\n{}\n\n", ctx.remediation_hint));
    result.push_str("**Timestamp:** ");
    result.push_str(&ctx.timestamp);
    result.push_str("\n\n");
    result
}

/// Formats the daemon registration information.
fn format_daemon_registration() -> String {
    let mut result = String::from("## Daemon Registration\n\n");
    match heartbeat_thread::read_daemon_registration() {
        Some(reg) => {
            let start_time = Utc.timestamp_opt(reg.start_time_unix as i64, 0);
            let start_str = match start_time {
                chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                _ => format!("{}s since epoch", reg.start_time_unix),
            };
            result.push_str(&format!(
                "- **PID:** {}\n\
                 - **Start Time:** {}\n\
                 - **Instance ID:** {}\n\
                 - **Log File:** {}",
                reg.pid, start_str, reg.instance_id, reg.log_file
            ));
        }
        None => {
            result.push_str("(No daemon registration found - file may have been deleted)");
        }
    }
    result
}
/// Formats the current worker states from state.json.
fn format_worker_states() -> String {
    let mut result = String::from("## Worker States\n\n");
    let state_path = state::get_state_path();
    match State::load(&state_path) {
        Ok(state) => {
            if state.workers.is_empty() {
                result.push_str("(No workers found)");
            } else {
                let mut worker_names: Vec<_> = state.workers.keys().collect();
                worker_names.sort();
                for name in worker_names {
                    let worker = &state.workers[name];
                    result.push_str(&format!("### Worker: {}\n", name));
                    result.push_str(&format!("- Status: {:?}\n", worker.status));
                    result.push_str(&format!("- Worktree: {}\n", worker.worktree_path));
                    result.push_str(&format!("- Branch: {}\n", worker.branch));
                    if !worker.current_prompt.is_empty() {
                        let truncated = truncate_string(&worker.current_prompt, 500);
                        result.push_str(&format!("- Current Prompt: {}\n", truncated));
                    }
                    if let Some(ref sha) = worker.commit_sha {
                        result.push_str(&format!("- Commit SHA: {}\n", sha));
                    }
                    if let Some(ref error) = worker.error_reason {
                        result.push_str(&format!("- Error Reason: {}\n", error));
                    }
                    result.push_str(&format!("- Crash Count: {}\n", worker.crash_count));
                    result.push_str(&format!("- Auto Retry Count: {}\n", worker.auto_retry_count));
                    result.push('\n');
                }
            }
            if let Some(unix) = state.last_task_completion_unix {
                let time = Utc.timestamp_opt(unix as i64, 0);
                let time_str = match time {
                    chrono::LocalResult::Single(dt) => {
                        dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
                    }
                    _ => format!("{}s since epoch", unix),
                };
                result.push_str(&format!("**Last Task Completion:** {}\n", time_str));
            }
        }
        Err(e) => {
            result.push_str(&format!("(Failed to load state: {})", e));
        }
    }
    result
}
/// Formats the git status of the main repository.
fn format_git_status(config: &Config) -> String {
    let mut result = String::from("## Git Status (Source Repository)\n\n");
    let repo_path = &config.repo.source;
    result.push_str(&format!("Repository: {}\n\n", repo_path));
    match run_git_status(repo_path) {
        Ok(status) => {
            if status.is_empty() {
                result.push_str("```\n(Clean working tree)\n```");
            } else {
                result.push_str(&format!("```\n{}\n```", truncate_string(&status, 5000)));
            }
        }
        Err(e) => {
            result.push_str(&format!("(Failed to get git status: {})", e));
        }
    }
    result
}
/// Runs `git status --short` in the given repository.
fn run_git_status(repo_path: &str) -> Result<String, String> {
    let output = Command::new("git")
        .args(["-C", repo_path, "status", "--short"])
        .output()
        .map_err(|e| format!("Failed to execute git status: {}", e))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("git status failed: {}", stderr.trim()))
    }
}
/// Formats log excerpts from all relevant log files.
fn format_log_excerpts() -> String {
    let mut result = String::from("## Log Excerpts\n\n");
    let daemon_log = auto_logging::auto_log_path();
    result.push_str("### Daemon Log (auto.log)\n\n");
    result.push_str(&format_log_file(&daemon_log, DEFAULT_LOG_LINES));
    result.push_str("\n\n");
    let task_discovery_log = auto_logging::task_discovery_log_path();
    result.push_str("### Task Discovery Log (task_discovery.log)\n\n");
    result.push_str(&format_log_file(&task_discovery_log, DEFAULT_LOG_LINES));
    result.push_str("\n\n");
    let post_accept_log = auto_logging::post_accept_log_path();
    result.push_str("### Post Accept Log (post_accept.log)\n\n");
    result.push_str(&format_log_file(&post_accept_log, DEFAULT_LOG_LINES));
    result
}
/// Reads the last N lines from a log file.
fn format_log_file(path: &Path, max_lines: usize) -> String {
    if !path.exists() {
        return format!("(File not found: {})", path.display());
    }
    match read_last_n_lines(path, max_lines) {
        Ok(lines) => {
            if lines.is_empty() {
                "(Empty file)".to_string()
            } else {
                let content = lines.join("\n");
                let truncated = truncate_string(&content, MAX_LOG_SECTION_CHARS);
                format!("```\n{}\n```", truncated)
            }
        }
        Err(e) => {
            info!(path = % path.display(), error = % e, "Failed to read log file");
            format!("(Failed to read: {})", e)
        }
    }
}
/// Reads the last N lines from a file.
///
/// Uses a reverse reading approach for efficiency with large files.
fn read_last_n_lines(path: &Path, n: usize) -> Result<Vec<String>, std::io::Error> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    let file_size = metadata.len();
    if file_size == 0 {
        return Ok(Vec::new());
    }
    let chunk_size: u64 = 8192;
    let mut reader = BufReader::new(file);
    let mut lines = Vec::new();
    let mut position = file_size;
    let mut remainder = String::new();
    while lines.len() < n && position > 0 {
        let read_size = chunk_size.min(position);
        position = position.saturating_sub(read_size);
        reader.seek(SeekFrom::Start(position))?;
        let mut buffer = vec![0u8; read_size as usize];
        std::io::Read::read_exact(&mut reader, &mut buffer)?;
        let chunk = String::from_utf8_lossy(&buffer);
        let combined = format!("{}{}", chunk, remainder);
        let mut chunk_lines: Vec<&str> = combined.lines().collect();
        if position > 0 && !chunk_lines.is_empty() {
            remainder = chunk_lines.remove(0).to_string();
        } else {
            remainder.clear();
        }
        for line in chunk_lines.into_iter().rev() {
            if lines.len() >= n {
                break;
            }
            lines.push(line.to_string());
        }
    }
    if !remainder.is_empty() && lines.len() < n {
        lines.push(remainder);
    }
    lines.reverse();
    if lines.len() > n {
        lines = lines.split_off(lines.len() - n);
    }
    debug!(path = % path.display(), line_count = lines.len(), "Read log file tail");
    Ok(lines)
}
/// Builds the recovery instructions section.
fn build_recovery_instructions_section() -> String {
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    format!(
        "# Recovery Instructions\n\n\
         After investigating and fixing the issue:\n\n\
         1. **If the issue is fixed:** Exit normally (Ctrl+C or complete your work). \
            The overseer will automatically restart the daemon.\n\n\
         2. **If the issue cannot be fixed automatically:** Create a file at:\n   \
            `~/.llmc/manual_intervention_needed_{}.txt`\n   \
            with an explanation of what went wrong and what manual steps are needed.\n   \
            The overseer will detect this file and terminate, alerting the operator.\n\n\
         **Important:**\n\
         - Review all log files for the root cause\n\
         - Check worker states and worktree git status\n\
         - Verify the task pool command is working correctly\n\
         - Consider whether this is a transient or persistent issue",
        timestamp
    )
}
/// Truncates a string to the given maximum length, adding an ellipsis if
/// truncated.
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let truncate_at = s.char_indices().take(max_len).last().map(|(i, _)| i).unwrap_or(max_len);
        format!("{}... (truncated)", &s[..truncate_at])
    }
}
