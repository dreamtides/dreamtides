use std::fs;
use std::io::Write as IoWrite;

use chrono::Utc;
use lattice::document::document_writer::WriteOptions;
use lattice::document::frontmatter_schema::{Frontmatter, TaskType};
use lattice::document::{document_reader, document_writer};
use lattice::error::error_types::LatticeError;
use lattice::id::lattice_id::LatticeId;
use tempfile::TempDir;

fn create_test_frontmatter() -> Frontmatter {
    Frontmatter {
        lattice_id: "LABCDT".parse::<LatticeId>().unwrap(),
        name: "test-doc".to_string(),
        description: "Test document".to_string(),
        parent_id: None,
        task_type: None,
        priority: None,
        labels: Vec::new(),
        blocking: Vec::new(),
        blocked_by: Vec::new(),
        discovered_from: Vec::new(),
        created_at: None,
        updated_at: None,
        closed_at: None,
        skill: false,
    }
}

fn create_task_frontmatter() -> Frontmatter {
    Frontmatter {
        lattice_id: "LABCDT".parse::<LatticeId>().unwrap(),
        name: "test-task".to_string(),
        description: "Test task".to_string(),
        parent_id: None,
        task_type: Some(TaskType::Bug),
        priority: Some(1),
        labels: vec!["urgent".to_string()],
        blocking: Vec::new(),
        blocked_by: Vec::new(),
        discovered_from: Vec::new(),
        created_at: None,
        updated_at: None,
        closed_at: None,
        skill: false,
    }
}

fn create_temp_document(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    let mut file = fs::File::create(&path).expect("Failed to create temp file");
    file.write_all(content.as_bytes()).expect("Failed to write temp file");
    path
}

// =============================================================================
// Basic Writing Tests
// =============================================================================

#[test]
fn write_new_creates_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("new_doc.md");
    let frontmatter = create_test_frontmatter();

    document_writer::write_new(&frontmatter, "", &path, &WriteOptions::default())
        .expect("Should write new document");

    assert!(path.exists(), "File should be created");
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("lattice-id: LABCDT"), "Content should have lattice ID");
    assert!(content.contains("name: test-doc"), "Content should have name");
}

#[test]
fn write_new_with_body() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("doc_with_body.md");
    let frontmatter = create_test_frontmatter();
    let body = "# Header\n\nBody content here.";

    document_writer::write_new(&frontmatter, body, &path, &WriteOptions::default())
        .expect("Should write document with body");

    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("# Header"), "Content should have body header");
    assert!(content.contains("Body content here."), "Content should have body text");
}

#[test]
fn write_task_document() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("task.md");
    let frontmatter = create_task_frontmatter();

    document_writer::write_new(&frontmatter, "", &path, &WriteOptions::default())
        .expect("Should write task document");

    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("task-type: bug"), "Content should have task type");
    assert!(content.contains("priority: 1"), "Content should have priority");
    assert!(content.contains("- urgent"), "Content should have labels");
}

// =============================================================================
// Atomic Write Tests
// =============================================================================

#[test]
fn write_is_atomic() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("atomic_test.md");
    let frontmatter = create_test_frontmatter();

    document_writer::write_new(&frontmatter, "Body", &path, &WriteOptions::default())
        .expect("Should write atomically");

    let doc = document_reader::read(&path).expect("Should be able to read back");
    assert_eq!(doc.frontmatter.lattice_id.as_str(), "LABCDT");
    assert!(doc.body.contains("Body"));
}

#[test]
fn write_overwrites_existing() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("overwrite.md");

    let mut frontmatter = create_test_frontmatter();
    document_writer::write_new(&frontmatter, "Original body", &path, &WriteOptions::default())
        .unwrap();

    frontmatter.description = "Updated description".to_string();
    document_writer::write_new(&frontmatter, "New body", &path, &WriteOptions::default()).unwrap();

    let doc = document_reader::read(&path).unwrap();
    assert_eq!(doc.frontmatter.description, "Updated description");
    assert!(doc.body.contains("New body"));
    assert!(!doc.body.contains("Original body"));
}

// =============================================================================
// Timestamp Update Tests
// =============================================================================

#[test]
fn write_without_timestamp_update() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("no_timestamp.md");
    let frontmatter = create_test_frontmatter();

    document_writer::write_new(&frontmatter, "", &path, &WriteOptions::default()).unwrap();

    let doc = document_reader::read(&path).unwrap();
    assert!(doc.frontmatter.updated_at.is_none(), "Timestamp should not be set");
}

#[test]
fn write_with_timestamp_update() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("with_timestamp.md");
    let frontmatter = create_test_frontmatter();
    let before = Utc::now();

    document_writer::write_new(&frontmatter, "", &path, &WriteOptions::with_timestamp()).unwrap();

    let doc = document_reader::read(&path).unwrap();
    assert!(doc.frontmatter.updated_at.is_some(), "Timestamp should be set");
    let timestamp = doc.frontmatter.updated_at.unwrap();
    assert!(timestamp >= before, "Timestamp should be recent");
}

// =============================================================================
// Create Parents Tests
// =============================================================================

#[test]
fn write_creates_parent_directories() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("nested").join("path").join("doc.md");
    let frontmatter = create_test_frontmatter();

    document_writer::write_new(&frontmatter, "", &path, &WriteOptions::with_parents())
        .expect("Should create parent directories");

    assert!(path.exists());
}

#[test]
fn write_fails_without_parent_creation() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("nonexistent").join("doc.md");
    let frontmatter = create_test_frontmatter();

    let err =
        document_writer::write_new(&frontmatter, "", &path, &WriteOptions::default()).unwrap_err();

    match err {
        LatticeError::WriteError { .. } => {}
        _ => panic!("Expected WriteError, got: {err:?}"),
    }
}

// =============================================================================
// Write Document Tests
// =============================================================================

#[test]
fn write_existing_document() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("doc.md");

    let frontmatter = create_test_frontmatter();
    document_writer::write_new(&frontmatter, "Original", &path, &WriteOptions::default()).unwrap();

    let mut doc = document_reader::read(&path).unwrap();
    doc.frontmatter.description = "Modified description".to_string();

    document_writer::write(&doc, &path, &WriteOptions::default()).unwrap();

    let reread = document_reader::read(&path).unwrap();
    assert_eq!(reread.frontmatter.description, "Modified description");
}

// =============================================================================
// Update Frontmatter Tests
// =============================================================================

#[test]
fn update_frontmatter_preserves_body() {
    let dir = TempDir::new().unwrap();
    let original_content = r#"---
lattice-id: LABCDT
name: test-doc
description: Original description
---

# Body Content

This should be preserved.
"#;
    let path = create_temp_document(&dir, "test-doc.md", original_content);

    let mut frontmatter = create_test_frontmatter();
    frontmatter.description = "Updated description".to_string();

    document_writer::update_frontmatter(&path, &frontmatter, &WriteOptions::default()).unwrap();

    let doc = document_reader::read(&path).unwrap();
    assert_eq!(doc.frontmatter.description, "Updated description");
    assert!(doc.body.contains("# Body Content"), "Body should be preserved");
    assert!(doc.body.contains("This should be preserved."), "Body content should be preserved");
}

// =============================================================================
// Update Body Tests
// =============================================================================

#[test]
fn update_body_preserves_frontmatter() {
    let dir = TempDir::new().unwrap();
    let original_content = r#"---
lattice-id: LABCDT
name: test-doc
description: Test description
labels:
  - important
---

Original body.
"#;
    let path = create_temp_document(&dir, "test-doc.md", original_content);

    document_writer::update_body(&path, "New body content.", &WriteOptions::default()).unwrap();

    let doc = document_reader::read(&path).unwrap();
    assert_eq!(doc.frontmatter.description, "Test description");
    assert_eq!(doc.frontmatter.labels, vec!["important"]);
    assert!(doc.body.contains("New body content."));
    assert!(!doc.body.contains("Original body."));
}

// =============================================================================
// Write Raw Tests
// =============================================================================

#[test]
fn write_raw_content() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("raw.md");
    let content = "Raw content without formatting.";

    document_writer::write_raw(&path, content, &WriteOptions::default()).unwrap();

    let read_content = fs::read_to_string(&path).unwrap();
    assert_eq!(read_content, content);
}

// =============================================================================
// WriteOptions Tests
// =============================================================================

#[test]
fn write_options_default() {
    let options = WriteOptions::default();
    assert!(!options.update_timestamp);
    assert!(!options.create_parents);
}

#[test]
fn write_options_with_timestamp() {
    let options = WriteOptions::with_timestamp();
    assert!(options.update_timestamp);
    assert!(!options.create_parents);
}

#[test]
fn write_options_with_parents() {
    let options = WriteOptions::with_parents();
    assert!(!options.update_timestamp);
    assert!(options.create_parents);
}

#[test]
fn write_options_with_timestamp_and_parents() {
    let options = WriteOptions::with_timestamp_and_parents();
    assert!(options.update_timestamp);
    assert!(options.create_parents);
}

// =============================================================================
// Round-Trip Tests
// =============================================================================

#[test]
fn round_trip_preserves_content() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("roundtrip.md");

    let original_frontmatter = create_task_frontmatter();
    let original_body = "# Test\n\nBody with *markdown* content.\n\n## Section\n\nMore content.";

    document_writer::write_new(
        &original_frontmatter,
        original_body,
        &path,
        &WriteOptions::default(),
    )
    .unwrap();

    let doc = document_reader::read(&path).unwrap();

    assert_eq!(doc.frontmatter.lattice_id, original_frontmatter.lattice_id);
    assert_eq!(doc.frontmatter.name, original_frontmatter.name);
    assert_eq!(doc.frontmatter.description, original_frontmatter.description);
    assert_eq!(doc.frontmatter.task_type, original_frontmatter.task_type);
    assert_eq!(doc.frontmatter.priority, original_frontmatter.priority);
    assert_eq!(doc.frontmatter.labels, original_frontmatter.labels);
    assert!(doc.body.contains("# Test"));
    assert!(doc.body.contains("*markdown*"));
}
