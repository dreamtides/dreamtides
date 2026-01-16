use std::env;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

pub fn git_root() -> Result<PathBuf> {
    let cwd = env::current_dir().context("Failed to read current directory")?;
    locate_project_root(&cwd)
}

pub fn git_root_for(path: &Path) -> Result<PathBuf> {
    let start = if path.is_dir() { path } else { path.parent().unwrap_or(path) };
    locate_project_root(start)
}

pub fn default_xlsm_path() -> Result<PathBuf> {
    Ok(git_root()?.join("client/Assets/StreamingAssets/Tabula.xlsm"))
}

pub fn default_toml_dir() -> Result<PathBuf> {
    Ok(git_root()?.join("client/Assets/StreamingAssets/Tabula"))
}

pub fn backup_dir_for(root: &Path) -> PathBuf {
    git_dir_for(root).join("excel-backups")
}

pub fn image_cache_dir_for(root: &Path) -> PathBuf {
    git_dir_for(root).join("xlsm_image_cache")
}

fn git_dir_for(root: &Path) -> PathBuf {
    let git_path = root.join(".git");
    if git_path.is_file() {
        // Git worktree: .git is a file containing "gitdir: /path/to/git/dir"
        if let Ok(content) = std::fs::read_to_string(&git_path)
            && let Some(gitdir) = content.strip_prefix("gitdir: ")
        {
            return PathBuf::from(gitdir.trim());
        }
    }
    git_path
}

fn locate_project_root(start: &Path) -> Result<PathBuf> {
    if let Some(found) = find_root(start) {
        return Ok(found);
    }
    if let Some(found) = manifest_root() {
        return Ok(found);
    }
    bail!("Unable to locate project root starting from {}", start.display());
}

fn find_root(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(dir) = current {
        let justfile = dir.join("justfile");
        if justfile.is_file() {
            return Some(dir.to_path_buf());
        }
        let git = dir.join(".git");
        if git.is_dir() {
            return Some(dir.to_path_buf());
        }
        current = dir.parent();
    }
    None
}

fn manifest_root() -> Option<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().and_then(|p| p.parent()).and_then(|p| p.parent()).map(Path::to_path_buf)
}
