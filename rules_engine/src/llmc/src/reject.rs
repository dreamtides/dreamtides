use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::cli::RejectArgs;
use crate::state::{self, AgentStatus};
use crate::{config, git_ops, runtime, time};

/// Re-run an agent with reviewer notes and the current diff.
pub fn run(args: &RejectArgs, repo_override: Option<&Path>) -> Result<()> {
    let paths = config::repo_paths(repo_override)?;
    let state_path = paths.llmc_dir.join("state.json");
    let state = state::load_state(&state_path)?;
    let Some(record) = state.agents.get(&args.agent) else {
        return Err(anyhow::anyhow!("Unknown agent id: {}", args.agent));
    };

    let notes = self::collect_notes(args)?;
    let diff = git_ops::diff_master_agent(&record.worktree_path, &record.branch)?;
    let mut updated_prompt = record.prompt.clone();

    if !notes.trim().is_empty() {
        updated_prompt = format!("{updated_prompt}\n\nReviewer notes:\n{notes}");
    }

    updated_prompt = format!("{updated_prompt}\n\nDiff:\n{diff}");

    let mut state = state::load_state(&state_path)?;
    let (runtime, worktree_path) = {
        let Some(record) = state.agents.get_mut(&args.agent) else {
            return Err(anyhow::anyhow!("Unknown agent id: {}", args.agent));
        };
        record.prompt = updated_prompt.clone();
        record.status = AgentStatus::Running;
        record.last_run_unix = time::unix_timestamp()?;
        (record.runtime, record.worktree_path.clone())
    };
    state::save_state(&state_path, &state)?;

    let outcome = runtime::run_runtime(runtime, &updated_prompt, &worktree_path, false);

    let mut state = state::load_state(&state_path)?;
    {
        let Some(record) = state.agents.get_mut(&args.agent) else {
            return Err(anyhow::anyhow!("Unknown agent id: {}", args.agent));
        };
        record.last_run_unix = time::unix_timestamp()?;
        record.last_pid = outcome.as_ref().ok().and_then(|outcome| outcome.pid);
        record.status = match outcome.as_ref() {
            Ok(outcome) if outcome.status.success() => AgentStatus::NeedsReview,
            Ok(_) => AgentStatus::Idle,
            Err(_) => AgentStatus::Idle,
        };
    }
    state::save_state(&state_path, &state)?;

    let outcome = outcome?;
    anyhow::ensure!(
        outcome.status.success(),
        "Runtime exited with status {status:?}",
        status = outcome.status
    );

    Ok(())
}

fn collect_notes(args: &RejectArgs) -> Result<String> {
    let mut notes = Vec::new();

    if let Some(note) = &args.notes {
        notes.push(note.clone());
    }

    if let Some(path) = &args.notes_file {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read notes file {path:?}"))?;
        notes.push(contents);
    }

    Ok(notes.join("\n\n"))
}
