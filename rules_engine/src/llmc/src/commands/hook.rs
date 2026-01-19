use std::io::{self, BufRead};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use tokio::time::timeout;

use crate::ipc::messages::{ClaudeHookInput, HookEvent};
use crate::ipc::socket;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);

pub async fn run_hook_stop(worker: &str) -> Result<()> {
    let input = read_stdin_json()?;
    let event = HookEvent::Stop {
        worker: worker.to_string(),
        session_id: input.session_id.unwrap_or_default(),
        timestamp: get_timestamp(),
    };
    send_event_gracefully(event, "stop").await;
    Ok(())
}

pub async fn run_hook_session_start(worker: &str) -> Result<()> {
    let input = read_stdin_json()?;
    let event = HookEvent::SessionStart {
        worker: worker.to_string(),
        session_id: input.session_id.unwrap_or_default(),
        timestamp: get_timestamp(),
    };
    send_event_gracefully(event, "session_start").await;
    Ok(())
}

pub async fn run_hook_session_end(worker: &str, cli_reason: &str) -> Result<()> {
    let input = read_stdin_json()?;
    let reason = input.reason.unwrap_or_else(|| cli_reason.to_string());
    let event =
        HookEvent::SessionEnd { worker: worker.to_string(), reason, timestamp: get_timestamp() };
    send_event_gracefully(event, "session_end").await;
    Ok(())
}

pub async fn run_hook_post_bash(worker: &str) -> Result<()> {
    let input = read_stdin_json()?;
    let command = input
        .tool_input
        .as_ref()
        .and_then(|v: &serde_json::Value| v.get("command"))
        .and_then(|v: &serde_json::Value| v.as_str())
        .map(String::from);
    let event = HookEvent::PostBash {
        worker: worker.to_string(),
        command,
        exit_code: None,
        timestamp: get_timestamp(),
    };
    send_event_gracefully(event, "post_bash").await;
    Ok(())
}

async fn send_event_gracefully(event: HookEvent, hook_name: &str) {
    let socket_path = socket::get_socket_path();

    if !socket_path.exists() {
        eprintln!("Hook {hook_name}: llmc daemon not running (socket not found)");
        return;
    }

    let send_future = socket::send_event(&socket_path, event);
    match timeout(CONNECT_TIMEOUT, send_future).await {
        Ok(Ok(response)) => {
            if !response.success
                && let Some(err) = response.error
            {
                eprintln!("Hook {hook_name} error: {err}");
            }
        }
        Ok(Err(e)) => {
            eprintln!("Hook {hook_name} failed: {e}");
        }
        Err(_) => {
            eprintln!("Hook {hook_name} timed out connecting to daemon");
        }
    }
}

fn get_timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn read_stdin_json() -> Result<ClaudeHookInput> {
    let stdin = io::stdin();
    let mut input = String::new();
    for line in stdin.lock().lines() {
        input.push_str(&line?);
    }
    if input.is_empty() {
        return Ok(ClaudeHookInput {
            session_id: None,
            transcript_path: None,
            cwd: None,
            hook_event_name: None,
            tool_name: None,
            tool_input: None,
            tool_response: None,
            reason: None,
        });
    }
    serde_json::from_str(&input).context("Failed to parse hook input JSON from stdin")
}
