use lattice::document::frontmatter_schema::TaskType;
use lattice::index::document_types::InsertDocument;
use lattice::index::{document_queries, schema_definition};
use lattice::lint::rule_engine::{LintConfig, LintContext, LintRule, Severity, execute_rules};
use lattice::lint::structure_rules::{InvalidDocumentNameFormatRule, all_structure_rules};
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
        false,
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
        false,
    )
}

// =============================================================================
// W020: Invalid Document Name Format
// =============================================================================

#[test]
fn w020_invalid_name_format_rule_interface() {
    let rule = InvalidDocumentNameFormatRule;
    assert_eq!(rule.codes(), &["W020"]);
    assert_eq!(rule.name(), "invalid-document-name-format");
    assert!(!rule.requires_document_body());
}

#[test]
fn w020_detects_single_word_filename() {
    let conn = create_test_db();
    let doc = create_kb_document("LDOCAA", "api/docs/fixbug.md", "fixbug");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = InvalidDocumentNameFormatRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.warning_count, 1, "Should detect single-word filename");
    assert!(
        summary.results[0].message.contains("at least two underscore-separated words"),
        "Message should suggest proper format"
    );
}

#[test]
fn w020_detects_hyphenated_filename() {
    let conn = create_test_db();
    let doc = create_kb_document("LDOCAA", "api/docs/fix-bug.md", "fix-bug");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = InvalidDocumentNameFormatRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.warning_count, 1, "Should detect hyphenated filename");
}

#[test]
fn w020_detects_uppercase_filename() {
    let conn = create_test_db();
    let doc = create_kb_document("LDOCAA", "api/docs/Fix_Bug.md", "fix-bug");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = InvalidDocumentNameFormatRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.warning_count, 1, "Should detect uppercase filename");
}

#[test]
fn w020_no_warning_for_valid_filename() {
    let conn = create_test_db();
    let doc = create_kb_document("LDOCAA", "api/docs/fix_bug.md", "fix-bug");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = InvalidDocumentNameFormatRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "W020"),
        "Valid underscore-separated filename should not trigger W020"
    );
}

#[test]
fn w020_no_warning_for_multi_word_filename() {
    let conn = create_test_db();
    let doc = create_task_document(
        "LDOCAA",
        "api/tasks/implement_user_authentication.md",
        "implement-user-authentication",
        2,
    );
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = InvalidDocumentNameFormatRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "W020"),
        "Multi-word underscore-separated filename should not trigger W020"
    );
}

#[test]
fn w020_no_warning_for_root_document() {
    let conn = create_test_db();
    let doc = create_kb_document("LDOCAA", "api/api.md", "api");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = InvalidDocumentNameFormatRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "W020"),
        "Root document should not trigger W020"
    );
}

#[test]
fn w020_allows_numbers_in_filename() {
    let conn = create_test_db();
    let doc = create_kb_document("LDOCAA", "api/docs/api_v2_migration.md", "api-v2-migration");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = InvalidDocumentNameFormatRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "W020"),
        "Numbers in filename should be allowed"
    );
}

// =============================================================================
// Integration: All Structure Rules
// =============================================================================

#[test]
fn all_structure_rules_returns_one_rule() {
    let rules = all_structure_rules();
    assert_eq!(rules.len(), 1, "Should have 1 structure rule");
}

#[test]
fn all_structure_rules_covers_all_structure_codes() {
    let rules = all_structure_rules();
    let mut codes: Vec<&str> = rules.iter().flat_map(|r| r.codes()).copied().collect();
    codes.sort();

    let expected = vec!["W020"];

    assert_eq!(codes, expected, "All structure codes should be covered");
}

#[test]
fn all_structure_rules_are_warning_severity() {
    let conn = create_test_db();
    let doc = create_kb_document("LDOCAA", "api/loose.md", "loose");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rules = all_structure_rules();
    let rule_refs: Vec<&dyn LintRule> = rules.iter().map(|r| r.as_ref()).collect();
    let summary = execute_rules(&ctx, &rule_refs, &config).expect("Execute should succeed");

    for result in &summary.results {
        assert_eq!(
            result.severity,
            Severity::Warning,
            "All structure rules should produce Warning severity, got {:?} for {}",
            result.severity,
            result.code
        );
    }
}
