//! Tests for the whitespace cleaner module.

use lattice::format::whitespace_cleaner::clean_whitespace;

#[test]
fn clean_whitespace_empty_input() {
    let result = clean_whitespace("");
    assert_eq!(result.content, "");
    assert!(!result.modified);
}

#[test]
fn clean_whitespace_already_clean() {
    let input = "Clean text.\n";
    let result = clean_whitespace(input);
    assert_eq!(result.content, input);
    assert!(!result.modified);
}

#[test]
fn clean_whitespace_removes_trailing_spaces() {
    let input = "Line with spaces   \n";
    let result = clean_whitespace(input);
    assert!(result.modified);
    assert_eq!(result.content, "Line with spaces\n");
}

#[test]
fn clean_whitespace_removes_trailing_tabs() {
    let input = "Line with tabs\t\t\n";
    let result = clean_whitespace(input);
    assert!(result.modified);
    assert_eq!(result.content, "Line with tabs\n");
}

#[test]
fn clean_whitespace_preserves_backslash_continuation() {
    let input = "Line continuation\\ \n";
    let result = clean_whitespace(input);
    assert!(!result.modified);
    assert!(result.content.contains("\\ "));
}

#[test]
fn clean_whitespace_collapses_multiple_blank_lines() {
    let input = "First.\n\n\n\nSecond.\n";
    let result = clean_whitespace(input);
    assert!(result.modified);
    assert_eq!(result.content, "First.\n\nSecond.\n");
}

#[test]
fn clean_whitespace_preserves_single_blank_line() {
    let input = "First.\n\nSecond.\n";
    let result = clean_whitespace(input);
    assert_eq!(result.content, input);
    assert!(!result.modified);
}

#[test]
fn clean_whitespace_adds_final_newline() {
    let input = "No newline at end";
    let result = clean_whitespace(input);
    assert!(result.modified);
    assert_eq!(result.content, "No newline at end\n");
}

#[test]
fn clean_whitespace_removes_extra_final_newlines() {
    let input = "Extra newlines\n\n\n";
    let result = clean_whitespace(input);
    assert!(result.modified);
    assert_eq!(result.content, "Extra newlines\n");
}

#[test]
fn clean_whitespace_multiple_lines_with_trailing() {
    let input = "Line 1  \nLine 2\t\nLine 3   \n";
    let result = clean_whitespace(input);
    assert!(result.modified);
    assert_eq!(result.content, "Line 1\nLine 2\nLine 3\n");
}

#[test]
fn clean_whitespace_only_blank_lines() {
    let input = "\n\n\n";
    let result = clean_whitespace(input);
    assert!(result.modified);
    assert_eq!(result.content, "\n");
}

#[test]
fn clean_whitespace_single_line_no_newline() {
    let input = "Single line";
    let result = clean_whitespace(input);
    assert!(result.modified);
    assert_eq!(result.content, "Single line\n");
}

#[test]
fn clean_whitespace_preserves_leading_whitespace() {
    let input = "    Indented line\n";
    let result = clean_whitespace(input);
    assert_eq!(result.content, input);
    assert!(!result.modified);
}

#[test]
fn clean_whitespace_preserves_content_between_lines() {
    let input = "First.\n\nSecond.\n\nThird.\n";
    let result = clean_whitespace(input);
    assert_eq!(result.content, input);
    assert!(!result.modified);
}

#[test]
fn clean_whitespace_mixed_issues() {
    let input = "Trailing spaces  \n\n\n\nNo final newline";
    let result = clean_whitespace(input);
    assert!(result.modified);
    assert_eq!(result.content, "Trailing spaces\n\nNo final newline\n");
}

#[test]
fn clean_whitespace_backslash_not_at_end() {
    let input = "Has \\ backslash  \n";
    let result = clean_whitespace(input);
    assert!(result.modified);
    assert_eq!(result.content, "Has \\ backslash\n");
}

#[test]
fn clean_whitespace_three_blank_lines_to_one() {
    let input = "A\n\n\nB\n";
    let result = clean_whitespace(input);
    assert!(result.modified);
    assert_eq!(result.content, "A\n\nB\n");
}

#[test]
fn clean_whitespace_blank_line_with_spaces() {
    let input = "Line 1\n   \nLine 2\n";
    let result = clean_whitespace(input);
    assert!(result.modified);
    assert_eq!(result.content, "Line 1\n\nLine 2\n");
}
