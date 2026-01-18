use std::path::PathBuf;

use anyhow::{Context, Result, bail};

use crate::llmc::state::{self, State};
use crate::llmc::tmux::sender::TmuxSender;
use crate::llmc::worker::{self, WorkerTransition};
use crate::llmc::{config, git};
/// Runs the rebase command, manually triggering a rebase for a worker
pub fn run_rebase(worker: &str, json: bool) -> Result<()> {
    let llmc_root = config::get_llmc_root();
    if !llmc_root.exists() {
        bail!(
            "LLMC workspace not initialized. Run 'llmc init' first.\n\
             Expected workspace at: {}",
            llmc_root.display()
        );
    }
    let state_path = state::get_state_path();
    let mut state = State::load(&state_path)?;
    let worker_record = state.get_worker(worker).ok_or_else(|| {
        anyhow::anyhow!(
            "Worker '{}' not found\n\
             Available workers: {}",
            worker,
            format_all_workers(&state)
        )
    })?;
    let worktree_path = PathBuf::from(&worker_record.worktree_path);
    let session_id = worker_record.session_id.clone();
    println!("Fetching latest master...");
    git::fetch_origin(&llmc_root)
        .with_context(|| format!("Failed to fetch origin for worker '{}'", worker))?;
    println!("Rebasing worker '{}' onto master...", worker);
    let rebase_result = git::rebase_onto(&worktree_path, "origin/master")
        .with_context(|| format!("Failed to rebase worker '{}'", worker))?;
    if rebase_result.success {
        if json {
            let output = crate::json_output::RebaseOutput {
                worker: worker.to_string(),
                success: true,
                conflicts: Vec::new(),
            };
            crate::json_output::print_json(&output);
        } else {
            println!("✓ Worker '{}' successfully rebased onto master", worker);
        }
        Ok(())
    } else {
        if !json {
            println!("⚠ Rebase encountered conflicts in {} file(s)", rebase_result.conflicts.len());
        }
        let worker_mut = state.get_worker_mut(worker).unwrap();
        let original_task = worker_mut.current_prompt.clone();
        worker::apply_transition(worker_mut, WorkerTransition::ToRebasing)?;
        state.save(&state_path)?;
        let conflict_prompt = build_conflict_prompt(&rebase_result.conflicts, &original_task);
        if !json {
            println!("Sending conflict resolution instructions to worker...");
        }
        let tmux_sender = TmuxSender::new();
        tmux_sender
            .send(&session_id, &conflict_prompt)
            .with_context(|| format!("Failed to send conflict prompt to worker '{}'", worker))?;
        if json {
            let output = crate::json_output::RebaseOutput {
                worker: worker.to_string(),
                success: false,
                conflicts: rebase_result.conflicts.clone(),
            };
            crate::json_output::print_json(&output);
        } else {
            println!("✓ Worker '{}' marked as rebasing", worker);
            println!("  Conflict resolution prompt sent to worker");
        }
        Ok(())
    }
}
pub fn build_conflict_prompt(conflicts: &[String], original_task: &str) -> String {
    let mut prompt = String::from(
        "A rebase onto master has encountered conflicts.\n\
         \n",
    );
    prompt
        .push_str(
            &format!(
                "IMPORTANT - Your original task:\n\
         \"{}\"\n\
         \n\
         DO NOT restart your task from scratch. Instead, INCORPORATE your existing changes/intent \n\
         into the new repository state. Your goal is to apply the same logical changes you already \n\
         made, but adapted to work with the new state of the files after master's changes.\n\
         \n",
                original_task.lines().take(3).collect::< Vec < _ >> ().join(" ")
            ),
        );
    prompt.push_str("Conflicting files:\n");
    for file in conflicts {
        let conflict_count = count_conflict_markers(file);
        prompt.push_str(&format!("- {} ({} conflict markers)\n", file, conflict_count));
    }
    prompt
        .push_str(
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
fn format_all_workers(state: &State) -> String {
    if state.workers.is_empty() {
        return "none".to_string();
    }
    state.workers.keys().map(String::as_str).collect::<Vec<_>>().join(", ")
}
