use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::cli::{AcceptArgs, AgentArgs};
use crate::{config, git_ops, rebase, state};

/// Accept an agent branch by rebasing, fast-forwarding, and cleaning up.
pub fn run(args: &AcceptArgs, repo_override: Option<&Path>) -> Result<()> {
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

    rebase::run(&AgentArgs { agent: args.agent.clone() }, repo_override)?;

    let commit = git_ops::rev_parse(&record.worktree_path, &record.branch)?;

    git_ops::checkout_master(&paths.repo_root)?;
    git_ops::merge_ff_only(&paths.repo_root, &record.branch)?;
    git_ops::worktree_remove(&paths.repo_root, &record.worktree_path)?;
    git_ops::branch_delete(&paths.repo_root, &record.branch)?;

    let mut state = state::load_state(&state_path)?;
    if state.agents.remove(&args.agent).is_none() {
        return Err(anyhow::anyhow!("Unknown agent id: {}", args.agent));
    }
    state::save_state(&state_path, &state)?;

    if args.pull {
        self::pull_to_source(&paths.repo_root, &args.agent, &commit)?;
    }

    Ok(())
}

fn pull_to_source(llmc_root: &Path, agent_id: &str, commit: &str) -> Result<()> {
    let source_root = self::source_repo_root(llmc_root)?;
    git_ops::ensure_clean_worktree(&source_root)?;
    git_ops::fetch_from(&source_root, llmc_root, commit)?;

    let range = "master..FETCH_HEAD";
    let ahead_count = git_ops::rev_list_count(&source_root, range)?;
    anyhow::ensure!(
        ahead_count == 1,
        "Expected one commit ahead of master, found {ahead_count} for {range}"
    );

    let branch = format!("llmc/{agent_id}");
    git_ops::branch_force(&source_root, &branch, "FETCH_HEAD")?;
    git_ops::checkout_branch(&source_root, &branch)?;

    let rebase_status = git_ops::rebase_onto_branch(&source_root, "master")?;
    anyhow::ensure!(rebase_status.success(), "git rebase master failed in {source_root:?}");

    git_ops::checkout_master(&source_root)?;
    git_ops::merge_ff_only(&source_root, &branch)?;
    git_ops::branch_delete(&source_root, &branch)?;

    Ok(())
}

fn source_repo_root(llmc_root: &Path) -> Result<PathBuf> {
    let origin = git_ops::remote_origin_url(llmc_root)?;
    let trimmed = origin.trim();
    anyhow::ensure!(!trimmed.is_empty(), "remote.origin.url is empty in {llmc_root:?}");

    let path = trimmed.strip_prefix("file://").unwrap_or(trimmed);
    let source_root = PathBuf::from(path);
    let source_root =
        if source_root.is_absolute() { source_root } else { llmc_root.join(source_root) };
    let source_root = fs::canonicalize(&source_root)
        .with_context(|| format!("Failed to canonicalize source repo {source_root:?}"))?;

    anyhow::ensure!(
        source_root.join(".git").exists(),
        "remote.origin.url is not a local git repository: {origin}"
    );

    Ok(source_root)
}
