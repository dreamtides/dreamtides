use std::fs;
use std::io::Write;
use std::path::Path;

use lattice::document::document_reader;
use lattice::error::error_types::LatticeError;
use tempfile::TempDir;

fn create_temp_document(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    let mut file = fs::File::create(&path).expect("Failed to create temp file");
    file.write_all(content.as_bytes()).expect("Failed to write temp file");
    path
}

fn minimal_document_content() -> &'static str {
    r#"---
lattice-id: LABCDT
name: test-doc
description: Test document
---
"#
}

fn document_with_body() -> &'static str {
    r#"---
lattice-id: LABCDT
name: test-doc
description: Test document
---

# Body Header

This is the body content.
"#
}

fn task_document_content() -> &'static str {
    r#"---
lattice-id: LABCDT
name: test-task
description: Test task
task-type: bug
priority: 1
---

# Bug Description

Details here.
"#
}

// =============================================================================
// Basic Reading Tests
// =============================================================================

#[test]
fn read_minimal_document() {
    let dir = TempDir::new().unwrap();
    let path = create_temp_document(&dir, "test_doc.md", minimal_document_content());

    let doc = document_reader::read(&path).expect("Should read document");

    assert_eq!(doc.frontmatter.lattice_id.as_str(), "LABCDT");
    assert_eq!(doc.frontmatter.name, "test-doc");
    assert_eq!(doc.frontmatter.description, "Test document");
    assert!(doc.body.is_empty(), "Minimal document should have empty body");
}

#[test]
fn read_document_with_body() {
    let dir = TempDir::new().unwrap();
    let path = create_temp_document(&dir, "test_doc.md", document_with_body());

    let doc = document_reader::read(&path).expect("Should read document");

    assert!(doc.body.contains("# Body Header"), "Body should contain header");
    assert!(doc.body.contains("body content"), "Body should contain content");
}

#[test]
fn read_task_document() {
    let dir = TempDir::new().unwrap();
    let path = create_temp_document(&dir, "test_task.md", task_document_content());

    let doc = document_reader::read(&path).expect("Should read task document");

    assert!(doc.is_task(), "Document should be a task");
    assert_eq!(doc.frontmatter.task_type.map(|t| t.to_string()), Some("bug".to_string()));
    assert_eq!(doc.frontmatter.priority, Some(1));
}

#[test]
fn read_knowledge_base_document() {
    let dir = TempDir::new().unwrap();
    let path = create_temp_document(&dir, "test_doc.md", minimal_document_content());

    let doc = document_reader::read(&path).expect("Should read KB document");

    assert!(doc.is_knowledge_base(), "Document should be a knowledge base document");
    assert!(!doc.is_task());
}

// =============================================================================
// Encoding Tests
// =============================================================================

#[test]
fn read_document_with_bom() {
    let dir = TempDir::new().unwrap();
    let content = format!("\u{feff}{}", minimal_document_content());
    let path = create_temp_document(&dir, "bom_doc.md", &content);

    let doc = document_reader::read(&path).expect("Should handle BOM");
    assert_eq!(doc.frontmatter.lattice_id.as_str(), "LABCDT");
}

#[test]
fn read_document_with_crlf() {
    let dir = TempDir::new().unwrap();
    let content = minimal_document_content().replace('\n', "\r\n");
    let path = create_temp_document(&dir, "crlf_doc.md", &content);

    let doc = document_reader::read(&path).expect("Should handle CRLF");
    assert_eq!(doc.frontmatter.lattice_id.as_str(), "LABCDT");
}

// =============================================================================
// Error Cases
// =============================================================================

#[test]
fn read_nonexistent_file() {
    let path = Path::new("/nonexistent/path/to/document.md");

    let err = document_reader::read(path).expect_err("Should fail for missing file");

    match err {
        LatticeError::FileNotFound { .. } => {}
        _ => panic!("Expected FileNotFound error, got: {err:?}"),
    }
}

#[test]
fn read_invalid_frontmatter() {
    let dir = TempDir::new().unwrap();
    let content = "Not a valid document";
    let path = create_temp_document(&dir, "invalid.md", content);

    let err = document_reader::read(&path).expect_err("Should fail for invalid frontmatter");

    match err {
        LatticeError::InvalidFrontmatter { .. } => {}
        _ => panic!("Expected InvalidFrontmatter error, got: {err:?}"),
    }
}

#[test]
fn read_malformed_yaml() {
    let dir = TempDir::new().unwrap();
    let content = r#"---
lattice-id: [unclosed
name: test
---
"#;
    let path = create_temp_document(&dir, "malformed.md", content);

    let err = document_reader::read(&path).expect_err("Should fail for malformed YAML");

    match err {
        LatticeError::YamlParseError { .. } => {}
        _ => panic!("Expected YamlParseError, got: {err:?}"),
    }
}

// =============================================================================
// Validation Tests
// =============================================================================

#[test]
fn read_and_validate_clean_document() {
    let dir = TempDir::new().unwrap();
    let path = create_temp_document(&dir, "test-doc.md", minimal_document_content());

    let result = document_reader::read_and_validate(&path).expect("Should read document");

    assert!(result.is_clean(), "Clean document should have no issues");
    assert!(result.unknown_keys.is_empty());
    assert!(result.field_errors.is_empty());
}

#[test]
fn read_and_validate_detects_unknown_keys() {
    let dir = TempDir::new().unwrap();
    let content = r#"---
lattice-id: LABCDT
name: test-doc
description: Test
unknown-key: value
---
"#;
    let path = create_temp_document(&dir, "test-doc.md", content);

    let result = document_reader::read_and_validate(&path).expect("Should read document");

    assert!(!result.is_clean(), "Document with unknown keys should not be clean");
    assert_eq!(result.unknown_keys.len(), 1);
    assert_eq!(result.unknown_keys[0].key, "unknown-key");
}

#[test]
fn read_and_validate_detects_field_errors() {
    let dir = TempDir::new().unwrap();
    let content = r#"---
lattice-id: LABCDT
name: INVALID_UPPERCASE
description: Test
---
"#;
    let path = create_temp_document(&dir, "test-doc.md", content);

    let result = document_reader::read_and_validate(&path).expect("Should read document");

    assert!(!result.is_clean(), "Document with invalid name should not be clean");
    assert!(!result.field_errors.is_empty(), "Should have field errors");
}

// =============================================================================
// Is Lattice Document Tests
// =============================================================================

#[test]
fn is_lattice_document_true_for_valid() {
    let dir = TempDir::new().unwrap();
    let path = create_temp_document(&dir, "doc.md", minimal_document_content());

    assert!(document_reader::is_lattice_document(&path).unwrap());
}

#[test]
fn is_lattice_document_false_for_regular_markdown() {
    let dir = TempDir::new().unwrap();
    let content = "# Regular Markdown\n\nNo frontmatter here.";
    let path = create_temp_document(&dir, "regular.md", content);

    assert!(!document_reader::is_lattice_document(&path).unwrap());
}

#[test]
fn is_lattice_document_true_with_bom() {
    let dir = TempDir::new().unwrap();
    let content = format!("\u{feff}{}", minimal_document_content());
    let path = create_temp_document(&dir, "bom.md", &content);

    assert!(document_reader::is_lattice_document(&path).unwrap());
}

#[test]
fn content_is_lattice_document_checks_content() {
    assert!(document_reader::content_is_lattice_document("---\nkey: value\n---\n"));
    assert!(document_reader::content_is_lattice_document("\u{feff}---\nkey: value\n---\n"));
    assert!(!document_reader::content_is_lattice_document("Regular content"));
    assert!(!document_reader::content_is_lattice_document(""));
}

// =============================================================================
// Document Methods Tests
// =============================================================================

#[test]
fn document_id_str() {
    let dir = TempDir::new().unwrap();
    let path = create_temp_document(&dir, "doc.md", minimal_document_content());

    let doc = document_reader::read(&path).unwrap();
    assert_eq!(doc.id_str(), "LABCDT");
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn read_document_with_empty_body_after_frontmatter() {
    let dir = TempDir::new().unwrap();
    let content = "---\nlattice-id: LABCDT\nname: test\ndescription: Test\n---\n";
    let path = create_temp_document(&dir, "test.md", content);

    let doc = document_reader::read(&path).expect("Should read document");
    assert!(doc.body.is_empty(), "Body should be empty");
}

#[test]
fn read_document_with_complex_yaml() {
    let dir = TempDir::new().unwrap();
    let content = r#"---
lattice-id: LABCDT
name: test-doc
description: Test document with special chars
labels:
  - label-one
  - label-two
blocking:
  - L22222
blocked-by:
  - L33333
---

Body content.
"#;
    let path = create_temp_document(&dir, "test-doc.md", content);

    let doc = document_reader::read(&path).expect("Should read complex document");

    assert_eq!(doc.frontmatter.labels.len(), 2);
    assert_eq!(doc.frontmatter.blocking.len(), 1);
    assert_eq!(doc.frontmatter.blocked_by.len(), 1);
}
