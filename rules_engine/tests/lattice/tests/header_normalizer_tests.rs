//! Tests for the header normalizer module.

use lattice::format::header_normalizer::normalize_headers;

#[test]
fn normalize_headers_empty_input() {
    let result = normalize_headers("");
    assert_eq!(result.content, "");
    assert!(!result.modified);
}

#[test]
fn normalize_headers_no_headers_unchanged() {
    let input = "Just some text.\n";
    let result = normalize_headers(input);
    assert_eq!(result.content, input);
    assert!(!result.modified);
}

#[test]
fn normalize_headers_atx_already_correct() {
    let input = "# Heading 1\n\nSome text.\n";
    let result = normalize_headers(input);
    assert_eq!(result.content, input);
    assert!(!result.modified);
}

#[test]
fn normalize_headers_setext_h1_converts_to_atx() {
    let input = "Heading 1\n=========\n";
    let result = normalize_headers(input);
    assert!(result.modified);
    assert!(result.content.contains("# Heading 1"));
    assert!(!result.content.contains("==="));
}

#[test]
fn normalize_headers_setext_h2_converts_to_atx() {
    let input = "Heading 2\n---------\n";
    let result = normalize_headers(input);
    assert!(result.modified);
    assert!(result.content.contains("## Heading 2"));
    assert!(!result.content.contains("---"));
}

#[test]
fn normalize_headers_atx_extra_spaces_normalized() {
    let input = "#  Too Many Spaces\n";
    let result = normalize_headers(input);
    assert!(result.modified);
    assert_eq!(result.content.trim(), "# Too Many Spaces");
}

#[test]
fn normalize_headers_atx_no_space_normalized() {
    let input = "#NoSpace\n";
    let result = normalize_headers(input);
    assert!(result.modified);
    assert_eq!(result.content.trim(), "# NoSpace");
}

#[test]
fn normalize_headers_adds_blank_line_before() {
    let input = "Some text.\n# Heading\n";
    let result = normalize_headers(input);
    assert!(result.modified);
    assert!(result.content.contains("Some text.\n\n# Heading"));
}

#[test]
fn normalize_headers_adds_blank_line_after() {
    let input = "# Heading\nSome text.\n";
    let result = normalize_headers(input);
    assert!(result.modified);
    assert!(result.content.contains("# Heading\n\nSome text."));
}

#[test]
fn normalize_headers_no_duplicate_blank_lines() {
    let input = "Some text.\n\n# Heading\n\nMore text.\n";
    let result = normalize_headers(input);
    assert!(!result.modified);
    assert_eq!(result.content, input);
}

#[test]
fn normalize_headers_preserves_level_hierarchy() {
    let input = "# H1\n\n## H2\n\n### H3\n";
    let result = normalize_headers(input);
    assert!(!result.modified);
    assert!(result.content.contains("# H1"));
    assert!(result.content.contains("## H2"));
    assert!(result.content.contains("### H3"));
}

#[test]
fn normalize_headers_h6_level() {
    let input = "###### H6\n";
    let result = normalize_headers(input);
    assert!(!result.modified);
    assert!(result.content.contains("###### H6"));
}

#[test]
fn normalize_headers_seven_hashes_not_header() {
    let input = "####### Not a header\n";
    let result = normalize_headers(input);
    assert!(!result.modified);
    assert!(result.content.contains("####### Not a header"));
}

#[test]
fn normalize_headers_first_line_header_no_blank_before() {
    let input = "# First Line Header\n\nText.\n";
    let result = normalize_headers(input);
    assert!(!result.modified);
    assert!(!result.content.starts_with('\n'));
}

#[test]
fn normalize_headers_setext_with_surrounding_text() {
    let input = "Before.\n\nTitle\n=====\n\nAfter.\n";
    let result = normalize_headers(input);
    assert!(result.modified);
    assert!(result.content.contains("Before."));
    assert!(result.content.contains("# Title"));
    assert!(result.content.contains("After."));
}

#[test]
fn normalize_headers_multiple_setext_headers() {
    let input = "First\n=====\n\nSecond\n------\n";
    let result = normalize_headers(input);
    assert!(result.modified);
    assert!(result.content.contains("# First"));
    assert!(result.content.contains("## Second"));
}

#[test]
fn normalize_headers_empty_header() {
    let input = "##\n";
    let result = normalize_headers(input);
    assert!(!result.modified);
    assert_eq!(result.content.trim(), "##");
}

#[test]
fn normalize_headers_setext_minimum_underline() {
    let input = "Head\n---\n";
    let result = normalize_headers(input);
    assert!(result.modified);
    assert!(result.content.contains("## Head"));
}

#[test]
fn normalize_headers_setext_too_short_underline() {
    let input = "Not Header\n--\n";
    let result = normalize_headers(input);
    assert!(!result.modified);
    assert!(result.content.contains("Not Header"));
    assert!(result.content.contains("--"));
}
