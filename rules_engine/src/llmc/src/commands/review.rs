use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result, bail};

use super::super::state::{State, WorkerStatus};
use super::super::{config, git};

#[derive(Debug, Clone, Copy)]
pub enum ReviewInterface {
    Difftastic,
    VSCode,
}

/// Runs the review command, showing a diff for a worker awaiting review
pub fn run_review(worker: Option<String>, interface: ReviewInterface) -> Result<()> {
    let llmc_root = config::get_llmc_root();
    if !llmc_root.exists() {
        eprintln!("LLMC workspace not initialized. Run 'llmc init' first.");
        eprintln!("Expected workspace at: {}", llmc_root.display());
        std::process::exit(1);
    }

    let (state, _config) = super::super::state::load_state_with_patrol()?;

    let worker_name = if let Some(name) = worker {
        if state.get_worker(&name).is_none() {
            eprintln!("Worker '{}' not found", name);
            eprintln!("Available workers: {}", format_all_workers(&state));
            std::process::exit(1);
        }
        name
    } else {
        let needs_review = state.get_workers_needing_review();
        if needs_review.is_empty() {
            eprintln!("No workers need review");
            eprintln!("\nRun 'llmc status' to see current worker states.");
            std::process::exit(1);
        }
        needs_review[0].name.clone()
    };

    let worker_record = state.get_worker(&worker_name).unwrap();

    if worker_record.status != WorkerStatus::NeedsReview {
        eprintln!(
            "Worker '{}' is in state {:?}, not needs_review",
            worker_name, worker_record.status
        );
        eprintln!("Workers needing review: {}", format_needs_review_workers(&state));
        std::process::exit(1);
    }

    let Some(commit_sha) = &worker_record.commit_sha else {
        bail!("Worker '{}' has no commit SHA", worker_name);
    };

    let worktree_path = PathBuf::from(&worker_record.worktree_path);

    println!("Fetching latest master...");
    git::fetch_origin(&llmc_root)?;

    let merge_base = git::get_merge_base(&worktree_path, "HEAD", "origin/master")?;
    let origin_master_sha = git::get_head_commit_of_ref(&llmc_root, "origin/master")?;

    if merge_base != origin_master_sha {
        println!("Worker needs rebase onto latest master. Rebasing...");

        if git::has_uncommitted_changes(&worktree_path)? {
            println!("Amending uncommitted changes before rebase...");
            git::amend_uncommitted_changes(&worktree_path)?;
        }

        let rebase_result = git::rebase_onto(&worktree_path, "origin/master")?;

        if !rebase_result.success {
            let (mut state, _) = super::super::state::load_state_with_patrol()?;
            let worker_mut = state.get_worker_mut(&worker_name).unwrap();
            super::super::worker::apply_transition(
                worker_mut,
                super::super::worker::WorkerTransition::ToRebasing,
            )?;

            let conflict_prompt = build_conflict_resolution_prompt(&rebase_result.conflicts);
            let sender = super::super::tmux::sender::TmuxSender::new();
            sender.send(&worker_mut.session_id, &conflict_prompt)?;

            state.save(&super::super::state::get_state_path())?;

            println!("\n✓ Agent rebase started");
            println!("  Worker '{}' transitioned to 'rebasing' state", worker_name);
            println!("  The agent will resolve conflicts and continue the rebase");
            println!("  Run 'llmc review {}' again once complete", worker_name);
            return Ok(());
        }

        println!("✓ Rebased onto latest master");
    }

    let commit_message = git::get_commit_message(&worktree_path, commit_sha)?;

    println!("Reviewing: {} ({})", worker_name, worker_record.branch);
    println!("Commit: {}", &commit_sha[..7.min(commit_sha.len())]);
    println!("Prompt: \"{}...\"", truncate_prompt(&worker_record.current_prompt, 50));
    println!();
    println!("{}", commit_message);
    println!();

    display_diff(&worktree_path, interface)?;

    println!();
    println!("Commands:");
    println!("  llmc accept        Accept these changes");
    println!("  llmc reject \"...\"  Request changes");

    save_last_reviewed(&worker_name)?;

    Ok(())
}

pub fn load_last_reviewed() -> Result<Option<String>> {
    let llmc_root = config::get_llmc_root();
    let last_reviewed_path = llmc_root.join(".last_reviewed");
    if !last_reviewed_path.exists() {
        return Ok(None);
    }
    let contents = fs::read_to_string(&last_reviewed_path)
        .with_context(|| format!("Failed to read {}", last_reviewed_path.display()))?;
    Ok(Some(contents.trim().to_string()))
}

fn display_diff(worktree_path: &PathBuf, interface: ReviewInterface) -> Result<()> {
    let current_branch = git::get_current_branch(worktree_path)?;
    let range = format!("origin/master...{}", current_branch);

    match interface {
        ReviewInterface::Difftastic => {
            let status = Command::new("git")
                .arg("-C")
                .arg(worktree_path)
                .arg("-c")
                .arg("diff.external=difft")
                .arg("diff")
                .arg(&range)
                .status()
                .context("Failed to execute git diff with difftastic")?;

            if !status.success() {
                bail!("git diff failed for {}", range);
            }
        }
        ReviewInterface::VSCode => {
            let output = Command::new("code")
                .arg("--diff")
                .arg("master")
                .arg(&current_branch)
                .current_dir(worktree_path)
                .output()
                .context("Failed to execute VS Code. Is 'code' in PATH?")?;

            if !output.status.success() {
                bail!(
                    "Failed to open diff in VS Code: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }

            println!("Opened diff in VS Code");
        }
    }

    Ok(())
}

fn save_last_reviewed(worker_name: &str) -> Result<()> {
    let llmc_root = config::get_llmc_root();
    let last_reviewed_path = llmc_root.join(".last_reviewed");
    fs::write(&last_reviewed_path, worker_name)
        .with_context(|| format!("Failed to write {}", last_reviewed_path.display()))?;
    Ok(())
}

fn truncate_prompt(prompt: &str, max_len: usize) -> String {
    let trimmed = prompt.trim();
    if trimmed.len() <= max_len {
        return trimmed.to_string();
    }

    let truncated = &trimmed[..max_len];
    let last_space = truncated.rfind(' ').unwrap_or(max_len);
    trimmed[..last_space].trim().to_string()
}

fn format_all_workers(state: &State) -> String {
    if state.workers.is_empty() {
        return "none".to_string();
    }
    state.workers.keys().map(String::as_str).collect::<Vec<_>>().join(", ")
}

fn format_needs_review_workers(state: &State) -> String {
    let needs_review = state.get_workers_needing_review();
    if needs_review.is_empty() {
        return "none".to_string();
    }
    needs_review.iter().map(|w| w.name.as_str()).collect::<Vec<_>>().join(", ")
}

fn build_conflict_resolution_prompt(conflicts: &[String]) -> String {
    let mut prompt = String::from(
        "A rebase onto master has encountered conflicts.\n\
         \n\
         Conflicting files:\n",
    );

    for file in conflicts {
        let conflict_count = count_conflict_markers(file);
        prompt.push_str(&format!("- {} ({} conflict markers)\n", file, conflict_count));
    }

    prompt.push_str(
        "\n\
         Resolution steps:\n\
         1. Examine conflict markers (<<<<<<, =======, >>>>>>>)\n\
         2. Decide how to resolve each conflict\n\
         3. Remove conflict markers\n\
         4. Stage resolved files: git add <file>\n\
         5. Continue rebase: git rebase --continue\n\
         6. Run validation: just review\n\
         7. IMPORTANT: If validation modified any files, amend them: git add -A && git commit --amend --no-edit\n\
         \n\
         Notes:\n\
         - View original versions: git show :2:<file> (ours) :3:<file> (theirs)\n\
         - To abort: git rebase --abort\n",
    );

    prompt
}

fn count_conflict_markers(file: &str) -> usize {
    std::fs::read_to_string(file).map(|content| content.matches("<<<<<<<").count()).unwrap_or(0)
}
