use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use tracing::{debug, info, warn};

use crate::document::{document_reader, document_writer, field_validation, frontmatter_parser};
use crate::error::error_types::LatticeError;
use crate::lint::rule_engine::LintResult;
use crate::lint::warning_rules;

/// Codes for all rules that can be automatically fixed.
pub const FIXABLE_CODES: &[&str] =
    &["W004", "W005", "W006", "W011", "W012", "W013", "W014", "W015", "E008"];

/// Summary of autofix operations on a single document.
#[derive(Debug, Default)]
pub struct FixResult {
    pub path: PathBuf,
    pub applied: Vec<String>,
    pub skipped: Vec<String>,
}

/// Summary of all autofix operations across documents.
#[derive(Debug, Default)]
pub struct AutofixSummary {
    pub documents_fixed: usize,
    pub total_fixes: usize,
    pub skipped_fixes: usize,
    pub results: Vec<FixResult>,
}

/// Returns true if the given lint code can be automatically fixed.
pub fn is_fixable(code: &str) -> bool {
    FIXABLE_CODES.contains(&code)
}

/// Applies automatic fixes for the given lint results.
///
/// Results are grouped by document path, and fixes are applied atomically
/// per document using temp file + rename.
pub fn apply_fixes(
    repo_root: &Path,
    results: &[LintResult],
) -> Result<AutofixSummary, LatticeError> {
    let fixable: Vec<_> = results.iter().filter(|r| is_fixable(&r.code)).collect();
    if fixable.is_empty() {
        debug!("No fixable issues found");
        return Ok(AutofixSummary::default());
    }
    let by_path = group_by_path(&fixable);
    info!(document_count = by_path.len(), "Applying fixes to documents");
    let mut summary = AutofixSummary::default();
    let mut sorted_paths: Vec<_> = by_path.keys().cloned().collect();
    sorted_paths.sort();
    for path in sorted_paths {
        let path_results = &by_path[&path];
        let full_path = repo_root.join(&path);
        match apply_fixes_to_document(&full_path, path_results) {
            Ok(result) => summary.add_result(result),
            Err(e) => {
                warn!(path = % path.display(), error = % e, "Failed to apply fixes");
                summary.results.push(FixResult {
                    path,
                    applied: vec![],
                    skipped: path_results.iter().map(|r| r.code.clone()).collect(),
                });
            }
        }
    }
    info!(
        documents = summary.documents_fixed,
        fixes = summary.total_fixes,
        skipped = summary.skipped_fixes,
        "Autofix complete"
    );
    Ok(summary)
}

/// Normalizes a name to lowercase-hyphen format.
pub fn normalize_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Removes trailing whitespace from all lines.
pub fn fix_trailing_whitespace(body: &str) -> String {
    body.lines().map(str::trim_end).collect::<Vec<_>>().join("\n")
        + if body.ends_with('\n') { "\n" } else { "" }
}

/// Collapses multiple consecutive blank lines to a single blank line.
pub fn fix_multiple_blank_lines(body: &str) -> String {
    let mut result = Vec::new();
    let mut prev_blank = false;
    let mut in_code_fence = false;
    let mut current_fence = "";
    for line in body.lines() {
        if !in_code_fence && is_fenced_code_start(line) {
            in_code_fence = true;
            current_fence = extract_fence(line);
            result.push(line);
            prev_blank = false;
            continue;
        }
        if in_code_fence {
            result.push(line);
            if is_matching_fence_end(line, current_fence) {
                in_code_fence = false;
                current_fence = "";
            }
            prev_blank = false;
            continue;
        }
        let is_blank = line.trim().is_empty();
        if is_blank && prev_blank {
            continue;
        }
        result.push(line);
        prev_blank = is_blank;
    }
    result.join("\n") + if body.ends_with('\n') { "\n" } else { "" }
}

/// Ensures the body ends with a newline.
pub fn fix_missing_final_newline(body: &str) -> String {
    if body.is_empty() || body.ends_with('\n') { body.to_string() } else { format!("{body}\n") }
}

/// Converts setext-style headers to ATX (#) style.
pub fn fix_setext_headers(body: &str) -> String {
    let lines: Vec<&str> = body.lines().collect();
    let mut result = Vec::new();
    let mut skip_next = false;
    let mut in_code_fence = false;
    let mut current_fence = "";
    for (i, line) in lines.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }
        if !in_code_fence && is_fenced_code_start(line) {
            in_code_fence = true;
            current_fence = extract_fence(line);
            result.push((*line).to_string());
            continue;
        }
        if in_code_fence {
            result.push((*line).to_string());
            if is_matching_fence_end(line, current_fence) {
                in_code_fence = false;
                current_fence = "";
            }
            continue;
        }
        if i + 1 < lines.len() && warning_rules::is_setext_underline(lines[i + 1]) {
            let header_text = line.trim();
            if !header_text.is_empty() {
                let underline = lines[i + 1].trim();
                let level = if underline.starts_with('=') { "#" } else { "##" };
                result.push(format!("{level} {header_text}"));
                skip_next = true;
                continue;
            }
        }
        result.push((*line).to_string());
    }
    result.join("\n") + if body.ends_with('\n') { "\n" } else { "" }
}

/// Converts all list markers to dashes.
pub fn fix_list_markers(body: &str) -> String {
    let mut result = Vec::new();
    let mut in_code_fence = false;
    let mut current_fence = "";
    for line in body.lines() {
        if !in_code_fence && is_fenced_code_start(line) {
            in_code_fence = true;
            current_fence = extract_fence(line);
            result.push(line.to_string());
            continue;
        }
        if in_code_fence {
            result.push(line.to_string());
            if is_matching_fence_end(line, current_fence) {
                in_code_fence = false;
                current_fence = "";
            }
            continue;
        }
        let trimmed = line.trim_start();
        let indent = &line[..line.len() - trimmed.len()];
        if let Some(rest) = trimmed.strip_prefix("* ") {
            result.push(format!("{indent}- {rest}"));
        } else if let Some(rest) = trimmed.strip_prefix("+ ") {
            result.push(format!("{indent}- {rest}"));
        } else {
            result.push(line.to_string());
        }
    }
    result.join("\n") + if body.ends_with('\n') { "\n" } else { "" }
}

/// Inserts blank lines before and after headings as needed.
pub fn fix_heading_blank_lines(body: &str) -> String {
    let lines: Vec<&str> = body.lines().collect();
    let mut result = Vec::new();
    let mut in_code_fence = false;
    let mut current_fence = "";
    for (i, line) in lines.iter().enumerate() {
        if !in_code_fence && is_fenced_code_start(line) {
            in_code_fence = true;
            current_fence = extract_fence(line);
            result.push((*line).to_string());
            continue;
        }
        if in_code_fence {
            result.push((*line).to_string());
            if is_matching_fence_end(line, current_fence) {
                in_code_fence = false;
                current_fence = "";
            }
            continue;
        }
        let is_heading = warning_rules::is_atx_heading(line);
        let prev_blank = i == 0 || lines[i - 1].trim().is_empty();
        let next_blank = i == lines.len() - 1 || lines[i + 1].trim().is_empty();
        if is_heading && !prev_blank && !result.is_empty() {
            result.push(String::new());
        }
        result.push((*line).to_string());
        if is_heading && !next_blank && i < lines.len() - 1 {
            result.push(String::new());
        }
    }
    result.join("\n") + if body.ends_with('\n') { "\n" } else { "" }
}

/// Inserts blank lines before and after lists as needed.
pub fn fix_list_blank_lines(body: &str) -> String {
    let lines: Vec<&str> = body.lines().collect();
    let mut result = Vec::new();
    let mut in_list = false;
    let mut in_code_fence = false;
    let mut current_fence = "";
    for (i, line) in lines.iter().enumerate() {
        if !in_code_fence && is_fenced_code_start(line) {
            in_code_fence = true;
            current_fence = extract_fence(line);
            result.push((*line).to_string());
            continue;
        }
        if in_code_fence {
            result.push((*line).to_string());
            if is_matching_fence_end(line, current_fence) {
                in_code_fence = false;
                current_fence = "";
            }
            continue;
        }
        let is_list_item = warning_rules::is_list_line(line);
        let is_blank = line.trim().is_empty();
        let is_continuation = warning_rules::is_list_continuation(line);
        if is_list_item && !in_list {
            in_list = true;
            let prev_blank = i == 0 || lines[i - 1].trim().is_empty();
            if !prev_blank && !result.is_empty() {
                result.push(String::new());
            }
        } else if !is_list_item && !is_continuation && in_list && !is_blank {
            in_list = false;
            if !result.is_empty() && !result.last().is_none_or(|l| l.trim().is_empty()) {
                result.push(String::new());
            }
        } else if is_blank {
            in_list = false;
        }
        result.push((*line).to_string());
    }
    result.join("\n") + if body.ends_with('\n') { "\n" } else { "" }
}

fn is_fenced_code_start(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("```") || trimmed.starts_with("~~~")
}

fn extract_fence(line: &str) -> &'static str {
    let trimmed = line.trim_start();
    if trimmed.starts_with("```") {
        "```"
    } else if trimmed.starts_with("~~~") {
        "~~~"
    } else {
        ""
    }
}

fn is_matching_fence_end(line: &str, fence: &str) -> bool {
    let trimmed = line.trim();
    trimmed == fence || (trimmed.starts_with(fence) && trimmed[fence.len()..].trim().is_empty())
}

impl AutofixSummary {
    fn add_result(&mut self, result: FixResult) {
        if !result.applied.is_empty() {
            self.documents_fixed += 1;
            self.total_fixes += result.applied.len();
        }
        self.skipped_fixes += result.skipped.len();
        self.results.push(result);
    }
}

fn group_by_path<'a>(results: &[&'a LintResult]) -> HashMap<PathBuf, Vec<&'a LintResult>> {
    let mut by_path: HashMap<PathBuf, Vec<&'a LintResult>> = HashMap::new();
    for result in results {
        by_path.entry(result.path.clone()).or_default().push(result);
    }
    by_path
}

fn apply_fixes_to_document(
    path: &Path,
    results: &[&LintResult],
) -> Result<FixResult, LatticeError> {
    let codes: HashSet<_> = results.iter().map(|r| r.code.as_str()).collect();
    debug!(path = % path.display(), codes = ? codes, "Applying fixes to document");
    let document = document_reader::read(path)?;
    let mut frontmatter = document.frontmatter.clone();
    let mut body = document.body.clone();
    let mut applied = Vec::new();
    let mut skipped = Vec::new();
    let has_e008 = codes.contains("E008");
    if has_e008 {
        if let Some(expected) = field_validation::derive_name_from_path(path) {
            debug!(
                path = % path.display(), old = % frontmatter.name, new = % expected,
                "Fixing E008"
            );
            frontmatter.name = expected;
            applied.push("E008".to_string());
        } else {
            warn!(path = % path.display(), "Cannot derive name from path for E008 fix");
            skipped.push("E008".to_string());
        }
    }
    let has_w004 = codes.contains("W004");
    if has_w004 && !has_e008 {
        let normalized = normalize_name(&frontmatter.name);
        if normalized != frontmatter.name {
            debug!(
                path = % path.display(), old = % frontmatter.name, new = % normalized,
                "Fixing W004"
            );
            frontmatter.name = normalized;
            applied.push("W004".to_string());
        }
    } else if has_w004 && has_e008 {
        debug!(path = % path.display(), "Skipping W004, already fixed by E008");
        skipped.push("W004".to_string());
    }
    if codes.contains("W011") {
        body = fix_trailing_whitespace(&body);
        applied.push("W011".to_string());
    }
    if codes.contains("W012") {
        body = fix_multiple_blank_lines(&body);
        applied.push("W012".to_string());
    }
    if codes.contains("W013") {
        body = fix_missing_final_newline(&body);
        applied.push("W013".to_string());
    }
    if codes.contains("W005") {
        body = fix_setext_headers(&body);
        applied.push("W005".to_string());
    }
    if codes.contains("W006") {
        body = fix_list_markers(&body);
        applied.push("W006".to_string());
    }
    if codes.contains("W014") {
        body = fix_heading_blank_lines(&body);
        applied.push("W014".to_string());
    }
    if codes.contains("W015") {
        body = fix_list_blank_lines(&body);
        applied.push("W015".to_string());
    }
    if applied.is_empty() {
        debug!(path = % path.display(), "No fixes applied");
        return Ok(FixResult { path: path.to_path_buf(), applied, skipped });
    }
    let content = frontmatter_parser::format_document(&frontmatter, &body)?;
    document_writer::write_raw(path, &content, &document_writer::WriteOptions::default())?;
    info!(path = % path.display(), fixes = ? applied, "Applied fixes");
    Ok(FixResult { path: path.to_path_buf(), applied, skipped })
}
