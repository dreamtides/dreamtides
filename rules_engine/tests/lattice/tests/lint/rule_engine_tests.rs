use std::path::PathBuf;

use lattice::document::frontmatter_schema::TaskType;
use lattice::index::document_types::InsertDocument;
use lattice::index::{document_queries, schema_definition};
use lattice::lint::rule_engine::{
    LintConfig, LintContext, LintDocument, LintResult, LintRule, LintSummary, Severity,
    execute_rules, execute_rules_on_documents,
};
use rusqlite::Connection;
use tempfile::TempDir;

fn create_test_db() -> Connection {
    let conn = Connection::open_in_memory().expect("Failed to create in-memory database");
    schema_definition::create_schema(&conn).expect("Failed to create schema");
    conn
}

fn create_test_document(id: &str, path: &str, name: &str) -> InsertDocument {
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

struct AlwaysErrorRule;

impl LintRule for AlwaysErrorRule {
    fn codes(&self) -> &[&str] {
        &["E999"]
    }

    fn name(&self) -> &str {
        "always-error"
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        vec![LintResult::error("E999", &doc.row.path, "Always fails")]
    }
}

struct AlwaysWarningRule;

impl LintRule for AlwaysWarningRule {
    fn codes(&self) -> &[&str] {
        &["W999"]
    }

    fn name(&self) -> &str {
        "always-warning"
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        vec![LintResult::warning("W999", &doc.row.path, "Always warns")]
    }
}

struct CleanRule;

impl LintRule for CleanRule {
    fn codes(&self) -> &[&str] {
        &["E000"]
    }

    fn name(&self) -> &str {
        "clean-rule"
    }

    fn check(&self, _doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        vec![]
    }
}

struct TaskOnlyRule;

impl LintRule for TaskOnlyRule {
    fn codes(&self) -> &[&str] {
        &["T001"]
    }

    fn name(&self) -> &str {
        "task-only"
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        if doc.row.task_type.is_some() {
            vec![LintResult::warning("T001", &doc.row.path, "Found task")]
        } else {
            vec![]
        }
    }
}

#[test]
fn lint_result_error_has_correct_fields() {
    let result = LintResult::error("E001", "test.md", "Test message");
    assert_eq!(result.code, "E001");
    assert_eq!(result.severity, Severity::Error);
    assert_eq!(result.path, PathBuf::from("test.md"));
    assert_eq!(result.message, "Test message");
    assert!(result.line.is_none());
}

#[test]
fn lint_result_warning_has_correct_fields() {
    let result = LintResult::warning("W001", "test.md", "Warning message");
    assert_eq!(result.code, "W001");
    assert_eq!(result.severity, Severity::Warning);
    assert_eq!(result.path, PathBuf::from("test.md"));
    assert_eq!(result.message, "Warning message");
}

#[test]
fn lint_result_with_line_adds_line_number() {
    let result = LintResult::error("E001", "test.md", "msg").with_line(42);
    assert_eq!(result.line, Some(42));
}

#[test]
fn severity_error_is_greater_than_warning() {
    assert!(Severity::Error > Severity::Warning);
}

#[test]
fn severity_is_error_returns_true_for_error() {
    assert!(Severity::Error.is_error());
    assert!(!Severity::Warning.is_error());
}

#[test]
fn severity_as_str_returns_correct_strings() {
    assert_eq!(Severity::Error.as_str(), "error");
    assert_eq!(Severity::Warning.as_str(), "warning");
}

#[test]
fn lint_summary_is_clean_when_no_issues() {
    let summary = LintSummary::default();
    assert!(summary.is_clean());
    assert!(!summary.has_errors());
    assert!(!summary.has_warnings());
}

#[test]
fn lint_summary_has_errors_when_error_count_nonzero() {
    let summary = LintSummary { error_count: 1, ..Default::default() };
    assert!(summary.has_errors());
    assert!(!summary.is_clean());
}

#[test]
fn lint_summary_has_warnings_when_warning_count_nonzero() {
    let summary = LintSummary { warning_count: 1, ..Default::default() };
    assert!(summary.has_warnings());
    assert!(!summary.is_clean());
}

#[test]
fn lint_config_default_has_no_filters() {
    let config = LintConfig::default();
    assert!(!config.errors_only);
    assert!(config.path_prefix.is_none());
}

#[test]
fn lint_config_with_errors_only_sets_flag() {
    let config = LintConfig::default().with_errors_only(true);
    assert!(config.errors_only);
}

#[test]
fn lint_config_with_path_prefix_sets_prefix() {
    let config = LintConfig::default().with_path_prefix("src/");
    assert_eq!(config.path_prefix, Some(PathBuf::from("src/")));
}

#[test]
fn execute_rules_returns_empty_summary_when_no_rules() {
    let conn = create_test_db();
    let doc = create_test_document("LDOC01", "api/api.md", "api");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rules: Vec<&dyn LintRule> = vec![];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.documents_checked, 1);
    assert!(summary.is_clean());
}

#[test]
fn execute_rules_counts_errors_correctly() {
    let conn = create_test_db();
    let doc1 = create_test_document("LDOC01", "api/api.md", "api");
    let doc2 = create_test_document("LDOC02", "db/db.md", "db");
    document_queries::insert(&conn, &doc1).expect("Insert should succeed");
    document_queries::insert(&conn, &doc2).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let error_rule = AlwaysErrorRule;
    let rules: Vec<&dyn LintRule> = vec![&error_rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.documents_checked, 2);
    assert_eq!(summary.error_count, 2);
    assert_eq!(summary.warning_count, 0);
    assert_eq!(summary.affected_documents, 2);
}

#[test]
fn execute_rules_counts_warnings_correctly() {
    let conn = create_test_db();
    let doc = create_test_document("LDOC01", "api/api.md", "api");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let warning_rule = AlwaysWarningRule;
    let rules: Vec<&dyn LintRule> = vec![&warning_rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.warning_count, 1);
    assert_eq!(summary.error_count, 0);
}

#[test]
fn execute_rules_combines_multiple_rule_results() {
    let conn = create_test_db();
    let doc = create_test_document("LDOC01", "api/api.md", "api");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let error_rule = AlwaysErrorRule;
    let warning_rule = AlwaysWarningRule;
    let rules: Vec<&dyn LintRule> = vec![&error_rule, &warning_rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.error_count, 1);
    assert_eq!(summary.warning_count, 1);
    assert_eq!(summary.results.len(), 2);
}

#[test]
fn execute_rules_errors_only_filters_warnings() {
    let conn = create_test_db();
    let doc = create_test_document("LDOC01", "api/api.md", "api");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default().with_errors_only(true);

    let error_rule = AlwaysErrorRule;
    let warning_rule = AlwaysWarningRule;
    let rules: Vec<&dyn LintRule> = vec![&error_rule, &warning_rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.error_count, 1);
    assert_eq!(summary.warning_count, 0, "Warnings should be filtered");
    assert_eq!(summary.results.len(), 1, "Only error should be in results");
}

#[test]
fn execute_rules_path_prefix_filters_documents() {
    let conn = create_test_db();
    let doc1 = create_test_document("LDOC01", "api/api.md", "api");
    let doc2 = create_test_document("LDOC02", "db/db.md", "db");
    document_queries::insert(&conn, &doc1).expect("Insert should succeed");
    document_queries::insert(&conn, &doc2).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default().with_path_prefix("api/");

    let error_rule = AlwaysErrorRule;
    let rules: Vec<&dyn LintRule> = vec![&error_rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.documents_checked, 1, "Only api/ doc should be checked");
    assert_eq!(summary.error_count, 1);
}

#[test]
fn execute_rules_clean_rule_produces_no_results() {
    let conn = create_test_db();
    let doc = create_test_document("LDOC01", "api/api.md", "api");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let clean_rule = CleanRule;
    let rules: Vec<&dyn LintRule> = vec![&clean_rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(summary.is_clean());
    assert_eq!(summary.documents_checked, 1);
}

#[test]
fn execute_rules_rule_can_access_document_metadata() {
    let conn = create_test_db();
    let kb_doc = create_test_document("LDOC01", "api/api.md", "api");
    let task_doc = create_task_document("LTASK1", "api/tasks/fix_bug.md", "fix-bug", 1);
    document_queries::insert(&conn, &kb_doc).expect("Insert should succeed");
    document_queries::insert(&conn, &task_doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let task_only_rule = TaskOnlyRule;
    let rules: Vec<&dyn LintRule> = vec![&task_only_rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.warning_count, 1, "Should warn for task only");
    assert_eq!(summary.documents_checked, 2);
}

#[test]
fn execute_rules_on_documents_handles_empty_list() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let error_rule = AlwaysErrorRule;
    let rules: Vec<&dyn LintRule> = vec![&error_rule];
    let summary =
        execute_rules_on_documents(&ctx, &rules, &config, vec![]).expect("Execute should succeed");

    assert_eq!(summary.documents_checked, 0);
    assert!(summary.is_clean());
}

#[test]
fn lint_context_lookup_document_finds_existing() {
    let conn = create_test_db();
    let doc = create_test_document("LDOC01", "api/api.md", "api");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());

    let result = ctx.lookup_document("LDOC01").expect("Lookup should succeed");
    assert!(result.is_some());
    assert_eq!(result.unwrap().id, "LDOC01");
}

#[test]
fn lint_context_lookup_document_returns_none_for_missing() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());

    let result = ctx.lookup_document("LMISSING").expect("Lookup should succeed");
    assert!(result.is_none());
}

#[test]
fn lint_context_document_exists_returns_true_for_existing() {
    let conn = create_test_db();
    let doc = create_test_document("LDOC01", "api/api.md", "api");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());

    assert!(ctx.document_exists("LDOC01").expect("Check should succeed"));
}

#[test]
fn lint_context_document_exists_returns_false_for_missing() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());

    assert!(!ctx.document_exists("LMISSING").expect("Check should succeed"));
}

#[test]
fn affected_documents_counts_unique_paths() {
    let conn = create_test_db();
    let doc = create_test_document("LDOC01", "api/api.md", "api");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let error_rule = AlwaysErrorRule;
    let warning_rule = AlwaysWarningRule;
    let rules: Vec<&dyn LintRule> = vec![&error_rule, &warning_rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.affected_documents, 1, "Same document should count once");
    assert_eq!(summary.results.len(), 2, "But should have 2 results");
}
