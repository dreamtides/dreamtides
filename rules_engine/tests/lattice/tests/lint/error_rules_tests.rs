use std::fs;
use std::io::Write;

use lattice::document::frontmatter_schema::TaskType;
use lattice::index::document_types::InsertDocument;
use lattice::index::link_queries::{self, InsertLink, LinkType};
use lattice::index::{document_queries, schema_definition};
use lattice::lint::error_rules::{
    CircularBlockingRule, DuplicateIdRule, InvalidFieldValueRule, InvalidIdFormatRule,
    InvalidKeyRule, MissingDescriptionRule, MissingNameRule, MissingPriorityRule,
    MissingReferenceRule, NameMismatchRule, NestedClosedRule, NonTaskInClosedRule, all_error_rules,
};
use lattice::lint::rule_engine::{LintConfig, LintContext, LintRule, Severity, execute_rules};
use rusqlite::Connection;
use tempfile::TempDir;

fn create_test_db() -> Connection {
    let conn = Connection::open_in_memory().expect("Failed to create in-memory database");
    schema_definition::create_schema(&conn).expect("Failed to create schema");
    conn
}

fn create_kb_document(id: &str, path: &str, name: &str) -> InsertDocument {
    InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        name.to_string(),
        format!("Description for {name}"),
        None,
        None,
        None,
        None,
        None,
        "abc123".to_string(),
        100,
    )
}

fn create_task_document(id: &str, path: &str, name: &str, priority: u8) -> InsertDocument {
    InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        name.to_string(),
        format!("Task: {name}"),
        Some(TaskType::Task),
        Some(priority),
        None,
        None,
        None,
        "def456".to_string(),
        200,
    )
}

fn create_task_without_priority(id: &str, path: &str, name: &str) -> InsertDocument {
    InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        name.to_string(),
        format!("Task: {name}"),
        Some(TaskType::Task),
        None,
        None,
        None,
        None,
        "def456".to_string(),
        200,
    )
}

fn create_document_with_empty_name(id: &str, path: &str) -> InsertDocument {
    InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        "".to_string(),
        "Description".to_string(),
        None,
        None,
        None,
        None,
        None,
        "abc123".to_string(),
        100,
    )
}

fn create_document_with_empty_description(id: &str, path: &str, name: &str) -> InsertDocument {
    InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        name.to_string(),
        "".to_string(),
        None,
        None,
        None,
        None,
        None,
        "abc123".to_string(),
        100,
    )
}

// =============================================================================
// E001: Duplicate Lattice ID
// =============================================================================

#[test]
fn e001_duplicate_id_rule_has_correct_interface() {
    // The database has a UNIQUE constraint on IDs, so we can't test actual
    // duplicate detection. Instead, verify the rule's interface.
    let rule = DuplicateIdRule;
    assert_eq!(rule.codes(), &["E001"]);
    assert_eq!(rule.name(), "duplicate-id");
    assert!(!rule.requires_document_body());
}

#[test]
fn e001_duplicate_id_no_error_for_unique_ids() {
    let conn = create_test_db();
    let doc1 = create_kb_document("LUNIQ1", "api/api.md", "api");
    let doc2 = create_kb_document("LUNIQ2", "db/db.md", "db");
    document_queries::insert(&conn, &doc1).expect("Insert should succeed");
    document_queries::insert(&conn, &doc2).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = DuplicateIdRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(summary.results.iter().all(|r| r.code != "E001"), "Unique IDs should not trigger E001");
}

// =============================================================================
// E002: Missing Reference Target
// =============================================================================

#[test]
fn e002_missing_reference_detects_broken_links() {
    let conn = create_test_db();
    let doc1 = create_kb_document("LDOC01", "api/api.md", "api");
    document_queries::insert(&conn, &doc1).expect("Insert should succeed");

    // Insert a link to a non-existent target
    let links = vec![InsertLink {
        source_id: "LDOC01",
        target_id: "LMISSING",
        link_type: LinkType::Body,
        position: 0,
    }];
    link_queries::insert_for_document(&conn, &links).expect("Insert links should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = MissingReferenceRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.error_count, 1, "Should detect missing reference target");
    assert!(summary.results[0].message.contains("LMISSING"));
}

#[test]
fn e002_missing_reference_no_error_for_valid_links() {
    let conn = create_test_db();
    let doc1 = create_kb_document("LDOC01", "api/api.md", "api");
    let doc2 = create_kb_document("LDOC02", "db/db.md", "db");
    document_queries::insert(&conn, &doc1).expect("Insert should succeed");
    document_queries::insert(&conn, &doc2).expect("Insert should succeed");

    // Insert a valid link
    let links = vec![InsertLink {
        source_id: "LDOC01",
        target_id: "LDOC02",
        link_type: LinkType::Body,
        position: 0,
    }];
    link_queries::insert_for_document(&conn, &links).expect("Insert links should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = MissingReferenceRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "E002"),
        "Valid links should not trigger E002"
    );
}

// =============================================================================
// E003: Invalid Frontmatter Key
// =============================================================================

fn create_temp_document(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    let mut file = fs::File::create(&path).expect("Failed to create temp file");
    file.write_all(content.as_bytes()).expect("Failed to write temp file");
    path
}

#[test]
fn e003_invalid_key_rule_requires_body() {
    let rule = InvalidKeyRule;
    assert_eq!(rule.codes(), &["E003"]);
    assert_eq!(rule.name(), "invalid-key");
    assert!(rule.requires_document_body(), "InvalidKeyRule must require document body");
}

#[test]
fn e003_invalid_key_detects_unknown_key() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create file with an unknown frontmatter key
    // Note: ID must be valid Base32 (L + A-Z/2-7), not containing 0, 1, 8, 9
    let content = r#"---
lattice-id: LABCDT
name: test-doc
description: Test document
unknown-key: some value
---
"#;
    create_temp_document(&temp_dir, "test_doc.md", content);

    // Insert document into database so linter can find it
    let doc = create_kb_document("LABCDT", "test_doc.md", "test-doc");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = InvalidKeyRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.error_count, 1, "Should detect unknown frontmatter key");
    assert!(
        summary.results[0].message.contains("invalid frontmatter key 'unknown-key'"),
        "Message should mention the invalid key"
    );
}

#[test]
fn e003_invalid_key_provides_typo_suggestion() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create file with typo in key (priorit instead of priority)
    // Note: ID must be valid Base32 (L + A-Z/2-7), not containing 0, 1, 8, 9
    let content = r#"---
lattice-id: LTASKA
name: fix-bug
description: Fix a bug
task-type: task
priorit: 2
---
"#;
    create_temp_document(&temp_dir, "fix_bug.md", content);

    let doc = create_task_document("LTASKA", "fix_bug.md", "fix-bug", 2);
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = InvalidKeyRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.error_count, 1, "Should detect typo in frontmatter key");
    assert!(
        summary.results[0].message.contains("did you mean 'priority'"),
        "Message should suggest the correct key"
    );
}

#[test]
fn e003_invalid_key_no_error_for_valid_keys() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create file with all valid keys
    // Note: ID must be valid Base32 (L + A-Z/2-7), not containing 0, 1, 8, 9
    let content = r#"---
lattice-id: LDOCAA
name: test-doc
description: A test document
---
"#;
    create_temp_document(&temp_dir, "test_doc.md", content);

    let doc = create_kb_document("LDOCAA", "test_doc.md", "test-doc");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = InvalidKeyRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(summary.results.iter().all(|r| r.code != "E003"), "Valid keys should not trigger E003");
}

// =============================================================================
// E004: Missing Required Field (Priority)
// =============================================================================

#[test]
fn e004_missing_priority_detects_task_without_priority() {
    let conn = create_test_db();
    let doc = create_task_without_priority("LTASK1", "api/tasks/fix_bug.md", "fix-bug");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = MissingPriorityRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.error_count, 1, "Should detect missing priority");
    assert!(summary.results[0].message.contains("missing 'priority' field"));
}

#[test]
fn e004_missing_priority_no_error_for_kb_documents() {
    let conn = create_test_db();
    let doc = create_kb_document("LDOC01", "api/api.md", "api");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = MissingPriorityRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "E004"),
        "KB documents without priority should not trigger E004"
    );
}

#[test]
fn e004_missing_priority_no_error_for_task_with_priority() {
    let conn = create_test_db();
    let doc = create_task_document("LTASK1", "api/tasks/fix_bug.md", "fix-bug", 2);
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = MissingPriorityRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "E004"),
        "Task with priority should not trigger E004"
    );
}

// =============================================================================
// E005: Invalid Field Value
// =============================================================================

#[test]
fn e005_invalid_priority_detects_out_of_range() {
    let conn = create_test_db();
    // Create task with priority 5 (out of range 0-4)
    let doc = InsertDocument::new(
        "LTASK1".to_string(),
        None,
        "api/tasks/fix_bug.md".to_string(),
        "fix-bug".to_string(),
        "Task description".to_string(),
        Some(TaskType::Task),
        Some(5), // Invalid priority
        None,
        None,
        None,
        "abc123".to_string(),
        100,
    );
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = InvalidFieldValueRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.error_count, 1, "Should detect invalid priority");
    assert!(summary.results[0].message.contains("allowed: 0-4"));
}

#[test]
fn e005_invalid_priority_no_error_for_valid_priority() {
    let conn = create_test_db();
    let doc = create_task_document("LTASK1", "api/tasks/fix_bug.md", "fix-bug", 4);
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = InvalidFieldValueRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "E005"),
        "Valid priority should not trigger E005"
    );
}

// =============================================================================
// E006: Circular Blocking
// =============================================================================

#[test]
fn e006_circular_blocking_detects_cycle() {
    let conn = create_test_db();
    let doc1 = create_task_document("LDOCA1", "api/tasks/task_a.md", "task-a", 2);
    let doc2 = create_task_document("LDOCB1", "api/tasks/task_b.md", "task-b", 2);
    let doc3 = create_task_document("LDOCC1", "api/tasks/task_c.md", "task-c", 2);
    document_queries::insert(&conn, &doc1).expect("Insert should succeed");
    document_queries::insert(&conn, &doc2).expect("Insert should succeed");
    document_queries::insert(&conn, &doc3).expect("Insert should succeed");

    // Create a cycle: A blocks B, B blocks C, C blocks A
    let links = vec![
        InsertLink {
            source_id: "LDOCA1",
            target_id: "LDOCB1",
            link_type: LinkType::Blocking,
            position: 0,
        },
        InsertLink {
            source_id: "LDOCB1",
            target_id: "LDOCC1",
            link_type: LinkType::Blocking,
            position: 0,
        },
        InsertLink {
            source_id: "LDOCC1",
            target_id: "LDOCA1",
            link_type: LinkType::Blocking,
            position: 0,
        },
    ];
    for link in &links {
        link_queries::insert_for_document(&conn, &[link.clone()]).expect("Insert should succeed");
    }

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = CircularBlockingRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(summary.error_count >= 1, "Should detect circular dependency");
    assert!(summary.results.iter().any(|r| r.code == "E006"));
}

#[test]
fn e006_circular_blocking_no_error_for_linear_deps() {
    let conn = create_test_db();
    let doc1 = create_task_document("LDOCA2", "api/tasks/task_a.md", "task-a", 2);
    let doc2 = create_task_document("LDOCB2", "api/tasks/task_b.md", "task-b", 2);
    document_queries::insert(&conn, &doc1).expect("Insert should succeed");
    document_queries::insert(&conn, &doc2).expect("Insert should succeed");

    // Create a linear dependency: A blocks B (no cycle)
    let links = vec![InsertLink {
        source_id: "LDOCA2",
        target_id: "LDOCB2",
        link_type: LinkType::Blocking,
        position: 0,
    }];
    link_queries::insert_for_document(&conn, &links).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = CircularBlockingRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "E006"),
        "Linear dependencies should not trigger E006"
    );
}

// =============================================================================
// E007: Invalid ID Format
// =============================================================================

#[test]
fn e007_invalid_id_format_detects_malformed_id() {
    let conn = create_test_db();
    // Create document with malformed ID (too short)
    let doc = create_kb_document("L12", "api/api.md", "api");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = InvalidIdFormatRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.error_count, 1, "Should detect malformed ID");
    assert!(summary.results[0].message.contains("malformed lattice-id"));
}

#[test]
fn e007_invalid_id_format_no_error_for_valid_id() {
    let conn = create_test_db();
    let doc = create_kb_document("LVALID", "api/api.md", "api");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = InvalidIdFormatRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(summary.results.iter().all(|r| r.code != "E007"), "Valid ID should not trigger E007");
}

// =============================================================================
// E008: Name-Filename Mismatch
// =============================================================================

#[test]
fn e008_name_mismatch_detects_wrong_name() {
    let conn = create_test_db();
    // Document at fix_bug.md has name "wrong-name" instead of "fix-bug"
    let doc = create_kb_document("LDOC01", "api/docs/fix_bug.md", "wrong-name");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = NameMismatchRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.error_count, 1, "Should detect name-filename mismatch");
    assert!(summary.results[0].message.contains("should be 'fix-bug'"));
}

#[test]
fn e008_name_mismatch_no_error_for_matching_name() {
    let conn = create_test_db();
    let doc = create_kb_document("LDOC01", "api/docs/fix_bug.md", "fix-bug");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = NameMismatchRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "E008"),
        "Matching name should not trigger E008"
    );
}

// =============================================================================
// E009: Missing Required Field (Name)
// =============================================================================

#[test]
fn e009_missing_name_detects_empty_name() {
    let conn = create_test_db();
    let doc = create_document_with_empty_name("LDOC01", "api/api.md");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = MissingNameRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.error_count, 1, "Should detect missing name");
    assert!(summary.results[0].message.contains("missing required 'name' field"));
}

#[test]
fn e009_missing_name_no_error_for_present_name() {
    let conn = create_test_db();
    let doc = create_kb_document("LDOC01", "api/api.md", "api");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = MissingNameRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "E009"),
        "Present name should not trigger E009"
    );
}

// =============================================================================
// E010: Missing Required Field (Description)
// =============================================================================

#[test]
fn e010_missing_description_detects_empty_description() {
    let conn = create_test_db();
    let doc = create_document_with_empty_description("LDOC01", "api/api.md", "api");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = MissingDescriptionRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.error_count, 1, "Should detect missing description");
    assert!(summary.results[0].message.contains("missing required 'description' field"));
}

#[test]
fn e010_missing_description_no_error_for_present_description() {
    let conn = create_test_db();
    let doc = create_kb_document("LDOC01", "api/api.md", "api");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = MissingDescriptionRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "E010"),
        "Present description should not trigger E010"
    );
}

// =============================================================================
// E011: Invalid Closed Directory Structure
// =============================================================================

#[test]
fn e011_nested_closed_detects_nested_directories() {
    let conn = create_test_db();
    // Document in nested .closed directories.
    // Path must have `/.closed/` appearing twice non-overlappingly.
    let doc = InsertDocument::new(
        "LTASK1".to_string(),
        None,
        "api/tasks/.closed/subdir/.closed/fix_bug.md".to_string(),
        "fix-bug".to_string(),
        "Task description".to_string(),
        Some(TaskType::Task),
        Some(2),
        None,
        None,
        None,
        "abc123".to_string(),
        100,
    );
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = NestedClosedRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.error_count, 1, "Should detect nested .closed directories");
    assert!(summary.results[0].message.contains("nested closed directory"));
}

#[test]
fn e011_nested_closed_no_error_for_single_closed() {
    let conn = create_test_db();
    // Document in single .closed directory
    let doc = InsertDocument::new(
        "LTASK1".to_string(),
        None,
        "api/tasks/.closed/fix_bug.md".to_string(),
        "fix-bug".to_string(),
        "Task description".to_string(),
        Some(TaskType::Task),
        Some(2),
        None,
        None,
        None,
        "abc123".to_string(),
        100,
    );
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = NestedClosedRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "E011"),
        "Single .closed should not trigger E011"
    );
}

// =============================================================================
// E012: Non-Task in Closed Directory
// =============================================================================

#[test]
fn e012_non_task_in_closed_detects_kb_in_closed() {
    let conn = create_test_db();
    // KB document in .closed directory
    let doc = InsertDocument::new(
        "LDOC01".to_string(),
        None,
        "api/tasks/.closed/design_doc.md".to_string(),
        "design-doc".to_string(),
        "A design document".to_string(),
        None, // No task_type - KB document
        None,
        None,
        None,
        None,
        "abc123".to_string(),
        100,
    );
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = NonTaskInClosedRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.error_count, 1, "Should detect KB document in .closed");
    assert!(summary.results[0].message.contains("knowledge base document in closed directory"));
}

#[test]
fn e012_non_task_in_closed_no_error_for_task_in_closed() {
    let conn = create_test_db();
    // Task document in .closed directory
    let doc = InsertDocument::new(
        "LTASK1".to_string(),
        None,
        "api/tasks/.closed/fix_bug.md".to_string(),
        "fix-bug".to_string(),
        "Task description".to_string(),
        Some(TaskType::Task),
        Some(2),
        None,
        None,
        None,
        "abc123".to_string(),
        100,
    );
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = NonTaskInClosedRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "E012"),
        "Task in .closed should not trigger E012"
    );
}

#[test]
fn e012_non_task_in_closed_no_error_for_kb_outside_closed() {
    let conn = create_test_db();
    let doc = create_kb_document("LDOC01", "api/docs/design_doc.md", "design-doc");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = NonTaskInClosedRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "E012"),
        "KB document outside .closed should not trigger E012"
    );
}

// =============================================================================
// Integration: All Error Rules
// =============================================================================

#[test]
fn all_error_rules_returns_thirteen_rules() {
    let rules = all_error_rules();
    assert_eq!(rules.len(), 13, "Should have 13 error rules");
}

#[test]
fn all_error_rules_covers_all_error_codes() {
    let rules = all_error_rules();
    let mut codes: Vec<&str> = rules.iter().flat_map(|r| r.codes()).copied().collect();
    codes.sort();

    let expected = vec![
        "E001", "E002", "E003", "E004", "E005", "E006", "E007", "E008", "E009", "E010", "E011",
        "E012", "E013",
    ];

    assert_eq!(codes, expected, "All error codes should be covered");
}

#[test]
fn all_error_rules_are_error_severity() {
    let conn = create_test_db();
    // Create a document that will trigger various errors
    let doc = create_document_with_empty_name("L12", "api/api.md");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rules = all_error_rules();
    let rule_refs: Vec<&dyn LintRule> = rules.iter().map(|r| r.as_ref()).collect();
    let summary = execute_rules(&ctx, &rule_refs, &config).expect("Execute should succeed");

    for result in &summary.results {
        assert_eq!(
            result.severity,
            Severity::Error,
            "All error rules should produce Error severity, got {:?} for {}",
            result.severity,
            result.code
        );
    }
}
