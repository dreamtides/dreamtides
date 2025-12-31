use std::path::Path;

use anyhow::{Context, Result};

use crate::cli::AgentArgs;
use crate::config;
use crate::state::{self, AgentStatus};

/// Display the current status of a named agent.
pub fn run(args: &AgentArgs, repo_override: Option<&Path>) -> Result<()> {
    let paths = config::repo_paths(repo_override)?;
    let state_path = paths.llmc_dir.join("state.json");
    let state = state::load_state(&state_path)?;
    let record = state
        .agents
        .get(&args.agent)
        .with_context(|| format!("Unknown agent id: {}", args.agent))?;

    println!("agent_id={}", record.agent_id);
    println!("status={}", self::status_label(record.status));

    Ok(())
}

fn status_label(status: AgentStatus) -> &'static str {
    match status {
        AgentStatus::Idle => "idle",
        AgentStatus::Running => "running",
        AgentStatus::Rebasing => "rebasing",
        AgentStatus::NeedsReview => "needs_review",
        AgentStatus::Accepted => "accepted",
        AgentStatus::Rejected => "rejected",
    }
}
