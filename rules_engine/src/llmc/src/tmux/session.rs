#![allow(dead_code)]

use std::path::Path;
use std::sync::OnceLock;

use anyhow::{Context, Result, bail};
use regex::Regex;
use tmux_interface::{
    CapturePane, DisplayMessage, HasSession, KillSession, ListSessions, NewSession, SetEnvironment,
    Tmux,
};

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
