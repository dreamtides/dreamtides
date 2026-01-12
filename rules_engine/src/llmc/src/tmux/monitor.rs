#![allow(dead_code)]

use std::thread::sleep;
use std::time::Duration;

use anyhow::Result;
use tmux_interface::{DisplayMessage, Tmux};

use super::sender::TmuxSender;
use super::session;

/// Claude's operational state - simplified to only deterministic states
#[derive(Debug, Clone, PartialEq)]
pub enum ClaudeState {
    Processing,
    Exited,
    Unknown,
}

/// Output event types for monitoring - simplified to deterministic events
#[derive(Debug, Clone, PartialEq)]
pub enum OutputEvent {
    Committed(String),
    CompletedNoCommit,
    Crashed,
}

/// Detects Claude's state from terminal output
pub struct StateDetector {
    sender: TmuxSender,
}

/// Monitors terminal output for specific events
pub struct OutputMonitor {
    sender: TmuxSender,
}

/// Process health status
#[derive(Debug, Clone, PartialEq)]
enum ProcessHealth {
    Running,
    Exited,
    Unknown,
}

impl StateDetector {
    pub fn new(sender: TmuxSender) -> Self {
        Self { sender }
    }

    /// Detects the current state of Claude in the given session
    /// Simplified to only check deterministic process health
    pub fn detect(&self, session: &str) -> Result<ClaudeState> {
        let health = check_process_health(session)?;
        match health {
            ProcessHealth::Exited => Ok(ClaudeState::Exited),
            ProcessHealth::Running => Ok(ClaudeState::Processing),
            ProcessHealth::Unknown => Ok(ClaudeState::Unknown),
        }
    }

    /// Accepts the bypass permissions warning dialog
    pub fn accept_bypass_warning(&self, session: &str) -> Result<()> {
        sleep(Duration::from_millis(1000));
        let output = session::capture_pane(session, 30)?;

        let lower = output.to_lowercase();
        let has_bypass_warning = lower.contains("bypass")
            || lower.contains("dangerously")
            || lower.contains("skip-permissions")
            || lower.contains("skip permissions");

        if !has_bypass_warning {
            return Ok(());
        }

        self.sender.send_keys_raw(session, "Down")?;
        sleep(Duration::from_millis(200));
        self.sender.send_keys_raw(session, "Enter")?;
        Ok(())
    }
}

impl OutputMonitor {
    pub fn new(sender: TmuxSender) -> Self {
        Self { sender }
    }

    /// Waits for and detects output events
    /// Simplified to only detect deterministic events (exits and crashes)
    pub fn wait_for_event(&self, session: &str) -> Result<OutputEvent> {
        let detector = StateDetector::new(self.sender.clone());
        loop {
            let state = detector.detect(session)?;
            match state {
                ClaudeState::Exited => {
                    // Check if it was a crash via exit code
                    if let Some(exit_code) = get_pane_exit_code(session)
                        && exit_code != 0
                        && exit_code != 130
                    {
                        return Ok(OutputEvent::Crashed);
                    }
                    return Ok(OutputEvent::CompletedNoCommit);
                }
                ClaudeState::Processing | ClaudeState::Unknown => {
                    sleep(Duration::from_millis(500));
                }
            }
        }
    }
}

fn check_process_health(session: &str) -> Result<ProcessHealth> {
    let cmd = session::get_pane_command(session)?;
    if session::is_claude_process(&cmd) {
        Ok(ProcessHealth::Running)
    } else if session::is_shell(&cmd) {
        Ok(ProcessHealth::Exited)
    } else {
        Ok(ProcessHealth::Unknown)
    }
}

fn get_pane_exit_code(session: &str) -> Option<i32> {
    Tmux::with_command(
        DisplayMessage::new().target_pane(session).print().message("#{pane_dead_status}"),
    )
    .output()
    .ok()
    .and_then(|output| output.to_string().trim().parse().ok())
}
