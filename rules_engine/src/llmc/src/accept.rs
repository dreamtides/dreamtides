use std::path::Path;

use anyhow::Result;

use crate::cli::AgentArgs;
use crate::{config, git_ops, rebase, state};

/// Accept an agent branch by rebasing, fast-forwarding, and cleaning up.
pub fn run(args: &AgentArgs, repo_override: Option<&Path>) -> Result<()> {
    let paths = config::repo_paths(repo_override)?;
    let state_path = paths.llmc_dir.join("state.json");
    let state = state::load_state(&state_path)?;
    let Some(record) = state.agents.get(&args.agent) else {
        return Err(anyhow::anyhow!("Unknown agent id: {}", args.agent));
    };

    git_ops::ensure_clean_worktree(&record.worktree_path)?;

    let range = format!("master..{}", record.branch);
    let ahead_count = git_ops::rev_list_count(&record.worktree_path, &range)?;
    anyhow::ensure!(
        ahead_count == 1,
        "Expected one commit ahead of master, found {ahead_count} for {range}"
    );

    rebase::run(args, repo_override)?;

    git_ops::checkout_master(&paths.repo_root)?;
    git_ops::merge_ff_only(&paths.repo_root, &record.branch)?;
    git_ops::worktree_remove(&paths.repo_root, &record.worktree_path)?;
    git_ops::branch_delete(&paths.repo_root, &record.branch)?;

    let mut state = state::load_state(&state_path)?;
    if state.agents.remove(&args.agent).is_none() {
        return Err(anyhow::anyhow!("Unknown agent id: {}", args.agent));
    }
    state::save_state(&state_path, &state)?;

    Ok(())
}
