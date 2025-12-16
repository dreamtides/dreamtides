use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;

use crate::core::{column_names, excel_reader, paths, toml_data};

pub fn build_toml(xlsm_path: Option<PathBuf>, output_dir: Option<PathBuf>) -> Result<()> {
    let spreadsheet_path = resolve_xlsm_path(xlsm_path)?;
    let output_dir = resolve_output_dir(output_dir)?;

    create_backup(&spreadsheet_path)?;

    let tables = excel_reader::extract_tables(&spreadsheet_path)?;
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("Cannot write to output directory {}", output_dir.display()))?;

    for table in tables {
        let toml_string = toml_data::table_to_toml(&table)?;
        let file_name = format!("{}.toml", column_names::normalize_table_name(table.name.as_str()));
        let file_path = output_dir.join(file_name);
        fs::write(&file_path, toml_string).with_context(|| {
            format!("Cannot write to output directory {}", output_dir.display())
        })?;
    }

    Ok(())
}

fn resolve_xlsm_path(xlsm_path: Option<PathBuf>) -> Result<PathBuf> {
    match xlsm_path {
        Some(path) => Ok(path),
        None => paths::default_xlsm_path(),
    }
}

fn resolve_output_dir(output_dir: Option<PathBuf>) -> Result<PathBuf> {
    match output_dir {
        Some(path) => Ok(path),
        None => paths::default_toml_dir(),
    }
}

fn create_backup(xlsm_path: &Path) -> Result<()> {
    let git_root = paths::git_root_for(xlsm_path)?;
    let backup_dir = paths::backup_dir_for(&git_root);
    fs::create_dir_all(&backup_dir)
        .with_context(|| format!("Cannot write to output directory {}", backup_dir.display()))?;
    let file_name = xlsm_path
        .file_name()
        .unwrap_or_else(|| OsStr::new("spreadsheet.xlsm"))
        .to_string_lossy()
        .to_string();
    let timestamp = Utc::now().format("%Y%m%d%H%M%S");
    let backup_path = backup_dir.join(format!("{timestamp}-{file_name}"));
    fs::copy(xlsm_path, &backup_path)
        .with_context(|| format!("Cannot write to output directory {}", backup_dir.display()))?;
    prune_backups(&backup_dir)?;
    Ok(())
}

fn prune_backups(backup_dir: &Path) -> Result<()> {
    let mut entries: Vec<_> = fs::read_dir(backup_dir)
        .with_context(|| format!("Cannot write to output directory {}", backup_dir.display()))?
        .filter_map(Result::ok)
        .collect();
    if entries.len() <= 50 {
        return Ok(());
    }
    entries.sort_by_key(|entry| std::cmp::Reverse(entry.file_name()));
    for entry in entries.into_iter().skip(50) {
        fs::remove_file(entry.path()).with_context(|| {
            format!("Cannot write to output directory {}", backup_dir.display())
        })?;
    }
    Ok(())
}
