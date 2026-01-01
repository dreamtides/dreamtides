use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

use anyhow::{Context, Result};

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

    command.arg("-p").arg(prompt).arg("--verbose").env("WORKTREE", worktree).current_dir(worktree);

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
    } else {
        command.stdin(Stdio::inherit()).stdout(Stdio::inherit()).stderr(Stdio::inherit());
    }

    let mut child =
        command.spawn().with_context(|| format!("Failed to spawn claude in {worktree:?}"))?;
    let pid = Some(child.id());
    let status =
        child.wait().with_context(|| format!("Failed to wait for claude in {worktree:?}"))?;

    Ok(RuntimeOutcome { status, pid })
}
