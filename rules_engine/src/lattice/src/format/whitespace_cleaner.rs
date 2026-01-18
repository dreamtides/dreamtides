//! Whitespace cleaning utilities for Lattice markdown documents.
//!
//! Handles trailing whitespace removal, multiple blank line collapse,
//! and final newline normalization.

use tracing::debug;

/// Result of cleaning whitespace in a document.
#[derive(Debug)]
pub struct WhitespaceCleanResult {
    /// The cleaned content.
    pub content: String,
    /// Whether any changes were made.
    pub modified: bool,
}

/// Cleans whitespace in the given markdown content.
///
/// - Removes trailing spaces and tabs from all lines
/// - Preserves intentional trailing backslash (line continuation)
/// - Collapses 2+ consecutive blank lines to single blank line
/// - Ensures file ends with exactly one newline character
/// - Does not add newline to empty files
pub fn clean_whitespace(content: &str) -> WhitespaceCleanResult {
    debug!("Cleaning whitespace in markdown content");
    if content.is_empty() {
        return WhitespaceCleanResult { content: String::new(), modified: false };
    }
    let original_has_trailing_newline = content.ends_with('\n');
    let original_newline_count = count_trailing_newlines(content);
    let (stripped, trailing_modified) = remove_trailing_whitespace(content);
    let (collapsed, collapse_modified) = collapse_multiple_blank_lines(&stripped);
    let (normalized, newline_modified) =
        ensure_final_newline(&collapsed, original_has_trailing_newline, original_newline_count);
    let modified = trailing_modified || collapse_modified || newline_modified;
    if modified {
        debug!("Whitespace cleaned");
    }
    WhitespaceCleanResult { content: normalized, modified }
}

/// Counts trailing newlines in the content.
fn count_trailing_newlines(content: &str) -> usize {
    content.len() - content.trim_end_matches('\n').len()
}

/// Removes trailing whitespace from all lines, preserving line continuation
/// backslashes.
fn remove_trailing_whitespace(content: &str) -> (String, bool) {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut modified = false;
    for line in lines {
        let cleaned = strip_trailing_whitespace_preserving_backslash(line);
        if cleaned != line {
            debug!(
                original_len = line.len(),
                cleaned_len = cleaned.len(),
                "Removed trailing whitespace"
            );
            modified = true;
        }
        result.push(cleaned);
    }
    (result.join("\n"), modified)
}

/// Strips trailing whitespace from a line, preserving trailing backslash for
/// line continuation.
fn strip_trailing_whitespace_preserving_backslash(line: &str) -> String {
    if line.ends_with("\\ ") || line.ends_with("\\\t") {
        let base = line.trim_end();
        if base.ends_with('\\') {
            return format!("{} ", base);
        }
    }
    line.trim_end().to_string()
}

/// Collapses multiple consecutive blank lines into a single blank line.
fn collapse_multiple_blank_lines(content: &str) -> (String, bool) {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut modified = false;
    let mut prev_blank = false;
    for line in lines {
        let is_blank = line.trim().is_empty();
        if is_blank && prev_blank {
            debug!("Collapsed multiple blank lines");
            modified = true;
            continue;
        }
        result.push(line.to_string());
        prev_blank = is_blank;
    }
    (result.join("\n"), modified)
}

/// Ensures the content ends with exactly one newline character.
fn ensure_final_newline(
    content: &str,
    original_had_trailing_newline: bool,
    original_newline_count: usize,
) -> (String, bool) {
    if content.is_empty() {
        if original_had_trailing_newline {
            return ("\n".to_string(), original_newline_count != 1);
        }
        return (String::new(), false);
    }
    let trimmed = content.trim_end_matches('\n');
    let current_newlines = content.len() - trimmed.len();
    if current_newlines == 1 {
        if original_newline_count == 1 {
            return (content.to_string(), false);
        }
        return (content.to_string(), original_newline_count > 1);
    }
    if current_newlines == 0 && original_had_trailing_newline && original_newline_count == 1 {
        return (format!("{}\n", trimmed), false);
    }
    debug!(current_newlines, "Normalizing final newline");
    (format!("{}\n", trimmed), true)
}
