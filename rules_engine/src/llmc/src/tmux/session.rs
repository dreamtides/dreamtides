#![allow(dead_code)]
use std::path::Path;
use std::sync::OnceLock;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use regex::Regex;
use tmux_interface::{
    CapturePane, DisplayMessage, HasSession, KillSession, ListSessions, NewSession, SetEnvironment,
    Tmux,
};

use super::sender::TmuxSender;
use crate::config::WorkerConfig;

/// Default TMUX session width (wide terminal to prevent message truncation)
pub const DEFAULT_SESSION_WIDTH: u32 = 500;
/// Default TMUX session height
pub const DEFAULT_SESSION_HEIGHT: u32 = 100;

/// Creates a detached TMUX session with specified dimensions
pub fn create_session(name: &str, cwd: &Path, width: u32, height: u32) -> Result<()> {
    if session_exists(name) {
        bail!("Session '{}' already exists", name);
    }

    let cwd_str = cwd.to_string_lossy();
    let new_session = NewSession::new()
        .detached()
        .session_name(name)
        .start_directory(cwd_str.as_ref())
        .width(width as usize)
        .height(height as usize);

    Tmux::new()
        .add_command(new_session)
        .output()
        .with_context(|| format!("Failed to create TMUX session '{}'", name))?;

    Ok(())
}

/// Kills a TMUX session by name
pub fn kill_session(name: &str) -> Result<()> {
    if !session_exists(name) {
        return Ok(());
    }

    Tmux::with_command(KillSession::new().target_session(name))
        .output()
        .with_context(|| format!("Failed to kill TMUX session '{}'", name))?;

    Ok(())
}

/// Checks if a TMUX session exists
pub fn session_exists(name: &str) -> bool {
    Tmux::with_command(HasSession::new().target_session(name))
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

/// Lists all active TMUX session names
pub fn list_sessions() -> Result<Vec<String>> {
    let output = Tmux::with_command(ListSessions::new().format("#{session_name}")).output();

    match output {
        Ok(output) => {
            if output.success() {
                Ok(output.to_string().lines().map(str::to_string).collect())
            } else {
                Ok(Vec::new())
            }
        }
        Err(e) => bail!("Failed to list TMUX sessions: {}", e),
    }
}

/// Gets the command running in the session's active pane
pub fn get_pane_command(session: &str) -> Result<String> {
    let output = Tmux::with_command(
        DisplayMessage::new().target_pane(session).print().message("#{pane_current_command}"),
    )
    .output()
    .with_context(|| format!("Failed to get pane command for session '{}'", session))?;

    if !output.success() {
        bail!("Session '{}' not found or inaccessible", session);
    }

    Ok(output.to_string().trim().to_string())
}

/// Captures recent lines from the session's active pane
pub fn capture_pane(session: &str, lines: u32) -> Result<String> {
    let output = Tmux::with_command(
        CapturePane::new().target_pane(session).stdout().start_line(format!("-{}", lines)),
    )
    .output()
    .with_context(|| format!("Failed to capture pane for session '{}'", session))?;

    if !output.success() {
        bail!("Session '{}' not found or inaccessible", session);
    }

    Ok(output.to_string())
}

/// Sets an environment variable in the session
pub fn set_env(session: &str, key: &str, value: &str) -> Result<()> {
    let output =
        Tmux::with_command(SetEnvironment::new().target_session(session).name(key).value(value))
            .output()
            .with_context(|| {
                format!("Failed to set environment variable in session '{}'", session)
            })?;

    if !output.success() {
        bail!("Failed to set environment variable '{}' in session '{}'", key, session);
    }

    Ok(())
}

/// Starts a complete worker session with Claude Code
pub fn start_worker_session(name: &str, worktree: &Path, config: &WorkerConfig) -> Result<()> {
    create_session(name, worktree, DEFAULT_SESSION_WIDTH, DEFAULT_SESSION_HEIGHT)?;

    let llmc_root = crate::config::get_llmc_root();
    set_env(name, "LLMC_WORKER", name)?;
    set_env(name, "LLMC_ROOT", llmc_root.to_str().unwrap())?;

    thread::sleep(Duration::from_millis(500));

    let sender = TmuxSender::new();
    let claude_cmd = build_claude_command(config);
    sender
        .send(name, &claude_cmd)
        .with_context(|| format!("Failed to send Claude command to session '{}'", name))?;

    wait_for_claude_ready(name, Duration::from_secs(30))?;

    accept_bypass_warning(name, &sender)?;

    sender
        .send(name, "/clear")
        .with_context(|| format!("Failed to send /clear to session '{}'", name))?;
    thread::sleep(Duration::from_millis(500));

    Ok(())
}

/// Converts a worker name to a session name
pub fn session_name_for_worker(worker: &str) -> String {
    format!("llmc-{}", worker)
}

/// Extracts worker name from a session name
pub fn worker_from_session_name(session: &str) -> Option<String> {
    session.strip_prefix("llmc-").map(str::to_string)
}

/// Checks if a command is a shell
pub fn is_shell(cmd: &str) -> bool {
    matches!(cmd, "bash" | "zsh" | "sh" | "fish" | "dash")
}

/// Checks if a command is a Claude process
pub fn is_claude_process(cmd: &str) -> bool {
    if matches!(cmd, "node" | "claude") {
        return true;
    }

    static SEMVER_PATTERN: OnceLock<Regex> = OnceLock::new();
    SEMVER_PATTERN.get_or_init(|| Regex::new(r"^\d+\.\d+\.\d+").unwrap()).is_match(cmd)
}

fn build_claude_command(config: &WorkerConfig) -> String {
    let mut cmd = String::from("claude");
    if let Some(model) = &config.model {
        cmd.push_str(&format!(" --model {}", model));
    }
    cmd.push_str(" --dangerously-skip-permissions");
    cmd
}

fn wait_for_claude_ready(session: &str, timeout: Duration) -> Result<()> {
    const POLL_INTERVAL_MS: u64 = 500;
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
        if let Ok(output) = capture_pane(session, 50) {
            if output.lines().rev().take(5).any(|line| {
                let trimmed = line.trim_start();
                trimmed.starts_with("> ") || trimmed == ">"
            }) {
                return Ok(());
            }

            if let Ok(command) = get_pane_command(session)
                && !is_claude_process(&command)
            {
                bail!(
                    "Claude process not found in session '{}', got command: {}",
                    session,
                    command
                );
            }
        }
    }

    bail!("Claude did not become ready after {} seconds", timeout.as_secs())
}

fn accept_bypass_warning(session: &str, sender: &TmuxSender) -> Result<()> {
    thread::sleep(Duration::from_millis(500));
    if let Ok(output) = capture_pane(session, 50)
        && (output.contains("bypass") || output.contains("dangerous"))
    {
        sender.send_keys_raw(session, "Down")?;
        thread::sleep(Duration::from_millis(200));
        sender.send_keys_raw(session, "Enter")?;
        thread::sleep(Duration::from_millis(500));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_name_for_worker() {
        assert_eq!(session_name_for_worker("adam"), "llmc-adam");
        assert_eq!(session_name_for_worker("baker"), "llmc-baker");
    }

    #[test]
    fn test_worker_from_session_name() {
        assert_eq!(worker_from_session_name("llmc-adam"), Some("adam".to_string()));
        assert_eq!(worker_from_session_name("llmc-baker"), Some("baker".to_string()));
        assert_eq!(worker_from_session_name("other-session"), None);
        assert_eq!(worker_from_session_name("llmc"), None);
    }

    #[test]
    fn test_is_shell() {
        assert!(is_shell("bash"));
        assert!(is_shell("zsh"));
        assert!(is_shell("sh"));
        assert!(is_shell("fish"));
        assert!(is_shell("dash"));
        assert!(!is_shell("node"));
        assert!(!is_shell("claude"));
        assert!(!is_shell("python"));
    }

    #[test]
    fn test_is_claude_process() {
        assert!(is_claude_process("node"));
        assert!(is_claude_process("claude"));
        assert!(is_claude_process("2.0.76"));
        assert!(is_claude_process("1.0.0"));
        assert!(!is_claude_process("bash"));
        assert!(!is_claude_process("python"));
        assert!(!is_claude_process("some-other-process"));
    }
}
