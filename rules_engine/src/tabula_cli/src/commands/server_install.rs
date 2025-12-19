use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

use anyhow::{Context, Result, bail};

pub fn server_install() -> Result<()> {
    let home = env::var("HOME").context("HOME environment variable not set")?;
    let target_dir =
        PathBuf::from(home).join("Library").join("Application Scripts").join("com.microsoft.Excel");
    fs::create_dir_all(&target_dir).with_context(|| {
        format!("Cannot create AppleScriptTask directory {}", target_dir.display())
    })?;
    let source_path = target_dir.join(SOURCE_NAME);
    fs::write(&source_path, APPLESCRIPT_SOURCE).with_context(|| {
        format!("Cannot write AppleScriptTask helper to {}", source_path.display())
    })?;
    let target_path = target_dir.join(SCRIPT_NAME);
    let status = Command::new("osacompile")
        .arg("-o")
        .arg(&target_path)
        .arg(&source_path)
        .status()
        .with_context(|| format!("Cannot run osacompile for {}", source_path.display()))?;
    if !status.success() {
        bail!("osacompile failed when creating {}", target_path.display());
    }
    Ok(())
}

const SCRIPT_NAME: &str = "tabula_server_curl.scpt";
const SOURCE_NAME: &str = "tabula_server_curl.applescript";
const APPLESCRIPT_SOURCE: &str = include_str!("../../applescript/tabula_server_curl.applescript");
