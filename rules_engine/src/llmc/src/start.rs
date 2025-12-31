use std::mem;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};

use crate::cli::StartArgs;
use crate::state::{
    AgentRecord, AgentStatus, Runtime, {self},
};
use crate::{config, nouns};

/// Prepare the initial agent record and state needed for llmc start.
pub fn run(args: &StartArgs, repo_override: Option<&Path>) -> Result<()> {
    let paths = config::repo_paths(repo_override)?;
    let state_path = paths.llmc_dir.join("state.json");
    let state = state::load_state(&state_path)?;

    let agent_id = match &args.agent {
        Some(agent) => agent.clone(),
        None => nouns::choose_agent_id(&state)?,
    };

    let now = self::unix_timestamp()?;

    mem::drop(AgentRecord {
        agent_id: agent_id.clone(),
        branch: format!("agent/{agent_id}"),
        worktree_path: paths.worktrees_dir.join(format!("agent-{agent_id}")),
        runtime: args.runtime.unwrap_or(Runtime::Codex),
        prompt: args.prompt.clone().unwrap_or_default(),
        created_at_unix: now,
        last_run_unix: now,
        status: AgentStatus::Idle,
        last_pid: None,
    });

    let repo_root = &paths.repo_root;

    anyhow::bail!("llmc start is not implemented yet for repo {repo_root:?}")
}

fn unix_timestamp() -> Result<u64> {
    let since_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .with_context(|| "System time was before the Unix epoch")?;

    Ok(since_epoch.as_secs())
}
