use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

use anyhow::{Context, Result};

#[derive(Clone, Debug)]
pub struct RepoPaths {
    pub repo_root: PathBuf,
    pub llmc_dir: PathBuf,
    pub worktrees_dir: PathBuf,
}

/// Resolve the repository root from an override or git rev-parse.
pub fn repo_root(repo_override: Option<&Path>) -> Result<PathBuf> {
    let Some(repo_override) = repo_override else {
        return self::git_repo_root();
    };

    fs::canonicalize(repo_override)
        .with_context(|| format!("Failed to canonicalize repo override {repo_override:?}"))
}

/// Build commonly used LLMC directories based on the repo root.
pub fn repo_paths(repo_override: Option<&Path>) -> Result<RepoPaths> {
    let repo_root = self::repo_root(repo_override)?;
    Ok(RepoPaths {
        llmc_dir: repo_root.join(".llmc"),
        worktrees_dir: repo_root.join(".worktrees"),
        repo_root,
    })
}

/// Default target directory for llmc setup when none is provided.
pub fn default_target_dir() -> Result<PathBuf> {
    let Some(home) = env::var_os("HOME") else {
        return Err(anyhow::anyhow!("HOME is not set"));
    };

    Ok(PathBuf::from(home).join("Documents").join("llmc"))
}

fn git_repo_root() -> Result<PathBuf> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()
        .with_context(|| "Failed to run git rev-parse --show-toplevel")?;

    anyhow::ensure!(output.status.success(), "git rev-parse --show-toplevel failed");

    let root = String::from_utf8(output.stdout)
        .with_context(|| "git rev-parse --show-toplevel output was not UTF-8")?;
    let root = root.trim();

    anyhow::ensure!(!root.is_empty(), "git rev-parse --show-toplevel returned empty output");

    Ok(PathBuf::from(root))
}
