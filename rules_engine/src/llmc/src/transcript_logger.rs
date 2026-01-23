use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use tracing::{debug, error, info, warn};

use crate::config;
/// Copies a Claude transcript file to the logs directory for archival.
///
/// Transcripts are stored in
/// `logs/transcripts/<worker>/<timestamp>_<session_id>.jsonl`. This allows
/// deep-dive analysis of worker sessions after task completion or failure.
pub fn archive_transcript(
    worker_name: &str,
    transcript_path: Option<&str>,
    session_id: Option<&str>,
) -> Result<Option<PathBuf>> {
    let Some(source_path) = transcript_path else {
        debug!(worker = % worker_name, "No transcript path available for archival");
        return Ok(None);
    };
    let source = Path::new(source_path);
    if !source.exists() {
        warn!(
            worker = % worker_name, path = % source_path,
            "Transcript file does not exist, skipping archival"
        );
        return Ok(None);
    }
    let llmc_root = config::get_llmc_root();
    let transcripts_dir = llmc_root.join("logs").join("transcripts").join(worker_name);
    fs::create_dir_all(&transcripts_dir).with_context(|| {
        format!("Failed to create transcripts directory: {}", transcripts_dir.display())
    })?;
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let session_suffix =
        session_id.map(|s| format!("_{}", sanitize_filename(s))).unwrap_or_default();
    let dest_filename = format!("{}{}.jsonl", timestamp, session_suffix);
    let dest_path = transcripts_dir.join(&dest_filename);
    match fs::copy(source, &dest_path) {
        Ok(bytes) => {
            info!(
                worker = % worker_name, source = % source_path, dest = % dest_path
                .display(), bytes = bytes, "Archived transcript for analysis"
            );
            Ok(Some(dest_path))
        }
        Err(e) => {
            error!(
                worker = % worker_name, source = % source_path, dest = % dest_path
                .display(), error = % e, "Failed to copy transcript file"
            );
            Err(e).context("Failed to archive transcript")
        }
    }
}
/// Clears the stored transcript info from a worker record.
///
/// Called after archival to prepare for the next task.
pub fn clear_transcript_info(
    transcript_session_id: &mut Option<String>,
    transcript_path: &mut Option<String>,
) {
    *transcript_session_id = None;
    *transcript_path = None;
}
fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .take(50)
        .collect()
}
