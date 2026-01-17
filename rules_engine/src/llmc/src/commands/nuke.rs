use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use super::super::lock::StateLock;
use super::super::state::{self, State};
use super::super::tmux::session;
use super::super::{config, git};
use super::console;

/// Runs the nuke command, permanently removing a worker or console session
pub fn run_nuke(name: Option<&str>, all: bool, json: bool) -> Result<()> {
    let llmc_root = config::get_llmc_root();
    if !llmc_root.exists() {
        bail!(
            "LLMC workspace not initialized. Run 'llmc init' first.\n\
             Expected workspace at: {}",
            llmc_root.display()
        );
    }

    // Check if this is a console session (consoles don't need state lock or state
    // modification)
    if let Some(name) = name
        && console::is_console_name(name)
    {
        return nuke_console(name, json);
    }

    let _lock = StateLock::acquire()?;
    let state_path = state::get_state_path();
    let mut state = State::load(&state_path)?;
    let mut removed_workers = Vec::new();
    if all {
        if name.is_some() {
            bail!("Cannot specify both --all and a worker name");
        }
        let worker_names: Vec<_> = state.workers.keys().cloned().collect();
        if worker_names.is_empty() {
            if json {
                crate::json_output::print_json(&crate::json_output::NukeOutput {
                    workers_removed: vec![],
                });
            } else {
                println!("No workers to nuke.");
            }
            return Ok(());
        }
        if !json {
            println!("This will permanently delete {} workers:", worker_names.len());
            for name in &worker_names {
                println!("  - {}", name);
            }
            println!("\nProceed? [y/N] ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Cancelled.");
                return Ok(());
            }
        }
        for name in worker_names {
            if nuke_worker(&mut state, &llmc_root, &name)? {
                removed_workers.push(name);
            }
        }
        if !removed_workers.is_empty() {
            state.save(&state_path)?;
        }
        if json {
            crate::json_output::print_json(&crate::json_output::NukeOutput {
                workers_removed: removed_workers,
            });
        } else if !removed_workers.is_empty() {
            println!("✓ {} worker(s) have been nuked", removed_workers.len());
        }
    } else {
        let Some(worker) = name else {
            bail!("Worker name required (or use --all to nuke all workers)");
        };
        if nuke_worker(&mut state, &llmc_root, worker)? {
            state.save(&state_path)?;
            removed_workers.push(worker.to_string());
            if json {
                crate::json_output::print_json(&crate::json_output::NukeOutput {
                    workers_removed: removed_workers,
                });
            } else {
                println!("✓ Worker '{}' has been nuked", worker);
            }
        } else if json {
            crate::json_output::print_json(&crate::json_output::NukeOutput {
                workers_removed: vec![],
            });
        }
    }
    Ok(())
}

/// Nukes a console session (just kills the TMUX session)
fn nuke_console(name: &str, json: bool) -> Result<()> {
    let session_id = console::normalize_console_name(name);

    if !session::session_exists(&session_id) {
        let consoles = console::list_console_sessions()?;
        let available = if consoles.is_empty() { "none".to_string() } else { consoles.join(", ") };
        bail!(
            "Console session '{}' does not exist\n\
             Available consoles: {}",
            session_id,
            available
        );
    }

    // Confirm before nuking
    if !json {
        println!(
            "This will terminate console session '{}'.\n\
             \n\
             Proceed? [y/N] ",
            session_id
        );
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    if let Err(e) = session::kill_session(&session_id) {
        bail!("Failed to kill console session '{}': {}", session_id, e);
    }

    if json {
        crate::json_output::print_json(&crate::json_output::NukeOutput {
            workers_removed: vec![session_id.clone()],
        });
    } else {
        println!("✓ Console session '{}' has been terminated", session_id);
    }

    Ok(())
}
fn nuke_worker(state: &mut State, llmc_root: &Path, worker: &str) -> Result<bool> {
    let worker_record = state.get_worker(worker).ok_or_else(|| {
        anyhow::anyhow!(
            "Worker '{}' not found\n\
             Available workers: {}",
            worker,
            format_all_workers(state)
        )
    })?;
    let session_id = worker_record.session_id.clone();
    let worktree_path = PathBuf::from(&worker_record.worktree_path);
    let branch = worker_record.branch.clone();
    if !confirm_nuke(worker, &session_id, &worktree_path, &branch)? {
        tracing::info!("User cancelled nuke operation for worker '{}'", worker);
        println!("Cancelled nuking '{}'.", worker);
        return Ok(false);
    }
    println!("Nuking worker '{}'...", worker);
    if let Err(e) = session::kill_session(&session_id) {
        eprintln!("  ⚠ Failed to kill TMUX session {}: {}", session_id, e);
    } else {
        println!("  ✓ Killed TMUX session: {}", session_id);
    }
    if worktree_path.exists() {
        if let Err(e) = git::remove_worktree(llmc_root, &worktree_path, true) {
            eprintln!("  ⚠ Failed to remove worktree: {}", e);
        } else {
            println!("  ✓ Removed worktree: {}", worktree_path.display());
        }
    }
    if let Err(e) = git::delete_branch(llmc_root, &branch, true) {
        eprintln!("  ⚠ Failed to delete branch {}: {}", branch, e);
    } else {
        println!("  ✓ Deleted branch: {}", branch);
    }
    state.remove_worker(worker);
    println!("  ✓ Removed from state.json");
    if let Err(e) = remove_worker_from_config(worker) {
        eprintln!("  ⚠ Failed to remove worker from config.toml: {}", e);
    } else {
        println!("  ✓ Removed from config.toml (if present)");
    }
    tracing::info!("Successfully nuked worker '{}'", worker);
    Ok(true)
}
fn confirm_nuke(
    worker: &str,
    session_id: &str,
    worktree_path: &Path,
    branch: &str,
) -> Result<bool> {
    println!(
        "This will permanently remove worker '{}':\n\
         \n\
         Removes:\n\
         - TMUX session: {}\n\
         - Worktree: {}\n\
         - Branch: {}\n\
         - Any uncommitted work will be LOST\n\
         \n\
         Proceed? [y/N] ",
        worker,
        session_id,
        worktree_path.display(),
        branch
    );
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().eq_ignore_ascii_case("y"))
}
fn format_all_workers(state: &State) -> String {
    if state.workers.is_empty() {
        return "none".to_string();
    }
    state.workers.keys().map(String::as_str).collect::<Vec<_>>().join(", ")
}
fn remove_worker_from_config(worker: &str) -> Result<()> {
    let config_path = config::get_config_path();
    let config_content = fs::read_to_string(&config_path).context("Failed to read config.toml")?;
    let section_header = format!("[workers.{}]", worker);
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
    let new_content = new_lines.join("\n");
    if !new_content.ends_with('\n') && !new_content.is_empty() {
        fs::write(&config_path, format!("{}\n", new_content))
            .context("Failed to write config.toml")?;
    } else {
        fs::write(&config_path, new_content).context("Failed to write config.toml")?;
    }
    Ok(())
}
