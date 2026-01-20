//! Tests for the `lat reopen` command.

use std::fs;

use lattice::cli::command_dispatch::CommandContext;
use lattice::cli::commands::{close_command, create_command, reopen_command};
use lattice::cli::global_options::GlobalOptions;
use lattice::cli::task_args::{CloseArgs, CreateArgs, ReopenArgs};
use lattice::document::document_reader;
use lattice::document::frontmatter_schema::TaskType;
use lattice::error::error_types::LatticeError;
use lattice::git::client_config::FakeClientIdStore;
use lattice::index::document_queries;
use lattice::task::closed_directory;
use lattice::test::test_environment::TestEnv;

fn create_context_from_env(env: &TestEnv, global: &GlobalOptions) -> CommandContext {
    let mut context = lattice::cli::command_dispatch::create_context(env.repo_root(), global)
        .expect("Create context");
    context.client_id_store = Box::new(FakeClientIdStore::new("WQN"));
    context
}

fn create_task(env: &TestEnv, parent: &str, description: &str) -> String {
    let args = CreateArgs {
        parent: Some(parent.to_string()),
        description: Some(description.to_string()),
        r#type: Some(TaskType::Task),
        priority: None,
        body_file: None,
        labels: Vec::new(),
        deps: None,
        interactive: false,
    };

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(env, &global);

    create_command::execute(ctx, args).expect("Create task");

    let docs = document_queries::all_ids(env.conn()).expect("Query IDs");
    docs.into_iter().last().expect("Should have created a document")
}

fn create_kb_doc(env: &TestEnv, parent: &str, description: &str) -> String {
    let args = CreateArgs {
        parent: Some(parent.to_string()),
        description: Some(description.to_string()),
        r#type: None,
        priority: None,
        body_file: None,
        labels: Vec::new(),
        deps: None,
        interactive: false,
    };

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(env, &global);

    create_command::execute(ctx, args).expect("Create KB doc");

    let docs = document_queries::all_ids(env.conn()).expect("Query IDs");
    docs.into_iter().last().expect("Should have created a document")
}

fn close_task(env: &TestEnv, task_id: &str) {
    let args = CloseArgs { ids: vec![task_id.to_string()], reason: None, dry_run: false };
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(env, &global);
    close_command::execute(ctx, args).expect("Close task");
}

fn reopen_args(ids: Vec<&str>) -> ReopenArgs {
    ReopenArgs { ids: ids.into_iter().map(String::from).collect(), dry_run: false }
}

// ============================================================================
// Basic Reopen Tests
// ============================================================================

#[test]
fn reopen_moves_task_from_closed_directory() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task_id = create_task(&env, "api/", "Fix login bug");
    close_task(&env, &task_id);

    let doc_row_before =
        document_queries::lookup_by_id(env.conn(), &task_id).expect("Query").expect("Document");
    assert!(
        closed_directory::is_in_closed(&doc_row_before.path),
        "Task should be in .closed/ before reopen"
    );

    let args = reopen_args(vec![&task_id]);

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = reopen_command::execute(ctx, args);
    assert!(result.is_ok(), "Reopen should succeed: {:?}", result);

    let doc_row_after =
        document_queries::lookup_by_id(env.conn(), &task_id).expect("Query").expect("Document");

    assert!(
        !closed_directory::is_in_closed(&doc_row_after.path),
        "Task should not be in .closed/ after reopen: {}",
        doc_row_after.path
    );
    assert!(!doc_row_after.is_closed, "is_closed flag should be cleared in index");

    let reopened_path = env.repo_root().join(&doc_row_after.path);
    assert!(reopened_path.exists(), "Reopened task file should exist at restored location");
}

#[test]
fn reopen_clears_closed_at_timestamp() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task_id = create_task(&env, "api/", "Fix login bug");
    close_task(&env, &task_id);

    let doc_row_before =
        document_queries::lookup_by_id(env.conn(), &task_id).expect("Query").expect("Document");
    assert!(doc_row_before.closed_at.is_some(), "closed_at should be set before reopen");

    let args = reopen_args(vec![&task_id]);

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    reopen_command::execute(ctx, args).expect("Reopen should succeed");

    let doc_row_after =
        document_queries::lookup_by_id(env.conn(), &task_id).expect("Query").expect("Document");

    assert!(doc_row_after.closed_at.is_none(), "closed_at should be cleared after reopen");

    let doc_path = env.repo_root().join(&doc_row_after.path);
    let document = document_reader::read(&doc_path).expect("Read document");
    assert!(
        document.frontmatter.closed_at.is_none(),
        "closed_at should be cleared in file frontmatter"
    );
}

// ============================================================================
// Error Case Tests
// ============================================================================

#[test]
fn reopen_rejects_non_closed_task() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task_id = create_task(&env, "api/", "Fix login bug");

    let args = reopen_args(vec![&task_id]);

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = reopen_command::execute(ctx, args);
    assert!(result.is_err(), "Reopen should fail for non-closed task");

    let err = result.unwrap_err();
    assert!(
        matches!(err, LatticeError::OperationNotAllowed { .. }),
        "Error should be OperationNotAllowed: {:?}",
        err
    );
}

#[test]
fn reopen_rejects_kb_document() {
    let env = TestEnv::new();
    env.create_dir("api/docs");

    let doc_id = create_kb_doc(&env, "api/", "Design document");

    let args = reopen_args(vec![&doc_id]);

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = reopen_command::execute(ctx, args);
    assert!(result.is_err(), "Reopen should fail for KB document");

    let err = result.unwrap_err();
    assert!(
        matches!(err, LatticeError::OperationNotAllowed { .. }),
        "Error should be OperationNotAllowed: {:?}",
        err
    );
}

#[test]
fn reopen_fails_for_nonexistent_id() {
    let env = TestEnv::new();

    let args = reopen_args(vec!["LNONEXIST"]);

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = reopen_command::execute(ctx, args);
    assert!(result.is_err(), "Reopen should fail for nonexistent ID");

    let err = result.unwrap_err();
    assert!(
        matches!(err, LatticeError::DocumentNotFound { .. }),
        "Error should be DocumentNotFound: {:?}",
        err
    );
}

#[test]
fn reopen_fails_if_target_path_exists() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task_id = create_task(&env, "api/", "Fix login bug");

    let doc_row =
        document_queries::lookup_by_id(env.conn(), &task_id).expect("Query").expect("Document");
    let original_path = env.repo_root().join(&doc_row.path);

    close_task(&env, &task_id);

    fs::write(&original_path, "conflicting file").expect("Create conflicting file");

    let args = reopen_args(vec![&task_id]);

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = reopen_command::execute(ctx, args);
    assert!(result.is_err(), "Reopen should fail when target path exists");

    let err = result.unwrap_err();
    assert!(
        matches!(err, LatticeError::PathAlreadyExists { .. }),
        "Error should be PathAlreadyExists: {:?}",
        err
    );
}

// ============================================================================
// Dry Run Tests
// ============================================================================

#[test]
fn reopen_dry_run_does_not_move_file() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task_id = create_task(&env, "api/", "Fix login bug");
    close_task(&env, &task_id);

    let doc_row_before =
        document_queries::lookup_by_id(env.conn(), &task_id).expect("Query").expect("Document");
    let closed_path = doc_row_before.path.clone();

    let args = ReopenArgs { ids: vec![task_id.clone()], dry_run: true };

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = reopen_command::execute(ctx, args);
    assert!(result.is_ok(), "Dry run should succeed: {:?}", result);

    let doc_row_after =
        document_queries::lookup_by_id(env.conn(), &task_id).expect("Query").expect("Document");

    assert_eq!(doc_row_after.path, closed_path, "Path should not change in dry run");
    assert!(doc_row_after.is_closed, "is_closed flag should not be cleared in dry run");

    let closed_file = env.repo_root().join(&closed_path);
    assert!(closed_file.exists(), "Closed file should still exist in dry run");
}

// ============================================================================
// Batch Reopen Tests
// ============================================================================

#[test]
fn reopen_handles_multiple_ids() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task1_id = create_task(&env, "api/", "First task");
    let task2_id = create_task(&env, "api/", "Second task");

    close_task(&env, &task1_id);
    close_task(&env, &task2_id);

    let args = reopen_args(vec![&task1_id, &task2_id]);

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = reopen_command::execute(ctx, args);
    assert!(result.is_ok(), "Batch reopen should succeed: {:?}", result);

    let doc1 =
        document_queries::lookup_by_id(env.conn(), &task1_id).expect("Query").expect("First doc");
    let doc2 =
        document_queries::lookup_by_id(env.conn(), &task2_id).expect("Query").expect("Second doc");

    assert!(!doc1.is_closed, "First task should be reopened");
    assert!(!doc2.is_closed, "Second task should be reopened");
    assert!(
        !closed_directory::is_in_closed(&doc1.path),
        "First task should not be in .closed/ directory"
    );
    assert!(
        !closed_directory::is_in_closed(&doc2.path),
        "Second task should not be in .closed/ directory"
    );
}

// ============================================================================
// Link Rewriting Tests
// ============================================================================

#[test]
fn reopen_rewrites_incoming_links() {
    let env = TestEnv::new();
    env.create_dir("api");

    let task_id = create_task(&env, "api/", "Fix login bug");

    let task_row =
        document_queries::lookup_by_id(env.conn(), &task_id).expect("Query").expect("Task");
    let task_filename =
        std::path::Path::new(&task_row.path).file_name().unwrap().to_string_lossy().to_string();

    close_task(&env, &task_id);

    // Create a linking document in the same directory with a relative link to
    // .closed/
    let linking_doc_content = format!(
        r#"---
lattice-id: LDOCABC
name: design-doc
description: Design document
created-at: 2026-01-01T00:00:00Z
updated-at: 2026-01-01T00:00:00Z
---

See the [fix login bug](.closed/{task_filename}#{task_id}) task for details.
"#
    );

    let doc_path = env.repo_root().join("api/design_doc.md");
    fs::write(&doc_path, &linking_doc_content).expect("Write linking doc");

    let insert_doc = lattice::index::document_types::InsertDocument::new(
        "LDOCABC".to_string(),
        None,
        "api/design_doc.md".to_string(),
        "design-doc".to_string(),
        "Design document".to_string(),
        None,
        None,
        None,
        None,
        None,
        "hash123".to_string(),
        100,
        false,
    );
    lattice::index::document_queries::insert(env.conn(), &insert_doc).expect("Insert doc");

    let insert_link = lattice::index::link_queries::InsertLink {
        source_id: "LDOCABC",
        target_id: &task_id,
        link_type: lattice::index::link_queries::LinkType::Body,
        position: 0,
    };
    lattice::index::link_queries::insert_for_document(env.conn(), &[insert_link])
        .expect("Insert link");

    let args = reopen_args(vec![&task_id]);

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    reopen_command::execute(ctx, args).expect("Reopen should succeed");

    let updated_content = fs::read_to_string(&doc_path).expect("Read updated doc");

    assert!(
        !updated_content.contains(".closed/"),
        "Link should be rewritten to remove .closed/ path: {}",
        updated_content
    );
    assert!(
        updated_content.contains(&task_filename),
        "Link should point to reopened task: {}",
        updated_content
    );
}
