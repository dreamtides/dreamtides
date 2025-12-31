use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::cli::StartArgs;
use crate::state::{self, AgentRecord, AgentStatus, Runtime};
use crate::{config, git_ops, nouns, prompt, runtime, time};

/// Prepare the initial agent record and state needed for llmc start.
pub fn run(args: &StartArgs, repo_override: Option<&Path>) -> Result<()> {
    let paths = config::repo_paths(repo_override)?;
    fs::create_dir_all(&paths.worktrees_dir).with_context(|| {
        format!(
            "Failed to create worktrees dir {worktrees_dir:?}",
            worktrees_dir = paths.worktrees_dir
        )
    })?;
    fs::create_dir_all(&paths.llmc_dir).with_context(|| {
        format!("Failed to create llmc dir {llmc_dir:?}", llmc_dir = paths.llmc_dir)
    })?;

    let state_path = paths.llmc_dir.join("state.json");
    let state = state::load_state(&state_path)?;

    let agent_id = match &args.agent {
        Some(agent) => agent.clone(),
        None => nouns::choose_agent_id(&state)?,
    };

    anyhow::ensure!(!state.agents.contains_key(&agent_id), "Agent id already exists: {agent_id}");

    let runtime = args.runtime.unwrap_or(Runtime::Codex);
    anyhow::ensure!(runtime == Runtime::Codex, "Runtime {runtime:?} is not supported yet");

    let worktree_path = paths.worktrees_dir.join(format!("agent-{agent_id}"));
    anyhow::ensure!(!worktree_path.exists(), "Worktree already exists: {worktree_path:?}");

    git_ops::worktree_add(&paths.repo_root, &worktree_path, &format!("agent/{agent_id}"))?;
    git_ops::ensure_clean_worktree(&worktree_path)?;

    let user_prompt = prompt::assemble_user_prompt(args.prompt.as_deref(), &args.prompt_file)?;
    let full_prompt = prompt::wrap_prompt(&paths.repo_root, &worktree_path, &user_prompt);

    let now = time::unix_timestamp()?;

    let record = AgentRecord {
        agent_id: agent_id.clone(),
        branch: format!("agent/{agent_id}"),
        worktree_path: worktree_path.clone(),
        runtime,
        prompt: full_prompt.clone(),
        created_at_unix: now,
        last_run_unix: now,
        status: AgentStatus::Running,
        last_pid: None,
    };

    let mut state = state::load_state(&state_path)?;
    state.agents.insert(agent_id.clone(), record);
    state::save_state(&state_path, &state)?;

    let outcome = runtime::run_runtime(
        runtime,
        &full_prompt,
        &paths.repo_root,
        &worktree_path,
        args.background,
    );

    let mut state = state::load_state(&state_path)?;
    {
        let Some(record) = state.agents.get_mut(&agent_id) else {
            return Err(anyhow::anyhow!("Agent record missing for {agent_id}"));
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
    self::print_agent_completed(&agent_id);

    Ok(())
}

fn print_agent_completed(agent_id: &str) {
    println!("Agent {agent_id} task completed");
}
