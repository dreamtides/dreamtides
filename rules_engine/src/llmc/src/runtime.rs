use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

use anyhow::{Context, Result};
use serde_json::Value;

use crate::state::{ClaudeConfig, Runtime};

#[derive(Debug)]
pub struct RuntimeOutcome {
    pub status: ExitStatus,
    pub pid: Option<u32>,
}

/// Run a supported runtime with the provided prompt.
pub fn run_runtime(
    runtime: Runtime,
    prompt: &str,
    repo_root: &Path,
    worktree: &Path,
    background: bool,
    claude_config: Option<ClaudeConfig>,
) -> Result<RuntimeOutcome> {
    println!("=== LLMC PROMPT ===");
    println!("{prompt}");
    println!("=== END PROMPT ===\n");

    match runtime {
        Runtime::Codex => self::run_codex(prompt, repo_root, worktree, background),
        Runtime::Claude => self::run_claude(prompt, repo_root, worktree, background, claude_config),
        _ => Err(anyhow::anyhow!("Runtime {runtime:?} is not supported yet")),
    }
}

/// Process Claude's stream-json output and print readable text.
fn process_claude_stream(reader: impl BufRead) -> Result<()> {
    for line in reader.lines() {
        let line = line.context("Failed to read line from claude output")?;
        if line.trim().is_empty() {
            continue;
        }

        let parsed: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => {
                // If not valid JSON, print as-is (might be error output)
                eprintln!("{line}");
                continue;
            }
        };

        // Extract event type
        let event_type = parsed.get("type").and_then(|v| v.as_str());

        match event_type {
            Some("stream_event") => {
                if let Some(event) = parsed.get("event") {
                    process_stream_event(event)?;
                }
            }
            Some("message_start") => {
                // Silence message_start events
            }
            Some("message_stop") => {
                // Silence message_stop events
            }
            Some("error") => {
                if let Some(error_msg) = parsed.get("error").and_then(|e| e.get("message")) {
                    eprintln!("\n[ERROR] {}", error_msg.as_str().unwrap_or("Unknown error"));
                }
            }
            _ => {
                // Print unknown event types for debugging
                eprintln!("[DEBUG] {}", serde_json::to_string_pretty(&parsed)?);
            }
        }
    }

    Ok(())
}

fn process_stream_event(event: &Value) -> Result<()> {
    let event_type = event.get("type").and_then(|v| v.as_str());

    match event_type {
        Some("content_block_start") => {
            if let Some(content_block) = event.get("content_block") {
                let block_type = content_block.get("type").and_then(|v| v.as_str());
                match block_type {
                    Some("thinking") => {
                        print!("\n[Thinking...] ");
                        std::io::Write::flush(&mut std::io::stdout())?;
                    }
                    Some("tool_use") => {
                        if let Some(name) = content_block.get("name").and_then(|v| v.as_str()) {
                            print!("\n[Tool: {name}] ");
                            std::io::Write::flush(&mut std::io::stdout())?;
                        }
                    }
                    _ => {}
                }
            }
        }
        Some("content_block_delta") => {
            if let Some(delta) = event.get("delta") {
                let delta_type = delta.get("type").and_then(|v| v.as_str());
                match delta_type {
                    Some("text_delta") => {
                        if let Some(text) = delta.get("text").and_then(|v| v.as_str()) {
                            print!("{text}");
                            std::io::Write::flush(&mut std::io::stdout())?;
                        }
                    }
                    Some("thinking_delta") => {
                        // Optionally print thinking text
                        if let Some(text) = delta.get("thinking").and_then(|v| v.as_str()) {
                            print!("{text}");
                            std::io::Write::flush(&mut std::io::stdout())?;
                        }
                    }
                    Some("input_json_delta") => {
                        // Tool input - could optionally display
                    }
                    _ => {}
                }
            }
        }
        Some("content_block_stop") => {
            println!(); // Newline after block completes
        }
        _ => {}
    }

    Ok(())
}

fn run_codex(
    prompt: &str,
    repo_root: &Path,
    worktree: &Path,
    background: bool,
) -> Result<RuntimeOutcome> {
    let mut command = Command::new("codex");
    command
        .arg("-a")
        .arg("never")
        .arg("exec")
        .arg("-C")
        .arg(worktree)
        .arg("--sandbox")
        .arg("workspace-write")
        .arg("--add-dir")
        .arg(repo_root.join(".git"))
        .arg(prompt)
        .env("WORKTREE", worktree)
        .current_dir(worktree);

    if background {
        command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    } else {
        command.stdin(Stdio::inherit()).stdout(Stdio::inherit()).stderr(Stdio::inherit());
    }

    let mut child =
        command.spawn().with_context(|| format!("Failed to spawn codex in {worktree:?}"))?;
    let pid = Some(child.id());
    let status =
        child.wait().with_context(|| format!("Failed to wait for codex in {worktree:?}"))?;

    Ok(RuntimeOutcome { status, pid })
}

fn run_claude(
    prompt: &str,
    _repo_root: &Path,
    worktree: &Path,
    background: bool,
    claude_config: Option<ClaudeConfig>,
) -> Result<RuntimeOutcome> {
    let config = claude_config.unwrap_or_default();
    let mut command = Command::new("claude");

    command
        .arg("-p")
        .arg(prompt)
        .arg("--verbose")
        .arg("--output-format")
        .arg("stream-json")
        .arg("--replay-user-messages")
        .arg("--include-partial-messages")
        .arg("--permission-mode")
        .arg("bypassPermissions")
        .env("WORKTREE", worktree)
        .current_dir(worktree);

    if let Some(model) = &config.model {
        command.arg("--model").arg(model);
    }

    if config.no_thinking {
        command.arg("--no-thinking");
    }

    if let Some(sandbox) = &config.sandbox {
        command.arg("--sandbox").arg(sandbox);
    }

    if config.skip_permissions {
        command.arg("--dangerously-skip-permissions");
    }

    if let Some(allowed_tools) = &config.allowed_tools {
        command.arg("--allowedTools").arg(allowed_tools);
    }

    for mcp_config in &config.mcp_config {
        command.arg("--mcp-config").arg(mcp_config);
    }

    if background {
        command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
        let mut child =
            command.spawn().with_context(|| format!("Failed to spawn claude in {worktree:?}"))?;
        let pid = Some(child.id());
        let status =
            child.wait().with_context(|| format!("Failed to wait for claude in {worktree:?}"))?;
        Ok(RuntimeOutcome { status, pid })
    } else {
        // Process stdout to convert JSON stream to readable text
        command.stdin(Stdio::inherit()).stdout(Stdio::piped()).stderr(Stdio::inherit());

        let mut child =
            command.spawn().with_context(|| format!("Failed to spawn claude in {worktree:?}"))?;
        let pid = Some(child.id());

        // Process the stdout stream
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            if let Err(e) = process_claude_stream(reader) {
                eprintln!("Warning: error processing claude stream: {e}");
            }
        }

        let status =
            child.wait().with_context(|| format!("Failed to wait for claude in {worktree:?}"))?;

        Ok(RuntimeOutcome { status, pid })
    }
}
