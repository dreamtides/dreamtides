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
        println!("      [verbose] Claude command sent, waiting for ready state...");
    }

    wait_for_claude_ready(name, Duration::from_secs(30), verbose)?;

    if verbose {
        println!("      [verbose] Claude is ready, checking for bypass warning...");
    }

    accept_bypass_warning(name, &sender, verbose)?;

    if verbose {
        println!("      [verbose] Sending /clear command...");
    }

    sender
        .send(name, "/clear")
        .with_context(|| format!("Failed to send /clear to session '{}'", name))?;
    thread::sleep(Duration::from_millis(500));

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

fn wait_for_claude_ready(session: &str, timeout: Duration, verbose: bool) -> Result<()> {
    const POLL_INTERVAL_MS: u64 = 500;
    let start = std::time::Instant::now();
    let mut poll_count = 0;

    while start.elapsed() < timeout {
        thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
        poll_count += 1;

        if verbose {
            println!(
                "        [verbose] Poll #{}: Checking Claude readiness ({}s elapsed)",
                poll_count,
                start.elapsed().as_secs()
            );
        }

        if let Ok(output) = capture_pane(session, 50) {
            if verbose {
                println!("        [verbose] Captured {} lines of output", output.lines().count());
                println!("        [verbose] Last 5 lines:");
                for line in output.lines().rev().take(5) {
                    println!("        [verbose]   | {}", line);
                }
            }

            // Check for the '>' prompt (Claude is ready)
            if output.lines().rev().take(5).any(|line| {
                let trimmed = line.trim_start();
                trimmed.starts_with("> ") || trimmed == ">" || trimmed.starts_with("❯")
            }) {
                if verbose {
                    println!("        [verbose] Found '>' prompt - Claude is ready!");
                }
                return Ok(());
            }

            // Check for bypass permissions prompt (Claude is waiting for confirmation)
            let lower = output.to_lowercase();
            if lower.contains("bypass") && lower.contains("permissions") {
                if verbose {
                    println!(
                        "        [verbose] Found bypass permissions prompt - Claude is ready (at confirmation)"
                    );
                }
                return Ok(());
            }

            if let Ok(command) = get_pane_command(session) {
                if verbose {
                    println!("        [verbose] Pane command: '{}'", command);
                    println!(
                        "        [verbose] Is Claude process: {}",
                        is_claude_process(&command)
                    );
                }

                if !is_claude_process(&command) {
                    bail!(
                        "Claude process not found in session '{}', got command: {}\n\
                         Last captured output:\n{}",
                        session,
                        command,
                        output
                            .lines()
                            .rev()
                            .take(20)
                            .collect::<Vec<_>>()
                            .iter()
                            .rev()
                            .cloned()
                            .collect::<Vec<_>>()
                            .join("\n")
                    );
                }
            } else if verbose {
                println!("        [verbose] Failed to get pane command");
            }
        } else if verbose {
            println!("        [verbose] Failed to capture pane output");
        }
    }

    if verbose {
        println!("        [verbose] Timeout reached after {} polls", poll_count);
    }

    bail!("Claude did not become ready after {} seconds", timeout.as_secs())
}

fn accept_bypass_warning(session: &str, sender: &TmuxSender, verbose: bool) -> Result<()> {
    thread::sleep(Duration::from_millis(500));
    if let Ok(output) = capture_pane(session, 50) {
        let lower = output.to_lowercase();
        let has_bypass_warning = lower.contains("bypass")
            || lower.contains("dangerously")
            || lower.contains("skip-permissions")
            || lower.contains("skip permissions");

        if verbose {
            println!(
                "        [verbose] Checking for bypass warning... found: {}",
                has_bypass_warning
            );
        }

        if has_bypass_warning {
            if verbose {
                println!("        [verbose] Accepting bypass warning (Down + Enter)");
            }
            sender.send_keys_raw(session, "Down")?;
            thread::sleep(Duration::from_millis(200));
            sender.send_keys_raw(session, "Enter")?;
            thread::sleep(Duration::from_millis(500));
        }
    } else if verbose {
        println!("        [verbose] Could not capture pane for bypass warning check");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_any_llmc_sessions_running() {
        let result = any_llmc_sessions_running();
        assert!(result.is_ok());
    }
}
