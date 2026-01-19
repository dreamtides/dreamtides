use std::io::{self, BufRead};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use tokio::time::timeout;

use crate::ipc::messages::{ClaudeHookInput, HookEvent};
use crate::ipc::socket;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);

/// Handles Stop hooks - the primary mechanism for detecting task completion.
///
/// This is critical for state transitions: when Claude stops working, this
/// hook fires and triggers the transition from Working -> NeedsReview (or
/// Reviewing -> NeedsReview for self-review completion).
pub async fn run_hook_stop(worker: &str) -> Result<()> {
    let input = match read_stdin_json() {
        Ok(input) => input,
        Err(e) => {
            tracing::warn!(
                worker = worker,
                error = %e,
                "Stop hook: failed to parse stdin JSON, using defaults"
            );
            ClaudeHookInput::default()
        }
    };
    let event = HookEvent::Stop {
        worker: worker.to_string(),
        session_id: input.session_id.unwrap_or_default(),
        timestamp: get_timestamp(),
    };
    send_event_gracefully(event, "stop").await;
    Ok(())
}

/// Handles SessionStart hooks - detects when Claude is ready to work.
///
/// Fired when a worker's Claude session starts up successfully.
pub async fn run_hook_session_start(worker: &str) -> Result<()> {
    let input = match read_stdin_json() {
        Ok(input) => input,
        Err(e) => {
            tracing::warn!(
                worker = worker,
                error = %e,
                "SessionStart hook: failed to parse stdin JSON, using defaults"
            );
            ClaudeHookInput::default()
        }
    };
    let event = HookEvent::SessionStart {
        worker: worker.to_string(),
        session_id: input.session_id.unwrap_or_default(),
        timestamp: get_timestamp(),
    };
    send_event_gracefully(event, "session_start").await;
    Ok(())
}

/// Handles SessionEnd hooks - detects when Claude exits.
///
/// Fired when a worker's Claude session ends (crash, shutdown, etc).
pub async fn run_hook_session_end(worker: &str, cli_reason: &str) -> Result<()> {
    let input = match read_stdin_json() {
        Ok(input) => input,
        Err(e) => {
            tracing::warn!(
                worker = worker,
                error = %e,
                "SessionEnd hook: failed to parse stdin JSON, using defaults"
            );
            ClaudeHookInput::default()
        }
    };
    let reason = input.reason.unwrap_or_else(|| cli_reason.to_string());
    let event =
        HookEvent::SessionEnd { worker: worker.to_string(), reason, timestamp: get_timestamp() };
    send_event_gracefully(event, "session_end").await;
    Ok(())
}

/// Sends a hook event to the daemon without printing errors to stderr.
///
/// Claude Code interprets any stderr output from hooks as an error, which
/// causes confusing "hook error" messages. Instead, we log issues via tracing
/// (which goes to the log file) and silently return on expected conditions
/// like the daemon not running.
async fn send_event_gracefully(event: HookEvent, hook_name: &str) {
    let socket_path = socket::get_socket_path();

    if !socket_path.exists() {
        tracing::debug!(
            hook = hook_name,
            socket_path = %socket_path.display(),
            "Hook skipped: daemon not running (socket not found)"
        );
        return;
    }

    let send_future = socket::send_event(&socket_path, event);
    match timeout(CONNECT_TIMEOUT, send_future).await {
        Ok(Ok(response)) => {
            if !response.success {
                let err = response.error.as_deref().unwrap_or("unknown error");
                tracing::warn!(hook = hook_name, error = err, "Hook event rejected by daemon");
            } else {
                tracing::debug!(hook = hook_name, "Hook event sent successfully");
            }
        }
        Ok(Err(e)) => {
            tracing::warn!(
                hook = hook_name,
                error = %e,
                "Hook failed to send event to daemon"
            );
        }
        Err(_) => {
            tracing::warn!(
                hook = hook_name,
                timeout_secs = CONNECT_TIMEOUT.as_secs(),
                "Hook timed out connecting to daemon"
            );
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
        return Ok(ClaudeHookInput::default());
    }
    serde_json::from_str(&input).context("Failed to parse hook input JSON from stdin")
}
