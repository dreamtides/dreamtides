use tracing::debug;
use unicode_width::UnicodeWidthStr;

/// Default line width for wrapping.
pub const DEFAULT_LINE_WIDTH: usize = 80;

/// Configuration for text wrapping.
#[derive(Debug, Clone)]
pub struct WrapConfig {
    /// Maximum line width in characters.
    pub line_width: usize,
}

/// Result of wrapping text.
#[derive(Debug)]
pub struct WrapResult {
    /// The wrapped content.
    pub content: String,
    /// Whether any changes were made.
    pub modified: bool,
}

/// Wraps the given markdown content at the configured line width.
///
/// Preserves structure elements:
/// - Code blocks (fenced and indented)
/// - Tables (lines containing |)
/// - List items (maintains structure)
/// - Links (doesn't break markdown link syntax)
pub fn wrap(content: &str, config: &WrapConfig) -> WrapResult {
    debug!(line_width = config.line_width, "Wrapping text");
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut i = 0;
    let mut modified = false;
    while i < lines.len() {
        let line = lines[i];
        if is_fenced_code_start(line) {
            i = process_fenced_code_block(&lines, i, &mut result);
            continue;
        }
        if should_preserve_unchanged(line) {
            result.push(line.to_string());
            i += 1;
            continue;
        }
        let (wrapped, was_modified) = wrap_line(line, config);
        result.extend(wrapped);
        modified = modified || was_modified;
        i += 1;
    }
    let mut content = result.join("\n");
    // Ensure trailing newline for non-empty content
    if !content.is_empty() {
        content.push('\n');
    }
    WrapResult { content, modified }
}

/// Processes a fenced code block starting at index `start`, adding lines to
/// result. Returns the new index after the code block.
fn process_fenced_code_block(lines: &[&str], start: usize, result: &mut Vec<String>) -> usize {
    let fence = extract_fence(lines[start]);
    result.push(lines[start].to_string());
    let mut i = start + 1;
    while i < lines.len() {
        let inner = lines[i];
        result.push(inner.to_string());
        i += 1;
        if is_matching_fence_end(inner, &fence) {
            break;
        }
    }
    i
}

/// Returns true if the line should be preserved without wrapping.
fn should_preserve_unchanged(line: &str) -> bool {
    is_indented_code(line)
        || is_table_line(line)
        || is_html_block(line)
        || is_heading(line)
        || line.trim().is_empty()
}

/// Wraps a line that requires wrapping (list item, blockquote, or paragraph).
fn wrap_line(line: &str, config: &WrapConfig) -> (Vec<String>, bool) {
    if is_list_item(line) {
        wrap_list_item(line, config)
    } else if is_blockquote(line) {
        wrap_blockquote(line, config)
    } else {
        wrap_paragraph_line(line, config)
    }
}

impl Default for WrapConfig {
    fn default() -> Self {
        Self { line_width: DEFAULT_LINE_WIDTH }
    }
}

impl WrapConfig {
    /// Creates a new wrap configuration with the specified line width.
    pub fn new(line_width: usize) -> Self {
        Self { line_width }
    }
}

/// Returns true if the line starts a fenced code block.
fn is_fenced_code_start(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("```") || trimmed.starts_with("~~~")
}

/// Extracts the fence characters (``` or ~~~) from the start of a code block.
fn extract_fence(line: &str) -> String {
    let trimmed = line.trim_start();
    if trimmed.starts_with("```") {
        "```".to_string()
    } else if trimmed.starts_with("~~~") {
        "~~~".to_string()
    } else {
        String::new()
    }
}

/// Returns true if the line ends a fenced code block with the matching fence.
fn is_matching_fence_end(line: &str, fence: &str) -> bool {
    let trimmed = line.trim();
    trimmed == fence || (trimmed.starts_with(fence) && trimmed[fence.len()..].trim().is_empty())
}

/// Returns true if the line is indented code (4+ spaces or tab at start).
fn is_indented_code(line: &str) -> bool {
    line.starts_with("    ") || line.starts_with('\t')
}

/// Returns true if the line appears to be part of a table.
fn is_table_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.contains('|') && !trimmed.starts_with('[')
}

/// Returns true if the line appears to be an HTML block or comment.
fn is_html_block(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with('<') && !trimmed.starts_with("<http")
}

/// Returns true if the line is a heading.
fn is_heading(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with('#')
}

/// Returns true if the line is a list item.
fn is_list_item(line: &str) -> bool {
    let trimmed = line.trim_start();
    if trimmed.starts_with("- ")
        || trimmed.starts_with("* ")
        || trimmed.starts_with("+ ")
        || trimmed == "-"
        || trimmed == "*"
        || trimmed == "+"
    {
        return true;
    }
    is_ordered_list_item(trimmed)
}

/// Returns true if the line starts with an ordered list marker.
fn is_ordered_list_item(trimmed: &str) -> bool {
    let mut chars = trimmed.chars().peekable();
    if !chars.next().is_some_and(|c| c.is_ascii_digit()) {
        return false;
    }
    while chars.peek().is_some_and(char::is_ascii_digit) {
        chars.next();
    }
    matches!(chars.next(), Some('.' | ')')) && matches!(chars.next(), Some(' ') | None)
}

/// Returns true if the line is a blockquote.
fn is_blockquote(line: &str) -> bool {
    line.trim_start().starts_with('>')
}

/// Wraps a list item, preserving the marker and indentation.
fn wrap_list_item(line: &str, config: &WrapConfig) -> (Vec<String>, bool) {
    let indent = line.len() - line.trim_start().len();
    let indent_str: String = line.chars().take(indent).collect();
    let trimmed = line.trim_start();
    let marker_end = find_list_marker_end(trimmed);
    let marker = &trimmed[..marker_end];
    let content = trimmed[marker_end..].trim_start();
    let continuation_indent = indent + UnicodeWidthStr::width(marker) + 1;
    let continuation_str = " ".repeat(continuation_indent);
    let first_line_width = config.line_width.saturating_sub(indent + marker.len() + 1);
    if content.is_empty() || UnicodeWidthStr::width(content) <= first_line_width {
        return (vec![line.to_string()], false);
    }
    let wrapped = wrap_text_with_indent(
        content,
        first_line_width,
        config.line_width.saturating_sub(continuation_indent),
    );
    let mut result = Vec::new();
    for (i, wrapped_line) in wrapped.iter().enumerate() {
        if i == 0 {
            result.push(format!("{}{} {}", indent_str, marker, wrapped_line));
        } else {
            result.push(format!("{}{}", continuation_str, wrapped_line));
        }
    }
    (result, wrapped.len() > 1)
}

/// Finds the end index of a list marker.
fn find_list_marker_end(line: &str) -> usize {
    if line.starts_with("- ") || line.starts_with("* ") || line.starts_with("+ ") {
        return 1;
    }
    if line == "-" || line == "*" || line == "+" {
        return 1;
    }
    let mut i = 0;
    let bytes = line.as_bytes();
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i < bytes.len() && (bytes[i] == b'.' || bytes[i] == b')') {
        return i + 1;
    }
    0
}

/// Wraps a blockquote line.
fn wrap_blockquote(line: &str, config: &WrapConfig) -> (Vec<String>, bool) {
    let indent = line.len() - line.trim_start().len();
    let indent_str: String = line.chars().take(indent).collect();
    let trimmed = line.trim_start();
    let marker_end = trimmed.find(|c: char| c != '>' && c != ' ').unwrap_or(trimmed.len());
    let marker = &trimmed[..marker_end];
    let content = &trimmed[marker_end..];
    let available_width = config.line_width.saturating_sub(indent + marker.len());
    if content.is_empty() || UnicodeWidthStr::width(content) <= available_width {
        return (vec![line.to_string()], false);
    }
    let wrapped = wrap_text_with_indent(content, available_width, available_width);
    let mut result = Vec::new();
    for wrapped_line in &wrapped {
        result.push(format!("{}{}{}", indent_str, marker, wrapped_line));
    }
    (result, wrapped.len() > 1)
}

/// Wraps a regular paragraph line.
fn wrap_paragraph_line(line: &str, config: &WrapConfig) -> (Vec<String>, bool) {
    let indent = line.len() - line.trim_start().len();
    let indent_str: String = line.chars().take(indent).collect();
    let content = line.trim_start();
    let available_width = config.line_width.saturating_sub(indent);
    if content.is_empty() || UnicodeWidthStr::width(content) <= available_width {
        return (vec![line.to_string()], false);
    }
    let wrapped = wrap_text_with_indent(content, available_width, available_width);
    let result: Vec<String> = wrapped.into_iter().map(|w| format!("{}{}", indent_str, w)).collect();
    let modified = result.len() > 1;
    (result, modified)
}

/// Wraps text content, respecting word boundaries and preserving links.
fn wrap_text_with_indent(
    content: &str,
    first_line_width: usize,
    subsequent_width: usize,
) -> Vec<String> {
    let tokens = tokenize(content);
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;
    let mut is_first_line = true;
    for token in tokens {
        let token_width = UnicodeWidthStr::width(token.as_str());
        let max_width = if is_first_line { first_line_width } else { subsequent_width };
        if current_line.is_empty() {
            current_line = token;
            current_width = token_width;
            continue;
        }
        let space_width = if current_line.ends_with(' ') { 0 } else { 1 };
        if current_width + space_width + token_width > max_width {
            lines.push(current_line.trim_end().to_string());
            current_line = token;
            current_width = token_width;
            is_first_line = false;
        } else {
            if !current_line.ends_with(' ') && !token.starts_with(' ') {
                current_line.push(' ');
                current_width += 1;
            }
            current_line.push_str(&token);
            current_width += token_width;
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line.trim_end().to_string());
    }
    lines
}

/// Tokenizes content into words, keeping markdown links intact.
fn tokenize(content: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = content.chars().peekable();
    let mut current = String::new();
    while let Some(c) = chars.next() {
        if c == '[' {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
            let link = capture_markdown_link(c, &mut chars);
            tokens.push(link);
        } else if c.is_whitespace() {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

/// Captures a complete markdown link starting from '['.
fn capture_markdown_link(start: char, chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut link = String::new();
    link.push(start);
    let mut bracket_depth = 1;
    let mut in_link_dest = false;
    let mut paren_depth = 0;
    while let Some(&c) = chars.peek() {
        link.push(c);
        chars.next();
        if !in_link_dest {
            match c {
                '[' => bracket_depth += 1,
                ']' => {
                    bracket_depth -= 1;
                    if bracket_depth == 0 {
                        if chars.peek() == Some(&'(') {
                            in_link_dest = true;
                            link.push('(');
                            chars.next();
                            paren_depth = 1;
                        } else {
                            break;
                        }
                    }
                }
                _ => {}
            }
        } else {
            match c {
                '(' => paren_depth += 1,
                ')' => {
                    paren_depth -= 1;
                    if paren_depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
        }
    }
    link
}
