use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use super::super::config::{self, Config};
use super::super::state::{State, WorkerRecord, WorkerStatus};
use super::super::tmux::sender::TmuxSender;
use super::super::{git, worker};

/// Runs the start command, assigning a task to an idle worker
pub fn run_start(
    worker: Option<String>,
    prompt: Option<String>,
    prompt_file: Option<PathBuf>,
) -> Result<()> {
    validate_prompt_args(&prompt, &prompt_file)?;

    let llmc_root = config::get_llmc_root();
    if !llmc_root.exists() {
        bail!(
            "LLMC workspace not initialized. Run 'llmc init' first.\n\
             Expected workspace at: {}",
            llmc_root.display()
        );
    }

    let (mut state, config) = super::super::state::load_state_with_patrol()?;

    let worker_name = select_worker(&worker, &config, &state)?;

    let worker_record =
        state.get_worker(&worker_name).context("Worker not found after selection")?;

    if worker_record.status != WorkerStatus::Idle {
        bail!(
            "Worker '{}' is not idle (current status: {:?})\n\
             Available idle workers: {}",
            worker_name,
            worker_record.status,
            format_idle_workers(&state)
        );
    }

    let worktree_path = PathBuf::from(&worker_record.worktree_path);

    println!("Pulling latest master into worker '{}'...", worker_name);
    git::pull_rebase(&worktree_path)?;

    println!("Copying Tabula.xlsm to worker worktree...");
    copy_tabula_xlsm(&config, &worktree_path)?;

    let user_prompt = load_prompt_content(&prompt, &prompt_file)?;
    let full_prompt = build_full_prompt(worker_record, &config, &worker_name, &user_prompt)?;

    println!("Sending prompt to worker '{}'...", worker_name);
    let tmux_sender = TmuxSender::new();

    tmux_sender
        .send(&worker_record.session_id, "/clear")
        .with_context(|| format!("Failed to send /clear to worker '{}'", worker_name))?;

    tmux_sender
        .send(&worker_record.session_id, &full_prompt)
        .with_context(|| format!("Failed to send prompt to worker '{}'", worker_name))?;

    let worker_mut = state.get_worker_mut(&worker_name).unwrap();
    worker::apply_transition(worker_mut, worker::WorkerTransition::ToWorking {
        prompt: full_prompt,
    })?;

    state.save(&super::super::state::get_state_path())?;

    println!("✓ Worker '{}' started on task", worker_name);
    Ok(())
}

fn validate_prompt_args(prompt: &Option<String>, prompt_file: &Option<PathBuf>) -> Result<()> {
    if prompt.is_none() && prompt_file.is_none() {
        bail!("Must provide either --prompt or --prompt-file");
    }

    if prompt.is_some() && prompt_file.is_some() {
        bail!("Cannot provide both --prompt and --prompt-file");
    }

    if let Some(file) = prompt_file
        && !file.exists()
    {
        bail!("Prompt file does not exist: {}", file.display());
    }

    Ok(())
}

fn select_worker(worker: &Option<String>, config: &Config, state: &State) -> Result<String> {
    if let Some(name) = worker {
        if state.get_worker(name).is_none() {
            bail!(
                "Worker '{}' not found\n\
                 Available workers: {}",
                name,
                format_all_workers(state)
            );
        }
        return Ok(name.clone());
    }

    let idle_workers = state.get_idle_workers();

    let available: Vec<_> = idle_workers
        .iter()
        .filter(|w| config.get_worker(&w.name).map(|c| !c.excluded_from_pool).unwrap_or(false))
        .collect();

    if available.is_empty() {
        bail!(
            "No idle workers available\n\n\
             Current worker states:\n{}",
            format_worker_states(state)
        );
    }

    Ok(available[0].name.clone())
}

fn load_prompt_content(prompt: &Option<String>, prompt_file: &Option<PathBuf>) -> Result<String> {
    if let Some(text) = prompt {
        if text.trim().is_empty() {
            bail!("Prompt cannot be empty");
        }
        return Ok(text.clone());
    }

    if let Some(path) = prompt_file {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read prompt file: {}", path.display()))?;

        if content.trim().is_empty() {
            bail!("Prompt file is empty: {}", path.display());
        }

        return Ok(content);
    }

    bail!("No prompt provided");
}

fn build_full_prompt(
    worker_record: &WorkerRecord,
    config: &Config,
    worker_name: &str,
    user_prompt: &str,
) -> Result<String> {
    let worktree_path = PathBuf::from(&worker_record.worktree_path);

    let repo_root = worktree_path
        .parent()
        .and_then(|p| p.parent())
        .context("Could not determine repository root")?;

    let mut prompt = format!(
        "You are working in: {}\n\
         Repository root: {}\n\
         \n\
         Follow the conventions in AGENTS.md\n\
         Run validation commands before committing\n\
         Create a single commit with your changes\n\
         Do NOT push to remote\n\
         \n",
        worktree_path.display(),
        repo_root.display()
    );

    if let Some(worker_config) = config.get_worker(worker_name)
        && let Some(role_prompt) = &worker_config.role_prompt
    {
        prompt.push_str(role_prompt);
        prompt.push_str("\n\n");
    }

    prompt.push_str(user_prompt);

    Ok(prompt)
}

fn copy_tabula_xlsm(config: &Config, worktree_path: &Path) -> Result<()> {
    let source_repo = PathBuf::from(&config.repo.source);
    let source_xlsm = source_repo.join("client/Assets/StreamingAssets/Tabula.xlsm");

    if !source_xlsm.exists() {
        return Ok(());
    }

    let dest_xlsm = worktree_path.join("Tabula.xlsm");

    fs::copy(&source_xlsm, &dest_xlsm).with_context(|| {
        format!("Failed to copy {} to {}", source_xlsm.display(), dest_xlsm.display())
    })?;

    Ok(())
}

fn format_idle_workers(state: &State) -> String {
    let idle = state.get_idle_workers();
    if idle.is_empty() {
        return "none".to_string();
    }
    idle.iter().map(|w| w.name.as_str()).collect::<Vec<_>>().join(", ")
}

fn format_all_workers(state: &State) -> String {
    if state.workers.is_empty() {
        return "none".to_string();
    }
    state.workers.keys().map(String::as_str).collect::<Vec<_>>().join(", ")
}

fn format_worker_states(state: &State) -> String {
    if state.workers.is_empty() {
        return "  (no workers)".to_string();
    }

    state
        .workers
        .values()
        .map(|w| format!("  {} - {:?}", w.name, w.status))
        .collect::<Vec<_>>()
        .join("\n")
}
