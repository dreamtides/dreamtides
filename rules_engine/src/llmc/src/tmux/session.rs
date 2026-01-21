use std::path::Path;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use tmux_interface::{
    CapturePane, HasSession, KillSession, ListSessions, NewSession, SetEnvironment, Tmux,
};

use crate::config::WorkerConfig;
use crate::tmux::sender::TmuxSender;
/// Default TMUX session width (wide terminal to prevent message truncation)
pub const DEFAULT_SESSION_WIDTH: u32 = 500;
/// Default TMUX session height
pub const DEFAULT_SESSION_HEIGHT: u32 = 100;
pub fn create_session(name: &str, cwd: &Path, width: u32, height: u32) -> Result<()> {
    if session_exists(name) {
        bail!("Session '{}' already exists", name);
    }
    let cwd_str = cwd.to_string_lossy();
    tracing::info!(
        operation = "session_create", session = name, working_directory = % cwd_str,
        width, height, "Creating TMUX session with working directory"
    );
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
    tracing::debug!(
        operation = "session_create",
        session = name,
        result = "success",
        "TMUX session created successfully"
    );
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
        .output()
        .map(|output| output.success())
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
/// Starts a complete worker session with Claude Code.
///
/// The daemon relies on the SessionStart hook to transition the worker from
/// Offline to Idle when Claude is ready.
pub fn start_worker_session(
    name: &str,
    worktree: &Path,
    config: &WorkerConfig,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("      [verbose] Creating TMUX session '{}'", name);
        println!("      [verbose] Working directory: {}", worktree.display());
        println!(
            "      [verbose] Session size: {}x{}",
            DEFAULT_SESSION_WIDTH, DEFAULT_SESSION_HEIGHT
        );
    }
    create_session(name, worktree, DEFAULT_SESSION_WIDTH, DEFAULT_SESSION_HEIGHT)?;
    if verbose {
        println!("      [verbose] TMUX session created successfully");
    }
    let llmc_root = crate::config::get_llmc_root();
    set_env(name, "LLMC_WORKER", name)?;
    set_env(name, "LLMC_ROOT", llmc_root.to_str().unwrap())?;
    if verbose {
        println!("      [verbose] Environment variables set");
    }
    thread::sleep(Duration::from_millis(500));
    let sender = TmuxSender::new();
    let claude_cmd = build_claude_command(config);
    if verbose {
        println!("      [verbose] Sending Claude command: {}", claude_cmd);
    }
    sender
        .send(name, &claude_cmd)
        .with_context(|| format!("Failed to send Claude command to session '{}'", name))?;
    if verbose {
        println!("      [verbose] Worker session startup complete");
    }
    Ok(())
}
/// Checks if any LLMC sessions are running
pub fn any_llmc_sessions_running() -> Result<bool> {
    let sessions = list_sessions()?;
    Ok(sessions.iter().any(|s| s.starts_with("llmc-")))
}
fn build_claude_command(config: &WorkerConfig) -> String {
    let mut cmd = String::from("claude");
    if let Some(model) = &config.model {
        cmd.push_str(&format!(" --model {}", model));
    }
    cmd.push_str(" --dangerously-skip-permissions");
    cmd
}
