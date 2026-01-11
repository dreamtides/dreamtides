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

    let (state, _config) = super::load_state_with_patrol()?;

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
    match interface {
        ReviewInterface::Difftastic => {
            let output = Command::new("difft")
                .arg("--display")
                .arg("side-by-side")
                .current_dir(worktree_path)
                .arg("master")
                .arg("HEAD")
                .output()
                .context("Failed to execute difft. Is difftastic installed?")?;

            if !output.status.success() {
                bail!(
                    "Failed to generate diff with difftastic: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }

            print!("{}", String::from_utf8_lossy(&output.stdout));
        }
        ReviewInterface::VSCode => {
            let output = Command::new("code")
                .arg("--diff")
                .arg("master")
                .arg("HEAD")
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
