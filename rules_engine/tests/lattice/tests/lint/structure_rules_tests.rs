use lattice::document::frontmatter_schema::TaskType;
use lattice::index::document_types::InsertDocument;
use lattice::index::{document_queries, schema_definition};
use lattice::lint::rule_engine::{LintConfig, LintContext, LintRule, Severity, execute_rules};
use lattice::lint::structure_rules::{
    InvalidDocumentNameFormatRule, KnowledgeBaseInTasksDirRule, NotInStandardLocationRule,
    TaskInDocsDirRule, all_structure_rules,
};
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
// W017: Document Not in Standard Location
// =============================================================================

#[test]
fn w017_not_in_standard_location_rule_interface() {
    let rule = NotInStandardLocationRule;
    assert_eq!(rule.codes(), &["W017"]);
    assert_eq!(rule.name(), "not-in-standard-location");
    assert!(!rule.requires_document_body());
}

#[test]
fn w017_detects_document_not_in_tasks_or_docs() {
    let conn = create_test_db();
    let doc = create_kb_document("LDOCAA", "api/loose_doc.md", "loose-doc");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = NotInStandardLocationRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.warning_count, 1, "Should detect document not in standard location");
    assert!(
        summary.results[0].message.contains("not in tasks/ or docs/"),
        "Message should explain the issue"
    );
}

#[test]
fn w017_no_warning_for_document_in_tasks() {
    let conn = create_test_db();
    let doc = create_task_document("LDOCAA", "api/tasks/fix_bug.md", "fix-bug", 2);
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = NotInStandardLocationRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "W017"),
        "Document in tasks/ should not trigger W017"
    );
}

#[test]
fn w017_no_warning_for_document_in_docs() {
    let conn = create_test_db();
    let doc = create_kb_document("LDOCAA", "api/docs/design_doc.md", "design-doc");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = NotInStandardLocationRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "W017"),
        "Document in docs/ should not trigger W017"
    );
}

#[test]
fn w017_no_warning_for_root_document() {
    let conn = create_test_db();
    let doc = create_kb_document("LDOCAA", "api/api.md", "api");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = NotInStandardLocationRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "W017"),
        "Root document should not trigger W017"
    );
}

// =============================================================================
// W018: Task in docs/ Directory
// =============================================================================

#[test]
fn w018_task_in_docs_rule_interface() {
    let rule = TaskInDocsDirRule;
    assert_eq!(rule.codes(), &["W018"]);
    assert_eq!(rule.name(), "task-in-docs-dir");
    assert!(!rule.requires_document_body());
}

#[test]
fn w018_detects_task_in_docs_directory() {
    let conn = create_test_db();
    let doc = create_task_document("LDOCAA", "api/docs/misplaced_task.md", "misplaced-task", 2);
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = TaskInDocsDirRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.warning_count, 1, "Should detect task in docs/ directory");
    assert!(
        summary.results[0].message.contains("is a task but located in docs/"),
        "Message should explain the issue"
    );
}

#[test]
fn w018_no_warning_for_task_in_tasks() {
    let conn = create_test_db();
    let doc = create_task_document("LDOCAA", "api/tasks/fix_bug.md", "fix-bug", 2);
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = TaskInDocsDirRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "W018"),
        "Task in tasks/ should not trigger W018"
    );
}

#[test]
fn w018_no_warning_for_kb_in_docs() {
    let conn = create_test_db();
    let doc = create_kb_document("LDOCAA", "api/docs/design_doc.md", "design-doc");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = TaskInDocsDirRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "W018"),
        "KB document in docs/ should not trigger W018"
    );
}

// =============================================================================
// W019: Knowledge Base Document in tasks/ Directory
// =============================================================================

#[test]
fn w019_kb_in_tasks_rule_interface() {
    let rule = KnowledgeBaseInTasksDirRule;
    assert_eq!(rule.codes(), &["W019"]);
    assert_eq!(rule.name(), "kb-in-tasks-dir");
    assert!(!rule.requires_document_body());
}

#[test]
fn w019_detects_kb_in_tasks_directory() {
    let conn = create_test_db();
    let doc = create_kb_document("LDOCAA", "api/tasks/misplaced_doc.md", "misplaced-doc");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = KnowledgeBaseInTasksDirRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.warning_count, 1, "Should detect KB document in tasks/ directory");
    assert!(
        summary.results[0].message.contains("is a knowledge base document but located in tasks/"),
        "Message should explain the issue"
    );
}

#[test]
fn w019_no_warning_for_kb_in_docs() {
    let conn = create_test_db();
    let doc = create_kb_document("LDOCAA", "api/docs/design_doc.md", "design-doc");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = KnowledgeBaseInTasksDirRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "W019"),
        "KB document in docs/ should not trigger W019"
    );
}

#[test]
fn w019_no_warning_for_task_in_tasks() {
    let conn = create_test_db();
    let doc = create_task_document("LDOCAA", "api/tasks/fix_bug.md", "fix-bug", 2);
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = KnowledgeBaseInTasksDirRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "W019"),
        "Task in tasks/ should not trigger W019"
    );
}

#[test]
fn w019_no_warning_for_kb_in_closed() {
    let conn = create_test_db();
    let doc = InsertDocument::new(
        "LDOCAA".to_string(),
        None,
        "api/tasks/.closed/old_doc.md".to_string(),
        "old-doc".to_string(),
        "Closed document".to_string(),
        None,
        None,
        None,
        None,
        None,
        "abc123".to_string(),
        100,
        false,
    );
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = KnowledgeBaseInTasksDirRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "W019"),
        "Document in .closed/ should not trigger W019"
    );
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
fn all_structure_rules_returns_four_rules() {
    let rules = all_structure_rules();
    assert_eq!(rules.len(), 4, "Should have 4 structure rules");
}

#[test]
fn all_structure_rules_covers_all_structure_codes() {
    let rules = all_structure_rules();
    let mut codes: Vec<&str> = rules.iter().flat_map(|r| r.codes()).copied().collect();
    codes.sort();

    let expected = vec!["W017", "W018", "W019", "W020"];

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
