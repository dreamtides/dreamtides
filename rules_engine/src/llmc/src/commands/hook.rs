use std::io::{self, BufRead};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use tokio::runtime::Builder;

use crate::ipc::messages::{ClaudeHookInput, HookEvent};
use crate::ipc::socket;

pub fn run_hook_stop(worker: &str) -> Result<()> {
    let input = read_stdin_json()?;
    let event = HookEvent::Stop {
        worker: worker.to_string(),
        session_id: input.session_id.unwrap_or_default(),
        timestamp: get_timestamp(),
    };
    let socket_path = socket::get_socket_path();
    let rt = Builder::new_current_thread().enable_all().build()?;
    let response = rt.block_on(socket::send_event(&socket_path, event))?;
    if !response.success
        && let Some(err) = response.error
    {
        eprintln!("Hook error: {}", err);
    }
    Ok(())
}

pub fn run_hook_session_start(worker: &str) -> Result<()> {
    let input = read_stdin_json()?;
    let event = HookEvent::SessionStart {
        worker: worker.to_string(),
        session_id: input.session_id.unwrap_or_default(),
        timestamp: get_timestamp(),
    };
    let socket_path = socket::get_socket_path();
    let rt = Builder::new_current_thread().enable_all().build()?;
    let response = rt.block_on(socket::send_event(&socket_path, event))?;
    if !response.success
        && let Some(err) = response.error
    {
        eprintln!("Hook error: {}", err);
    }
    Ok(())
}

pub fn run_hook_session_end(worker: &str, cli_reason: &str) -> Result<()> {
    let input = read_stdin_json()?;
    let reason = input.reason.unwrap_or_else(|| cli_reason.to_string());
    let event =
        HookEvent::SessionEnd { worker: worker.to_string(), reason, timestamp: get_timestamp() };
    let socket_path = socket::get_socket_path();
    let rt = Builder::new_current_thread().enable_all().build()?;
    let response = rt.block_on(socket::send_event(&socket_path, event))?;
    if !response.success
        && let Some(err) = response.error
    {
        eprintln!("Hook error: {}", err);
    }
    Ok(())
}

pub fn run_hook_post_bash(worker: &str) -> Result<()> {
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
    let socket_path = socket::get_socket_path();
    let rt = Builder::new_current_thread().enable_all().build()?;
    let response = rt.block_on(socket::send_event(&socket_path, event))?;
    if !response.success
        && let Some(err) = response.error
    {
        eprintln!("Hook error: {}", err);
    }
    Ok(())
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
