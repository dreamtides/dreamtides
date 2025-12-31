use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use atomic_write_file::AtomicWriteFile;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentRecord {
    pub agent_id: String,
    pub branch: String,
    pub worktree_path: PathBuf,
    pub runtime: Runtime,
    pub prompt: String,
    pub created_at_unix: u64,
    pub last_run_unix: u64,
    pub status: AgentStatus,
    pub last_pid: Option<u32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct StateFile {
    pub agents: BTreeMap<String, AgentRecord>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Idle,
    Running,
    Rebasing,
    NeedsReview,
    Accepted,
    Rejected,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
#[clap(rename_all = "lowercase")]
pub enum Runtime {
    Claude,
    Codex,
    Gemini,
    Cursor,
}

/// Load the LLMC state file if present, otherwise return a default state.
pub fn load_state(path: &Path) -> Result<StateFile> {
    if !path.exists() {
        return Ok(StateFile::default());
    }

    let data = fs::read(path).with_context(|| format!("Failed to read state file {path:?}"))?;
    serde_json::from_slice(&data).with_context(|| format!("Failed to parse state file {path:?}"))
}

/// Write the LLMC state file atomically to disk.
pub fn save_state(path: &Path, state: &StateFile) -> Result<()> {
    let Some(parent) = path.parent() else {
        return Err(anyhow::anyhow!("State path has no parent: {path:?}"));
    };

    fs::create_dir_all(parent)
        .with_context(|| format!("Failed to create state directory {parent:?}"))?;

    let data = serde_json::to_vec_pretty(state)
        .with_context(|| format!("Failed to serialize state file {path:?}"))?;

    let mut file = AtomicWriteFile::options()
        .open(path)
        .with_context(|| format!("Failed to open state file {path:?}"))?;
    file.write_all(&data).with_context(|| format!("Failed to write state file {path:?}"))?;
    file.commit().with_context(|| format!("Failed to commit state file {path:?}"))?;

    Ok(())
}
