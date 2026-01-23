// TODO: Remove this allow once overseer_session is integrated with
// overseer_loop and remediation_executor
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{fs, thread};

use anyhow::{Context, Result};

use crate::config::{self, Config};
use crate::tmux::sender::TmuxSender;
use crate::tmux::session;

/// Returns the overseer session name for this LLMC instance.
///
/// Uses the session prefix to ensure multiple LLMC instances don't conflict.
pub fn get_overseer_session_name() -> String {
    format!("{}-overseer", config::get_session_prefix())
}

/// Returns true if the given session name is the overseer session.
///
/// This is used by other modules to exclude the overseer session from
/// kill operations (e.g., `llmc down`, `llmc nuke --all`).
pub fn is_overseer_session(session_name: &str) -> bool {
    session_name == get_overseer_session_name()
}

/// Checks if the overseer TMUX session exists and is healthy.
///
/// A session is considered healthy if:
/// 1. The TMUX session exists
/// 2. The session has content (Claude Code is running)
pub fn is_overseer_session_healthy() -> bool {
    if !session::session_exists(&get_overseer_session_name()) {
        tracing::debug!(session = &get_overseer_session_name(), "Overseer session does not exist");
        return false;
    }
    match session::capture_pane(&get_overseer_session_name(), 5) {
        Ok(content) => {
            let is_active = !content.trim().is_empty();
            tracing::debug!(
                session = &get_overseer_session_name(),
                has_content = is_active,
                "Overseer session health check"
            );
            is_active
        }
        Err(e) => {
            tracing::info!(
                session = &get_overseer_session_name(),
                error = %e,
                "Failed to capture overseer session pane - session may be restarting or \
                 may have crashed. Overseer health check will retry on next poll."
            );
            false
        }
    }
}

/// Creates the overseer TMUX session in the main project directory.
///
/// The session runs in repo.source (the main project directory), NOT a
/// worktree. It uses a wide terminal width to prevent message truncation and
/// sets environment variables for identification.
pub fn create_overseer_session(config: &Config) -> Result<()> {
    let project_dir = PathBuf::from(&config.repo.source);
    if !project_dir.exists() {
        anyhow::bail!("Project directory does not exist: {}", project_dir.display());
    }
    tracing::info!(
        session = &get_overseer_session_name(),
        working_directory = %project_dir.display(),
        "Creating overseer TMUX session"
    );
    session::create_session(
        &get_overseer_session_name(),
        &project_dir,
        session::DEFAULT_SESSION_WIDTH,
        session::DEFAULT_SESSION_HEIGHT,
    )
    .with_context(|| {
        format!("Failed to create overseer TMUX session '{}'", &get_overseer_session_name())
    })?;
    let llmc_root = crate::config::get_llmc_root();
    session::set_env(&get_overseer_session_name(), "LLMC_OVERSEER", "true")?;
    session::set_env(
        &get_overseer_session_name(),
        "LLMC_ROOT",
        llmc_root.to_str().unwrap_or_default(),
    )?;
    create_overseer_claude_hooks(&project_dir)?;
    thread::sleep(Duration::from_millis(500));
    let claude_cmd = build_overseer_claude_command(config);
    tracing::info!(
        session = &get_overseer_session_name(),
        command = %claude_cmd,
        "Starting Claude Code in overseer session"
    );
    let sender = TmuxSender::new();
    sender.send(&get_overseer_session_name(), &claude_cmd).with_context(|| {
        format!(
            "Failed to send Claude command to overseer session '{}'",
            &get_overseer_session_name()
        )
    })?;
    tracing::info!(
        session = &get_overseer_session_name(),
        "Overseer session created and Claude Code started"
    );
    Ok(())
}

/// Ensures the overseer session is running and healthy.
///
/// If the session doesn't exist or is unhealthy, it will be (re)created.
/// This is an AUTO recovery operation - the overseer handles it without
/// external help.
pub fn ensure_overseer_session(config: &Config) -> Result<()> {
    if is_overseer_session_healthy() {
        tracing::debug!(session = &get_overseer_session_name(), "Overseer session is healthy");
        return Ok(());
    }
    if session::session_exists(&get_overseer_session_name()) {
        tracing::info!(
            session = &get_overseer_session_name(),
            "Overseer session exists but is unhealthy, killing and recreating"
        );
        session::kill_session(&get_overseer_session_name())?;
        thread::sleep(Duration::from_millis(500));
    } else {
        tracing::info!(
            session = &get_overseer_session_name(),
            "Overseer session does not exist, creating"
        );
    }
    create_overseer_session(config)
}

/// Kills the overseer session if it exists.
///
/// This should only be called during overseer shutdown.
pub fn kill_overseer_session() -> Result<()> {
    if session::session_exists(&get_overseer_session_name()) {
        tracing::info!(session = &get_overseer_session_name(), "Killing overseer TMUX session");
        session::kill_session(&get_overseer_session_name())?;
    }
    Ok(())
}

/// Sends a message to the overseer Claude Code session.
///
/// Used by the remediation executor to send prompts to Claude.
pub fn send_to_overseer(message: &str) -> Result<()> {
    if !session::session_exists(&get_overseer_session_name()) {
        anyhow::bail!("Overseer session '{}' does not exist", &get_overseer_session_name());
    }
    let sender = TmuxSender::new();
    sender.send(&get_overseer_session_name(), message)
}

/// Sends a /clear command to the overseer session to reset Claude's context.
pub fn clear_overseer_session() -> Result<()> {
    tracing::info!(session = &get_overseer_session_name(), "Sending /clear to overseer session");
    send_to_overseer("/clear")?;
    thread::sleep(Duration::from_secs(2));
    Ok(())
}

/// Captures recent output from the overseer session.
pub fn capture_overseer_output(lines: u32) -> Result<String> {
    session::capture_pane(&get_overseer_session_name(), lines)
}

/// Creates Claude hook settings for the overseer with an explicit llmc_root.
///
/// Uses the remediation socket path to avoid conflicts with the daemon's
/// socket. Use this in tests to avoid depending on the LLMC_ROOT environment
/// variable.
pub fn create_overseer_claude_hooks_with_root(project_dir: &Path, llmc_root: &Path) -> Result<()> {
    let claude_dir = project_dir.join(".claude");
    let settings_path = claude_dir.join("settings.json");
    let llmc_root_str = llmc_root.to_string_lossy();
    let remediation_socket = llmc_root.join("llmc_remediation.sock");
    let remediation_socket_str = remediation_socket.to_string_lossy();
    let expected_socket_arg = format!("--socket {}", remediation_socket_str);
    if settings_path.exists() {
        let content = fs::read_to_string(&settings_path)?;
        if content.contains("llmc hook") && content.contains(&expected_socket_arg) {
            tracing::debug!(
                path = %settings_path.display(),
                llmc_root = %llmc_root_str,
                "Overseer Claude hooks already configured with correct remediation socket"
            );
            return Ok(());
        }
        if content.contains("llmc hook") {
            tracing::info!(
                path = %settings_path.display(),
                expected_socket = %remediation_socket_str,
                "Overseer Claude hooks exist but socket path doesn't match, regenerating"
            );
        }
    }
    fs::create_dir_all(&claude_dir).context("Failed to create .claude directory")?;
    let llmc_bin = std::env::current_exe()
        .unwrap_or_else(|_| PathBuf::from("llmc"))
        .to_string_lossy()
        .to_string();
    let settings = serde_json::json!({
        "hooks": {
            "Stop": [{
                "hooks": [{
                    "type": "command",
                    "command": format!("LLMC_ROOT={} {} hook stop --worker overseer --socket {}", llmc_root_str, llmc_bin, remediation_socket_str),
                    "timeout": 5
                }]
            }],
            "SessionStart": [{
                "hooks": [{
                    "type": "command",
                    "command": format!("LLMC_ROOT={} {} hook session-start --worker overseer --socket {}", llmc_root_str, llmc_bin, remediation_socket_str),
                    "timeout": 5
                }]
            }],
            "SessionEnd": [{
                "hooks": [{
                    "type": "command",
                    "command": format!("LLMC_ROOT={} {} hook session-end --worker overseer --socket {}", llmc_root_str, llmc_bin, remediation_socket_str),
                    "timeout": 5
                }]
            }]
        }
    });
    let content =
        serde_json::to_string_pretty(&settings).context("Failed to serialize hook settings")?;
    fs::write(&settings_path, content).context("Failed to write .claude/settings.json")?;
    tracing::info!(
        path = %settings_path.display(),
        llmc_root = %llmc_root_str,
        remediation_socket = %remediation_socket_str,
        "Created Claude hook settings for overseer with remediation socket"
    );
    Ok(())
}

fn build_overseer_claude_command(config: &Config) -> String {
    let mut cmd = String::from("claude");
    cmd.push_str(&format!(" --model {}", config.defaults.model));
    cmd.push_str(" --dangerously-skip-permissions");
    cmd
}

fn create_overseer_claude_hooks(project_dir: &Path) -> Result<()> {
    let llmc_root = config::get_llmc_root();
    create_overseer_claude_hooks_with_root(project_dir, &llmc_root)
}
