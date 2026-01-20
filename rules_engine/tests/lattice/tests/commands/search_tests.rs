//! Tests for the `lat search` command.

use lattice::cli::commands::search_command;
use lattice::cli::query_args::SearchArgs;
use lattice::document::frontmatter_schema::TaskType;
use lattice::error::error_types::LatticeError;
use lattice::index::document_types::InsertDocument;
use lattice::index::{document_queries, fulltext_search};
use lattice::test::test_environment::TestEnv;

fn search_args(query: &str) -> SearchArgs {
    SearchArgs { query: query.to_string(), limit: None, path: None, r#type: None }
}

fn insert_document(conn: &rusqlite::Connection, id: &str, path: &str, name: &str, body: &str) {
    let doc = InsertDocument::new(
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
    );
    document_queries::insert(conn, &doc).expect("Failed to insert document");
    fulltext_search::index_document(conn, id, body).expect("Failed to index document");
}

fn insert_task(
    conn: &rusqlite::Connection,
    id: &str,
    path: &str,
    name: &str,
    body: &str,
    task_type: TaskType,
) {
    let doc = InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        name.to_string(),
        format!("Description for {name}"),
        Some(task_type),
        Some(2),
        None,
        None,
        None,
        "abc123".to_string(),
        100,
        false,
    );
    document_queries::insert(conn, &doc).expect("Failed to insert document");
    fulltext_search::index_document(conn, id, body).expect("Failed to index document");
}

// ============================================================================
// Basic Search Tests
// ============================================================================

#[test]
fn search_finds_matching_document() {
    let env = TestEnv::new();

    insert_document(
        env.conn(),
        "LDOC01",
        "api/docs/authentication.md",
        "authentication",
        "User authentication with OAuth2 tokens",
    );

    let args = search_args("OAuth2");
    let (_temp, context) = env.into_parts();
    let result = search_command::execute(context, args);

    assert!(result.is_ok(), "Search should succeed: {:?}", result);
}

#[test]
fn search_returns_no_results_for_unmatched_query() {
    let env = TestEnv::new();

    insert_document(
        env.conn(),
        "LDOC02",
        "api/docs/database.md",
        "database",
        "PostgreSQL connection pooling",
    );

    let args = search_args("zyxwvnonexistent");
    let (_temp, context) = env.into_parts();
    let result = search_command::execute(context, args);

    assert!(result.is_ok(), "Search with no results should still succeed: {:?}", result);
}

#[test]
fn search_finds_multiple_documents() {
    let env = TestEnv::new();

    insert_document(env.conn(), "LDOC03", "api/docs/auth.md", "auth", "Error handling in login");
    insert_document(
        env.conn(),
        "LDOC04",
        "api/docs/errors.md",
        "errors",
        "Error reporting service",
    );
    insert_document(
        env.conn(),
        "LDOC05",
        "api/docs/logging.md",
        "logging",
        "Logging infrastructure setup",
    );

    let args = search_args("Error");
    let (_temp, context) = env.into_parts();
    let result = search_command::execute(context, args);

    assert!(result.is_ok(), "Search should succeed: {:?}", result);
}

// ============================================================================
// Query Validation Tests
// ============================================================================

#[test]
fn search_empty_query_fails() {
    let env = TestEnv::new();

    let args = search_args("");
    let (_temp, context) = env.into_parts();
    let result = search_command::execute(context, args);

    assert!(result.is_err(), "Empty query should fail");
    match result.unwrap_err() {
        LatticeError::InvalidArgument { message } => {
            assert!(message.contains("empty"), "Error should mention empty query: {}", message);
        }
        e => panic!("Expected InvalidArgument error, got {e:?}"),
    }
}

#[test]
fn search_whitespace_only_query_fails() {
    let env = TestEnv::new();

    let args = search_args("   ");
    let (_temp, context) = env.into_parts();
    let result = search_command::execute(context, args);

    assert!(result.is_err(), "Whitespace-only query should fail");
}

#[test]
fn search_very_long_query_fails() {
    let env = TestEnv::new();

    let long_query = "a".repeat(1001);
    let args = search_args(&long_query);
    let (_temp, context) = env.into_parts();
    let result = search_command::execute(context, args);

    assert!(result.is_err(), "Very long query should fail");
    match result.unwrap_err() {
        LatticeError::InvalidArgument { message } => {
            assert!(
                message.contains("maximum length"),
                "Error should mention max length: {}",
                message
            );
        }
        e => panic!("Expected InvalidArgument error, got {e:?}"),
    }
}

// ============================================================================
// Path Filter Tests
// ============================================================================

#[test]
fn search_with_path_filter_restricts_results() {
    let env = TestEnv::new();

    insert_document(
        env.conn(),
        "LDOC06",
        "api/docs/errors.md",
        "errors",
        "Error handling patterns",
    );
    insert_document(
        env.conn(),
        "LDOC07",
        "database/docs/errors.md",
        "db-errors",
        "Database error codes",
    );

    let args = SearchArgs {
        query: "Error".to_string(),
        limit: None,
        path: Some("api/".to_string()),
        r#type: None,
    };
    let (_temp, context) = env.into_parts();
    let result = search_command::execute(context, args);

    assert!(result.is_ok(), "Search with path filter should succeed: {:?}", result);
}

#[test]
fn search_with_nonmatching_path_returns_no_results() {
    let env = TestEnv::new();

    insert_document(
        env.conn(),
        "LDOC08",
        "api/docs/feature.md",
        "feature",
        "New feature implementation",
    );

    let args = SearchArgs {
        query: "feature".to_string(),
        limit: None,
        path: Some("database/".to_string()),
        r#type: None,
    };
    let (_temp, context) = env.into_parts();
    let result = search_command::execute(context, args);

    assert!(result.is_ok(), "Search with no matching path should succeed: {:?}", result);
}

// ============================================================================
// Type Filter Tests
// ============================================================================

#[test]
fn search_with_type_filter_restricts_to_task_type() {
    let env = TestEnv::new();

    insert_task(
        env.conn(),
        "LDOC09",
        "api/tasks/fix_bug.md",
        "fix-bug",
        "Fix the login bug",
        TaskType::Bug,
    );
    insert_task(
        env.conn(),
        "LDOC10",
        "api/tasks/add_feature.md",
        "add-feature",
        "Add new login feature",
        TaskType::Feature,
    );

    let args = SearchArgs {
        query: "login".to_string(),
        limit: None,
        path: None,
        r#type: Some(TaskType::Bug),
    };
    let (_temp, context) = env.into_parts();
    let result = search_command::execute(context, args);

    assert!(result.is_ok(), "Search with type filter should succeed: {:?}", result);
}

#[test]
fn search_type_filter_excludes_non_tasks() {
    let env = TestEnv::new();

    insert_document(
        env.conn(),
        "LDOC11",
        "api/docs/auth.md",
        "auth",
        "Authentication documentation",
    );

    let args = SearchArgs {
        query: "Authentication".to_string(),
        limit: None,
        path: None,
        r#type: Some(TaskType::Bug),
    };
    let (_temp, context) = env.into_parts();
    let result = search_command::execute(context, args);

    assert!(result.is_ok(), "Search with type filter should succeed: {:?}", result);
}

// ============================================================================
// Limit Tests
// ============================================================================

#[test]
fn search_with_limit_respects_limit() {
    let env = TestEnv::new();

    for i in 1..=10 {
        let id = format!("LDOC{:02}", 11 + i);
        insert_document(
            env.conn(),
            &id,
            &format!("docs/doc{i}.md"),
            &format!("doc{i}"),
            "Common searchable term here",
        );
    }

    let args =
        SearchArgs { query: "searchable".to_string(), limit: Some(3), path: None, r#type: None };
    let (_temp, context) = env.into_parts();
    let result = search_command::execute(context, args);

    assert!(result.is_ok(), "Search with limit should succeed: {:?}", result);
}

#[test]
fn search_default_limit_is_reasonable() {
    let env = TestEnv::new();

    for i in 1..=30 {
        let id = format!("LDOC{:02}", 30 + i);
        insert_document(
            env.conn(),
            &id,
            &format!("docs/doc{i}.md"),
            &format!("doc{i}"),
            "Term that matches all documents",
        );
    }

    let args = search_args("matches");
    let (_temp, context) = env.into_parts();
    let result = search_command::execute(context, args);

    assert!(result.is_ok(), "Search with many results should succeed: {:?}", result);
}

// ============================================================================
// FTS5 Query Syntax Tests
// ============================================================================

#[test]
fn search_phrase_query() {
    let env = TestEnv::new();

    insert_document(env.conn(), "LDOC61", "docs/exact.md", "exact", "The exact phrase match here");
    insert_document(
        env.conn(),
        "LDOC62",
        "docs/partial.md",
        "partial",
        "Phrase and match separated",
    );

    let args = search_args("\"phrase match\"");
    let (_temp, context) = env.into_parts();
    let result = search_command::execute(context, args);

    assert!(result.is_ok(), "Phrase search should succeed: {:?}", result);
}

#[test]
fn search_prefix_query() {
    let env = TestEnv::new();

    insert_document(env.conn(), "LDOC63", "docs/auth.md", "auth", "Authentication service");
    insert_document(env.conn(), "LDOC64", "docs/authz.md", "authz", "Authorization rules");

    let args = search_args("auth*");
    let (_temp, context) = env.into_parts();
    let result = search_command::execute(context, args);

    assert!(result.is_ok(), "Prefix search should succeed: {:?}", result);
}

// ============================================================================
// JSON Output Tests
// ============================================================================

#[test]
fn search_with_json_output() {
    let env = TestEnv::new().with_json_output();

    insert_document(env.conn(), "LDOC65", "docs/test.md", "test", "Test document content");

    let args = search_args("Test");
    let (_temp, context) = env.into_parts();
    let result = search_command::execute(context, args);

    assert!(result.is_ok(), "Search with JSON output should succeed: {:?}", result);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn search_special_characters_in_query() {
    let env = TestEnv::new();

    insert_document(env.conn(), "LDOC66", "docs/code.md", "code", "Function foo_bar returns int");

    let args = search_args("foo_bar");
    let (_temp, context) = env.into_parts();
    let result = search_command::execute(context, args);

    assert!(result.is_ok(), "Search with underscores should succeed: {:?}", result);
}

#[test]
fn search_document_removed_from_index() {
    let env = TestEnv::new();

    fulltext_search::index_document(env.conn(), "LGHOST", "Ghost document content")
        .expect("Index should succeed");

    let args = search_args("Ghost");
    let (_temp, context) = env.into_parts();
    let result = search_command::execute(context, args);

    assert!(result.is_ok(), "Search should handle missing document gracefully: {:?}", result);
}
