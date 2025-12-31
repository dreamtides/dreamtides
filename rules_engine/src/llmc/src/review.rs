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
    let agent_id = state::resolve_agent_id(args.agent.as_deref(), &state)?;
    let record =
        state.agents.get(&agent_id).with_context(|| format!("Unknown agent id: {agent_id}"))?;
    println!("agent_id={agent_id}");

    match args.interface {
        ReviewInterface::Diff => self::run_diff(record),
        ReviewInterface::Difftastic => self::run_difftastic(record),
        ReviewInterface::Vscode => anyhow::bail!("Review interface vscode is not implemented yet"),
        ReviewInterface::Forgejo => {
            anyhow::bail!("Review interface forgejo is not implemented yet")
        }
    }
}

fn run_diff(record: &AgentRecord) -> Result<()> {
    let diff = git_ops::diff_master_agent(&record.worktree_path, &record.branch)?;
    let status = git_ops::status_porcelain(&record.worktree_path)?;
    let commit_status = self::commit_status(record)?;
    if let Some(message) = self::commit_warning(&commit_status, !diff.trim().is_empty(), &status) {
        self::print_warning(&message);
    }

    if !diff.trim().is_empty() {
        print!("{diff}");
        return Ok(());
    }

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

fn run_difftastic(record: &AgentRecord) -> Result<()> {
    let diff = git_ops::diff_master_agent(&record.worktree_path, &record.branch)?;
    let status = git_ops::status_porcelain(&record.worktree_path)?;
    let commit_status = self::commit_status(record)?;
    if let Some(message) = self::commit_warning(&commit_status, !diff.trim().is_empty(), &status) {
        self::print_warning(&message);
    }

    if !diff.trim().is_empty() {
        return git_ops::diff_master_agent_difftastic(&record.worktree_path, &record.branch);
    }

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
        git_ops::diff_cached_difftastic(&record.worktree_path)?;
        if !unstaged.trim().is_empty() {
            println!();
        }
    }

    if !unstaged.trim().is_empty() {
        git_ops::diff_worktree_difftastic(&record.worktree_path)?;
    }

    Ok(())
}

fn commit_status(record: &AgentRecord) -> Result<CommitStatus> {
    let range = format!("master..{}", record.branch);
    let commit_count = git_ops::rev_list_count(&record.worktree_path, &range)?;
    if commit_count == 0 {
        return Ok(CommitStatus { commit_count, message_word_count: None, message_ok: false });
    }

    let message = git_ops::commit_subject(&record.worktree_path, &record.branch)?;
    let message_word_count = message.split_whitespace().count();

    Ok(CommitStatus {
        commit_count,
        message_word_count: Some(message_word_count),
        message_ok: (8..=12).contains(&message_word_count),
    })
}

fn commit_warning(
    status: &CommitStatus,
    has_commit_diff: bool,
    worktree_status: &str,
) -> Option<String> {
    if !has_commit_diff && worktree_status.trim().is_empty() {
        return None;
    }

    if status.commit_count == 0 {
        return Some("Review warning: no commit found on agent worktree".to_string());
    }

    if !status.message_ok {
        let word_count = status.message_word_count.unwrap_or_default();
        return Some(format!(
            "Review warning: commit message should be ~10 words (got {word_count})"
        ));
    }

    None
}

fn print_warning(message: &str) {
    println!("\x1b[31m{message}\x1b[0m");
}

struct CommitStatus {
    commit_count: usize,
    message_word_count: Option<usize>,
    message_ok: bool,
}
