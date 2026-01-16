use std::path::Path;

use lattice::document::frontmatter_parser;
use lattice::error::error_types::LatticeError;

fn test_path() -> &'static Path {
    Path::new("test/doc.md")
}

// =============================================================================
// Successful Parsing Tests
// =============================================================================

#[test]
fn parses_minimal_frontmatter() {
    let content = r#"---
lattice-id: LABCDT
name: test-doc
description: Test document
---
"#;

    let result = frontmatter_parser::parse(content, test_path());
    let parsed = result.expect("Should parse minimal frontmatter");

    assert_eq!(parsed.frontmatter.lattice_id.as_str(), "LABCDT");
    assert_eq!(parsed.frontmatter.name, "test-doc");
    assert_eq!(parsed.frontmatter.description, "Test document");
}

#[test]
fn parses_frontmatter_with_body() {
    let content = r#"---
lattice-id: LABCDT
name: test-doc
description: Test document
---

# Document Body

This is the body content.
"#;

    let result = frontmatter_parser::parse(content, test_path());
    let parsed = result.expect("Should parse frontmatter with body");

    assert_eq!(parsed.frontmatter.lattice_id.as_str(), "LABCDT");
    assert!(parsed.body.contains("# Document Body"), "Body should contain header");
    assert!(parsed.body.contains("This is the body content."), "Body should contain content");
}

#[test]
fn preserves_raw_yaml_for_round_tripping() {
    let content = r#"---
lattice-id: LABCDT
name: test-doc
description: Test document
---
"#;

    let result = frontmatter_parser::parse(content, test_path());
    let parsed = result.expect("Should parse");

    assert!(parsed.raw_yaml.contains("lattice-id: LABCDT"), "Raw YAML should be preserved");
}

#[test]
fn handles_bom_at_start() {
    let content = "\u{feff}---\nlattice-id: LABCDT\nname: test\ndescription: Test\n---\n";

    let result = frontmatter_parser::parse(content, test_path());
    assert!(result.is_ok(), "Should handle UTF-8 BOM at start");
}

#[test]
fn handles_crlf_line_endings() {
    let content = "---\r\nlattice-id: LABCDT\r\nname: test\r\ndescription: Test\r\n---\r\n";

    let result = frontmatter_parser::parse(content, test_path());
    assert!(result.is_ok(), "Should handle CRLF line endings");
}

// =============================================================================
// Error: Missing Delimiters
// =============================================================================

#[test]
fn rejects_missing_opening_delimiter() {
    let content = r#"lattice-id: LABCDT
name: test-doc
description: Test document
---
"#;

    let result = frontmatter_parser::parse(content, test_path());
    let err = result.expect_err("Should reject missing opening delimiter");

    match err {
        LatticeError::InvalidFrontmatter { reason, .. } => {
            assert!(
                reason.contains("start with '---'"),
                "Error should mention missing opening delimiter: {reason}"
            );
        }
        _ => panic!("Expected InvalidFrontmatter error, got: {err:?}"),
    }
}

#[test]
fn rejects_missing_closing_delimiter() {
    let content = r#"---
lattice-id: LABCDT
name: test-doc
description: Test document

# Body without closing delimiter
"#;

    let result = frontmatter_parser::parse(content, test_path());
    let err = result.expect_err("Should reject missing closing delimiter");

    match err {
        LatticeError::InvalidFrontmatter { reason, .. } => {
            assert!(
                reason.contains("missing closing"),
                "Error should mention missing closing delimiter: {reason}"
            );
        }
        _ => panic!("Expected InvalidFrontmatter error, got: {err:?}"),
    }
}

// =============================================================================
// Error: Empty Frontmatter
// =============================================================================

#[test]
fn rejects_empty_frontmatter() {
    let content = "---\n---\n";

    let result = frontmatter_parser::parse(content, test_path());
    let err = result.expect_err("Should reject empty frontmatter");

    match err {
        LatticeError::InvalidFrontmatter { reason, .. } => {
            assert!(reason.contains("cannot be empty"), "Error should mention empty: {reason}");
        }
        _ => panic!("Expected InvalidFrontmatter error, got: {err:?}"),
    }
}

#[test]
fn rejects_whitespace_only_frontmatter() {
    let content = "---\n   \n\t\n---\n";

    let result = frontmatter_parser::parse(content, test_path());
    let err = result.expect_err("Should reject whitespace-only frontmatter");

    match err {
        LatticeError::InvalidFrontmatter { reason, .. } => {
            assert!(reason.contains("cannot be empty"), "Error should mention empty: {reason}");
        }
        _ => panic!("Expected InvalidFrontmatter error, got: {err:?}"),
    }
}

// =============================================================================
// Error: Malformed YAML
// =============================================================================

#[test]
fn rejects_invalid_yaml_syntax() {
    let content = r#"---
lattice-id: LABCDT
name: [unclosed bracket
description: Test
---
"#;

    let result = frontmatter_parser::parse(content, test_path());
    let err = result.expect_err("Should reject invalid YAML");

    match err {
        LatticeError::YamlParseError { reason, .. } => {
            assert!(!reason.is_empty(), "Should have error reason");
        }
        _ => panic!("Expected YamlParseError, got: {err:?}"),
    }
}

#[test]
fn rejects_non_mapping_frontmatter() {
    let content = r#"---
- item1
- item2
---
"#;

    let result = frontmatter_parser::parse_with_unknown_key_detection(content, test_path());
    let err = result.expect_err("Should reject list-based frontmatter");

    match err {
        LatticeError::InvalidFrontmatter { reason, .. } => {
            assert!(reason.contains("mapping"), "Error should mention mapping: {reason}");
        }
        _ => panic!("Expected InvalidFrontmatter error, got: {err:?}"),
    }
}

// =============================================================================
// Unknown Key Detection
// =============================================================================

#[test]
fn detects_unknown_keys() {
    let content = r#"---
lattice-id: LABCDT
name: test-doc
description: Test
unknown-field: value
---
"#;

    let result = frontmatter_parser::parse_with_unknown_key_detection(content, test_path());
    let (_, unknown_keys) = result.expect("Should parse despite unknown key");

    assert_eq!(unknown_keys.len(), 1, "Should find one unknown key");
    assert_eq!(unknown_keys[0].key, "unknown-field");
}

#[test]
fn suggests_correction_for_typo() {
    let content = r#"---
lattice-id: LABCDT
name: test-doc
description: Test
priorty: 1
---
"#;

    let result = frontmatter_parser::parse_with_unknown_key_detection(content, test_path());
    let (_, unknown_keys) = result.expect("Should parse despite typo");

    assert_eq!(unknown_keys.len(), 1, "Should find typo key");
    assert_eq!(unknown_keys[0].key, "priorty");
    assert_eq!(
        unknown_keys[0].suggestion,
        Some("priority".to_string()),
        "Should suggest 'priority'"
    );
}

#[test]
fn suggests_correction_for_task_type_typo() {
    let content = r#"---
lattice-id: LABCDT
name: test-doc
description: Test
taks-type: bug
---
"#;

    let result = frontmatter_parser::parse_with_unknown_key_detection(content, test_path());
    let (_, unknown_keys) = result.expect("Should parse despite typo");

    assert_eq!(unknown_keys.len(), 1);
    assert_eq!(unknown_keys[0].key, "taks-type");
    assert_eq!(unknown_keys[0].suggestion, Some("task-type".to_string()));
}

#[test]
fn no_suggestion_for_completely_different_key() {
    let content = r#"---
lattice-id: LABCDT
name: test-doc
description: Test
completely-random: value
---
"#;

    let result = frontmatter_parser::parse_with_unknown_key_detection(content, test_path());
    let (_, unknown_keys) = result.expect("Should parse");

    assert_eq!(unknown_keys.len(), 1);
    assert_eq!(unknown_keys[0].key, "completely-random");
    assert!(unknown_keys[0].suggestion.is_none(), "Should not suggest for unrelated keys");
}

// =============================================================================
// Serialization Tests
// =============================================================================

#[test]
fn serializes_frontmatter_to_yaml() {
    let content = r#"---
lattice-id: LABCDT
name: test-doc
description: Test document
---
"#;

    let parsed = frontmatter_parser::parse(content, test_path()).expect("Should parse");
    let serialized = frontmatter_parser::serialize(&parsed.frontmatter).expect("Should serialize");

    assert!(serialized.contains("lattice-id: LABCDT"), "Should contain lattice ID");
    assert!(serialized.contains("name: test-doc"), "Should contain name");
}

// =============================================================================
// Document Formatting Tests
// =============================================================================

#[test]
fn formats_document_with_empty_body() {
    let content = r#"---
lattice-id: LABCDT
name: test-doc
description: Test document
---
"#;

    let parsed = frontmatter_parser::parse(content, test_path()).expect("Should parse");
    let formatted =
        frontmatter_parser::format_document(&parsed.frontmatter, "").expect("Should format");

    assert!(formatted.starts_with("---\n"), "Should start with delimiter");
    assert!(formatted.ends_with("---\n"), "Should end with delimiter and newline");
}

#[test]
fn formats_document_with_body() {
    let content = r#"---
lattice-id: LABCDT
name: test-doc
description: Test document
---

# Body
"#;

    let parsed = frontmatter_parser::parse(content, test_path()).expect("Should parse");
    let body = "# New Body\n\nContent here.";
    let formatted =
        frontmatter_parser::format_document(&parsed.frontmatter, body).expect("Should format");

    assert!(formatted.contains("---\n"), "Should have delimiters");
    assert!(formatted.contains("# New Body"), "Should contain body");
    assert!(formatted.contains("Content here."), "Should contain body content");
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn handles_triple_dashes_in_body() {
    let content = r#"---
lattice-id: LABCDT
name: test-doc
description: Test document
---

# Code Example

```yaml
---
key: value
---
```
"#;

    let result = frontmatter_parser::parse(content, test_path());
    let parsed = result.expect("Should parse document with dashes in body");

    assert!(parsed.body.contains("```yaml"), "Body should contain code block");
    assert!(parsed.body.contains("key: value"), "Body should contain YAML in code block");
}

#[test]
fn handles_delimiter_at_end_of_file_without_newline() {
    let content = "---\nlattice-id: LABCDT\nname: test\ndescription: Test\n---";

    let result = frontmatter_parser::parse(content, test_path());
    assert!(result.is_ok(), "Should handle delimiter at EOF without newline");
}
