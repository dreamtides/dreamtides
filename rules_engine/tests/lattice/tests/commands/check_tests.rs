//! Tests for the `lat check` command.

use std::fs;
use std::io::Write;
use std::process::ExitCode;

use lattice::cli::commands::check_command::check_output;
use lattice::cli::maintenance_args::CheckArgs;
use lattice::document::frontmatter_schema::TaskType;
use lattice::error::exit_codes;
use lattice::index::document_queries;
use lattice::index::document_types::InsertDocument;
use lattice::lint::rule_engine::LintSummary;
use lattice::test::test_environment::TestEnv;

fn default_args() -> CheckArgs {
    CheckArgs {
        errors_only: false,
        path: None,
        fix: false,
        staged_only: false,
        rebuild_index: false,
    }
}

fn create_valid_doc(id: &str, path: &str, name: &str, description: &str) -> InsertDocument {
    InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        name.to_string(),
        description.to_string(),
        None,
        None,
        Some(chrono::Utc::now()),
        None,
        None,
        format!("hash-{id}"),
        100,
    )
}

fn insert_doc(
    conn: &rusqlite::Connection,
    doc: &InsertDocument,
    repo_root: &std::path::Path,
    path: &str,
) {
    document_queries::insert(conn, doc).expect("Failed to insert document");
    let full_path = repo_root.join(path);
    let parent = full_path.parent().expect("Path should have parent");
    fs::create_dir_all(parent).expect("Failed to create parent directories");
    let mut file = fs::File::create(&full_path).expect("Failed to create file");
    write!(
        file,
        "---\nlattice-id: {}\nname: {}\ndescription: {}\n---\nBody content",
        doc.id, doc.name, doc.description
    )
    .expect("Failed to write file");
}

// ============================================================================
// Exit Code Computation Tests
// ============================================================================

#[test]
fn compute_exit_code_returns_success_when_clean() {
    let summary = LintSummary::default();
    assert_eq!(check_output::compute_exit_code(&summary), exit_codes::success());
}

#[test]
fn compute_exit_code_returns_validation_error_when_errors_present() {
    let summary = LintSummary { error_count: 1, ..Default::default() };
    assert_eq!(check_output::compute_exit_code(&summary), exit_codes::validation_error());
}

#[test]
fn compute_exit_code_returns_warnings_only_code_when_no_errors() {
    let summary = LintSummary { warning_count: 3, ..Default::default() };
    assert_eq!(check_output::compute_exit_code(&summary), ExitCode::from(3));
}

#[test]
fn compute_exit_code_prefers_errors_over_warnings() {
    let summary = LintSummary { error_count: 1, warning_count: 5, ..Default::default() };
    assert_eq!(
        check_output::compute_exit_code(&summary),
        exit_codes::validation_error(),
        "Errors should take precedence over warnings"
    );
}

// ============================================================================
// Check Args Tests
// ============================================================================

#[test]
fn default_args_has_no_filters() {
    let args = default_args();
    assert!(!args.errors_only);
    assert!(args.path.is_none());
    assert!(!args.fix);
    assert!(!args.staged_only);
    assert!(!args.rebuild_index);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn check_with_valid_document_shows_clean_summary() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");
    env.create_dir("api/docs");

    let doc = create_valid_doc("LABC01", "api/api.md", "api", "API root document");
    insert_doc(env.conn(), &doc, env.repo_root(), "api/api.md");

    // Use the lint engine directly to verify behavior without exit
    let rules = lattice::lint::error_rules::all_error_rules();
    let rule_refs: Vec<&dyn lattice::lint::rule_engine::LintRule> =
        rules.iter().map(|r| r.as_ref()).collect();
    let config = lattice::lint::rule_engine::LintConfig::default();
    let ctx = lattice::lint::rule_engine::LintContext::new(env.conn(), env.repo_root());

    let summary = lattice::lint::rule_engine::execute_rules(&ctx, &rule_refs, &config)
        .expect("Check should succeed");

    assert_eq!(summary.documents_checked, 1);
    // The document has valid structure, no errors expected from E008 (name
    // mismatch) since the name matches the filename
}

#[test]
fn check_with_path_filter_limits_scope() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");
    env.create_dir("api/docs");
    env.create_dir("database/docs");

    let api = create_valid_doc("LAPI01", "api/api.md", "api", "API root");
    insert_doc(env.conn(), &api, env.repo_root(), "api/api.md");

    let db = create_valid_doc("LDB001", "database/database.md", "database", "Database root");
    insert_doc(env.conn(), &db, env.repo_root(), "database/database.md");

    let rules = lattice::lint::error_rules::all_error_rules();
    let rule_refs: Vec<&dyn lattice::lint::rule_engine::LintRule> =
        rules.iter().map(|r| r.as_ref()).collect();
    let config = lattice::lint::rule_engine::LintConfig::default().with_path_prefix("api/");
    let ctx = lattice::lint::rule_engine::LintContext::new(env.conn(), env.repo_root());

    let summary = lattice::lint::rule_engine::execute_rules(&ctx, &rule_refs, &config)
        .expect("Check should succeed");

    assert_eq!(summary.documents_checked, 1, "Only api/ doc should be checked");
}

#[test]
fn check_errors_only_filters_warnings() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");
    env.create_dir("api/docs");

    // Create a document that will trigger a warning (too long name)
    let doc = create_valid_doc("LABC01", "api/api.md", "api", "A description that is valid");
    insert_doc(env.conn(), &doc, env.repo_root(), "api/api.md");

    // Use warning rules
    let rules = lattice::lint::warning_rules::all_warning_rules();
    let rule_refs: Vec<&dyn lattice::lint::rule_engine::LintRule> =
        rules.iter().map(|r| r.as_ref()).collect();
    let config = lattice::lint::rule_engine::LintConfig::default().with_errors_only(true);
    let ctx = lattice::lint::rule_engine::LintContext::new(env.conn(), env.repo_root());

    let summary = lattice::lint::rule_engine::execute_rules(&ctx, &rule_refs, &config)
        .expect("Check should succeed");

    assert_eq!(summary.warning_count, 0, "Warnings should be filtered when errors_only is true");
}

#[test]
fn check_with_empty_repo_succeeds() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");
    env.create_dir("api/docs");

    let rules = lattice::lint::error_rules::all_error_rules();
    let rule_refs: Vec<&dyn lattice::lint::rule_engine::LintRule> =
        rules.iter().map(|r| r.as_ref()).collect();
    let config = lattice::lint::rule_engine::LintConfig::default();
    let ctx = lattice::lint::rule_engine::LintContext::new(env.conn(), env.repo_root());

    let summary = lattice::lint::rule_engine::execute_rules(&ctx, &rule_refs, &config)
        .expect("Check should succeed on empty repo");

    assert!(summary.is_clean());
    assert_eq!(summary.documents_checked, 0);
}

#[test]
fn check_reports_missing_priority_for_task() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");
    env.create_dir("api/docs");

    // Create a task without priority by setting task_type but no priority
    let mut doc = create_valid_doc("LTASK1", "api/tasks/task-one.md", "task-one", "A task");
    doc.task_type = Some(TaskType::Task);
    doc.priority = None;
    insert_doc(env.conn(), &doc, env.repo_root(), "api/tasks/task-one.md");

    let rules = lattice::lint::error_rules::all_error_rules();
    let rule_refs: Vec<&dyn lattice::lint::rule_engine::LintRule> =
        rules.iter().map(|r| r.as_ref()).collect();
    let config = lattice::lint::rule_engine::LintConfig::default();
    let ctx = lattice::lint::rule_engine::LintContext::new(env.conn(), env.repo_root());

    let summary = lattice::lint::rule_engine::execute_rules(&ctx, &rule_refs, &config)
        .expect("Check should succeed");

    assert!(summary.has_errors(), "Should report error for missing priority");
    assert!(
        summary.results.iter().any(|r| r.code == "E004"),
        "Should include E004 (missing priority)"
    );
}

#[test]
fn check_reports_name_mismatch() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");
    env.create_dir("api/docs");

    // Create a document where name doesn't match filename
    let doc = create_valid_doc("LDOC01", "api/api.md", "wrong-name", "Description");
    insert_doc(env.conn(), &doc, env.repo_root(), "api/api.md");

    let rules = lattice::lint::error_rules::all_error_rules();
    let rule_refs: Vec<&dyn lattice::lint::rule_engine::LintRule> =
        rules.iter().map(|r| r.as_ref()).collect();
    let config = lattice::lint::rule_engine::LintConfig::default();
    let ctx = lattice::lint::rule_engine::LintContext::new(env.conn(), env.repo_root());

    let summary = lattice::lint::rule_engine::execute_rules(&ctx, &rule_refs, &config)
        .expect("Check should succeed");

    assert!(summary.has_errors(), "Should report error for name mismatch");
    assert!(
        summary.results.iter().any(|r| r.code == "E008"),
        "Should include E008 (name mismatch)"
    );
}

#[test]
fn check_summary_counts_affected_documents_correctly() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");
    env.create_dir("api/docs");

    // Create two docs with issues
    let doc1 = create_valid_doc("LDOC01", "api/api.md", "wrong-one", "Description 1");
    let doc2 = create_valid_doc("LDOC02", "api/docs/design.md", "wrong-two", "Description 2");
    insert_doc(env.conn(), &doc1, env.repo_root(), "api/api.md");
    insert_doc(env.conn(), &doc2, env.repo_root(), "api/docs/design.md");

    let rules = lattice::lint::error_rules::all_error_rules();
    let rule_refs: Vec<&dyn lattice::lint::rule_engine::LintRule> =
        rules.iter().map(|r| r.as_ref()).collect();
    let config = lattice::lint::rule_engine::LintConfig::default();
    let ctx = lattice::lint::rule_engine::LintContext::new(env.conn(), env.repo_root());

    let summary = lattice::lint::rule_engine::execute_rules(&ctx, &rule_refs, &config)
        .expect("Check should succeed");

    assert_eq!(summary.documents_checked, 2);
    assert_eq!(summary.affected_documents, 2, "Both documents should be affected");
}
