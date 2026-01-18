//! Header normalization utilities for Lattice markdown documents.
//!
//! Handles conversion of setext-style headers to ATX style and ensures proper
//! blank line spacing around headings.

use tracing::debug;

/// Result of normalizing headers in a document.
#[derive(Debug)]
pub struct HeaderNormalizeResult {
    /// The normalized content.
    pub content: String,
    /// Whether any changes were made.
    pub modified: bool,
}

/// Normalizes headers in the given markdown content.
///
/// - Converts setext-style headers (underlines) to ATX style (#)
/// - Normalizes header spacing (single space after #)
/// - Ensures blank line before each heading
/// - Ensures blank line after each heading
pub fn normalize_headers(content: &str) -> HeaderNormalizeResult {
    debug!("Normalizing headers in markdown content");
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return HeaderNormalizeResult { content: String::new(), modified: false };
    }
    let (converted_lines, setext_modified) = convert_setext_to_atx(&lines);
    let (spaced_lines, spacing_modified) = ensure_header_spacing(&converted_lines);
    let (normalized_lines, style_modified) = normalize_atx_style(&spaced_lines);
    let mut result = normalized_lines.join("\n");
    if !result.is_empty() && !content.is_empty() {
        result.push('\n');
    }
    let modified = setext_modified || spacing_modified || style_modified;
    if modified {
        debug!("Headers normalized");
    }
    HeaderNormalizeResult { content: result, modified }
}

/// Converts setext-style headers to ATX style.
fn convert_setext_to_atx(lines: &[&str]) -> (Vec<String>, bool) {
    let mut result = Vec::new();
    let mut modified = false;
    let mut i = 0;
    while i < lines.len() {
        if i + 1 < lines.len() && is_setext_underline(lines[i + 1]) {
            let header_text = lines[i].trim();
            let level = if lines[i + 1].starts_with('=') { 1 } else { 2 };
            let prefix = "#".repeat(level);
            result.push(format!("{} {}", prefix, header_text));
            debug!(original = lines[i], level, "Converted setext header to ATX");
            modified = true;
            i += 2;
            continue;
        }
        result.push(lines[i].to_string());
        i += 1;
    }
    (result, modified)
}

/// Returns true if the line is a setext-style underline.
fn is_setext_underline(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }
    let is_equals = trimmed.chars().all(|c| c == '=') && trimmed.len() >= 3;
    let is_dashes = trimmed.chars().all(|c| c == '-') && trimmed.len() >= 3;
    is_equals || is_dashes
}

/// Ensures blank lines before and after headers.
fn ensure_header_spacing(lines: &[String]) -> (Vec<String>, bool) {
    let mut result = Vec::new();
    let mut modified = false;
    for (i, line) in lines.iter().enumerate() {
        let is_header = is_atx_header(line);
        let prev_blank = i == 0 || lines[i - 1].trim().is_empty();
        let next_blank = i + 1 >= lines.len() || lines[i + 1].trim().is_empty();
        if is_header && i > 0 && !prev_blank {
            result.push(String::new());
            modified = true;
            debug!(line_num = i + 1, "Added blank line before header");
        }
        result.push(line.clone());
        if is_header && i + 1 < lines.len() && !next_blank {
            result.push(String::new());
            modified = true;
            debug!(line_num = i + 1, "Added blank line after header");
        }
    }
    (result, modified)
}

/// Returns true if the line is an ATX-style header (or looks like one that
/// needs normalization).
fn is_atx_header(line: &str) -> bool {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('#') {
        return false;
    }
    let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
    if hash_count == 0 || hash_count > 6 {
        return false;
    }
    let rest = &trimmed[hash_count..];
    rest.is_empty() || rest.starts_with(' ') || rest.chars().next().is_some_and(char::is_alphabetic)
}

/// Normalizes ATX header style to have exactly one space after #.
fn normalize_atx_style(lines: &[String]) -> (Vec<String>, bool) {
    let mut result = Vec::new();
    let mut modified = false;
    for line in lines {
        if is_atx_header(line) {
            let trimmed = line.trim_start();
            let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
            let rest = trimmed[hash_count..].trim_start();
            let normalized = if rest.is_empty() {
                "#".repeat(hash_count)
            } else {
                format!("{} {}", "#".repeat(hash_count), rest)
            };
            if normalized != *line {
                debug!(original = %line, normalized = %normalized, "Normalized ATX header spacing");
                modified = true;
            }
            result.push(normalized);
        } else {
            result.push(line.clone());
        }
    }
    (result, modified)
}
