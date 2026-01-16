use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use super::super::config::{self, Config};
use super::super::state::{State, WorkerRecord, WorkerStatus};
use super::super::tmux::sender::TmuxSender;
use super::super::tmux::session;
use super::super::{git, worker};
use crate::lock::StateLock;

/// Runs the start command, assigning a task to an idle worker
pub fn run_start(
    worker: Option<String>,
    prompt: Option<String>,
    prompt_file: Option<PathBuf>,
    prompt_cmd: Option<String>,
    skip_review: bool,
) -> Result<()> {
    validate_prompt_args(&prompt, &prompt_file, &prompt_cmd)?;

    let llmc_root = config::get_llmc_root();
    if !llmc_root.exists() {
        bail!(
            "LLMC workspace not initialized. Run 'llmc init' first.\n\
             Expected workspace at: {}",
            llmc_root.display()
        );
    }

    if !session::any_llmc_sessions_running()? {
        bail!(
            "LLMC daemon is not running (no llmc-* TMUX sessions detected).\n\
             Run 'llmc up' to start the daemon."
        );
    }

    let _lock = StateLock::acquire()?;

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

    copy_tabula_xlsm(&config, &worktree_path)?;

    let user_prompt = load_prompt_content(&prompt, &prompt_file, &prompt_cmd)?;
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
    worker_mut.skip_review = skip_review;
    worker::apply_transition(worker_mut, worker::WorkerTransition::ToWorking {
        prompt: full_prompt,
    })?;

    state.save(&super::super::state::get_state_path())?;

    if skip_review {
        println!("✓ Worker '{}' started on task (review phase will be skipped)", worker_name);
    } else {
        println!("✓ Worker '{}' started on task", worker_name);
    }
    Ok(())
}

fn validate_prompt_args(
    prompt: &Option<String>,
    prompt_file: &Option<PathBuf>,
    prompt_cmd: &Option<String>,
) -> Result<()> {
    let specified_count =
        prompt.is_some() as u8 + prompt_file.is_some() as u8 + prompt_cmd.is_some() as u8;

    if specified_count > 1 {
        bail!("Cannot provide more than one of --prompt, --prompt-file, or --prompt-cmd");
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

fn load_prompt_content(
    prompt: &Option<String>,
    prompt_file: &Option<PathBuf>,
    prompt_cmd: &Option<String>,
) -> Result<String> {
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

    if let Some(cmd) = prompt_cmd {
        return execute_prompt_command(cmd);
    }

    open_editor_for_prompt()
}

fn execute_prompt_command(cmd: &str) -> Result<String> {
    println!("Executing prompt command: {}", cmd);

    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .with_context(|| format!("Failed to execute prompt command: {cmd}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "Prompt command failed with exit code {:?}\nCommand: {}\nStderr: {}",
            output.status.code(),
            cmd,
            stderr.trim()
        );
    }

    let content = String::from_utf8(output.stdout)
        .with_context(|| "Prompt command output is not valid UTF-8")?;

    if content.trim().is_empty() {
        bail!("Prompt command produced empty output: {}", cmd);
    }

    println!("Prompt command generated {} bytes of output", content.len());

    Ok(content)
}

fn open_editor_for_prompt() -> Result<String> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("llmc-prompt-{}.md", std::process::id()));

    let status = Command::new(&editor)
        .arg(&temp_file)
        .status()
        .with_context(|| format!("Failed to launch editor: {}", editor))?;

    if !status.success() {
        bail!("Editor exited with non-zero status: {}", status);
    }

    let content = fs::read_to_string(&temp_file).with_context(|| {
        format!("Failed to read prompt from temporary file: {}", temp_file.display())
    })?;

    let _ = fs::remove_file(&temp_file);

    if content.trim().is_empty() {
        bail!("Prompt cannot be empty");
    }

    Ok(content)
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

    let dest_xlsm = worktree_path.join("client/Assets/StreamingAssets/Tabula.xlsm");

    if dest_xlsm.exists() {
        return Ok(());
    }

    if let Some(parent) = dest_xlsm.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }

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
