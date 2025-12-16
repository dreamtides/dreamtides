use std::env;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

pub fn git_root() -> Result<PathBuf> {
    let cwd = env::current_dir().context("Failed to read current directory")?;
    find_git_root(&cwd)
}

pub fn git_root_for(path: &Path) -> Result<PathBuf> {
    let start = if path.is_dir() { path } else { path.parent().unwrap_or(path) };
    find_git_root(start)
}

pub fn default_xlsm_path() -> Result<PathBuf> {
    Ok(git_root()?.join("client/Assets/StreamingAssets/Tabula.xlsm"))
}

pub fn default_toml_dir() -> Result<PathBuf> {
    Ok(git_root()?.join("client/Assets/StreamingAssets/Tabula"))
}

pub fn backup_dir_for(root: &Path) -> PathBuf {
    root.join(".git/excel-backups")
}

pub fn image_cache_dir_for(root: &Path) -> PathBuf {
    root.join(".git/xlsm_image_cache")
}

fn find_git_root(start: &Path) -> Result<PathBuf> {
    let mut current = Some(start);
    while let Some(dir) = current {
        let candidate = dir.join(".git");
        if candidate.is_dir() {
            return Ok(dir.to_path_buf());
        }
        current = dir.parent();
    }
    bail!("Unable to locate .git directory starting from {}", start.display());
}
