use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::Result;
use serde::Deserialize;

/// Information about API errors found in a transcript.
#[derive(Debug, Clone, Default)]
pub struct ApiErrorInfo {
    /// Number of API errors found in the transcript
    pub error_count: usize,
    /// The most recent API error message, if any
    pub last_error_message: Option<String>,
    /// Whether any errors were 500-series server errors
    pub has_500_error: bool,
    /// Whether any errors were rate limit (429) errors
    pub has_rate_limit_error: bool,
}

/// Scans a Claude transcript file for API error entries.
///
/// Claude Code logs API errors in transcript files with `"isApiErrorMessage":
/// true`. This function reads the transcript and extracts information about any
/// API errors.
pub fn scan_transcript_for_api_errors(
    transcript_path: &Path,
    max_entries: usize,
) -> Result<ApiErrorInfo> {
    let file = File::open(transcript_path)?;
    let reader = BufReader::new(file);
    let mut entries: Vec<(bool, Option<String>)> = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(entry) = serde_json::from_str::<TranscriptEntry>(&line)
            && entry.is_api_error_message
        {
            let error_text =
                entry.message.and_then(|m| m.content.into_iter().next()).and_then(|c| c.text);
            entries.push((true, error_text));
        }
    }
    let recent_entries: Vec<_> = if max_entries > 0 && entries.len() > max_entries {
        entries.into_iter().rev().take(max_entries).collect()
    } else {
        entries
    };
    let mut has_500_error = false;
    let mut has_rate_limit_error = false;
    let mut last_error_message = None;
    for (_, error_text) in &recent_entries {
        if let Some(text) = error_text {
            last_error_message = error_text.clone();
            let text_lower = text.to_lowercase();
            if text_lower.contains("500")
                || text_lower.contains("502")
                || text_lower.contains("503")
                || text_lower.contains("504")
                || text_lower.contains("internal server error")
                || text_lower.contains("api_error")
            {
                has_500_error = true;
            }
            if text_lower.contains("429")
                || text_lower.contains("rate limit")
                || text_lower.contains("too many requests")
            {
                has_rate_limit_error = true;
            }
        }
    }
    Ok(ApiErrorInfo {
        error_count: recent_entries.len(),
        last_error_message,
        has_500_error,
        has_rate_limit_error,
    })
}

/// Represents a parsed entry from a Claude transcript file.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TranscriptEntry {
    #[serde(default)]
    is_api_error_message: bool,
    #[serde(default)]
    message: Option<TranscriptMessage>,
}

#[derive(Debug, Deserialize)]
struct TranscriptMessage {
    #[serde(default)]
    content: Vec<ContentBlock>,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(default)]
    text: Option<String>,
}

impl ApiErrorInfo {
    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }
}
