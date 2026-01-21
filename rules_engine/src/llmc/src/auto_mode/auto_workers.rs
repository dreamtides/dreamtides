// TODO: Remove this allow once auto_workers is integrated with
// auto_orchestrator
#![allow(dead_code)]

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use tracing::info;

use crate::commands::add;
use crate::config::Config;
use crate::state::{State, WorkerRecord, WorkerStatus};
use crate::tmux::session;
use crate::{config, git};

/// Prefix used for auto worker names.
pub const AUTO_WORKER_PREFIX: &str = "auto-";

/// Returns the name for an auto worker at the given index (1-based).
///
/// Auto workers are named `auto-1`, `auto-2`, etc.
pub fn auto_worker_name(index: u32) -> String {
    format!("{}{}", AUTO_WORKER_PREFIX, index)
}

/// Returns true if the given worker name is an auto worker.
///
/// Auto workers have names that start with "auto-".
pub fn is_auto_worker(name: &str) -> bool {
    name.starts_with(AUTO_WORKER_PREFIX)
}

/// Generates the list of auto worker names for a given concurrency.
///
/// For concurrency N, returns ["auto-1", "auto-2", ..., "auto-N"].
pub fn generate_auto_worker_names(concurrency: u32) -> Vec<String> {
    (1..=concurrency).map(auto_worker_name).collect()
}

/// Ensures all auto workers exist up to the configured concurrency.
///
/// Creates missing auto workers with `excluded_from_pool: true`.
/// Returns the names of all auto workers (existing and newly created).
pub fn ensure_auto_workers_exist(
    state: &mut State,
    config: &Config,
    concurrency: u32,
) -> Result<Vec<String>> {
    let worker_names = generate_auto_worker_names(concurrency);
    let mut created = Vec::new();
    for name in &worker_names {
        if state.get_worker(name).is_none() {
            info!(worker = %name, "Creating auto worker");
            create_auto_worker(state, config, name)?;
            created.push(name.clone());
        }
    }
    if !created.is_empty() {
        info!(workers = ?created, "Created {} auto worker(s)", created.len());
    }
    Ok(worker_names)
}

/// Starts the TMUX session for an auto worker if not already running.
pub fn start_auto_worker_session(worker: &WorkerRecord, config: &Config) -> Result<()> {
    if session::session_exists(&worker.session_id) {
        return Ok(());
    }
    let worktree_path = std::path::Path::new(&worker.worktree_path);
    let Some(worker_config) = config.get_worker(&worker.name) else {
        anyhow::bail!(
            "Auto worker '{}' not found in config.toml. \
             This indicates a configuration error during auto worker creation.",
            worker.name
        );
    };
    session::start_worker_session(&worker.session_id, worktree_path, worker_config, false)
        .with_context(|| format!("Failed to start session for auto worker '{}'", worker.name))
}

/// Gets all idle auto workers from state.
pub fn get_idle_auto_workers(state: &State) -> Vec<&WorkerRecord> {
    state
        .workers
        .values()
        .filter(|w| is_auto_worker(&w.name) && w.status == WorkerStatus::Idle)
        .collect()
}

/// Removes auto workers from state and cleans up their resources.
///
/// This is called during auto mode shutdown to clean up auto-specific state
/// while preserving the workers themselves (which can be nuked manually).
pub fn clear_auto_mode_state(state: &mut State) {
    state.auto_mode = false;
    state.auto_workers.clear();
}

/// Records that auto mode is active with the given workers.
pub fn set_auto_mode_active(state: &mut State, worker_names: Vec<String>) {
    state.auto_mode = true;
    state.auto_workers = worker_names;
}

/// Updates the last task completion timestamp.
pub fn record_task_completion(state: &mut State) {
    state.last_task_completion_unix = Some(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|e| panic!("System time before UNIX epoch: {}", e))
            .as_secs(),
    );
}

/// Creates a single auto worker.
///
/// Auto workers are created with:
/// - `excluded_from_pool: true` (cannot receive manual `llmc start` tasks)
/// - Worktree at `.worktrees/auto-N`
/// - TMUX session named `llmc-auto-N`
fn create_auto_worker(state: &mut State, config: &Config, name: &str) -> Result<()> {
    let llmc_root = config::get_llmc_root();
    git::fetch_origin(&llmc_root).context("Failed to fetch origin")?;
    let branch_name = format!("llmc/{}", name);
    let worktree_path = llmc_root.join(".worktrees").join(name);
    create_branch_if_missing(&llmc_root, &branch_name)?;
    create_worktree_if_missing(&llmc_root, &branch_name, &worktree_path)?;
    add::copy_tabula_to_worktree(&llmc_root, &worktree_path)?;
    add::create_serena_project(&worktree_path, name)?;
    add::create_claude_hook_settings_silent(&worktree_path, name)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|e| panic!("System time before UNIX epoch: {}", e))
        .as_secs();
    let worker_record = WorkerRecord {
        name: name.to_string(),
        worktree_path: worktree_path.to_string_lossy().to_string(),
        branch: branch_name,
        status: WorkerStatus::Offline,
        current_prompt: String::new(),
        prompt_cmd: None,
        created_at_unix: now,
        last_activity_unix: now,
        commit_sha: None,
        session_id: format!("llmc-{}", name),
        crash_count: 0,
        last_crash_unix: None,
        on_complete_sent_unix: None,
        self_review: false,
        pending_self_review: false,
        commits_first_detected_unix: None,
        pending_rebase_prompt: false,
        error_reason: None,
    };
    state.add_worker(worker_record);
    add_auto_worker_to_config(name, config)?;
    Ok(())
}

/// Creates the git branch if it doesn't exist.
fn create_branch_if_missing(repo: &Path, branch_name: &str) -> Result<()> {
    if git::branch_exists(repo, branch_name) {
        return Ok(());
    }
    git::create_branch(repo, branch_name, "origin/master")
        .with_context(|| format!("Failed to create branch {}", branch_name))
}

/// Creates the worktree if it doesn't exist.
fn create_worktree_if_missing(repo: &Path, branch_name: &str, worktree_path: &Path) -> Result<()> {
    if worktree_path.exists() {
        return Ok(());
    }
    git::create_worktree(repo, branch_name, worktree_path)
        .with_context(|| format!("Failed to create worktree at {}", worktree_path.display()))
}

/// Adds the auto worker to config.toml with excluded_from_pool = true.
fn add_auto_worker_to_config(name: &str, _config: &Config) -> Result<()> {
    let config_path = config::get_config_path();
    let config_content =
        std::fs::read_to_string(&config_path).context("Failed to read config.toml")?;
    let section_header = format!("[workers.{}]", name);
    if config_content.contains(&section_header) {
        return Ok(());
    }
    let worker_section = format!("\n{}\nexcluded_from_pool = true\n", section_header);
    let mut updated_content = config_content;
    if !updated_content.ends_with('\n') && !updated_content.is_empty() {
        updated_content.push('\n');
    }
    updated_content.push_str(&worker_section);
    std::fs::write(&config_path, updated_content).context("Failed to write config.toml")?;
    Config::load(&config_path).context("Failed to validate updated config")?;
    Ok(())
}
