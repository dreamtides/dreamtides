use std::os::unix::process::CommandExt;
use std::process::Command;
use std::time::Duration;

use anyhow::{Context, Result, bail};

use super::super::config::{self, Config, WorkerConfig};
use super::super::tmux::sender::TmuxSender;
use super::super::tmux::session::{self, DEFAULT_SESSION_HEIGHT, DEFAULT_SESSION_WIDTH};

/// Prefix for console session names
pub const CONSOLE_PREFIX: &str = "llmc-console";

/// Runs the console command, creating a new console session and attaching to it
pub fn run_console() -> Result<()> {
    let llmc_root = config::get_llmc_root();
    if !llmc_root.exists() {
        bail!(
            "LLMC workspace not initialized. Run 'llmc init' first.\n\
             Expected workspace at: {}",
            llmc_root.display()
        );
    }

    let config_path = config::get_config_path();
    let config = Config::load(&config_path)?;

    // Find next available console number
    let session_name = find_next_console_name()?;
    println!("Creating console session: {}", session_name);

    // Create the session in the LLMC root directory (no worktree needed)
    session::create_session(
        &session_name,
        &llmc_root,
        DEFAULT_SESSION_WIDTH,
        DEFAULT_SESSION_HEIGHT,
    )
    .with_context(|| format!("Failed to create console session '{}'", session_name))?;

    // Build a default worker config from the global defaults
    let worker_config = WorkerConfig {
        model: Some(config.defaults.model.clone()),
        role_prompt: None,
        excluded_from_pool: true,
        self_review: None,
    };

    // Start Claude in the session
    let sender = TmuxSender::new();
    let claude_cmd = build_claude_command(&worker_config);

    println!("Starting Claude Code...");
    sender
        .send(&session_name, &claude_cmd)
        .with_context(|| format!("Failed to send Claude command to session '{}'", session_name))?;

    // Wait for Claude to be ready
    if let Err(e) = session::wait_for_claude_ready(&session_name, Duration::from_secs(30), false) {
        // Clean up on failure
        let _ = session::kill_session(&session_name);
        return Err(e);
    }

    // Accept bypass warning if present
    session::accept_bypass_warning(&session_name, &sender, false)?;

    println!("Console ready. Attaching to session...");

    // Attach to the session (this replaces the current process)
    let err = Command::new("tmux").arg("attach-session").arg("-t").arg(&session_name).exec();

    Err(anyhow::anyhow!("Failed to exec tmux attach-session: {}", err))
}

/// Lists all active console session names
pub fn list_console_sessions() -> Result<Vec<String>> {
    let sessions = session::list_sessions()?;
    Ok(sessions.into_iter().filter(|s| s.starts_with(CONSOLE_PREFIX)).collect())
}

/// Checks if a name refers to a console session
pub fn is_console_name(name: &str) -> bool {
    // Accept both "console1" and "llmc-console1" formats
    name.starts_with("console") || name.starts_with(CONSOLE_PREFIX)
}

/// Normalizes a console name to the full session ID format
pub fn normalize_console_name(name: &str) -> String {
    if name.starts_with(CONSOLE_PREFIX) {
        name.to_string()
    } else if let Some(num) = name.strip_prefix("console") {
        format!("{}{}", CONSOLE_PREFIX, num)
    } else {
        name.to_string()
    }
}

/// Finds the next available console number
fn find_next_console_name() -> Result<String> {
    let sessions = session::list_sessions()?;
    let mut max_num = 0;

    for session in sessions {
        if let Some(num_str) = session.strip_prefix(CONSOLE_PREFIX)
            && let Ok(num) = num_str.parse::<u32>()
        {
            max_num = max_num.max(num);
        }
    }

    Ok(format!("{}{}", CONSOLE_PREFIX, max_num + 1))
}

fn build_claude_command(config: &WorkerConfig) -> String {
    let mut cmd = String::from("claude");
    if let Some(model) = &config.model {
        cmd.push_str(&format!(" --model {}", model));
    }
    cmd.push_str(" --dangerously-skip-permissions");
    cmd
}
