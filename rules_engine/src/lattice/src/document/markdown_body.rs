/// Soft limit for document line count before linter warning.
pub const SOFT_LINE_LIMIT: usize = 500;

/// Statistics about a markdown body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BodyStats {
    /// Total number of lines in the body.
    pub line_count: usize,
    /// Number of non-empty lines.
    pub non_empty_lines: usize,
    /// Number of top-level headers (lines starting with `# `).
    pub top_level_headers: usize,
}

/// Counts the number of lines in a body string.
///
/// An empty body has 0 lines. A body with content but no newline at the end
/// counts as 1 line.
pub fn count_lines(body: &str) -> usize {
    if body.is_empty() {
        return 0;
    }
    body.lines().count()
}

/// Returns true if the body exceeds the soft line limit.
pub fn exceeds_soft_limit(body: &str) -> bool {
    count_lines(body) > SOFT_LINE_LIMIT
}

/// Computes statistics about the markdown body.
pub fn compute_stats(body: &str) -> BodyStats {
    let mut line_count = 0;
    let mut non_empty_lines = 0;
    let mut top_level_headers = 0;

    for line in body.lines() {
        line_count += 1;
        if !line.trim().is_empty() {
            non_empty_lines += 1;
        }
        if line.starts_with("# ") {
            top_level_headers += 1;
        }
    }

    BodyStats { line_count, non_empty_lines, top_level_headers }
}

/// Appends content to the end of a body.
///
/// Ensures there's a blank line between existing content and appended content.
pub fn append_content(body: &str, content: &str) -> String {
    if body.is_empty() {
        return content.to_string();
    }

    let body_trimmed = body.trim_end();
    if body_trimmed.is_empty() {
        return content.to_string();
    }

    format!("{body_trimmed}\n\n{content}")
}

/// Prepends content to the start of a body.
///
/// Ensures there's a blank line between prepended content and existing content.
pub fn prepend_content(body: &str, content: &str) -> String {
    if body.is_empty() {
        return content.to_string();
    }

    let body_trimmed = body.trim_start();
    if body_trimmed.is_empty() {
        return content.to_string();
    }

    format!("{content}\n\n{body_trimmed}")
}

/// Extracts sections from a markdown body by top-level headers.
///
/// Returns a vector of (header_text, section_content) tuples.
/// Content before the first header is returned with an empty header.
/// The section_content includes everything from the header line to the next
/// top-level header (or end of document).
pub fn extract_sections(body: &str) -> Vec<(String, String)> {
    let mut sections = Vec::new();
    let mut current_header = String::new();
    let mut current_content = String::new();

    for line in body.lines() {
        if let Some(header_text) = line.strip_prefix("# ") {
            if !current_header.is_empty() || !current_content.trim().is_empty() {
                sections.push((current_header, current_content.trim().to_string()));
            }
            current_header = header_text.trim().to_string();
            current_content = format!("{line}\n");
        } else {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    if !current_header.is_empty() || !current_content.trim().is_empty() {
        sections.push((current_header, current_content.trim().to_string()));
    }

    sections
}

/// Finds the end position of frontmatter in raw document content.
///
/// Returns the byte offset immediately after the closing `---` delimiter and
/// any trailing newlines, or None if frontmatter is not properly formed.
/// The returned offset is relative to the original content, including any BOM.
pub fn find_body_start(content: &str) -> Option<usize> {
    const DELIMITER: &str = "---";
    const BOM: char = '\u{feff}';

    let bom_len = if content.starts_with(BOM) { BOM.len_utf8() } else { 0 };
    let content_without_bom = &content[bom_len..];

    if !content_without_bom.starts_with(DELIMITER) {
        return None;
    }

    let after_opening = &content_without_bom[DELIMITER.len()..];
    let mut pos = 0;

    for line in after_opening.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\n', '\r']);
        if trimmed == DELIMITER {
            let body_start = DELIMITER.len() + pos + line.len();
            let remaining = &content_without_bom[body_start..];
            let skip_count = remaining.len() - remaining.trim_start_matches(['\n', '\r']).len();
            return Some(bom_len + body_start + skip_count);
        }
        pos += line.len();
    }

    let remaining = &after_opening[pos..];
    if remaining.trim() == DELIMITER { Some(content.len()) } else { None }
}

/// Checks if a line is a Lattice section marker.
///
/// Lattice template sections are marked with a bracketed Lattice prefix, e.g.,
/// a header containing `## ` followed by the Lattice marker and `Context`.
pub fn is_lattice_section(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.contains("[Lattice]") && trimmed.starts_with('#')
}

/// Extracts Lattice template sections from a body.
///
/// Returns a tuple of (context_section, acceptance_section) where each is
/// the full section content including the header, or None if not present.
pub fn extract_lattice_sections(body: &str) -> (Option<String>, Option<String>) {
    let sections = extract_sections(body);
    let mut context = None;
    let mut acceptance = None;

    for (header, content) in sections {
        if header.contains("[Lattice] Context") {
            context = Some(content);
        } else if header.contains("[Lattice] Acceptance Criteria") {
            acceptance = Some(content);
        }
    }

    (context, acceptance)
}

/// Removes Lattice template sections from a body.
///
/// Returns the body with the Context and Acceptance Criteria sections that
/// have the Lattice marker removed.
pub fn strip_lattice_sections(body: &str) -> String {
    let sections = extract_sections(body);
    let mut result = String::new();

    for (header, content) in sections {
        if !header.contains("[Lattice] Context")
            && !header.contains("[Lattice] Acceptance Criteria")
        {
            if !result.is_empty() && !content.is_empty() {
                result.push_str("\n\n");
            }
            result.push_str(&content);
        }
    }

    result
}
