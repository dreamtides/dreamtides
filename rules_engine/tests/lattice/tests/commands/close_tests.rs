//! Tests for the `lat close` command.

use std::fs;

use lattice::cli::command_dispatch::{CommandContext, create_context};
use lattice::cli::commands::{close_command, create_command};
use lattice::cli::global_options::GlobalOptions;
use lattice::cli::task_args::{CloseArgs, CreateArgs};
use lattice::document::document_reader;
use lattice::document::frontmatter_schema::TaskType;
use lattice::error::error_types::LatticeError;
use lattice::index::{document_queries, schema_definition};
use lattice::task::closed_directory;

fn create_test_repo() -> (tempfile::TempDir, CommandContext) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");

    let global = GlobalOptions::default();
    let context = create_context(repo_root, &global).expect("Failed to create context");

    schema_definition::create_schema(&context.conn).expect("Failed to create schema");

    (temp_dir, context)
}

fn create_task(context: &CommandContext, parent: &str, description: &str) -> String {
    let args = CreateArgs {
        parent: parent.to_string(),
        description: description.to_string(),
        r#type: Some(TaskType::Task),
        priority: None,
        body_file: None,
        labels: Vec::new(),
        deps: None,
    };

    let global = GlobalOptions::default();
    let ctx = create_context(&context.repo_root, &global).expect("Create context");

    create_command::execute(ctx, args).expect("Create task");

    let docs = document_queries::all_ids(&context.conn).expect("Query IDs");
    docs.into_iter().last().expect("Should have created a document")
}

fn create_kb_doc(context: &CommandContext, parent: &str, description: &str) -> String {
    let args = CreateArgs {
        parent: parent.to_string(),
        description: description.to_string(),
        r#type: None,
        priority: None,
        body_file: None,
        labels: Vec::new(),
        deps: None,
    };

    let global = GlobalOptions::default();
    let ctx = create_context(&context.repo_root, &global).expect("Create context");

    create_command::execute(ctx, args).expect("Create KB doc");

    let docs = document_queries::all_ids(&context.conn).expect("Query IDs");
    docs.into_iter().last().expect("Should have created a document")
}

fn close_args(ids: Vec<&str>) -> CloseArgs {
    CloseArgs { ids: ids.into_iter().map(String::from).collect(), reason: None, dry_run: false }
}

// ============================================================================
// Basic Close Tests
// ============================================================================

#[test]
fn close_moves_task_to_closed_directory() {
    let (temp_dir, context) = create_test_repo();

    fs::create_dir_all(temp_dir.path().join("api/tasks")).expect("Create dirs");

    let task_id = create_task(&context, "api/", "Fix login bug");

    let doc_row_before =
        document_queries::lookup_by_id(&context.conn, &task_id).expect("Query").expect("Document");
    assert!(
        !closed_directory::is_in_closed(&doc_row_before.path),
        "Task should not be in .closed/ before close"
    );

    let args = close_args(vec![&task_id]);

    let global = GlobalOptions::default();
    let ctx = create_context(temp_dir.path(), &global).expect("Create context");

    let result = close_command::execute(ctx, args);
    assert!(result.is_ok(), "Close should succeed: {:?}", result);

    let doc_row_after =
        document_queries::lookup_by_id(&context.conn, &task_id).expect("Query").expect("Document");

    assert!(
        closed_directory::is_in_closed(&doc_row_after.path),
        "Task should be in .closed/ after close: {}",
        doc_row_after.path
    );
    assert!(doc_row_after.is_closed, "is_closed flag should be set in index");

    let closed_path = temp_dir.path().join(&doc_row_after.path);
    assert!(closed_path.exists(), "Closed task file should exist at new location");
}

#[test]
fn close_sets_closed_at_timestamp() {
    let (temp_dir, context) = create_test_repo();

    fs::create_dir_all(temp_dir.path().join("api/tasks")).expect("Create dirs");

    let task_id = create_task(&context, "api/", "Fix login bug");

    let doc_row_before =
        document_queries::lookup_by_id(&context.conn, &task_id).expect("Query").expect("Document");
    assert!(doc_row_before.closed_at.is_none(), "closed_at should be None before close");

    let args = close_args(vec![&task_id]);

    let global = GlobalOptions::default();
    let ctx = create_context(temp_dir.path(), &global).expect("Create context");

    close_command::execute(ctx, args).expect("Close should succeed");

    let doc_row_after =
        document_queries::lookup_by_id(&context.conn, &task_id).expect("Query").expect("Document");

    assert!(doc_row_after.closed_at.is_some(), "closed_at should be set after close");

    let doc_path = temp_dir.path().join(&doc_row_after.path);
    let document = document_reader::read(&doc_path).expect("Read document");
    assert!(
        document.frontmatter.closed_at.is_some(),
        "closed_at should be set in file frontmatter"
    );
}

// ============================================================================
// Error Case Tests
// ============================================================================

#[test]
fn close_rejects_kb_document() {
    let (temp_dir, context) = create_test_repo();

    fs::create_dir_all(temp_dir.path().join("api/docs")).expect("Create dirs");

    let doc_id = create_kb_doc(&context, "api/", "Design document");

    let args = close_args(vec![&doc_id]);

    let global = GlobalOptions::default();
    let ctx = create_context(temp_dir.path(), &global).expect("Create context");

    let result = close_command::execute(ctx, args);
    assert!(result.is_err(), "Close should fail for KB document");

    let err = result.unwrap_err();
    assert!(
        matches!(err, LatticeError::OperationNotAllowed { .. }),
        "Error should be OperationNotAllowed: {:?}",
        err
    );
}

#[test]
fn close_rejects_already_closed_task() {
    let (temp_dir, context) = create_test_repo();

    fs::create_dir_all(temp_dir.path().join("api/tasks")).expect("Create dirs");

    let task_id = create_task(&context, "api/", "Fix login bug");

    let args = close_args(vec![&task_id]);
    let global = GlobalOptions::default();
    let ctx = create_context(temp_dir.path(), &global).expect("Create context");
    close_command::execute(ctx, args).expect("First close should succeed");

    let args = close_args(vec![&task_id]);
    let global = GlobalOptions::default();
    let ctx = create_context(temp_dir.path(), &global).expect("Create context");
    let result = close_command::execute(ctx, args);

    assert!(result.is_err(), "Second close should fail");

    let err = result.unwrap_err();
    assert!(
        matches!(err, LatticeError::OperationNotAllowed { .. }),
        "Error should be OperationNotAllowed: {:?}",
        err
    );
}

#[test]
fn close_fails_for_nonexistent_id() {
    let (temp_dir, _context) = create_test_repo();

    let args = close_args(vec!["LNONEXIST"]);

    let global = GlobalOptions::default();
    let ctx = create_context(temp_dir.path(), &global).expect("Create context");

    let result = close_command::execute(ctx, args);
    assert!(result.is_err(), "Close should fail for nonexistent ID");

    let err = result.unwrap_err();
    assert!(
        matches!(err, LatticeError::DocumentNotFound { .. }),
        "Error should be DocumentNotFound: {:?}",
        err
    );
}

// ============================================================================
// Dry Run Tests
// ============================================================================

#[test]
fn close_dry_run_does_not_move_file() {
    let (temp_dir, context) = create_test_repo();

    fs::create_dir_all(temp_dir.path().join("api/tasks")).expect("Create dirs");

    let task_id = create_task(&context, "api/", "Fix login bug");

    let doc_row_before =
        document_queries::lookup_by_id(&context.conn, &task_id).expect("Query").expect("Document");
    let original_path = doc_row_before.path.clone();

    let args = CloseArgs { ids: vec![task_id.clone()], reason: None, dry_run: true };

    let global = GlobalOptions::default();
    let ctx = create_context(temp_dir.path(), &global).expect("Create context");

    let result = close_command::execute(ctx, args);
    assert!(result.is_ok(), "Dry run should succeed: {:?}", result);

    let doc_row_after =
        document_queries::lookup_by_id(&context.conn, &task_id).expect("Query").expect("Document");

    assert_eq!(doc_row_after.path, original_path, "Path should not change in dry run");
    assert!(!doc_row_after.is_closed, "is_closed flag should not be set in dry run");

    let original_file = temp_dir.path().join(&original_path);
    assert!(original_file.exists(), "Original file should still exist in dry run");
}

// ============================================================================
// Reason Tests
// ============================================================================

#[test]
fn close_with_reason_appends_to_body() {
    let (temp_dir, context) = create_test_repo();

    fs::create_dir_all(temp_dir.path().join("api/tasks")).expect("Create dirs");

    let task_id = create_task(&context, "api/", "Fix login bug");

    let args = CloseArgs {
        ids: vec![task_id.clone()],
        reason: Some("Fixed in commit abc123".to_string()),
        dry_run: false,
    };

    let global = GlobalOptions::default();
    let ctx = create_context(temp_dir.path(), &global).expect("Create context");

    close_command::execute(ctx, args).expect("Close should succeed");

    let doc_row =
        document_queries::lookup_by_id(&context.conn, &task_id).expect("Query").expect("Document");
    let doc_path = temp_dir.path().join(&doc_row.path);
    let document = document_reader::read(&doc_path).expect("Read document");

    assert!(
        document.body.contains("## Closure Reason"),
        "Body should contain Closure Reason heading"
    );
    assert!(
        document.body.contains("Fixed in commit abc123"),
        "Body should contain the reason text"
    );
}

// ============================================================================
// Batch Close Tests
// ============================================================================

#[test]
fn close_handles_multiple_ids() {
    let (temp_dir, context) = create_test_repo();

    fs::create_dir_all(temp_dir.path().join("api/tasks")).expect("Create dirs");

    let task1_id = create_task(&context, "api/", "First task");
    let task2_id = create_task(&context, "api/", "Second task");

    let args = close_args(vec![&task1_id, &task2_id]);

    let global = GlobalOptions::default();
    let ctx = create_context(temp_dir.path(), &global).expect("Create context");

    let result = close_command::execute(ctx, args);
    assert!(result.is_ok(), "Batch close should succeed: {:?}", result);

    let doc1 = document_queries::lookup_by_id(&context.conn, &task1_id)
        .expect("Query")
        .expect("First doc");
    let doc2 = document_queries::lookup_by_id(&context.conn, &task2_id)
        .expect("Query")
        .expect("Second doc");

    assert!(doc1.is_closed, "First task should be closed");
    assert!(doc2.is_closed, "Second task should be closed");
    assert!(
        closed_directory::is_in_closed(&doc1.path),
        "First task should be in .closed/ directory"
    );
    assert!(
        closed_directory::is_in_closed(&doc2.path),
        "Second task should be in .closed/ directory"
    );
}

// ============================================================================
// Link Rewriting Tests
// ============================================================================

#[test]
fn close_rewrites_incoming_links() {
    let (temp_dir, context) = create_test_repo();

    fs::create_dir_all(temp_dir.path().join("api/tasks")).expect("Create dirs");
    fs::create_dir_all(temp_dir.path().join("api/docs")).expect("Create dirs");

    let task_id = create_task(&context, "api/", "Fix login bug");

    let task_row =
        document_queries::lookup_by_id(&context.conn, &task_id).expect("Query").expect("Task");
    let task_path = task_row.path.clone();

    let linking_doc_content = format!(
        r#"---
lattice-id: LDOCABC
name: design-doc
description: Design document
created-at: 2026-01-01T00:00:00Z
updated-at: 2026-01-01T00:00:00Z
---

See the [fix login bug](../tasks/{task_filename}#{task_id}) task for details.
"#,
        task_filename = std::path::Path::new(&task_path).file_name().unwrap().to_string_lossy(),
        task_id = task_id
    );

    let doc_path = temp_dir.path().join("api/docs/design_doc.md");
    fs::write(&doc_path, &linking_doc_content).expect("Write linking doc");

    let insert_doc = lattice::index::document_types::InsertDocument::new(
        "LDOCABC".to_string(),
        None,
        "api/docs/design_doc.md".to_string(),
        "design-doc".to_string(),
        "Design document".to_string(),
        None,
        None,
        None,
        None,
        None,
        "hash123".to_string(),
        100,
    );
    lattice::index::document_queries::insert(&context.conn, &insert_doc).expect("Insert doc");

    let insert_link = lattice::index::link_queries::InsertLink {
        source_id: "LDOCABC",
        target_id: &task_id,
        link_type: lattice::index::link_queries::LinkType::Body,
        position: 0,
    };
    lattice::index::link_queries::insert_for_document(&context.conn, &[insert_link])
        .expect("Insert link");

    let args = close_args(vec![&task_id]);

    let global = GlobalOptions::default();
    let ctx = create_context(temp_dir.path(), &global).expect("Create context");

    close_command::execute(ctx, args).expect("Close should succeed");

    let updated_content = fs::read_to_string(&doc_path).expect("Read updated doc");

    assert!(
        updated_content.contains(".closed/"),
        "Link should be rewritten to include .closed/ path: {}",
        updated_content
    );
}
