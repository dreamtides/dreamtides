//! Remediation execution and logging for overseer mode.
//!
//! This module handles executing remediation prompts via the overseer's Claude
//! Code session, monitoring for completion, and logging the entire remediation
//! process.

#![allow(dead_code)]

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use chrono::Utc;
use tokio::runtime::Handle;
use tokio::sync::mpsc::Receiver;
use tokio::task::block_in_place;
use tokio::time;
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::ipc::messages::{HookEvent, HookMessage};
use crate::ipc::socket;
use crate::overseer_mode::overseer_session;

const REMEDIATION_TIMEOUT_SECS: u64 = 1800;
const COMPLETION_POLL_INTERVAL_SECS: u64 = 5;
const POST_PROMPT_DELAY_SECS: u64 = 3;

/// Executes the remediation process.
///
/// Steps:
/// 1. Clear the overseer Claude session context
/// 2. Start an IPC listener to receive completion events
/// 3. Send the remediation prompt
/// 4. Wait for completion (Stop event or timeout)
/// 5. Log the entire remediation to a file
pub fn execute_remediation(
    prompt: &str,
    config: &Config,
    shutdown_count: &Arc<AtomicU32>,
) -> Result<()> {
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let log_path = remediation_log_path(&timestamp);
    let mut log_file = create_remediation_log(&log_path, prompt)?;

    info!(log_path = %log_path.display(), "Starting remediation");
    println!("Remediation log: {}", log_path.display());

    let start_time = Instant::now();
    overseer_session::clear_overseer_session().context("Failed to clear overseer session")?;
    write_log_entry(&mut log_file, "Cleared overseer session")?;

    let ipc_receiver = start_ipc_listener_for_remediation()?;
    write_log_entry(&mut log_file, "Started IPC listener for completion detection")?;

    std::thread::sleep(Duration::from_secs(POST_PROMPT_DELAY_SECS));

    info!(prompt_length = prompt.len(), "Sending remediation prompt");
    overseer_session::send_to_overseer(prompt).context("Failed to send remediation prompt")?;
    write_log_entry(&mut log_file, &format!("Sent remediation prompt ({} chars)", prompt.len()))?;

    println!("Waiting for remediation to complete...");
    let completion_result = block_in_place(|| {
        Handle::current().block_on(async {
            wait_for_completion(ipc_receiver, shutdown_count, &mut log_file).await
        })
    });

    let elapsed = start_time.elapsed();
    capture_and_log_output(&mut log_file)?;

    match completion_result {
        Ok(true) => {
            write_log_entry(
                &mut log_file,
                &format!("Remediation completed successfully in {:.1}s", elapsed.as_secs_f64()),
            )?;
            info!(elapsed_secs = elapsed.as_secs(), "Remediation completed successfully");
            println!("✓ Remediation completed in {:.1}s", elapsed.as_secs_f64());
        }
        Ok(false) => {
            write_log_entry(&mut log_file, "Remediation interrupted by shutdown signal")?;
            info!("Remediation interrupted by shutdown signal");
            println!("⚠ Remediation interrupted");
        }
        Err(e) => {
            write_log_entry(&mut log_file, &format!("Remediation failed: {}", e))?;
            error!(error = %e, "Remediation failed");
            println!("⚠ Remediation issue: {}", e);
        }
    }

    let _ = config;
    Ok(())
}

/// Returns the path for a remediation log file.
pub fn remediation_log_path(timestamp: &str) -> PathBuf {
    let logs_dir = crate::config::get_llmc_root().join("logs");
    logs_dir.join(format!("remediation_{}.txt", timestamp))
}

/// Starts an IPC listener for receiving remediation completion events.
fn start_ipc_listener_for_remediation() -> Result<Receiver<HookMessage>> {
    let socket_path = socket::get_socket_path();

    if socket_path.exists() {
        debug!(path = %socket_path.display(), "Removing existing socket");
        fs::remove_file(&socket_path).context("Failed to remove existing socket")?;
    }

    info!(path = %socket_path.display(), "Starting IPC listener for remediation");
    socket::spawn_ipc_listener(socket_path).context("Failed to spawn IPC listener")
}

/// Waits for a Stop event from the overseer or timeout.
async fn wait_for_completion(
    mut receiver: Receiver<HookMessage>,
    shutdown_count: &Arc<AtomicU32>,
    log_file: &mut File,
) -> Result<bool> {
    let deadline = Instant::now() + Duration::from_secs(REMEDIATION_TIMEOUT_SECS);

    loop {
        if shutdown_count.load(Ordering::SeqCst) > 0 {
            return Ok(false);
        }

        if Instant::now() > deadline {
            bail!("Remediation timed out after {} seconds", REMEDIATION_TIMEOUT_SECS);
        }

        let timeout_duration = Duration::from_secs(COMPLETION_POLL_INTERVAL_SECS);
        match time::timeout(timeout_duration, receiver.recv()).await {
            Ok(Some(msg)) => {
                let event_desc = format!("{:?}", msg.event);
                write_log_entry(log_file, &format!("Received hook event: {}", event_desc))
                    .unwrap_or_else(|e| warn!(error = %e, "Failed to write log entry"));

                info!(event = ?msg.event, "Received hook event during remediation");

                if let HookEvent::Stop { worker, .. } = &msg.event
                    && worker == "overseer"
                {
                    info!("Overseer stop event received - remediation complete");
                    return Ok(true);
                }

                if let HookEvent::SessionEnd { worker, reason, .. } = &msg.event
                    && worker == "overseer"
                {
                    warn!(reason = %reason, "Overseer session ended during remediation");
                    bail!("Overseer session ended unexpectedly: {}", reason);
                }
            }
            Ok(None) => {
                bail!("IPC channel closed unexpectedly");
            }
            Err(_) => {
                debug!("Still waiting for remediation completion...");
            }
        }
    }
}

/// Creates the remediation log file with initial content.
fn create_remediation_log(path: &PathBuf, prompt: &str) -> Result<File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Failed to create logs directory")?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .with_context(|| format!("Failed to create remediation log at {}", path.display()))?;

    writeln!(file, "# Remediation Log")?;
    writeln!(file, "Started: {}", Utc::now().format("%Y-%m-%d %H:%M:%S UTC"))?;
    writeln!(file)?;
    writeln!(file, "## Remediation Prompt")?;
    writeln!(file)?;
    writeln!(file, "```")?;
    writeln!(file, "{}", prompt)?;
    writeln!(file, "```")?;
    writeln!(file)?;
    writeln!(file, "## Events")?;
    writeln!(file)?;

    file.flush()?;
    Ok(file)
}

/// Writes a timestamped log entry.
fn write_log_entry(file: &mut File, message: &str) -> Result<()> {
    let timestamp = Utc::now().format("%H:%M:%S");
    writeln!(file, "[{}] {}", timestamp, message)?;
    file.flush()?;
    Ok(())
}

/// Captures the final overseer output and writes it to the log.
fn capture_and_log_output(log_file: &mut File) -> Result<()> {
    writeln!(log_file)?;
    writeln!(log_file, "## Session Output (final 200 lines)")?;
    writeln!(log_file)?;

    match overseer_session::capture_overseer_output(200) {
        Ok(output) => {
            writeln!(log_file, "```")?;
            writeln!(log_file, "{}", output)?;
            writeln!(log_file, "```")?;
        }
        Err(e) => {
            warn!(error = %e, "Failed to capture overseer output");
            writeln!(log_file, "(Failed to capture: {})", e)?;
        }
    }

    log_file.flush()?;
    Ok(())
}
