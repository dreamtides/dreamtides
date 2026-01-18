//! Tests for the list normalizer module.

use lattice::format::list_normalizer::normalize_lists;

#[test]
fn normalize_lists_empty_input() {
    let result = normalize_lists("");
    assert_eq!(result.content, "");
    assert!(!result.modified);
}

#[test]
fn normalize_lists_no_lists_unchanged() {
    let input = "Just some text.\n";
    let result = normalize_lists(input);
    assert_eq!(result.content, input);
    assert!(!result.modified);
}

#[test]
fn normalize_lists_dash_list_unchanged() {
    let input = "- Item 1\n- Item 2\n";
    let result = normalize_lists(input);
    assert_eq!(result.content, input);
    assert!(!result.modified);
}

#[test]
fn normalize_lists_asterisk_converts_to_dash() {
    let input = "* Item 1\n* Item 2\n";
    let result = normalize_lists(input);
    assert!(result.modified);
    assert!(result.content.contains("- Item 1"));
    assert!(result.content.contains("- Item 2"));
    assert!(!result.content.contains('*'));
}

#[test]
fn normalize_lists_plus_converts_to_dash() {
    let input = "+ Item 1\n+ Item 2\n";
    let result = normalize_lists(input);
    assert!(result.modified);
    assert!(result.content.contains("- Item 1"));
    assert!(result.content.contains("- Item 2"));
    assert!(!result.content.contains('+'));
}

#[test]
fn normalize_lists_mixed_markers_converted() {
    let input = "* Item 1\n+ Item 2\n- Item 3\n";
    let result = normalize_lists(input);
    assert!(result.modified);
    let lines: Vec<&str> = result.content.lines().collect();
    assert!(lines[0].starts_with("- "));
    assert!(lines[1].starts_with("- "));
    assert!(lines[2].starts_with("- "));
}

#[test]
fn normalize_lists_preserves_indentation() {
    let input = "  * Nested item\n";
    let result = normalize_lists(input);
    assert!(result.modified);
    assert!(result.content.starts_with("  - "));
    assert!(result.content.contains("Nested item"));
}

#[test]
fn normalize_lists_adds_blank_line_before() {
    let input = "Some text.\n- Item 1\n";
    let result = normalize_lists(input);
    assert!(result.modified);
    assert!(result.content.contains("Some text.\n\n- Item 1"));
}

#[test]
fn normalize_lists_adds_blank_line_after() {
    let input = "- Item 1\nSome text.\n";
    let result = normalize_lists(input);
    assert!(result.modified);
    assert!(result.content.contains("- Item 1\n\nSome text."));
}

#[test]
fn normalize_lists_no_duplicate_blank_lines() {
    let input = "Some text.\n\n- Item 1\n\nMore text.\n";
    let result = normalize_lists(input);
    assert!(!result.modified);
    assert_eq!(result.content, input);
}

#[test]
fn normalize_lists_ordered_list_unchanged() {
    let input = "1. First\n2. Second\n";
    let result = normalize_lists(input);
    assert!(!result.modified);
    assert!(result.content.contains("1. First"));
    assert!(result.content.contains("2. Second"));
}

#[test]
fn normalize_lists_ordered_list_with_paren() {
    let input = "1) First\n2) Second\n";
    let result = normalize_lists(input);
    assert!(!result.modified);
    assert!(result.content.contains("1) First"));
}

#[test]
fn normalize_lists_nested_list_preserves_structure() {
    let input = "- Item 1\n  - Nested 1\n  - Nested 2\n- Item 2\n";
    let result = normalize_lists(input);
    assert!(!result.modified);
    assert!(result.content.contains("- Item 1"));
    assert!(result.content.contains("  - Nested 1"));
}

#[test]
fn normalize_lists_first_line_list_no_blank_before() {
    let input = "- First item\n- Second item\n";
    let result = normalize_lists(input);
    assert!(!result.modified);
    assert!(!result.content.starts_with('\n'));
}

#[test]
fn normalize_lists_bare_asterisk_converts() {
    let input = "*\n";
    let result = normalize_lists(input);
    assert!(result.modified);
    assert_eq!(result.content.trim(), "-");
}

#[test]
fn normalize_lists_bare_plus_converts() {
    let input = "+\n";
    let result = normalize_lists(input);
    assert!(result.modified);
    assert_eq!(result.content.trim(), "-");
}

#[test]
fn normalize_lists_preserves_content_after_marker() {
    let input = "* Item with **bold** and `code`\n";
    let result = normalize_lists(input);
    assert!(result.modified);
    assert!(result.content.contains("- Item with **bold** and `code`"));
}

#[test]
fn normalize_lists_large_ordered_number() {
    let input = "999. Large number\n";
    let result = normalize_lists(input);
    assert!(!result.modified);
    assert!(result.content.contains("999. Large number"));
}

#[test]
fn normalize_lists_consecutive_lists() {
    let input = "* List A\n\n* List B\n";
    let result = normalize_lists(input);
    assert!(result.modified);
    assert!(result.content.contains("- List A"));
    assert!(result.content.contains("- List B"));
}
