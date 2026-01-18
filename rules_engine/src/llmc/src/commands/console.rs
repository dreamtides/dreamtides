use std::os::unix::process::CommandExt;
use std::process::Command;
use std::time::Duration;

use anyhow::{Context, Result, bail};

use crate::llmc::config::{self, Config, WorkerConfig};
use crate::llmc::tmux::sender::TmuxSender;
use crate::llmc::tmux::session::{self, DEFAULT_SESSION_HEIGHT, DEFAULT_SESSION_WIDTH};
/// Prefix for console session names
pub const CONSOLE_PREFIX: &str = "llmc-console";
/// Runs the console command, creating a new console session or attaching to an
/// existing one
pub fn run_console(name: Option<&str>) -> Result<()> {
    let llmc_root = config::get_llmc_root();
    if !llmc_root.exists() {
        bail!(
            "LLMC workspace not initialized. Run 'llmc init' first.\n\
             Expected workspace at: {}",
            llmc_root.display()
        );
    }
    if let Some(requested_name) = name {
        let session_name = normalize_console_name(requested_name);
        if session::session_exists(&session_name) {
            println!("Attaching to existing console session: {}", session_name);
            let err =
                Command::new("tmux").arg("attach-session").arg("-t").arg(&session_name).exec();
            return Err(anyhow::anyhow!("Failed to exec tmux attach-session: {}", err));
        }
    }
    let config_path = config::get_config_path();
    let config = Config::load(&config_path)?;
    let working_dir = std::path::PathBuf::from(&config.repo.source);
    if !working_dir.exists() {
        bail!(
            "Master repository not found at: {}\n\
             Check the repo.source setting in config.toml",
            working_dir.display()
        );
    }
    let session_name = if let Some(requested_name) = name {
        normalize_console_name(requested_name)
    } else {
        find_next_console_name()?
    };
    println!("Creating console session: {}", session_name);
    println!("Working directory: {}", working_dir.display());
    session::create_session(
        &session_name,
        &working_dir,
        DEFAULT_SESSION_WIDTH,
        DEFAULT_SESSION_HEIGHT,
    )
    .with_context(|| format!("Failed to create console session '{}'", session_name))?;
    let worker_config = WorkerConfig {
        model: Some(config.defaults.model.clone()),
        role_prompt: None,
        excluded_from_pool: true,
        self_review: None,
    };
    let sender = TmuxSender::new();
    let claude_cmd = build_claude_command(&worker_config);
    println!("Starting Claude Code...");
    sender
        .send(&session_name, &claude_cmd)
        .with_context(|| format!("Failed to send Claude command to session '{}'", session_name))?;
    if let Err(e) = session::wait_for_claude_ready(&session_name, Duration::from_secs(30), false) {
        let _ = session::kill_session(&session_name);
        return Err(e);
    }
    session::accept_bypass_warning(&session_name, &sender, false)?;
    println!("Console ready. Attaching to session...");
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
