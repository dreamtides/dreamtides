//! List normalization utilities for Lattice markdown documents.
//!
//! Handles conversion of list markers to dashes and ensures proper blank line
//! spacing around lists.

use tracing::debug;

/// Result of normalizing lists in a document.
#[derive(Debug)]
pub struct ListNormalizeResult {
    /// The normalized content.
    pub content: String,
    /// Whether any changes were made.
    pub modified: bool,
}

/// Normalizes lists in the given markdown content.
///
/// - Converts all unordered list markers (*, +) to dashes (-)
/// - Preserves indentation for nested lists
/// - Ensures blank line before list start
/// - Ensures blank line after list end
pub fn normalize_lists(content: &str) -> ListNormalizeResult {
    debug!("Normalizing lists in markdown content");
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return ListNormalizeResult { content: String::new(), modified: false };
    }
    let (converted_lines, marker_modified) = convert_list_markers(&lines);
    let (spaced_lines, spacing_modified) = ensure_list_spacing(&converted_lines);
    let mut result = spaced_lines.join("\n");
    if !result.is_empty() && !content.is_empty() {
        result.push('\n');
    }
    let modified = marker_modified || spacing_modified;
    if modified {
        debug!("Lists normalized");
    }
    ListNormalizeResult { content: result, modified }
}

/// Converts all unordered list markers to dashes.
fn convert_list_markers(lines: &[&str]) -> (Vec<String>, bool) {
    let mut result = Vec::new();
    let mut modified = false;
    for line in lines {
        if let Some(converted) = convert_unordered_marker(line) {
            if converted != *line {
                debug!(original = %line, converted = %converted, "Converted list marker to dash");
                modified = true;
            }
            result.push(converted);
        } else {
            result.push((*line).to_string());
        }
    }
    (result, modified)
}

/// Converts a single unordered list line's marker to a dash.
fn convert_unordered_marker(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
        let indent_len = line.len() - trimmed.len();
        let indent: String = line.chars().take(indent_len).collect();
        let rest = &trimmed[2..];
        return Some(format!("{}- {}", indent, rest));
    }
    if trimmed == "*" || trimmed == "+" {
        let indent_len = line.len() - trimmed.len();
        let indent: String = line.chars().take(indent_len).collect();
        return Some(format!("{}-", indent));
    }
    None
}

/// Ensures blank lines before and after lists.
fn ensure_list_spacing(lines: &[String]) -> (Vec<String>, bool) {
    let mut result = Vec::new();
    let mut modified = false;
    let list_status = compute_list_status(lines);
    for (i, line) in lines.iter().enumerate() {
        let is_list_start = list_status[i] && (i == 0 || !list_status[i - 1]);
        let is_list_end = list_status[i] && (i + 1 >= lines.len() || !list_status[i + 1]);
        if is_list_start && i > 0 && !lines[i - 1].trim().is_empty() {
            result.push(String::new());
            modified = true;
            debug!(line_num = i + 1, "Added blank line before list");
        }
        result.push(line.clone());
        if is_list_end && i + 1 < lines.len() && !lines[i + 1].trim().is_empty() {
            result.push(String::new());
            modified = true;
            debug!(line_num = i + 1, "Added blank line after list");
        }
    }
    (result, modified)
}

/// Computes which lines are part of a list.
fn compute_list_status(lines: &[String]) -> Vec<bool> {
    let mut status = Vec::with_capacity(lines.len());
    for line in lines {
        status.push(is_list_line(line));
    }
    status
}

/// Returns true if the line is part of a list (list item or continuation).
fn is_list_line(line: &str) -> bool {
    is_unordered_list_item(line) || is_ordered_list_item(line) || is_list_continuation(line)
}

/// Returns true if the line is an unordered list item.
fn is_unordered_list_item(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("- ")
        || trimmed.starts_with("* ")
        || trimmed.starts_with("+ ")
        || trimmed == "-"
        || trimmed == "*"
        || trimmed == "+"
}

/// Returns true if the line is an ordered list item.
fn is_ordered_list_item(line: &str) -> bool {
    let trimmed = line.trim_start();
    let mut chars = trimmed.chars().peekable();
    if !chars.next().is_some_and(|c| c.is_ascii_digit()) {
        return false;
    }
    while chars.peek().is_some_and(char::is_ascii_digit) {
        chars.next();
    }
    matches!(chars.next(), Some('.' | ')')) && matches!(chars.next(), Some(' ') | None)
}

/// Returns true if the line appears to be a continuation of a list item.
fn is_list_continuation(line: &str) -> bool {
    if line.trim().is_empty() {
        return false;
    }
    let indent = line.len() - line.trim_start().len();
    indent >= 2
}
