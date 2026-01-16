use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};

use super::super::config::Config;
use super::super::state::{State, WorkerRecord, WorkerStatus};
use super::super::tmux::session;
use super::super::{config, git, state};

/// Adds a new worker to the LLMC system
pub fn run_add(
    name: &str,
    model: Option<String>,
    role_prompt: Option<String>,
    excluded_from_pool: bool,
    self_review: bool,
) -> Result<()> {
    validate_worker_name(name)?;

    if let Some(ref m) = model {
        config::validate_model(m)?;
    }

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

    if state.get_worker(name).is_some() {
        bail!(
            "Worker '{}' already exists.\n\
             Use 'llmc nuke {}' to remove it first, or choose a different name.",
            name,
            name
        );
    }

    let session_id = format!("llmc-{}", name);
    if session::session_exists(&session_id) {
        bail!(
            "A TMUX session named '{}' already exists.\n\
             This might be an orphaned session from a previous worker.\n\
             \n\
             To fix this:\n\
             • Run 'llmc doctor --repair' to clean up orphaned sessions automatically\n\
             • Or manually kill it: tmux kill-session -t {}",
            session_id,
            session_id
        );
    }

    println!("Adding worker: {}", name);

    println!("Fetching latest master...");
    git::fetch_origin(&llmc_root)?;

    let branch_name = format!("llmc/{}", name);
    let worktree_path = llmc_root.join(".worktrees").join(name);

    create_branch(&llmc_root, &branch_name)?;
    create_worktree_for_worker(&llmc_root, &branch_name, &worktree_path)?;
    copy_tabula_to_worktree(&llmc_root, &worktree_path)?;
    create_serena_project(&worktree_path, name)?;

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
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
        self_review,
    };

    state.add_worker(worker_record);
    state.save(&state_path)?;

    add_worker_to_config(name, model, role_prompt, excluded_from_pool, self_review)?;

    println!("✓ Worker '{}' added successfully!", name);
    println!("\nWorktree: {}", worktree_path.display());
    println!("Branch: llmc/{}", name);

    let daemon_running = is_daemon_running();
    if daemon_running {
        println!("\nDaemon is running, starting worker session...");
        match start_worker_immediately(name) {
            Ok(()) => {
                println!("✓ Worker session started and ready for tasks");
                println!("\nNext step:");
                println!("  Run 'llmc start {}' to assign a task", name);
            }
            Err(e) => {
                eprintln!("Warning: Failed to start worker session: {}", e);
                println!("The daemon will retry starting this worker automatically.");
                println!("\nNext step:");
                println!(
                    "  Run 'llmc status' to check when worker is ready, then 'llmc start {}'",
                    name
                );
            }
        }
    } else {
        println!("\nNext steps:");
        println!("  1. Run 'llmc up' to start the daemon and bring this worker online");
        println!("  2. Run 'llmc start {}' to assign a task", name);
    }

    Ok(())
}

/// Recreates a missing worktree for an existing worker
pub fn recreate_missing_worktree(
    worker_name: &str,
    branch_name: &str,
    worktree_path: &Path,
) -> Result<()> {
    let llmc_root = config::get_llmc_root();

    tracing::info!(
        "Recreating missing worktree for worker '{}' at {}",
        worker_name,
        worktree_path.display()
    );

    git::remove_worktree(&llmc_root, worktree_path, true)
        .context("Failed to remove stale worktree registration")?;

    git::fetch_origin(&llmc_root).context("Failed to fetch latest master")?;

    create_branch(&llmc_root, branch_name)?;
    create_worktree_for_worker(&llmc_root, branch_name, worktree_path)?;
    copy_tabula_to_worktree(&llmc_root, worktree_path)?;
    create_serena_project(worktree_path, worker_name)?;

    Ok(())
}

pub fn copy_tabula_to_worktree(repo: &Path, worktree_path: &Path) -> Result<()> {
    let source_tabula = repo.join("Tabula.xlsm");

    if !source_tabula.exists() {
        return Ok(());
    }

    let target_tabula = worktree_path.join("client/Assets/StreamingAssets/Tabula.xlsm");

    if target_tabula.exists() {
        return Ok(());
    }

    println!("Copying Tabula.xlsm to worktree...");

    if let Some(parent) = target_tabula.parent() {
        fs::create_dir_all(parent).context("Failed to create StreamingAssets directory")?;
    }

    fs::copy(&source_tabula, &target_tabula).context("Failed to copy Tabula.xlsm to worktree")?;

    Ok(())
}

/// Creates .serena/project.yml in the worktree with a unique project name
/// and registers the project path in Serena's global config.
/// This file is gitignored, so each worktree gets its own copy.
pub fn create_serena_project(worktree_path: &Path, worker_name: &str) -> Result<()> {
    let serena_dir = worktree_path.join(".serena");
    let project_yml = serena_dir.join("project.yml");

    fs::create_dir_all(&serena_dir).context("Failed to create .serena directory")?;

    let content = format!(
        r#"languages:
- rust

encoding: "utf-8"

ignore_all_files_in_gitignore: true

ignored_paths: []

read_only: false

excluded_tools: []

initial_prompt: ""

project_name: "dreamtides-{}"
included_optional_tools: []
"#,
        worker_name
    );

    fs::write(&project_yml, content).context("Failed to write .serena/project.yml")?;
    println!("Created Serena project config: dreamtides-{}", worker_name);

    // Register the project in Serena's global config
    register_serena_project(worktree_path)?;

    Ok(())
}

pub fn is_daemon_running() -> bool {
    session::list_sessions()
        .ok()
        .map(|sessions| sessions.iter().any(|s| s.starts_with("llmc-")))
        .unwrap_or(false)
}

/// Registers a project path in Serena's global config
/// (~/.serena/serena_config.yml)
fn register_serena_project(worktree_path: &Path) -> Result<()> {
    let home = std::env::var("HOME").context("HOME environment variable not set")?;
    let config_path = PathBuf::from(home).join(".serena/serena_config.yml");

    if !config_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&config_path).context("Failed to read Serena config")?;
    let worktree_str = worktree_path.to_string_lossy();

    // Check if already registered
    if content.contains(&*worktree_str) {
        return Ok(());
    }

    // Find the projects: section and add the new path
    let updated = if let Some(pos) = content.find("\nprojects:\n") {
        let insert_pos = pos + "\nprojects:\n".len();
        let (before, after) = content.split_at(insert_pos);
        format!("{}- {}\n{}", before, worktree_str, after)
    } else if let Some(pos) = content.find("projects:\n") {
        let insert_pos = pos + "projects:\n".len();
        let (before, after) = content.split_at(insert_pos);
        format!("{}- {}\n{}", before, worktree_str, after)
    } else {
        // No projects section found, append one
        format!("{}\nprojects:\n- {}\n", content.trim_end(), worktree_str)
    };

    fs::write(&config_path, updated).context("Failed to update Serena config")?;
    println!("Registered project with Serena: {}", worktree_str);

    Ok(())
}

fn validate_worker_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("Worker name cannot be empty");
    }

    if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        bail!(
            "Invalid worker name: '{}'\n\
             Worker names must contain only alphanumeric characters, underscores, and hyphens.",
            name
        );
    }

    Ok(())
}

fn create_branch(repo: &Path, branch_name: &str) -> Result<()> {
    println!("Creating branch {} from origin/master...", branch_name);

    if git::branch_exists(repo, branch_name) {
        println!("  Branch already exists (reusing)");
        return Ok(());
    }

    git::create_branch(repo, branch_name, "origin/master")?;
    Ok(())
}

fn create_worktree_for_worker(repo: &Path, branch_name: &str, worktree_path: &Path) -> Result<()> {
    println!("Creating worktree at {}...", worktree_path.display());

    if worktree_path.exists() {
        bail!(
            "Worktree path already exists: {}\n\
             This should not happen. Please remove it manually and try again.",
            worktree_path.display()
        );
    }

    git::create_worktree(repo, branch_name, worktree_path)?;
    Ok(())
}

fn add_worker_to_config(
    name: &str,
    model: Option<String>,
    role_prompt: Option<String>,
    excluded_from_pool: bool,
    self_review: bool,
) -> Result<()> {
    println!("Adding worker to config.toml...");

    let config_path = config::get_config_path();
    let config_content = fs::read_to_string(&config_path).context("Failed to read config.toml")?;

    let section_header = format!("[workers.{}]", name);
    let lines: Vec<&str> = config_content.lines().collect();

    let mut new_lines = Vec::new();
    let mut skip_section = false;

    for line in lines {
        let trimmed = line.trim();

        if trimmed == section_header {
            skip_section = true;
            continue;
        }

        if skip_section {
            if trimmed.starts_with('[') {
                skip_section = false;
            } else {
                continue;
            }
        }

        new_lines.push(line);
    }

    let mut config_content = new_lines.join("\n");
    if !config_content.ends_with('\n') && !config_content.is_empty() {
        config_content.push('\n');
    }

    let worker_config_section = format!("\n[workers.{}]\n", name);

    let mut worker_lines = Vec::new();
    if let Some(m) = model {
        worker_lines.push(format!("model = \"{}\"", m));
    }
    if let Some(rp) = role_prompt {
        worker_lines.push(format!("role_prompt = \"{}\"", rp));
    }
    if excluded_from_pool {
        worker_lines.push("excluded_from_pool = true".to_string());
    }
    if self_review {
        worker_lines.push("self_review = true".to_string());
    }

    if worker_lines.is_empty() {
        worker_lines.push("# Uses defaults from [defaults] section".to_string());
    }

    let worker_config = format!("{}{}\n", worker_config_section, worker_lines.join("\n"));
    config_content.push_str(&worker_config);

    fs::write(&config_path, config_content).context("Failed to write config.toml")?;

    Config::load(&config_path)?;

    Ok(())
}

fn start_worker_immediately(name: &str) -> Result<()> {
    let config_path = config::get_config_path();
    let config = Config::load(&config_path)?;
    let worker_config = config.get_worker(name).with_context(|| {
        format!("Worker '{}' not found in config after adding. This should not happen.", name)
    })?;

    let state_path = state::get_state_path();
    let mut state = State::load(&state_path)?;
    let worker_record = state.get_worker(name).with_context(|| {
        format!("Worker '{}' not found in state after adding. This should not happen.", name)
    })?;

    let worktree_path = Path::new(&worker_record.worktree_path);
    let session_id = &worker_record.session_id;

    if session::session_exists(session_id) {
        return Ok(());
    }

    session::start_worker_session(session_id, worktree_path, worker_config, false)?;

    let worker_mut = state.get_worker_mut(name).unwrap();
    worker_mut.status = WorkerStatus::Idle;
    worker_mut.last_activity_unix = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    state.save(&state_path)?;

    Ok(())
}
