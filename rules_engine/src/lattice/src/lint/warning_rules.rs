use std::path::Path;

use regex::Regex;
use tracing::debug;

use crate::link::link_extractor;
use crate::lint::rule_engine::{LintContext, LintDocument, LintResult, LintRule};

/// Maximum recommended document line count.
const MAX_DOCUMENT_LINES: usize = 500;

/// Maximum name length in characters.
const MAX_NAME_LENGTH: usize = 64;

/// Maximum description length in characters.
const MAX_DESCRIPTION_LENGTH: usize = 1024;

/// W001: Document too large.
///
/// Document exceeds the recommended 500 line limit.
pub struct DocumentTooLargeRule;

/// W002: Name too long.
///
/// Document name exceeds 64 characters.
pub struct NameTooLongRule;

/// W003: Description too long.
///
/// Description exceeds 1024 characters.
pub struct DescriptionTooLongRule;

/// W004: Invalid name characters.
///
/// Name contains characters other than lowercase letters, numbers, hyphens.
pub struct InvalidNameCharactersRule;

/// W005: Inconsistent header style.
///
/// Document mixes ATX (`#`) and setext (underline) headers.
pub struct InconsistentHeaderStyleRule;

/// W006: Inconsistent list markers.
///
/// Document mixes list markers (`-`, `*`, `+`).
pub struct InconsistentListMarkersRule;

/// W007: Bare URL.
///
/// URL appears without markdown link syntax.
pub struct BareUrlRule;

/// W008: Self-reference.
///
/// Document contains a link to itself.
pub struct SelfReferenceRule;

/// W009: Backslash in path.
///
/// Path contains backslashes (Windows-style).
pub struct BackslashInPathRule;

/// W010: Link path mismatch.
///
/// Link file path doesn't match the target document's actual location.
pub struct LinkPathMismatchRule;

/// W010b: Missing link fragment.
///
/// Link has file path but no Lattice ID fragment.
pub struct MissingLinkFragmentRule;

/// W011: Trailing whitespace.
///
/// Lines end with unnecessary whitespace.
pub struct TrailingWhitespaceRule;

/// W012: Multiple blank lines.
///
/// More than one consecutive blank line in document.
pub struct MultipleBlankLinesRule;

/// W013: Missing final newline.
///
/// File does not end with a newline character.
pub struct MissingFinalNewlineRule;

/// W014: Heading without blank lines.
///
/// Heading not surrounded by blank lines.
pub struct HeadingWithoutBlankLinesRule;

/// W015: List without blank lines.
///
/// List not surrounded by blank lines.
pub struct ListWithoutBlankLinesRule;

/// W016: Template section in non-root.
///
/// Non-root document contains template sections (markdown sections with headers
/// like "## Dependencies" that are meant only for root documents).
pub struct TemplateSectionInNonRootRule;

/// Returns all warning-level lint rules.
pub fn all_warning_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(DocumentTooLargeRule),
        Box::new(NameTooLongRule),
        Box::new(DescriptionTooLongRule),
        Box::new(InvalidNameCharactersRule),
        Box::new(InconsistentHeaderStyleRule),
        Box::new(InconsistentListMarkersRule),
        Box::new(BareUrlRule),
        Box::new(SelfReferenceRule),
        Box::new(BackslashInPathRule),
        Box::new(LinkPathMismatchRule),
        Box::new(MissingLinkFragmentRule),
        Box::new(TrailingWhitespaceRule),
        Box::new(MultipleBlankLinesRule),
        Box::new(MissingFinalNewlineRule),
        Box::new(HeadingWithoutBlankLinesRule),
        Box::new(ListWithoutBlankLinesRule),
        Box::new(TemplateSectionInNonRootRule),
    ]
}

impl LintRule for DocumentTooLargeRule {
    fn codes(&self) -> &[&str] {
        &["W001"]
    }

    fn name(&self) -> &str {
        "document-too-large"
    }

    fn requires_document_body(&self) -> bool {
        true
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        let Some(document) = &doc.document else {
            return vec![];
        };

        let line_count = document.body.lines().count();
        if line_count >= MAX_DOCUMENT_LINES {
            let message = format!("{line_count} lines (recommended max: {MAX_DOCUMENT_LINES})");
            debug!(path = %doc.row.path, lines = line_count, "Document exceeds line limit");
            return vec![LintResult::warning("W001", &doc.row.path, message)];
        }

        vec![]
    }
}

impl LintRule for NameTooLongRule {
    fn codes(&self) -> &[&str] {
        &["W002"]
    }

    fn name(&self) -> &str {
        "name-too-long"
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        let name_len = doc.row.name.chars().count();
        if name_len > MAX_NAME_LENGTH {
            let message = format!("name is {name_len} characters (max: {MAX_NAME_LENGTH})");
            debug!(path = %doc.row.path, len = name_len, "Name exceeds max length");
            return vec![LintResult::warning("W002", &doc.row.path, message)];
        }

        vec![]
    }
}

impl LintRule for DescriptionTooLongRule {
    fn codes(&self) -> &[&str] {
        &["W003"]
    }

    fn name(&self) -> &str {
        "description-too-long"
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        let desc_len = doc.row.description.chars().count();
        if desc_len > MAX_DESCRIPTION_LENGTH {
            let message =
                format!("description is {desc_len} characters (max: {MAX_DESCRIPTION_LENGTH})");
            debug!(path = %doc.row.path, len = desc_len, "Description exceeds max length");
            return vec![LintResult::warning("W003", &doc.row.path, message)];
        }

        vec![]
    }
}

impl LintRule for InvalidNameCharactersRule {
    fn codes(&self) -> &[&str] {
        &["W004"]
    }

    fn name(&self) -> &str {
        "invalid-name-characters"
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        let name = &doc.row.name;
        if name.is_empty() {
            return vec![];
        }

        let valid_name_regex = Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$")
            .unwrap_or_else(|e| panic!("Invalid regex for name validation: {e}"));

        if !valid_name_regex.is_match(name) {
            let message = format!(
                "name '{}' contains invalid characters (use lowercase-hyphen-format)",
                name
            );
            debug!(path = %doc.row.path, name = %name, "Invalid name characters");
            return vec![LintResult::warning("W004", &doc.row.path, message)];
        }

        vec![]
    }
}

impl LintRule for InconsistentHeaderStyleRule {
    fn codes(&self) -> &[&str] {
        &["W005"]
    }

    fn name(&self) -> &str {
        "inconsistent-header-style"
    }

    fn requires_document_body(&self) -> bool {
        true
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        let Some(document) = &doc.document else {
            return vec![];
        };

        let lines: Vec<&str> = document.body.lines().collect();
        let mut has_atx = false;
        let mut has_setext = false;

        for (i, line) in lines.iter().enumerate() {
            if line.starts_with('#') && !line.starts_with("#!") {
                has_atx = true;
            }

            if i > 0 && is_setext_underline(line) && !lines[i - 1].is_empty() {
                has_setext = true;
            }
        }

        if has_atx && has_setext {
            let message = "mixes header styles (use ATX # headers consistently)";
            debug!(path = %doc.row.path, "Mixed header styles detected");
            return vec![LintResult::warning("W005", &doc.row.path, message)];
        }

        vec![]
    }
}

impl LintRule for InconsistentListMarkersRule {
    fn codes(&self) -> &[&str] {
        &["W006"]
    }

    fn name(&self) -> &str {
        "inconsistent-list-markers"
    }

    fn requires_document_body(&self) -> bool {
        true
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        let Some(document) = &doc.document else {
            return vec![];
        };

        let mut markers_found = Vec::new();

        for line in document.body.lines() {
            let trimmed = line.trim_start();
            if let Some(marker) = extract_list_marker(trimmed)
                && !markers_found.contains(&marker)
            {
                markers_found.push(marker);
            }
        }

        if markers_found.len() > 1 {
            let message = "mixes list markers (use - consistently)";
            debug!(path = %doc.row.path, markers = ?markers_found, "Mixed list markers");
            return vec![LintResult::warning("W006", &doc.row.path, message)];
        }

        vec![]
    }
}

impl LintRule for BareUrlRule {
    fn codes(&self) -> &[&str] {
        &["W007"]
    }

    fn name(&self) -> &str {
        "bare-url"
    }

    fn requires_document_body(&self) -> bool {
        true
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        let Some(document) = &doc.document else {
            return vec![];
        };

        let mut results = Vec::new();
        let url_regex = Regex::new(r"https?://[^\s\)>\]]+")
            .unwrap_or_else(|e| panic!("Invalid URL regex: {e}"));

        let mut in_code_block = false;
        for (line_num, line) in document.body.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }

            for m in url_regex.find_iter(line) {
                let url = m.as_str();
                if !is_url_in_markdown_link(line, url) {
                    let message = "has bare URL (use [text](url) format)";
                    debug!(path = %doc.row.path, line = line_num + 1, url = %url, "Bare URL found");
                    results.push(
                        LintResult::warning("W007", &doc.row.path, message).with_line(line_num + 1),
                    );
                }
            }
        }

        results
    }
}

impl LintRule for SelfReferenceRule {
    fn codes(&self) -> &[&str] {
        &["W008"]
    }

    fn name(&self) -> &str {
        "self-reference"
    }

    fn requires_document_body(&self) -> bool {
        true
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        let Some(document) = &doc.document else {
            return vec![];
        };

        let extracted = link_extractor::extract(&document.body);
        let mut results = Vec::new();

        for link in extracted.links {
            if let Some(fragment) = &link.fragment
                && fragment.as_str() == doc.row.id
            {
                let message = format!("contains self-reference at line {}", link.line);
                debug!(path = %doc.row.path, line = link.line, "Self-reference found");
                results
                    .push(LintResult::warning("W008", &doc.row.path, message).with_line(link.line));
            }
        }

        results
    }
}

impl LintRule for BackslashInPathRule {
    fn codes(&self) -> &[&str] {
        &["W009"]
    }

    fn name(&self) -> &str {
        "backslash-in-path"
    }

    fn requires_document_body(&self) -> bool {
        true
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        let Some(document) = &doc.document else {
            return vec![];
        };

        let extracted = link_extractor::extract(&document.body);
        let mut results = Vec::new();

        for link in extracted.links {
            if let Some(path) = &link.path
                && path.contains('\\')
            {
                let message = "uses backslash in path (use forward slashes)";
                debug!(path = %doc.row.path, line = link.line, link_path = %path, "Backslash in path");
                results
                    .push(LintResult::warning("W009", &doc.row.path, message).with_line(link.line));
            }
        }

        results
    }
}

impl LintRule for LinkPathMismatchRule {
    fn codes(&self) -> &[&str] {
        &["W010"]
    }

    fn name(&self) -> &str {
        "link-path-mismatch"
    }

    fn requires_document_body(&self) -> bool {
        true
    }

    fn check(&self, doc: &LintDocument, ctx: &LintContext<'_>) -> Vec<LintResult> {
        let Some(document) = &doc.document else {
            return vec![];
        };

        let extracted = link_extractor::extract(&document.body);
        let mut results = Vec::new();

        let source_dir = Path::new(&doc.row.path).parent();

        for link in extracted.links {
            let (Some(link_path), Some(fragment)) = (&link.path, &link.fragment) else {
                continue;
            };

            let target_doc = match ctx.lookup_document(fragment.as_str()) {
                Ok(Some(doc)) => doc,
                Ok(None) => continue,
                Err(e) => {
                    debug!(error = %e, target_id = %fragment, "Failed to lookup target document");
                    continue;
                }
            };

            let Some(source_dir) = source_dir else {
                continue;
            };

            let resolved_path = source_dir.join(link_path);
            let normalized = normalize_path(&resolved_path);
            let target_path = Path::new(&target_doc.path);

            if normalized != target_path {
                let expected_rel = compute_relative_path(source_dir, target_path);
                let message = format!(
                    "has stale link path (expected {}#{})",
                    expected_rel.display(),
                    fragment
                );
                debug!(
                    path = %doc.row.path,
                    line = link.line,
                    found = %link_path,
                    expected = %expected_rel.display(),
                    "Link path mismatch"
                );
                results
                    .push(LintResult::warning("W010", &doc.row.path, message).with_line(link.line));
            }
        }

        results
    }
}

impl LintRule for MissingLinkFragmentRule {
    fn codes(&self) -> &[&str] {
        &["W010b"]
    }

    fn name(&self) -> &str {
        "missing-link-fragment"
    }

    fn requires_document_body(&self) -> bool {
        true
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        let Some(document) = &doc.document else {
            return vec![];
        };

        let extracted = link_extractor::extract(&document.body);
        let mut results = Vec::new();

        for link in extracted.links {
            if let Some(path) = &link.path
                && link.fragment.is_none()
                && path.ends_with(".md")
            {
                let message =
                    format!("link missing Lattice ID fragment: [{}]({})", link.text, path);
                debug!(path = %doc.row.path, line = link.line, link_path = %path, "Missing fragment");
                results.push(
                    LintResult::warning("W010b", &doc.row.path, message).with_line(link.line),
                );
            }
        }

        results
    }
}

impl LintRule for TrailingWhitespaceRule {
    fn codes(&self) -> &[&str] {
        &["W011"]
    }

    fn name(&self) -> &str {
        "trailing-whitespace"
    }

    fn requires_document_body(&self) -> bool {
        true
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        let Some(document) = &doc.document else {
            return vec![];
        };

        let mut results = Vec::new();

        for (line_num, line) in document.body.lines().enumerate() {
            if line != line.trim_end() {
                let message = "has trailing whitespace";
                results.push(
                    LintResult::warning("W011", &doc.row.path, message).with_line(line_num + 1),
                );
            }
        }

        if results.len() > 3 {
            debug!(path = %doc.row.path, count = results.len(), "Multiple trailing whitespace lines");
            let message = format!("{} lines with trailing whitespace", results.len());
            return vec![LintResult::warning("W011", &doc.row.path, message)];
        }

        results
    }
}

impl LintRule for MultipleBlankLinesRule {
    fn codes(&self) -> &[&str] {
        &["W012"]
    }

    fn name(&self) -> &str {
        "multiple-blank-lines"
    }

    fn requires_document_body(&self) -> bool {
        true
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        let Some(document) = &doc.document else {
            return vec![];
        };

        let mut results = Vec::new();
        let mut consecutive_blank = 0;
        let mut blank_start_line = 0;

        for (line_num, line) in document.body.lines().enumerate() {
            if line.trim().is_empty() {
                if consecutive_blank == 0 {
                    blank_start_line = line_num + 1;
                }
                consecutive_blank += 1;
            } else {
                if consecutive_blank > 1 {
                    let message =
                        format!("has {} consecutive blank lines (max: 1)", consecutive_blank);
                    debug!(
                        path = %doc.row.path,
                        line = blank_start_line,
                        count = consecutive_blank,
                        "Multiple blank lines"
                    );
                    results.push(
                        LintResult::warning("W012", &doc.row.path, message)
                            .with_line(blank_start_line),
                    );
                }
                consecutive_blank = 0;
            }
        }

        if consecutive_blank > 1 {
            let message = format!("has {} consecutive blank lines (max: 1)", consecutive_blank);
            results.push(
                LintResult::warning("W012", &doc.row.path, message).with_line(blank_start_line),
            );
        }

        results
    }
}

impl LintRule for MissingFinalNewlineRule {
    fn codes(&self) -> &[&str] {
        &["W013"]
    }

    fn name(&self) -> &str {
        "missing-final-newline"
    }

    fn requires_document_body(&self) -> bool {
        true
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        let Some(document) = &doc.document else {
            return vec![];
        };

        if !document.body.is_empty() && !document.body.ends_with('\n') {
            let message = "does not end with newline";
            debug!(path = %doc.row.path, "Missing final newline");
            return vec![LintResult::warning("W013", &doc.row.path, message)];
        }

        vec![]
    }
}

impl LintRule for HeadingWithoutBlankLinesRule {
    fn codes(&self) -> &[&str] {
        &["W014"]
    }

    fn name(&self) -> &str {
        "heading-without-blank-lines"
    }

    fn requires_document_body(&self) -> bool {
        true
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        let Some(document) = &doc.document else {
            return vec![];
        };

        let lines: Vec<&str> = document.body.lines().collect();
        let mut results = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            if !is_atx_heading(line) {
                continue;
            }

            let needs_blank_before = i > 0 && !lines[i - 1].trim().is_empty();
            let needs_blank_after = i < lines.len() - 1 && !lines[i + 1].trim().is_empty();

            if needs_blank_before || needs_blank_after {
                let message = "heading should have blank line before/after";
                results.push(LintResult::warning("W014", &doc.row.path, message).with_line(i + 1));
            }
        }

        results
    }
}

impl LintRule for ListWithoutBlankLinesRule {
    fn codes(&self) -> &[&str] {
        &["W015"]
    }

    fn name(&self) -> &str {
        "list-without-blank-lines"
    }

    fn requires_document_body(&self) -> bool {
        true
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        let Some(document) = &doc.document else {
            return vec![];
        };

        let lines: Vec<&str> = document.body.lines().collect();
        let mut results = Vec::new();
        let mut in_list = false;
        let mut list_start = 0;

        for (i, line) in lines.iter().enumerate() {
            let is_list_item = is_list_line(line);

            if is_list_item && !in_list {
                in_list = true;
                list_start = i;

                let prev_is_blank = i == 0 || lines[i - 1].trim().is_empty();
                if !prev_is_blank {
                    let message = "list should have blank line before";
                    results
                        .push(LintResult::warning("W015", &doc.row.path, message).with_line(i + 1));
                }
            } else if !is_list_item && in_list && !line.trim().is_empty() {
                if !is_list_continuation(line) {
                    in_list = false;
                    let message = "list should have blank line after";
                    results.push(
                        LintResult::warning("W015", &doc.row.path, message)
                            .with_line(list_start + 1),
                    );
                }
            } else if line.trim().is_empty() {
                in_list = false;
            }
        }

        results
    }
}

impl LintRule for TemplateSectionInNonRootRule {
    fn codes(&self) -> &[&str] {
        &["W016"]
    }

    fn name(&self) -> &str {
        "template-section-in-non-root"
    }

    fn requires_document_body(&self) -> bool {
        true
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        if doc.row.is_root {
            return vec![];
        }

        let Some(document) = &doc.document else {
            return vec![];
        };

        let lattice_heading_regex = Regex::new(r"^#{1,6}\s+\[Lattice\]")
            .unwrap_or_else(|e| panic!("Invalid Lattice heading regex: {e}"));

        for (line_num, line) in document.body.lines().enumerate() {
            if lattice_heading_regex.is_match(line) {
                let message = "has [Lattice] sections but is not a root document";
                debug!(path = %doc.row.path, line = line_num + 1, "Template section in non-root");
                return vec![
                    LintResult::warning("W016", &doc.row.path, message).with_line(line_num + 1),
                ];
            }
        }

        vec![]
    }
}

fn is_setext_underline(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }
    trimmed.chars().all(|c| c == '=' || c == '-') && trimmed.len() >= 3
}

fn extract_list_marker(line: &str) -> Option<char> {
    let mut chars = line.chars();
    match chars.next()? {
        '-' | '*' | '+' => {
            if chars.next() == Some(' ') {
                return line.chars().next();
            }
            None
        }
        c if c.is_ascii_digit() => {
            for ch in chars {
                if ch == '.' || ch == ')' {
                    return None;
                }
                if !ch.is_ascii_digit() {
                    return None;
                }
            }
            None
        }
        _ => None,
    }
}

fn is_url_in_markdown_link(line: &str, url: &str) -> bool {
    if let Some(idx) = line.find(url) {
        let before = &line[..idx];
        let after_idx = idx + url.len();

        if before.ends_with('(') || before.ends_with('<') {
            return true;
        }

        if before.ends_with("](") {
            return true;
        }

        if after_idx < line.len() {
            let next_char = line.chars().nth(after_idx);
            if (next_char == Some(')') || next_char == Some('>'))
                && (before.ends_with('(') || before.ends_with('<') || before.contains("]("))
            {
                return true;
            }
        }
    }
    false
}

fn is_atx_heading(line: &str) -> bool {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('#') {
        return false;
    }

    let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
    if hash_count > 6 {
        return false;
    }

    let after_hashes = &trimmed[hash_count..];
    after_hashes.is_empty() || after_hashes.starts_with(' ')
}

fn is_list_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
        return true;
    }

    let mut chars = trimmed.chars().peekable();
    if chars.peek().is_some_and(char::is_ascii_digit) {
        while chars.peek().is_some_and(char::is_ascii_digit) {
            chars.next();
        }
        let delimiter = chars.next();
        if matches!(delimiter, Some('.' | ')')) && chars.next() == Some(' ') {
            return true;
        }
    }

    false
}

fn is_list_continuation(line: &str) -> bool {
    let leading_spaces = line.len() - line.trim_start().len();
    leading_spaces >= 2
}

fn normalize_path(path: &Path) -> std::path::PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::CurDir => {}
            _ => {
                components.push(component);
            }
        }
    }
    components.iter().collect()
}

fn compute_relative_path(from: &Path, to: &Path) -> std::path::PathBuf {
    let from_components: Vec<_> = from.components().collect();
    let to_components: Vec<_> = to.components().collect();

    let common_len =
        from_components.iter().zip(to_components.iter()).take_while(|(a, b)| a == b).count();

    let mut result = std::path::PathBuf::new();

    for _ in common_len..from_components.len() {
        result.push("..");
    }

    for component in &to_components[common_len..] {
        result.push(component);
    }

    result
}
