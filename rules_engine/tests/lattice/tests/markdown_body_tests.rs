use lattice::document::markdown_body;

// =============================================================================
// Line Counting Tests
// =============================================================================

#[test]
fn count_lines_empty_body() {
    assert_eq!(markdown_body::count_lines(""), 0, "Empty body should have 0 lines");
}

#[test]
fn count_lines_single_line() {
    assert_eq!(markdown_body::count_lines("Hello"), 1, "Single line without newline");
    assert_eq!(markdown_body::count_lines("Hello\n"), 1, "Single line with newline");
}

#[test]
fn count_lines_multiple_lines() {
    let body = "Line 1\nLine 2\nLine 3";
    assert_eq!(markdown_body::count_lines(body), 3, "Three lines without trailing newline");

    let body_with_trailing = "Line 1\nLine 2\nLine 3\n";
    assert_eq!(
        markdown_body::count_lines(body_with_trailing),
        3,
        "Three lines with trailing newline"
    );
}

#[test]
fn count_lines_with_empty_lines() {
    let body = "Line 1\n\nLine 3\n\n";
    assert_eq!(markdown_body::count_lines(body), 4, "Lines including empty ones");
}

// =============================================================================
// Soft Limit Tests
// =============================================================================

#[test]
fn exceeds_soft_limit_under_limit() {
    let body = "line\n".repeat(500);
    assert!(!markdown_body::exceeds_soft_limit(&body), "500 lines should not exceed soft limit");
}

#[test]
fn exceeds_soft_limit_over_limit() {
    let body = "line\n".repeat(501);
    assert!(markdown_body::exceeds_soft_limit(&body), "501 lines should exceed soft limit");
}

// =============================================================================
// Body Statistics Tests
// =============================================================================

#[test]
fn compute_stats_empty_body() {
    let stats = markdown_body::compute_stats("");
    assert_eq!(stats.line_count, 0);
    assert_eq!(stats.non_empty_lines, 0);
    assert_eq!(stats.top_level_headers, 0);
}

#[test]
fn compute_stats_with_content() {
    let body = "# Header 1\n\nSome content.\n\n# Header 2\n\nMore content.";
    let stats = markdown_body::compute_stats(body);
    assert_eq!(stats.line_count, 7);
    assert_eq!(stats.non_empty_lines, 4);
    assert_eq!(stats.top_level_headers, 2);
}

#[test]
fn compute_stats_no_headers() {
    let body = "Just some text\nwith multiple lines\nbut no headers.";
    let stats = markdown_body::compute_stats(body);
    assert_eq!(stats.line_count, 3);
    assert_eq!(stats.non_empty_lines, 3);
    assert_eq!(stats.top_level_headers, 0);
}

// =============================================================================
// Append Content Tests
// =============================================================================

#[test]
fn append_content_to_empty() {
    let result = markdown_body::append_content("", "New content");
    assert_eq!(result, "New content");
}

#[test]
fn append_content_to_existing() {
    let result = markdown_body::append_content("Existing", "New");
    assert_eq!(result, "Existing\n\nNew");
}

#[test]
fn append_content_trims_trailing_whitespace() {
    let result = markdown_body::append_content("Existing\n\n\n", "New");
    assert_eq!(result, "Existing\n\nNew");
}

#[test]
fn append_content_whitespace_only_body() {
    let result = markdown_body::append_content("   \n\t\n  ", "New");
    assert_eq!(result, "New");
}

// =============================================================================
// Prepend Content Tests
// =============================================================================

#[test]
fn prepend_content_to_empty() {
    let result = markdown_body::prepend_content("", "New content");
    assert_eq!(result, "New content");
}

#[test]
fn prepend_content_to_existing() {
    let result = markdown_body::prepend_content("Existing", "New");
    assert_eq!(result, "New\n\nExisting");
}

#[test]
fn prepend_content_trims_leading_whitespace() {
    let result = markdown_body::prepend_content("\n\n\nExisting", "New");
    assert_eq!(result, "New\n\nExisting");
}

// =============================================================================
// Extract Sections Tests
// =============================================================================

#[test]
fn extract_sections_no_headers() {
    let body = "Some content without headers.";
    let sections = markdown_body::extract_sections(body);
    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].0, "");
    assert_eq!(sections[0].1, "Some content without headers.");
}

#[test]
fn extract_sections_single_header() {
    let body = "# Header\n\nContent under header.";
    let sections = markdown_body::extract_sections(body);
    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].0, "Header");
    assert!(sections[0].1.contains("# Header"));
    assert!(sections[0].1.contains("Content under header."));
}

#[test]
fn extract_sections_multiple_headers() {
    let body = "# First\n\nFirst content.\n\n# Second\n\nSecond content.";
    let sections = markdown_body::extract_sections(body);
    assert_eq!(sections.len(), 2);
    assert_eq!(sections[0].0, "First");
    assert_eq!(sections[1].0, "Second");
}

#[test]
fn extract_sections_content_before_first_header() {
    let body = "Preamble content.\n\n# Header\n\nHeader content.";
    let sections = markdown_body::extract_sections(body);
    assert_eq!(sections.len(), 2);
    assert_eq!(sections[0].0, "");
    assert!(sections[0].1.contains("Preamble content."));
    assert_eq!(sections[1].0, "Header");
}

// =============================================================================
// Find Body Start Tests
// =============================================================================

#[test]
fn find_body_start_valid_frontmatter() {
    let content = "---\nkey: value\n---\n\nBody content";
    let start = markdown_body::find_body_start(content);
    assert!(start.is_some());
    let body = &content[start.unwrap()..];
    assert_eq!(body, "Body content");
}

#[test]
fn find_body_start_with_bom() {
    let content = "\u{feff}---\nkey: value\n---\n\nBody";
    let start = markdown_body::find_body_start(content);
    assert!(start.is_some(), "Should find body start with BOM");
    // Verify the returned offset works correctly with the original content
    let body = &content[start.unwrap()..];
    assert_eq!(body, "Body", "Offset should point to body content after BOM");
}

#[test]
fn find_body_start_no_frontmatter() {
    let content = "Just regular content";
    let start = markdown_body::find_body_start(content);
    assert!(start.is_none());
}

#[test]
fn find_body_start_unclosed_frontmatter() {
    let content = "---\nkey: value\nNo closing delimiter";
    let start = markdown_body::find_body_start(content);
    assert!(start.is_none());
}

// =============================================================================
// Lattice Section Tests
// =============================================================================

#[test]
fn is_lattice_section_context() {
    assert!(markdown_body::is_lattice_section("## [Lattice] Context"));
    assert!(markdown_body::is_lattice_section("# [Lattice] Context"));
}

#[test]
fn is_lattice_section_acceptance() {
    assert!(markdown_body::is_lattice_section("## [Lattice] Acceptance Criteria"));
}

#[test]
fn is_lattice_section_false_positives() {
    assert!(!markdown_body::is_lattice_section("[Lattice] without header"));
    assert!(!markdown_body::is_lattice_section("Regular header"));
    assert!(!markdown_body::is_lattice_section("## Regular header"));
}

#[test]
fn extract_lattice_sections_finds_both() {
    let body =
        "# [Lattice] Context\n\nContext content.\n\n# [Lattice] Acceptance Criteria\n\nCriteria.";
    let (context, acceptance) = markdown_body::extract_lattice_sections(body);
    assert!(context.is_some());
    assert!(acceptance.is_some());
    assert!(context.unwrap().contains("Context content."));
    assert!(acceptance.unwrap().contains("Criteria."));
}

#[test]
fn extract_lattice_sections_none_present() {
    let body = "# Regular Header\n\nRegular content.";
    let (context, acceptance) = markdown_body::extract_lattice_sections(body);
    assert!(context.is_none());
    assert!(acceptance.is_none());
}

#[test]
fn strip_lattice_sections_removes_both() {
    let body =
        "# Keep This\n\nContent.\n\n# [Lattice] Context\n\nRemove.\n\n# Also Keep\n\nMore content.";
    let stripped = markdown_body::strip_lattice_sections(body);
    assert!(stripped.contains("# Keep This"));
    assert!(stripped.contains("# Also Keep"));
    assert!(!stripped.contains("[Lattice] Context"));
}

#[test]
fn strip_lattice_sections_preserves_other_content() {
    let body = "# Regular\n\nContent only.";
    let stripped = markdown_body::strip_lattice_sections(body);
    assert!(stripped.contains("# Regular"));
    assert!(stripped.contains("Content only."));
}
