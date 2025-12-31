use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

use anyhow::{Context, Result};

use crate::state::Runtime;

#[derive(Debug)]
pub struct RuntimeOutcome {
    pub status: ExitStatus,
    pub pid: Option<u32>,
}

/// Run a supported runtime with the provided prompt.
pub fn run_runtime(
    runtime: Runtime,
    prompt: &str,
    worktree: &Path,
    background: bool,
) -> Result<RuntimeOutcome> {
    match runtime {
        Runtime::Codex => self::run_codex(prompt, worktree, background),
        _ => Err(anyhow::anyhow!("Runtime {runtime:?} is not supported yet")),
    }
}

fn run_codex(prompt: &str, worktree: &Path, background: bool) -> Result<RuntimeOutcome> {
    let mut command = Command::new("codex");
    command
        .arg("-a")
        .arg("never")
        .arg("exec")
        .arg("-C")
        .arg(worktree)
        .arg("--sandbox")
        .arg("workspace-write")
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
