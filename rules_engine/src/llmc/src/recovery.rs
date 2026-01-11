#![allow(dead_code)]

use std::fs::{self, OpenOptions};
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::state::{State, WorkerRecord, WorkerStatus};
use crate::tmux::monitor::{ClaudeState, StateDetector};
use crate::tmux::sender::TmuxSender;
use crate::tmux::session;
use crate::worker::Worker;

/// Recovery manager for handling various failure modes
pub struct RecoveryManager {
    logs_dir: PathBuf,
    state_detector: StateDetector,
}

/// Types of recovery actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryAction {
    LostInput { attempt: u32, method: String },
    SessionCrash { crash_type: String },
    StuckWorker { action: String },
    PartialSend { attempt: u32 },
    StateCorruption { action: String },
}

/// Log entry for recovery actions
#[derive(Debug, Serialize, Deserialize)]
pub struct RecoveryLogEntry {
    pub timestamp: DateTime<Utc>,
    pub worker: String,
    pub action: RecoveryAction,
    pub context: String,
    pub pane_output: Option<String>,
    pub git_status: Option<String>,
    pub success: bool,
}

/// Crash classification
#[derive(Debug, Clone)]
pub enum CrashType {
    UserExit,
    RateLimit,
    Fatal,
    Unknown,
}

/// Result of input verification
pub enum InputResult {
    Received,
    Lost,
}

/// Reset crash count if appropriate
pub fn should_reset_crash_count(worker: &WorkerRecord, now_unix: u64) -> bool {
    if let Some(last_crash) = worker.last_crash_unix {
        let time_since_crash = now_unix.saturating_sub(last_crash);
        time_since_crash >= 24 * 60 * 60
    } else {
        false
    }
}

impl RecoveryManager {
    pub fn new(logs_dir: PathBuf) -> Self {
        Self { logs_dir, state_detector: StateDetector::new(TmuxSender::default()) }
    }

    /// Handle lost input recovery with retry logic
    pub fn handle_lost_input(&self, worker: &mut Worker, message: &str) -> Result<()> {
        tracing::warn!(
            worker = %worker.name,
            session = %worker.session_id,
            message_len = message.len(),
            "Lost input detected, initiating recovery"
        );

        for attempt in 1..=3 {
            let _method = match attempt {
                1 => {
                    tracing::info!(
                        worker = %worker.name,
                        attempt,
                        method = "increased_debounce",
                        extra_ms = 200,
                        "Retrying with increased debounce"
                    );
                    self.log_recovery_action(
                        &worker.name,
                        RecoveryAction::LostInput {
                            attempt,
                            method: "increased_debounce".to_string(),
                        },
                        "Retry with increased debounce",
                        false,
                    )?;

                    if self.retry_with_increased_debounce(worker, message, 200)? {
                        tracing::info!(
                            worker = %worker.name,
                            attempt,
                            "Lost input recovery successful"
                        );
                        self.mark_recovery_success(&worker.name, attempt)?;
                        return Ok(());
                    }
                    "increased_debounce"
                }
                2 => {
                    tracing::info!(
                        worker = %worker.name,
                        attempt,
                        method = "load_buffer",
                        "Retrying via load-buffer method"
                    );
                    self.log_recovery_action(
                        &worker.name,
                        RecoveryAction::LostInput { attempt, method: "load_buffer".to_string() },
                        "Retry with load-buffer",
                        false,
                    )?;

                    if self.retry_with_load_buffer(worker, message)? {
                        tracing::info!(
                            worker = %worker.name,
                            attempt,
                            "Lost input recovery successful"
                        );
                        self.mark_recovery_success(&worker.name, attempt)?;
                        return Ok(());
                    }
                    "load_buffer"
                }
                3 => {
                    tracing::info!(
                        worker = %worker.name,
                        attempt,
                        method = "respawn",
                        "Respawning Claude and retrying"
                    );
                    self.log_recovery_action(
                        &worker.name,
                        RecoveryAction::LostInput { attempt, method: "respawn".to_string() },
                        "Respawn Claude and retry",
                        false,
                    )?;

                    if self.respawn_and_retry(worker, message)? {
                        tracing::info!(
                            worker = %worker.name,
                            attempt,
                            "Lost input recovery successful after respawn"
                        );
                        self.mark_recovery_success(&worker.name, attempt)?;
                        return Ok(());
                    }
                    "respawn"
                }
                _ => unreachable!(),
            };

            thread::sleep(Duration::from_secs(2_u64.pow(attempt)));
        }

        self.escalate_to_user(
            &worker.name,
            "Lost input after 3 retry attempts",
            message,
            &worker.session_id,
        )?;
        bail!("Failed to recover from lost input after 3 attempts");
    }

    fn retry_with_increased_debounce(
        &self,
        worker: &mut Worker,
        message: &str,
        extra_ms: u64,
    ) -> Result<bool> {
        let custom_sender = TmuxSender::with_timing(500 + extra_ms as u32, 100, 2000, 3, 200);

        custom_sender
            .send(&worker.session_id, message)
            .with_context(|| format!("Failed to send with increased debounce (+{}ms)", extra_ms))?;

        self.verify_input_received(&worker.session_id, Duration::from_secs(10))
    }

    fn retry_with_load_buffer(&self, worker: &mut Worker, message: &str) -> Result<bool> {
        worker
            .sender
            .send_large_message(&worker.session_id, message)
            .context("Failed to send via load-buffer method")?;

        self.verify_input_received(&worker.session_id, Duration::from_secs(10))
    }

    fn respawn_and_retry(&self, worker: &mut Worker, message: &str) -> Result<bool> {
        self.respawn_claude(&worker.session_id)?;

        thread::sleep(Duration::from_secs(2));

        worker.sender.send(&worker.session_id, "/clear")?;
        thread::sleep(Duration::from_millis(500));

        worker.sender.send(&worker.session_id, message)?;

        self.verify_input_received(&worker.session_id, Duration::from_secs(10))
    }

    fn verify_input_received(&self, session_id: &str, timeout: Duration) -> Result<bool> {
        let deadline = SystemTime::now() + timeout;
        let initial_state = self.state_detector.detect(session_id)?;

        while SystemTime::now() < deadline {
            thread::sleep(Duration::from_millis(500));
            let current_state = self.state_detector.detect(session_id)?;

            if initial_state == ClaudeState::Ready && current_state == ClaudeState::Processing {
                return Ok(true);
            }

            if self.pane_content_changed_significantly(session_id)? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn pane_content_changed_significantly(&self, session_id: &str) -> Result<bool> {
        let before = session::capture_pane(session_id, 20)?;

        thread::sleep(Duration::from_millis(200));

        let after = session::capture_pane(session_id, 20)?;

        Ok(before != after && after.len() > before.len())
    }

    /// Handle session crash recovery
    pub fn handle_session_crash(&self, worker: &mut WorkerRecord) -> Result<()> {
        tracing::warn!(
            worker = %worker.name,
            session = %worker.session_id,
            status = ?worker.status,
            "Session crash detected, initiating recovery"
        );

        let crash_type = self.classify_crash(&worker.session_id)?;

        match crash_type {
            CrashType::UserExit => {
                tracing::info!(
                    worker = %worker.name,
                    crash_type = "user_exit",
                    "Clean exit detected, resetting to idle"
                );
                self.log_recovery_action(
                    &worker.name,
                    RecoveryAction::SessionCrash { crash_type: "user_exit".to_string() },
                    "Reset to idle",
                    true,
                )?;
                worker.status = WorkerStatus::Idle;
                worker.current_prompt.clear();
                worker.commit_sha = None;
                worker.crash_count = 0;
                Ok(())
            }
            CrashType::RateLimit => {
                tracing::warn!(
                    worker = %worker.name,
                    crash_type = "rate_limit",
                    crash_count = worker.crash_count + 1,
                    "Rate limit detected, waiting 5 minutes before retry"
                );
                self.log_recovery_action(
                    &worker.name,
                    RecoveryAction::SessionCrash { crash_type: "rate_limit".to_string() },
                    "Wait 5 minutes and retry",
                    true,
                )?;

                thread::sleep(Duration::from_secs(300));

                worker.crash_count += 1;
                if worker.crash_count >= 3 {
                    tracing::error!(
                        worker = %worker.name,
                        crash_count = worker.crash_count,
                        "Rate limit exceeded after 3 attempts, escalating"
                    );
                    worker.status = WorkerStatus::Error;
                    self.escalate_crash_to_user(&worker.name, "Rate limit after 3 retries")?;
                    bail!("Rate limit exceeded after 3 attempts");
                }

                self.restart_worker_session(worker)
            }
            CrashType::Fatal | CrashType::Unknown => {
                worker.crash_count += 1;
                worker.last_crash_unix =
                    Some(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs());

                tracing::warn!(
                    worker = %worker.name,
                    crash_type = ?crash_type,
                    crash_count = worker.crash_count,
                    "Unexpected crash detected, attempting auto-restart"
                );
                self.log_recovery_action(
                    &worker.name,
                    RecoveryAction::SessionCrash {
                        crash_type: format!("{:?}", crash_type).to_lowercase(),
                    },
                    &format!("Auto-restart (crash #{})", worker.crash_count),
                    true,
                )?;

                if worker.crash_count >= 3 {
                    tracing::error!(
                        worker = %worker.name,
                        crash_count = worker.crash_count,
                        crash_type = ?crash_type,
                        "Worker crashed 3 times, escalating to user"
                    );
                    worker.status = WorkerStatus::Error;
                    self.escalate_crash_to_user(
                        &worker.name,
                        &format!("3 crashes detected: {:?}", crash_type),
                    )?;
                    bail!("Worker crashed 3 times, marking as ERROR");
                }

                self.restart_worker_session(worker)
            }
        }
    }

    fn classify_crash(&self, session_id: &str) -> Result<CrashType> {
        let output = session::capture_pane(session_id, 50)?;

        if output.contains("rate limit") || output.contains("429") {
            return Ok(CrashType::RateLimit);
        }
        if output.contains("/exit") || output.contains("Goodbye") {
            return Ok(CrashType::UserExit);
        }
        if output.contains("FATAL") || output.contains("panic") {
            return Ok(CrashType::Fatal);
        }

        Ok(CrashType::Unknown)
    }

    fn restart_worker_session(&self, worker: &mut WorkerRecord) -> Result<()> {
        let prompt_backup = worker.current_prompt.clone();
        let was_working = worker.status == WorkerStatus::Working;

        self.respawn_claude(&worker.session_id)?;

        thread::sleep(Duration::from_secs(2));

        if was_working && !prompt_backup.is_empty() {
            let context_msg = format!(
                "{}\n\nNote: The session crashed during processing. Previous partial work may be \
                 visible in the git diff. Please continue from where you left off.",
                prompt_backup
            );

            let sender = TmuxSender::default();
            sender.send(&worker.session_id, "/clear")?;
            thread::sleep(Duration::from_millis(500));
            sender.send(&worker.session_id, &context_msg)?;

            worker.status = WorkerStatus::Working;
            worker.current_prompt = prompt_backup;
        } else {
            worker.status = WorkerStatus::Idle;
            worker.current_prompt.clear();
        }

        Ok(())
    }

    fn respawn_claude(&self, session_id: &str) -> Result<()> {
        let target = format!("{}:0", session_id);

        Command::new("tmux")
            .args(["respawn-pane", "-k", "-t", &target])
            .output()
            .context("Failed to respawn pane")?;

        thread::sleep(Duration::from_millis(500));

        let sender = TmuxSender::default();
        sender
            .send(session_id, "claude --dangerously-skip-permissions")
            .context("Failed to send claude command")?;

        self.wait_for_claude_ready_after_respawn(session_id)?;

        self.accept_bypass_warning_after_respawn(session_id, &sender)?;

        Ok(())
    }

    fn wait_for_claude_ready_after_respawn(&self, session_id: &str) -> Result<()> {
        const MAX_ATTEMPTS: u32 = 60;
        const POLL_INTERVAL_MS: u64 = 500;

        for _ in 0..MAX_ATTEMPTS {
            thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));

            let Ok(output) = session::capture_pane(session_id, 50) else {
                continue;
            };

            if output.lines().rev().take(5).any(|line| {
                let trimmed = line.trim_start();
                trimmed.starts_with("> ") || trimmed == ">" || trimmed.starts_with("❯")
            }) {
                return Ok(());
            }

            if let Ok(command) = session::get_pane_command(session_id)
                && session::is_shell(&command)
            {
                bail!(
                    "Claude process exited unexpectedly after respawn, shell detected: {}",
                    command
                );
            }
        }

        bail!("Claude did not become ready after respawn (30 seconds timeout)")
    }

    fn accept_bypass_warning_after_respawn(
        &self,
        session_id: &str,
        sender: &TmuxSender,
    ) -> Result<()> {
        thread::sleep(Duration::from_millis(500));

        let Ok(output) = session::capture_pane(session_id, 50) else {
            return Ok(());
        };

        if output.contains("bypass") || output.contains("dangerous") || output.contains("Bypass") {
            sender.send_keys_raw(session_id, "Down")?;
            thread::sleep(Duration::from_millis(200));
            sender.send_keys_raw(session_id, "Enter")?;
            thread::sleep(Duration::from_millis(500));
        }

        Ok(())
    }

    /// Handle stuck worker recovery
    pub fn handle_stuck_worker(&self, worker: &mut WorkerRecord) -> Result<()> {
        tracing::warn!("Handling stuck worker '{}'", worker.name);

        let current_state = self.state_detector.detect(&worker.session_id)?;

        if current_state == ClaudeState::Ready {
            if let Some(commit_sha) = self.check_for_new_commit(&worker.worktree_path)? {
                tracing::info!("Found commit, transitioning to needs_review");
                worker.status = WorkerStatus::NeedsReview;
                worker.commit_sha = Some(commit_sha);
                self.log_recovery_action(
                    &worker.name,
                    RecoveryAction::StuckWorker { action: "found_commit".to_string() },
                    "Transitioned to needs_review",
                    true,
                )?;
                return Ok(());
            }

            tracing::info!("No commit found, transitioning to needs_input");
            worker.status = WorkerStatus::NeedsInput;
            self.log_recovery_action(
                &worker.name,
                RecoveryAction::StuckWorker { action: "no_commit".to_string() },
                "Transitioned to needs_input",
                true,
            )?;
            return Ok(());
        }

        let working_duration =
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() - worker.last_activity_unix;

        if working_duration >= 30 * 60 {
            self.send_nudge(&worker.name, &worker.session_id, "first")?;
            thread::sleep(Duration::from_secs(5 * 60));

            let state_after_nudge = self.state_detector.detect(&worker.session_id)?;
            if state_after_nudge == ClaudeState::Processing {
                return Ok(());
            }

            if working_duration >= 40 * 60 {
                self.send_nudge(&worker.name, &worker.session_id, "final")?;
                thread::sleep(Duration::from_secs(5 * 60));

                let final_state = self.state_detector.detect(&worker.session_id)?;
                if final_state == ClaudeState::Processing {
                    return Ok(());
                }

                worker.status = WorkerStatus::NeedsInput;
                self.log_recovery_action(
                    &worker.name,
                    RecoveryAction::StuckWorker { action: "timeout".to_string() },
                    "Marked as needs_input after timeout",
                    true,
                )?;
            }
        }

        Ok(())
    }

    fn check_for_new_commit(&self, worktree_path: &str) -> Result<Option<String>> {
        let output = Command::new("git")
            .args(["log", "-1", "--format=%H", "HEAD"])
            .current_dir(worktree_path)
            .output()
            .context("Failed to get HEAD commit")?;

        if !output.status.success() {
            return Ok(None);
        }

        let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if sha.is_empty() { Ok(None) } else { Ok(Some(sha)) }
    }

    fn send_nudge(&self, worker_name: &str, session_id: &str, nudge_type: &str) -> Result<()> {
        let message = match nudge_type {
            "first" => {
                "Status check: You've been working on this task for 30 minutes. Are you making \
                 progress or blocked on something? Please provide a brief update."
            }
            "final" => {
                "This task will be flagged for human review if there's no response in 5 minutes. \
                 If you're blocked, please describe the issue. If you're still working, please \
                 commit your progress so far."
            }
            _ => return Ok(()),
        };

        self.log_recovery_action(
            worker_name,
            RecoveryAction::StuckWorker { action: format!("nudge_{}", nudge_type) },
            message,
            true,
        )?;

        let sender = TmuxSender::default();
        sender.send(session_id, message)?;

        Ok(())
    }

    /// Handle partial send recovery
    pub fn handle_partial_send(&self, worker: &mut Worker, message: &str) -> Result<()> {
        tracing::warn!("Handling partial send for worker '{}'", worker.name);

        for attempt in 1..=3 {
            self.log_recovery_action(
                &worker.name,
                RecoveryAction::PartialSend { attempt },
                "Clear and retry",
                false,
            )?;

            if self.clear_and_retry(worker, message)? {
                self.mark_recovery_success(&worker.name, attempt)?;
                return Ok(());
            }

            thread::sleep(Duration::from_millis(100 * attempt as u64));
        }

        self.escalate_to_user(
            &worker.name,
            "Partial send after 3 retry attempts",
            message,
            &worker.session_id,
        )?;
        bail!("Failed to recover from partial send after 3 attempts");
    }

    fn clear_and_retry(&self, worker: &mut Worker, message: &str) -> Result<bool> {
        Command::new("tmux")
            .args(["send-keys", "-t", &worker.session_id, "C-u"])
            .output()
            .context("Failed to send Ctrl-U")?;

        thread::sleep(Duration::from_millis(100));

        let output = session::capture_pane(&worker.session_id, 5)?;
        if !output.trim().ends_with('>') && !output.trim().ends_with('❯') && !output.contains("> ") {
            Command::new("tmux")
                .args(["send-keys", "-t", &worker.session_id, "C-c"])
                .output()
                .context("Failed to send Ctrl-C")?;
            thread::sleep(Duration::from_millis(200));
        }

        worker.sender.send(&worker.session_id, message)?;

        self.verify_input_received(&worker.session_id, Duration::from_secs(5))
    }

    /// Handle state corruption recovery
    pub fn handle_state_corruption(state_path: &Path) -> Result<State> {
        tracing::error!("Handling state corruption for {}", state_path.display());

        let backup_path = state_path.with_extension("json.bak");

        if backup_path.exists() {
            tracing::info!("Attempting to restore from backup");
            let backup_content =
                fs::read_to_string(&backup_path).context("Failed to read backup file")?;

            match serde_json::from_str::<State>(&backup_content) {
                Ok(state) => {
                    tracing::info!("Successfully restored state from backup");
                    fs::copy(&backup_path, state_path)?;
                    return Ok(state);
                }
                Err(e) => {
                    tracing::error!("Backup file also corrupted: {}", e);
                }
            }
        }

        eprintln!("\n╭─────────────────────────────────────────────────────────────╮");
        eprintln!("│ ⚠️  Critical: State file corruption detected");
        eprintln!("╰─────────────────────────────────────────────────────────────╯");
        eprintln!("\n📋 Issue:");
        eprintln!("  The LLMC state file at {} is corrupted and the backup", state_path.display());
        eprintln!("  is either missing or also corrupted.");
        eprintln!("\n💡 Recovery Options:");
        eprintln!("  1. llmc doctor --rebuild");
        eprintln!("     Reconstruct state from filesystem (preserves worker data)");
        eprintln!("     ⭐ RECOMMENDED - Try this first");
        eprintln!();
        eprintln!("  2. llmc init --force");
        eprintln!("     Initialize fresh state (⚠️  LOSES all worker history)");
        eprintln!();
        eprintln!("  3. Manual recovery:");
        eprintln!("     - Check backup: {}", backup_path.display());
        eprintln!("     - Edit state.json manually if partially readable");
        eprintln!();
        eprintln!("⚠️  Do NOT proceed without recovering state to avoid data loss.\n");

        bail!(
            "State file corrupted and cannot be automatically recovered. Run 'llmc doctor --rebuild' to attempt recovery."
        );
    }

    fn log_recovery_action(
        &self,
        worker: &str,
        action: RecoveryAction,
        context: &str,
        success: bool,
    ) -> Result<()> {
        let entry = RecoveryLogEntry {
            timestamp: Utc::now(),
            worker: worker.to_string(),
            action,
            context: context.to_string(),
            pane_output: self.capture_pane_output(worker).ok(),
            git_status: self.get_git_status(worker).ok(),
            success,
        };

        let log_path = self.logs_dir.join(format!("{}.log", worker));
        fs::create_dir_all(&self.logs_dir)?;

        let mut file = OpenOptions::new().create(true).append(true).open(&log_path)?;

        writeln!(file, "{}", serde_json::to_string(&entry)?)?;

        Ok(())
    }

    fn mark_recovery_success(&self, worker: &str, attempt: u32) -> Result<()> {
        tracing::info!("Recovery successful for '{}' on attempt {}", worker, attempt);
        Ok(())
    }

    fn capture_pane_output(&self, worker: &str) -> Result<String> {
        let session_id = format!("llmc-{}", worker);
        session::capture_pane(&session_id, 30)
            .context("Failed to capture pane output for diagnostics")
    }

    fn get_git_status(&self, worker: &str) -> Result<String> {
        let worktree_path = self
            .logs_dir
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid logs directory"))?
            .join(".worktrees")
            .join(worker);

        let output = Command::new("git")
            .args(["status", "--short"])
            .current_dir(&worktree_path)
            .output()
            .context("Failed to get git status for diagnostics")?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn escalate_to_user(
        &self,
        worker_name: &str,
        error: &str,
        prompt: &str,
        session_id: &str,
    ) -> Result<()> {
        let pane_output = session::capture_pane(session_id, 30)
            .unwrap_or_else(|e| format!("[Unable to capture pane output: {}]", e));
        let git_status = self
            .get_git_status(worker_name)
            .unwrap_or_else(|e| format!("[Unable to get git status: {}]", e));

        eprintln!("\n╭─────────────────────────────────────────────────────────────╮");
        eprintln!("│ ⚠️  Worker '{}' requires manual intervention", worker_name);
        eprintln!("╰─────────────────────────────────────────────────────────────╯");
        eprintln!("\n📋 Error Details:");
        eprintln!("  {}", error);
        eprintln!("\n🔍 Diagnostics:");
        eprintln!(
            "  Task: \"{}\"",
            if prompt.len() > 60 { format!("{}...", &prompt[..57]) } else { prompt.to_string() }
        );

        if !git_status.trim().is_empty() {
            eprintln!("\n  Git Status:");
            for line in git_status.lines() {
                eprintln!("    {}", line);
            }
        }

        eprintln!("\n  Recent Terminal Output:");
        let lines: Vec<_> = pane_output.lines().collect();
        let display_count = 15.min(lines.len());
        for line in lines.iter().rev().take(display_count).rev() {
            eprintln!("    {}", line);
        }

        eprintln!("\n💡 Suggested Actions:");
        eprintln!("  1. llmc attach {}          - Interactive session access", worker_name);
        eprintln!("  2. llmc message {} \"...\"   - Send clarification", worker_name);
        eprintln!("  3. llmc review {}          - Check current work", worker_name);
        eprintln!(
            "  4. llmc nuke {} && llmc add {}  - Full worker reset",
            worker_name, worker_name
        );
        eprintln!("\n📖 Logs: {}/{}.log\n", self.logs_dir.display(), worker_name);

        Ok(())
    }

    fn escalate_crash_to_user(&self, worker_name: &str, reason: &str) -> Result<()> {
        let pane_output = self
            .capture_pane_output(worker_name)
            .unwrap_or_else(|e| format!("[Unable to capture output: {}]", e));

        eprintln!("\n╭─────────────────────────────────────────────────────────────╮");
        eprintln!("│ ⚠️  Worker '{}' crashed and requires intervention", worker_name);
        eprintln!("╰─────────────────────────────────────────────────────────────╯");
        eprintln!("\n📋 Crash Details:");
        eprintln!("  {}", reason);

        eprintln!("\n  Last Terminal Output:");
        let lines: Vec<_> = pane_output.lines().collect();
        let display_count = 20.min(lines.len());
        for line in lines.iter().rev().take(display_count).rev() {
            eprintln!("    {}", line);
        }

        eprintln!("\n💡 Recovery Options:");
        eprintln!("  1. llmc attach {}          - Inspect crash state", worker_name);
        eprintln!("  2. View logs: {}/{}.log", self.logs_dir.display(), worker_name);
        eprintln!(
            "  3. llmc nuke {} && llmc add {}  - Reset worker (loses state)",
            worker_name, worker_name
        );
        eprintln!("\n⚠️  Note: After 3 crashes, manual intervention is required.\n");

        Ok(())
    }
}
