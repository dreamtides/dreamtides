use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::config::Config;
use crate::state::State;
use crate::{config, git};
/// Initializes a new LLMC workspace
pub fn run_init(source: Option<PathBuf>, target: Option<PathBuf>, force: bool) -> Result<()> {
    let target_dir = target.unwrap_or_else(config::get_llmc_root);
    if target_dir.exists() {
        if force {
            println!("Removing existing directory at {}", target_dir.display());
            fs::remove_dir_all(&target_dir).context("Failed to remove existing directory")?;
        } else {
            bail!(
                "Target directory already exists: {}\nPlease remove it first, use --force to overwrite, or specify a different target with --target",
                target_dir.display()
            );
        }
    }
    let source_dir = if let Some(src) = source {
        src
    } else {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .context("Could not determine home directory")?;
        PathBuf::from(home).join("Documents/GoogleDrive/dreamtides")
    };
    if !source_dir.exists() {
        bail!("Source directory does not exist: {}", source_dir.display());
    }
    if !is_git_repo(&source_dir)? {
        bail!("Source directory is not a git repository: {}", source_dir.display());
    }
    println!("Initializing LLMC workspace at {}", target_dir.display());
    println!("Source repository: {}", source_dir.display());
    clone_repository(&source_dir, &target_dir)?;
    configure_git(&target_dir)?;
    install_lfs_if_available(&target_dir);
    create_directory_structure(&target_dir)?;
    create_config_file(&target_dir, &source_dir)?;
    create_initial_state(&target_dir)?;
    copy_tabula_if_present(&source_dir, &target_dir)?;
    println!("\n✓ LLMC workspace initialized successfully!");
    println!("\nNext steps:");
    println!("  1. Review and customize {}", target_dir.join("config.toml").display());
    println!("  2. Run 'llmc add <name>' to create workers");
    println!("  3. Run 'llmc up' to start the daemon");
    Ok(())
}
fn is_git_repo(path: &Path) -> Result<bool> {
    Ok(path.join(".git").exists())
}
fn clone_repository(source: &Path, target: &Path) -> Result<()> {
    println!("Cloning repository with --local...");
    let output = Command::new("git")
        .arg("clone")
        .arg("--local")
        .arg(source)
        .arg(target)
        .output()
        .context("Failed to execute git clone")?;
    if !output.status.success() {
        bail!("Failed to clone repository: {}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}
fn configure_git(repo: &Path) -> Result<()> {
    println!("Configuring git rerere...");
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("config")
        .arg("rerere.enabled")
        .arg("true")
        .output()
        .context("Failed to configure git rerere")?;
    if !output.status.success() {
        eprintln!("Warning: Failed to enable git rerere (non-fatal)");
    }
    Ok(())
}
fn install_lfs_if_available(repo: &Path) {
    println!("Installing git LFS hooks (if available)...");
    let output = Command::new("git").arg("-C").arg(repo).arg("lfs").arg("install").output();
    match output {
        Ok(out) if out.status.success() => {
            println!("✓ Git LFS installed");
        }
        _ => {
            println!("  Git LFS not available (skipping)");
        }
    }
}
fn create_directory_structure(target: &Path) -> Result<()> {
    println!("Creating directory structure...");
    fs::create_dir_all(target.join("logs")).context("Failed to create logs directory")?;
    fs::create_dir_all(target.join(".worktrees"))
        .context("Failed to create .worktrees directory")?;
    Ok(())
}
fn create_config_file(target: &Path, source: &Path) -> Result<()> {
    println!("Creating config.toml...");
    let config_path = target.join("config.toml");
    let source_str = source.to_string_lossy();
    // Detect the default branch from the cloned repository
    let default_branch = git::detect_default_branch(target);
    println!("  Detected default branch: {}", default_branch);
    let config_content = format!(
        r#"[defaults]
model = "opus"
skip_permissions = true
# allowed_tools = ["Bash", "Edit", "Read", "Write", "Glob", "Grep"]
# patrol_interval_secs = 60
# sound_on_review = true

[repo]
source = "{}"
default_branch = "{}"

# Example worker configuration:
# [workers.example]
# model = "sonnet"
# role_prompt = "You are Example, focused on..."
"#,
        source_str, default_branch
    );
    fs::write(&config_path, config_content).context("Failed to write config.toml")?;
    let _ = Config::load(&config_path)?;
    Ok(())
}
fn create_initial_state(target: &Path) -> Result<()> {
    println!("Creating state.json...");
    let state_path = target.join("state.json");
    let state = State::new();
    state.save(&state_path)?;
    Ok(())
}
fn copy_tabula_if_present(source: &Path, target: &Path) -> Result<()> {
    let source_tabula = source.join("client/Assets/StreamingAssets/Tabula.xlsm");
    if source_tabula.exists() {
        println!("Copying Tabula.xlsm...");
        let target_tabula = target.join("Tabula.xlsm");
        fs::copy(&source_tabula, &target_tabula).context("Failed to copy Tabula.xlsm")?;
    } else {
        println!("  Tabula.xlsm not found in source (skipping)");
    }
    Ok(())
}
