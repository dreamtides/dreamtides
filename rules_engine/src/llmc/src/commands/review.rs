use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::state::{self, State, WorkerStatus};
use crate::{config, git};

#[derive(Debug, Clone, Copy)]
pub enum ReviewInterface {
    Difftastic,
    VSCode,
}

/// Runs the review command, showing a diff for a worker awaiting review
pub fn run_review(
    worker: Option<String>,
    interface: ReviewInterface,
    name_only: bool,
    json: bool,
) -> Result<()> {
    let llmc_root = config::get_llmc_root();
    if !llmc_root.exists() {
        eprintln!("LLMC workspace not initialized. Run 'llmc init' first.");
        eprintln!("Expected workspace at: {}", llmc_root.display());
        std::process::exit(1);
    }

    let (state, config) = super::super::state::load_state_with_patrol()?;

    let worker_name = if let Some(name) = worker {
        if state.get_worker(&name).is_none() {
            eprintln!("Worker '{}' not found", name);
            eprintln!("Available workers: {}", format_all_workers(&state));
            std::process::exit(1);
        }
        name
    } else {
        let needs_review = state.get_workers_truly_needing_review(&config);
        if needs_review.is_empty() {
            eprintln!("No workers need review");
            eprintln!("\nRun 'llmc status' to see current worker states.");
            std::process::exit(1);
        }
        needs_review[0].name.clone()
    };

    let worker_record = state.get_worker(&worker_name).unwrap();

    // Check if worker is truly ready for human review
    if !state::is_truly_needs_review(worker_record, &config) {
        if worker_record.status == WorkerStatus::NeedsReview {
            // Worker is in NeedsReview but waiting for self-review prompt
            eprintln!(
                "Worker '{}' is awaiting self-review (self-review prompt not yet sent)",
                worker_name
            );
            eprintln!("The worker will be ready for human review after completing self-review.");
            eprintln!("\nRun 'llmc status' to see current worker states.");
        } else if worker_record.status == WorkerStatus::Reviewing {
            eprintln!("Worker '{}' is currently performing self-review", worker_name);
            eprintln!("Wait for the worker to complete self-review before reviewing.");
            eprintln!("\nRun 'llmc status' to see current worker states.");
        } else {
            eprintln!(
                "Worker '{}' is in state {:?}, not needs_review",
                worker_name, worker_record.status
            );
            eprintln!("Workers needing review: {}", format_needs_review_workers(&state, &config));
        }
        std::process::exit(1);
    }

    let Some(commit_sha) = &worker_record.commit_sha else {
        bail!("Worker '{}' has no commit SHA", worker_name);
    };

    let worktree_path = PathBuf::from(&worker_record.worktree_path);

    if json {
        eprintln!("Fetching latest master...");
    } else {
        println!("Fetching latest master...");
    }
    git::fetch_origin(&llmc_root)?;

    let merge_base = git::get_merge_base(&worktree_path, "HEAD", "origin/master")?;
    let origin_master_sha = git::get_head_commit_of_ref(&llmc_root, "origin/master")?;

    if merge_base != origin_master_sha {
        if json {
            eprintln!("Worker needs rebase onto latest master. Rebasing...");
        } else {
            println!("Worker needs rebase onto latest master. Rebasing...");
        }

        if git::has_uncommitted_changes(&worktree_path)? {
            if json {
                eprintln!("Amending uncommitted changes before rebase...");
            } else {
                println!("Amending uncommitted changes before rebase...");
            }
            git::amend_uncommitted_changes(&worktree_path)?;
        }

        let rebase_result = git::rebase_onto(&worktree_path, "origin/master")?;

        if !rebase_result.success {
            let (mut state, _) = super::super::state::load_state_with_patrol()?;
            let worker_mut = state.get_worker_mut(&worker_name).unwrap();
            let original_task = worker_mut.current_prompt.clone();
            super::super::worker::apply_transition(
                worker_mut,
                super::super::worker::WorkerTransition::ToRebasing,
            )?;

            let conflict_prompt =
                build_conflict_resolution_prompt(&rebase_result.conflicts, &original_task);
            let sender = super::super::tmux::sender::TmuxSender::new();
            sender.send(&worker_mut.session_id, &conflict_prompt)?;

            state.save(&super::super::state::get_state_path())?;

            if json {
                eprintln!("\n✓ Agent rebase started");
                eprintln!("  Worker '{}' transitioned to 'rebasing' state", worker_name);
                eprintln!("  The agent will resolve conflicts and continue the rebase");
                eprintln!("  Run 'llmc review {}' again once complete", worker_name);
            } else {
                println!("\n✓ Agent rebase started");
                println!("  Worker '{}' transitioned to 'rebasing' state", worker_name);
                println!("  The agent will resolve conflicts and continue the rebase");
                println!("  Run 'llmc review {}' again once complete", worker_name);
            }
            return Ok(());
        }

        if json {
            eprintln!("✓ Rebased onto latest master");
        } else {
            println!("✓ Rebased onto latest master");
        }
    }

    let commit_message = git::get_commit_message(&worktree_path, commit_sha)?;

    if json {
        eprintln!("Reviewing: {} ({})", worker_name, worker_record.branch);
        eprintln!("Commit: {}", &commit_sha[..7.min(commit_sha.len())]);
        eprintln!("Prompt: \"{}...\"", truncate_prompt(&worker_record.current_prompt, 50));
        eprintln!();
        eprintln!("{}", commit_message);
        eprintln!();
    } else {
        println!("Reviewing: {} ({})", worker_name, worker_record.branch);
        println!("Commit: {}", &commit_sha[..7.min(commit_sha.len())]);
        println!("Prompt: \"{}...\"", truncate_prompt(&worker_record.current_prompt, 50));
        println!();
        println!("{}", commit_message);
        println!();
    }

    if json {
        let changed_files = get_changed_files(&worktree_path)?;
        let output = crate::json_output::ReviewOutput {
            worker: worker_name.clone(),
            status: format!("{:?}", worker_record.status).to_lowercase(),
            commit_sha: worker_record.commit_sha.clone(),
            changed_files,
        };
        crate::json_output::print_json(&output);
    } else {
        display_diff(&worktree_path, interface, name_only)?;

        println!();
        println!("Commands:");
        println!("  llmc accept        Accept these changes");
        println!("  llmc reject \"...\"  Request changes");
    }

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

fn get_changed_files(worktree_path: &PathBuf) -> Result<Vec<String>> {
    let current_branch = git::get_current_branch(worktree_path)?;
    let range = format!("origin/master...{}", current_branch);

    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(worktree_path)
        .arg("diff")
        .arg("--name-only")
        .arg(&range)
        .output()
        .context("Failed to execute git diff --name-only")?;

    if !output.status.success() {
        bail!("git diff --name-only failed for {}", range);
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(std::string::ToString::to_string)
        .collect())
}

fn display_diff(
    worktree_path: &PathBuf,
    interface: ReviewInterface,
    name_only: bool,
) -> Result<()> {
    let current_branch = git::get_current_branch(worktree_path)?;
    let range = format!("origin/master...{}", current_branch);

    // If name_only flag is set, just show file names regardless of interface
    if name_only {
        let output = Command::new("git")
            .arg("-C")
            .arg(worktree_path)
            .arg("diff")
            .arg("--name-only")
            .arg(&range)
            .output()
            .context("Failed to execute git diff --name-only")?;

        if !output.status.success() {
            bail!("git diff --name-only failed for {}", range);
        }

        print!("{}", String::from_utf8_lossy(&output.stdout));
        return Ok(());
    }

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
            // Open the worktree in VSCode
            let output = Command::new("code")
                .arg(worktree_path)
                .output()
                .context("Failed to execute VS Code. Is 'code' in PATH?")?;

            if !output.status.success() {
                bail!("Failed to open VS Code: {}", String::from_utf8_lossy(&output.stderr));
            }

            println!("✓ Opened worktree in VS Code: {}", worktree_path.display());
            println!();
            println!("To view the diff:");
            println!("  1. Open the Source Control panel (View → Source Control or Ctrl+Shift+G)");
            println!("  2. The changes compared to origin/master will be shown");
            println!("  3. Click on any file to see its diff");
            println!();
            println!("Alternative ways to view changes:");
            println!("  • Command Palette (Ctrl+Shift+P or Cmd+Shift+P):");
            println!("    - Search 'Git: View File History' to see commit history");
            println!("    - Search 'Git: Compare with...' then select 'origin/master'");
            println!("  • Timeline view (click clock icon in Explorer) shows file history");
            println!();
            println!("The worktree is on branch '{}' and includes all changes.", current_branch);
            println!(
                "You can edit files directly and they will be reflected in the worker's state."
            );
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

fn format_needs_review_workers(state: &State, config: &super::super::config::Config) -> String {
    let needs_review = state.get_workers_truly_needing_review(config);
    if needs_review.is_empty() {
        return "none".to_string();
    }
    needs_review.iter().map(|w| w.name.as_str()).collect::<Vec<_>>().join(", ")
}

fn build_conflict_resolution_prompt(conflicts: &[String], original_task: &str) -> String {
    let mut prompt = String::from(
        "A rebase onto master has encountered conflicts.\n\
         \n",
    );

    prompt.push_str(&format!(
        "IMPORTANT - Your original task:\n\
         \"{}\"\n\
         \n\
         DO NOT restart your task from scratch. Instead, INCORPORATE your existing changes/intent \n\
         into the new repository state. Your goal is to apply the same logical changes you already \n\
         made, but adapted to work with the new state of the files after master's changes.\n\
         \n",
        original_task.lines().take(3).collect::<Vec<_>>().join(" ")
    ));

    prompt.push_str("Conflicting files:\n");

    for file in conflicts {
        let conflict_count = count_conflict_markers(file);
        prompt.push_str(&format!("- {} ({} conflict markers)\n", file, conflict_count));
    }

    prompt.push_str(
        "\n\
         Resolution steps:\n\
         1. Examine conflict markers (<<<<<<, =======, >>>>>>>)\n\
         2. Understand what master changed (their version) and what you changed (our version)\n\
         3. Decide how to INCORPORATE YOUR CHANGES into the new state - do NOT just accept theirs\n\
         4. Remove conflict markers and apply your intended changes\n\
         5. Stage resolved files: git add <file>\n\
         6. Continue rebase: git rebase --continue\n\
         7. Run validation: just review\n\
         8. IMPORTANT: If validation modified any files, amend them: git add -A && git commit --amend --no-edit\n\
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
