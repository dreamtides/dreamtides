use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
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
pub async fn run_hook_stop(worker: &str, socket_override: Option<&Path>) -> Result<()> {
    let start_time = std::time::Instant::now();
    let timestamp = get_timestamp();
    tracing::info!(worker = worker, timestamp = timestamp, "Stop hook invoked");
    let input = match read_stdin_json() {
        Ok(input) => {
            tracing::debug!(
                worker = worker,
                session_id = ?input.session_id,
                "Stop hook: parsed stdin JSON successfully"
            );
            input
        }
        Err(e) => {
            tracing::info!(
                worker = worker,
                error = %e,
                "Stop hook: failed to parse stdin JSON, using defaults (this is normal \
                 if Claude Code didn't provide JSON input)"
            );
            ClaudeHookInput::default()
        }
    };
    let session_id = input.session_id.unwrap_or_default();
    let transcript_path = input.transcript_path.clone();
    let event = HookEvent::Stop {
        worker: worker.to_string(),
        session_id: session_id.clone(),
        timestamp,
        transcript_path,
    };
    send_event_gracefully(event, "stop", worker, &session_id, start_time, socket_override).await;
    Ok(())
}

/// Handles SessionStart hooks - detects when Claude is ready to work.
///
/// Fired when a worker's Claude session starts up successfully.
pub async fn run_hook_session_start(worker: &str, socket_override: Option<&Path>) -> Result<()> {
    let start_time = std::time::Instant::now();
    let timestamp = get_timestamp();
    tracing::info!(worker = worker, timestamp = timestamp, "SessionStart hook invoked");
    let input = match read_stdin_json() {
        Ok(input) => input,
        Err(e) => {
            tracing::info!(
                worker = worker,
                error = %e,
                "SessionStart hook: failed to parse stdin JSON, using defaults (this is \
                 normal if Claude Code didn't provide JSON input)"
            );
            ClaudeHookInput::default()
        }
    };
    let session_id = input.session_id.unwrap_or_default();
    let transcript_path = input.transcript_path.clone();
    let event = HookEvent::SessionStart {
        worker: worker.to_string(),
        session_id: session_id.clone(),
        timestamp,
        transcript_path,
    };
    send_event_gracefully(event, "session_start", worker, &session_id, start_time, socket_override)
        .await;
    Ok(())
}

/// Handles SessionEnd hooks - detects when Claude exits.
///
/// Fired when a worker's Claude session ends (crash, shutdown, etc).
pub async fn run_hook_session_end(
    worker: &str,
    cli_reason: &str,
    socket_override: Option<&Path>,
) -> Result<()> {
    let start_time = std::time::Instant::now();
    let timestamp = get_timestamp();
    tracing::info!(
        worker = worker,
        cli_reason = cli_reason,
        timestamp = timestamp,
        "SessionEnd hook invoked"
    );
    let input = match read_stdin_json() {
        Ok(input) => input,
        Err(e) => {
            tracing::info!(
                worker = worker,
                error = %e,
                "SessionEnd hook: failed to parse stdin JSON, using defaults (this is \
                 normal if Claude Code didn't provide JSON input)"
            );
            ClaudeHookInput::default()
        }
    };
    let reason = input.reason.unwrap_or_else(|| cli_reason.to_string());
    let transcript_path = input.transcript_path.clone();
    let event = HookEvent::SessionEnd {
        worker: worker.to_string(),
        reason: reason.clone(),
        timestamp,
        transcript_path,
    };
    send_event_gracefully(event, "session_end", worker, &reason, start_time, socket_override).await;
    Ok(())
}

/// Sends a hook event to the daemon without printing errors to stderr.
///
/// Claude Code interprets any stderr output from hooks as an error, which
/// causes confusing "hook error" messages. Instead, we log issues via tracing
/// (which goes to the log file) and silently return on expected conditions
/// like the daemon not running.
async fn send_event_gracefully(
    event: HookEvent,
    hook_name: &str,
    worker: &str,
    context: &str,
    start_time: std::time::Instant,
    socket_override: Option<&Path>,
) {
    let socket_path = socket_override.map(PathBuf::from).unwrap_or_else(socket::get_socket_path);

    if !socket_path.exists() {
        let elapsed_ms = start_time.elapsed().as_millis();
        tracing::info!(
            hook = hook_name,
            worker = worker,
            context = context,
            socket_path = %socket_path.display(),
            elapsed_ms = elapsed_ms,
            "Hook skipped: daemon not running (socket not found) - this is expected \
             when llmc is not active"
        );
        return;
    }

    let send_future = socket::send_event(&socket_path, event);
    match timeout(CONNECT_TIMEOUT, send_future).await {
        Ok(Ok(response)) => {
            let elapsed_ms = start_time.elapsed().as_millis();
            if !response.success {
                let err = response.error.as_deref().unwrap_or("unknown error");
                tracing::error!(
                    hook = hook_name,
                    worker = worker,
                    context = context,
                    error = err,
                    elapsed_ms = elapsed_ms,
                    "Hook event rejected by daemon - daemon received event but could not \
                     process it. Check daemon logs for details."
                );
            } else {
                tracing::info!(
                    hook = hook_name,
                    worker = worker,
                    context = context,
                    elapsed_ms = elapsed_ms,
                    "Hook event sent successfully"
                );
            }
        }
        Ok(Err(e)) => {
            let elapsed_ms = start_time.elapsed().as_millis();
            tracing::error!(
                hook = hook_name,
                worker = worker,
                context = context,
                error = %e,
                elapsed_ms = elapsed_ms,
                socket_path = %socket_path.display(),
                "Hook failed to send event to daemon - socket exists but connection failed. \
                 Daemon may have crashed. Worker state changes may not be detected."
            );
        }
        Err(_) => {
            let elapsed_ms = start_time.elapsed().as_millis();
            tracing::error!(
                hook = hook_name,
                worker = worker,
                context = context,
                timeout_secs = CONNECT_TIMEOUT.as_secs(),
                elapsed_ms = elapsed_ms,
                socket_path = %socket_path.display(),
                "Hook timed out connecting to daemon - daemon may be overloaded or frozen. \
                 Worker state changes may not be detected. Consider restarting with 'llmc down && llmc up'."
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
