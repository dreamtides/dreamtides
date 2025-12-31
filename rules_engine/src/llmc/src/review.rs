use std::path::Path;

use anyhow::{Context, Result};

use crate::cli::{ReviewArgs, ReviewInterface};
use crate::state::{self, AgentRecord};
use crate::{config, git_ops};

/// Run the selected review interface for an agent.
pub fn run(args: &ReviewArgs, repo_override: Option<&Path>) -> Result<()> {
    let paths = config::repo_paths(repo_override)?;
    let state_path = paths.llmc_dir.join("state.json");
    let state = state::load_state(&state_path)?;
    let record = state
        .agents
        .get(&args.agent)
        .with_context(|| format!("Unknown agent id: {}", args.agent))?;

    match args.interface {
        ReviewInterface::Diff => self::run_diff(record),
        ReviewInterface::Difftastic => {
            anyhow::bail!("Review interface difftastic is not implemented yet")
        }
        ReviewInterface::Vscode => anyhow::bail!("Review interface vscode is not implemented yet"),
        ReviewInterface::Forgejo => {
            anyhow::bail!("Review interface forgejo is not implemented yet")
        }
    }
}

fn run_diff(record: &AgentRecord) -> Result<()> {
    let diff = git_ops::diff_master_agent(&record.worktree_path, &record.branch)?;
    if !diff.trim().is_empty() {
        print!("{diff}");
        return Ok(());
    }

    let status = git_ops::status_porcelain(&record.worktree_path)?;
    if status.trim().is_empty() {
        println!("Nothing to review, working directory clean");
        return Ok(());
    }

    let staged = git_ops::diff_cached(&record.worktree_path)?;
    let unstaged = git_ops::diff_worktree(&record.worktree_path)?;

    if staged.trim().is_empty() && unstaged.trim().is_empty() {
        println!("No diff output; working directory has untracked changes");
        return Ok(());
    }

    if !staged.trim().is_empty() {
        print!("{staged}");
        if !unstaged.trim().is_empty() {
            println!();
        }
    }

    if !unstaged.trim().is_empty() {
        print!("{unstaged}");
    }

    Ok(())
}
