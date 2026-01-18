use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use regex::Regex;

use super::super::config::{self, Config};
use super::super::state::{State, WorkerRecord, WorkerStatus};
use super::super::tmux::sender::TmuxSender;
use super::super::tmux::session;
use super::super::{git, worker};
use crate::lock::StateLock;

/// Runs the start command, assigning a task to an idle worker
pub fn run_start(
    worker: Option<String>,
    prefix: Option<String>,
    prompt: Option<String>,
    prompt_file: Option<PathBuf>,
    prompt_cmd: Option<String>,
    self_review: bool,
    json: bool,
) -> Result<()> {
    validate_prompt_args(&prompt, &prompt_file, &prompt_cmd)?;
    validate_worker_args(&worker, &prefix)?;

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

    let worker_name = select_worker(&worker, &prefix, &config, &state)?;

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

    if json {
        eprintln!("Pulling latest master into worker '{}'...", worker_name);
    } else {
        println!("Pulling latest master into worker '{}'...", worker_name);
    }

    // First, check for stale commits from a previous task. If the worker is idle
    // but has commits ahead of origin/master, those are leftover from a prior task
    // that was already merged (or rejected). Reset to origin/master to start clean.
    if git::has_commits_ahead_of(&worktree_path, "origin/master")? {
        tracing::info!(
            worker = %worker_name,
            "Worker has stale commits from previous task, resetting to origin/master"
        );
        if !json {
            println!(
                "  Resetting worker to origin/master (removing stale commits from previous task)..."
            );
        }
        git::reset_to_ref(&worktree_path, "origin/master")?;
    }

    git::pull_rebase(&worktree_path)?;

    copy_tabula_xlsm(&config, &worktree_path)?;
    copy_serena_config(&config, &worktree_path)?;

    let user_prompt = load_prompt_content(&prompt, &prompt_file, &prompt_cmd, json)?;

    // Warn if the prompt contains absolute paths to the source repository
    warn_about_source_repo_paths(&user_prompt, &config, &worktree_path)?;

    let full_prompt = build_full_prompt(worker_record, &config, &worker_name, &user_prompt)?;

    // Log full prompt content for debugging
    tracing::info!(
        operation = "worker_start",
        worker = %worker_name,
        worktree = %worktree_path.display(),
        prompt_length = full_prompt.len(),
        prompt_cmd = ?prompt_cmd,
        self_review,
        "Starting worker with prompt"
    );
    tracing::debug!(
        operation = "worker_start_prompt",
        worker = %worker_name,
        full_prompt = %full_prompt,
        "Full prompt content being sent to worker"
    );

    if json {
        eprintln!("Sending prompt to worker '{}'...", worker_name);
    } else {
        println!("Sending prompt to worker '{}'...", worker_name);
    }
    let tmux_sender = TmuxSender::new();

    tmux_sender
        .send(&worker_record.session_id, "/clear")
        .with_context(|| format!("Failed to send /clear to worker '{}'", worker_name))?;

    tmux_sender
        .send(&worker_record.session_id, &full_prompt)
        .with_context(|| format!("Failed to send prompt to worker '{}'", worker_name))?;

    let worker_mut =
        state.get_worker_mut(&worker_name).expect("Worker disappeared after validation");
    // Use CLI flag if set, otherwise check worker config
    worker_mut.self_review =
        self_review || config.get_worker(&worker_name).and_then(|c| c.self_review).unwrap_or(false);
    worker::apply_transition(worker_mut, worker::WorkerTransition::ToWorking {
        prompt: full_prompt,
        prompt_cmd: prompt_cmd.clone(),
    })?;

    let self_review_enabled =
        state.get_worker(&worker_name).map(|w| w.self_review).unwrap_or(false);

    state.save(&super::super::state::get_state_path())?;

    if json {
        let worker_record = state.get_worker(&worker_name).unwrap();
        let output = crate::json_output::StartOutput {
            worker: worker_name,
            status: "working".to_string(),
            self_review_enabled,
            worktree_path: worker_record.worktree_path.clone(),
            branch: worker_record.branch.clone(),
        };
        crate::json_output::print_json(&output);
    } else if self_review_enabled {
        println!("✓ Worker '{}' started on task (self-review enabled)", worker_name);
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

fn validate_worker_args(worker: &Option<String>, prefix: &Option<String>) -> Result<()> {
    if worker.is_some() && prefix.is_some() {
        bail!("Cannot provide both --worker and --prefix");
    }
    Ok(())
}

fn select_worker(
    worker: &Option<String>,
    prefix: &Option<String>,
    config: &Config,
    state: &State,
) -> Result<String> {
    // If a specific worker name is provided, use that worker
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

    // Filter to only non-excluded workers
    let mut available: Vec<_> = idle_workers
        .iter()
        .filter(|w| config.get_worker(&w.name).map(|c| !c.excluded_from_pool).unwrap_or(false))
        .collect();

    // If a prefix is provided, further filter to workers matching that prefix
    if let Some(prefix) = prefix {
        available.retain(|w| w.name.starts_with(prefix));

        if available.is_empty() {
            let all_matching: Vec<_> = state
                .workers
                .values()
                .filter(|w| w.name.starts_with(prefix))
                .map(|w| format!("{} ({:?})", w.name, w.status))
                .collect();

            if all_matching.is_empty() {
                bail!(
                    "No workers found with prefix '{}'\n\
                     Available workers: {}",
                    prefix,
                    format_all_workers(state)
                );
            } else {
                bail!(
                    "No idle workers available with prefix '{}'\n\n\
                     Workers matching prefix:\n  {}",
                    prefix,
                    all_matching.join("\n  ")
                );
            }
        }
    } else if available.is_empty() {
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
    json: bool,
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
        return execute_prompt_command(cmd, json);
    }

    open_editor_for_prompt()
}

fn execute_prompt_command(cmd: &str, json: bool) -> Result<String> {
    if json {
        eprintln!("Executing prompt command: {}", cmd);
    } else {
        println!("Executing prompt command: {}", cmd);
    }

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

    if json {
        eprintln!("Prompt command generated {} bytes of output", content.len());
    } else {
        println!("Prompt command generated {} bytes of output", content.len());
    }

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

fn copy_serena_config(config: &Config, worktree_path: &Path) -> Result<()> {
    let source_repo = PathBuf::from(&config.repo.source);
    let source_serena = source_repo.join(".serena");

    if !source_serena.exists() {
        return Ok(());
    }

    let dest_serena = worktree_path.join(".serena");

    // If destination already exists, check if it needs updating
    if dest_serena.exists() {
        return Ok(());
    }

    // Create the .serena directory in the worktree
    fs::create_dir_all(&dest_serena)
        .with_context(|| format!("Failed to create directory {}", dest_serena.display()))?;

    // Copy all files from source .serena to worktree .serena
    for entry in fs::read_dir(&source_serena)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let source_file = entry.path();
        let dest_file = dest_serena.join(&file_name);

        // Skip directories (like memories) - only copy config files
        if source_file.is_file() {
            fs::copy(&source_file, &dest_file).with_context(|| {
                format!("Failed to copy {} to {}", source_file.display(), dest_file.display())
            })?;
        }
    }

    tracing::debug!(
        operation = "copy_serena_config",
        source = %source_serena.display(),
        dest = %dest_serena.display(),
        "Copied Serena config to worktree"
    );

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

/// Detects if the prompt contains absolute paths pointing to the source
/// repository instead of relative paths. This is a common mistake that can
/// cause workers to modify the main repo instead of their worktree.
///
/// Returns a vector of (original_path, suggested_relative_path) pairs.
fn detect_source_repo_paths(prompt: &str, config: &Config) -> Vec<(String, String)> {
    let source_path = PathBuf::from(&config.repo.source);
    let source_str = source_path.to_string_lossy();

    // Look for @path references that point to the source repo
    // Matches patterns like @/path/to/source/repo/...
    let pattern = format!(r#"@"?{}/?([\w/._-]*)"?"#, regex::escape(&source_str));
    let re = Regex::new(&pattern).expect("Invalid regex pattern");

    let mut found_paths = Vec::new();
    for cap in re.captures_iter(prompt) {
        let full_match = cap.get(0).unwrap().as_str().to_string();
        let relative_part = cap.get(1).map_or("", |m| m.as_str());

        // Suggest using a relative path instead
        let suggested =
            if relative_part.is_empty() { "@.".to_string() } else { format!("@{}", relative_part) };

        found_paths.push((full_match, suggested));
    }

    found_paths
}

fn warn_about_source_repo_paths(prompt: &str, config: &Config, worktree_path: &Path) -> Result<()> {
    let problematic_paths = detect_source_repo_paths(prompt, config);

    if problematic_paths.is_empty() {
        return Ok(());
    }

    // Log the detected paths for debugging
    tracing::warn!(
        operation = "path_detection",
        source_repo = %config.repo.source,
        worktree = %worktree_path.display(),
        detected_count = problematic_paths.len(),
        detected_paths = ?problematic_paths,
        "Source repository paths detected in prompt - worker may modify main repo instead of worktree"
    );

    eprintln!("\n⚠️  WARNING: Source repository paths detected in prompt!\n");
    eprintln!("The worker's working directory is: {}\n", worktree_path.display());
    eprintln!("But your prompt contains absolute paths to the source repository:");
    eprintln!("  Source repo: {}\n", config.repo.source);

    for (original, suggested) in &problematic_paths {
        eprintln!("  Found:     {}", original);
        eprintln!("  Suggested: {}\n", suggested);
    }

    eprintln!("Using absolute paths to the source repository will cause the");
    eprintln!("worker to modify files in the main repo instead of its worktree.");
    eprintln!();
    eprintln!("Recommendations:");
    eprintln!("  • Use relative paths like @rules_engine/src/...");
    eprintln!("  • Or use the worktree path: @{}/...", worktree_path.display());
    eprintln!();

    eprint!("Continue anyway? [y/N] ");
    io::stderr().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if !input.trim().eq_ignore_ascii_case("y") {
        tracing::info!(
            operation = "path_detection",
            result = "aborted",
            "User aborted start due to source repo paths in prompt"
        );
        bail!("Aborted by user. Please fix the paths in your prompt and try again.");
    }

    tracing::warn!(
        operation = "path_detection",
        result = "user_override",
        "User chose to proceed despite source repo paths in prompt"
    );

    eprintln!();
    Ok(())
}
