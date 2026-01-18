use lattice::lint::autofix_engine::{
    FIXABLE_CODES, fix_heading_blank_lines, fix_list_blank_lines, fix_list_markers,
    fix_missing_final_newline, fix_multiple_blank_lines, fix_setext_headers,
    fix_trailing_whitespace, is_fixable, normalize_name,
};

#[test]
fn fixable_codes_contains_expected_rules() {
    assert!(FIXABLE_CODES.contains(&"W004"), "W004 should be fixable");
    assert!(FIXABLE_CODES.contains(&"W005"), "W005 should be fixable");
    assert!(FIXABLE_CODES.contains(&"W006"), "W006 should be fixable");
    assert!(FIXABLE_CODES.contains(&"W011"), "W011 should be fixable");
    assert!(FIXABLE_CODES.contains(&"W012"), "W012 should be fixable");
    assert!(FIXABLE_CODES.contains(&"W013"), "W013 should be fixable");
    assert!(FIXABLE_CODES.contains(&"W014"), "W014 should be fixable");
    assert!(FIXABLE_CODES.contains(&"W015"), "W015 should be fixable");
    assert!(FIXABLE_CODES.contains(&"E008"), "E008 should be fixable");
}

#[test]
fn is_fixable_returns_true_for_fixable_codes() {
    assert!(is_fixable("W004"), "W004 should be fixable");
    assert!(is_fixable("E008"), "E008 should be fixable");
}

#[test]
fn is_fixable_returns_false_for_non_fixable_codes() {
    assert!(!is_fixable("E001"), "E001 should not be fixable");
    assert!(!is_fixable("W001"), "W001 should not be fixable");
    assert!(!is_fixable("INVALID"), "INVALID should not be fixable");
}

#[test]
fn normalize_name_converts_underscores_to_hyphens() {
    assert_eq!(normalize_name("hello_world"), "hello-world", "underscores should become hyphens");
}

#[test]
fn normalize_name_lowercases_characters() {
    assert_eq!(normalize_name("UPPER"), "upper", "uppercase letters should be lowercased");
}

#[test]
fn normalize_name_collapses_multiple_hyphens() {
    assert_eq!(
        normalize_name("test--name"),
        "test-name",
        "multiple hyphens should collapse to one"
    );
}

#[test]
fn normalize_name_converts_spaces_to_hyphens() {
    assert_eq!(normalize_name("with spaces"), "with-spaces", "spaces should become hyphens");
}

#[test]
fn normalize_name_handles_mixed_separators() {
    assert_eq!(
        normalize_name("a__b--c"),
        "a-b-c",
        "mixed separators should normalize to single hyphens"
    );
}

#[test]
fn fix_trailing_whitespace_removes_trailing_spaces() {
    let input = "hello  \nworld\t\n";
    let expected = "hello\nworld\n";
    assert_eq!(fix_trailing_whitespace(input), expected, "trailing whitespace should be removed");
}

#[test]
fn fix_trailing_whitespace_preserves_content_without_trailing() {
    let input = "no trailing";
    assert_eq!(
        fix_trailing_whitespace(input),
        input,
        "content without trailing whitespace should be unchanged"
    );
}

#[test]
fn fix_multiple_blank_lines_collapses_to_single() {
    let input = "a\n\n\nb\n";
    let expected = "a\n\nb\n";
    assert_eq!(
        fix_multiple_blank_lines(input),
        expected,
        "multiple blank lines should collapse to one"
    );
}

#[test]
fn fix_multiple_blank_lines_preserves_single_blank() {
    let input = "a\n\nb\n";
    assert_eq!(fix_multiple_blank_lines(input), input, "single blank lines should be preserved");
}

#[test]
fn fix_missing_final_newline_adds_newline() {
    let input = "hello";
    let expected = "hello\n";
    assert_eq!(fix_missing_final_newline(input), expected, "missing final newline should be added");
}

#[test]
fn fix_missing_final_newline_preserves_existing() {
    let input = "hello\n";
    assert_eq!(
        fix_missing_final_newline(input),
        input,
        "existing final newline should be preserved"
    );
}

#[test]
fn fix_missing_final_newline_handles_empty_string() {
    assert_eq!(fix_missing_final_newline(""), "", "empty string should remain empty");
}

#[test]
fn fix_setext_headers_converts_level_one() {
    let input = "Title\n=====\n";
    let expected = "# Title\n";
    assert_eq!(fix_setext_headers(input), expected, "setext level 1 header should convert to ATX");
}

#[test]
fn fix_setext_headers_converts_level_two() {
    let input = "Title\n-----\n";
    let expected = "## Title\n";
    assert_eq!(fix_setext_headers(input), expected, "setext level 2 header should convert to ATX");
}

#[test]
fn fix_setext_headers_converts_multiple() {
    let input = "Title\n=====\n\nSubtitle\n--------\n";
    let expected = "# Title\n\n## Subtitle\n";
    assert_eq!(
        fix_setext_headers(input),
        expected,
        "multiple setext headers should convert to ATX"
    );
}

#[test]
fn fix_list_markers_converts_asterisks() {
    let input = "* item1\n* item2\n";
    let expected = "- item1\n- item2\n";
    assert_eq!(fix_list_markers(input), expected, "asterisk markers should convert to dashes");
}

#[test]
fn fix_list_markers_converts_plus_signs() {
    let input = "+ item1\n+ item2\n";
    let expected = "- item1\n- item2\n";
    assert_eq!(fix_list_markers(input), expected, "plus sign markers should convert to dashes");
}

#[test]
fn fix_list_markers_preserves_dashes() {
    let input = "- item1\n- item2\n";
    assert_eq!(fix_list_markers(input), input, "dash markers should be preserved");
}

#[test]
fn fix_list_markers_preserves_indentation() {
    let input = "  * nested\n    * deeper\n";
    let expected = "  - nested\n    - deeper\n";
    assert_eq!(
        fix_list_markers(input),
        expected,
        "indentation should be preserved during conversion"
    );
}

#[test]
fn fix_list_markers_handles_mixed() {
    let input = "* item1\n+ item2\n- item3\n";
    let expected = "- item1\n- item2\n- item3\n";
    assert_eq!(fix_list_markers(input), expected, "mixed markers should all convert to dashes");
}

#[test]
fn fix_heading_blank_lines_adds_before() {
    let input = "text\n# Heading\n";
    let expected = "text\n\n# Heading\n";
    assert_eq!(
        fix_heading_blank_lines(input),
        expected,
        "blank line should be added before heading"
    );
}

#[test]
fn fix_heading_blank_lines_adds_after() {
    let input = "# Heading\ntext\n";
    let expected = "# Heading\n\ntext\n";
    assert_eq!(
        fix_heading_blank_lines(input),
        expected,
        "blank line should be added after heading"
    );
}

#[test]
fn fix_heading_blank_lines_preserves_existing() {
    let input = "text\n\n# Heading\n\nmore text\n";
    assert_eq!(fix_heading_blank_lines(input), input, "existing blank lines should be preserved");
}

#[test]
fn fix_list_blank_lines_adds_before() {
    let input = "text\n- item\n";
    let expected = "text\n\n- item\n";
    assert_eq!(fix_list_blank_lines(input), expected, "blank line should be added before list");
}

#[test]
fn fix_list_blank_lines_preserves_existing() {
    let input = "text\n\n- item\n\nmore\n";
    assert_eq!(fix_list_blank_lines(input), input, "existing blank lines should be preserved");
}
