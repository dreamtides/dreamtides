// TODO: Remove this allow once overseer_session is integrated with
// overseer_loop and remediation_executor
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{fs, thread};

use anyhow::{Context, Result};

use crate::config::Config;
use crate::tmux::sender::TmuxSender;
use crate::tmux::session;

pub const OVERSEER_SESSION_NAME: &str = "llmc-overseer";

/// Returns true if the given session name is the overseer session.
///
/// This is used by other modules to exclude the overseer session from
/// kill operations (e.g., `llmc down`, `llmc nuke --all`).
pub fn is_overseer_session(session_name: &str) -> bool {
    session_name == OVERSEER_SESSION_NAME
}

/// Checks if the overseer TMUX session exists and is healthy.
///
/// A session is considered healthy if:
/// 1. The TMUX session exists
/// 2. The session has content (Claude Code is running)
pub fn is_overseer_session_healthy() -> bool {
    if !session::session_exists(OVERSEER_SESSION_NAME) {
        tracing::debug!(session = OVERSEER_SESSION_NAME, "Overseer session does not exist");
        return false;
    }
    match session::capture_pane(OVERSEER_SESSION_NAME, 5) {
        Ok(content) => {
            let is_active = !content.trim().is_empty();
            tracing::debug!(
                session = OVERSEER_SESSION_NAME,
                has_content = is_active,
                "Overseer session health check"
            );
            is_active
        }
        Err(e) => {
            tracing::warn!(
                session = OVERSEER_SESSION_NAME,
                error = %e,
                "Failed to capture overseer session pane"
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
        session = OVERSEER_SESSION_NAME,
        working_directory = %project_dir.display(),
        "Creating overseer TMUX session"
    );
    session::create_session(
        OVERSEER_SESSION_NAME,
        &project_dir,
        session::DEFAULT_SESSION_WIDTH,
        session::DEFAULT_SESSION_HEIGHT,
    )
    .with_context(|| {
        format!("Failed to create overseer TMUX session '{}'", OVERSEER_SESSION_NAME)
    })?;
    let llmc_root = crate::config::get_llmc_root();
    session::set_env(OVERSEER_SESSION_NAME, "LLMC_OVERSEER", "true")?;
    session::set_env(OVERSEER_SESSION_NAME, "LLMC_ROOT", llmc_root.to_str().unwrap_or_default())?;
    create_overseer_claude_hooks(&project_dir)?;
    thread::sleep(Duration::from_millis(500));
    let claude_cmd = build_overseer_claude_command(config);
    tracing::info!(
        session = OVERSEER_SESSION_NAME,
        command = %claude_cmd,
        "Starting Claude Code in overseer session"
    );
    let sender = TmuxSender::new();
    sender.send(OVERSEER_SESSION_NAME, &claude_cmd).with_context(|| {
        format!("Failed to send Claude command to overseer session '{}'", OVERSEER_SESSION_NAME)
    })?;
    tracing::info!(
        session = OVERSEER_SESSION_NAME,
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
        tracing::debug!(session = OVERSEER_SESSION_NAME, "Overseer session is healthy");
        return Ok(());
    }
    if session::session_exists(OVERSEER_SESSION_NAME) {
        tracing::info!(
            session = OVERSEER_SESSION_NAME,
            "Overseer session exists but is unhealthy, killing and recreating"
        );
        session::kill_session(OVERSEER_SESSION_NAME)?;
        thread::sleep(Duration::from_millis(500));
    } else {
        tracing::info!(
            session = OVERSEER_SESSION_NAME,
            "Overseer session does not exist, creating"
        );
    }
    create_overseer_session(config)
}

/// Kills the overseer session if it exists.
///
/// This should only be called during overseer shutdown.
pub fn kill_overseer_session() -> Result<()> {
    if session::session_exists(OVERSEER_SESSION_NAME) {
        tracing::info!(session = OVERSEER_SESSION_NAME, "Killing overseer TMUX session");
        session::kill_session(OVERSEER_SESSION_NAME)?;
    }
    Ok(())
}

/// Sends a message to the overseer Claude Code session.
///
/// Used by the remediation executor to send prompts to Claude.
pub fn send_to_overseer(message: &str) -> Result<()> {
    if !session::session_exists(OVERSEER_SESSION_NAME) {
        anyhow::bail!("Overseer session '{}' does not exist", OVERSEER_SESSION_NAME);
    }
    let sender = TmuxSender::new();
    sender.send(OVERSEER_SESSION_NAME, message)
}

/// Sends a /clear command to the overseer session to reset Claude's context.
pub fn clear_overseer_session() -> Result<()> {
    tracing::info!(session = OVERSEER_SESSION_NAME, "Sending /clear to overseer session");
    send_to_overseer("/clear")?;
    thread::sleep(Duration::from_secs(2));
    Ok(())
}

/// Captures recent output from the overseer session.
pub fn capture_overseer_output(lines: u32) -> Result<String> {
    session::capture_pane(OVERSEER_SESSION_NAME, lines)
}

fn build_overseer_claude_command(config: &Config) -> String {
    let mut cmd = String::from("claude");
    cmd.push_str(&format!(" --model {}", config.defaults.model));
    cmd.push_str(" --dangerously-skip-permissions");
    cmd
}

fn create_overseer_claude_hooks(project_dir: &Path) -> Result<()> {
    let claude_dir = project_dir.join(".claude");
    let settings_path = claude_dir.join("settings.json");
    if settings_path.exists() {
        let content = fs::read_to_string(&settings_path)?;
        if content.contains("llmc hook") {
            tracing::debug!(
                path = %settings_path.display(),
                "Overseer Claude hooks already configured"
            );
            return Ok(());
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
                    "command": format!("{} hook stop --worker overseer", llmc_bin),
                    "timeout": 5
                }]
            }],
            "SessionStart": [{
                "hooks": [{
                    "type": "command",
                    "command": format!("{} hook session-start --worker overseer", llmc_bin),
                    "timeout": 5
                }]
            }],
            "SessionEnd": [{
                "hooks": [{
                    "type": "command",
                    "command": format!("{} hook session-end --worker overseer", llmc_bin),
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
        "Created Claude hook settings for overseer"
    );
    Ok(())
}
