//! Tests for the `lat mv` command.

use std::fs;

use lattice::cli::command_dispatch::CommandContext;
use lattice::cli::commands::{create_command, mv_command};
use lattice::cli::global_options::GlobalOptions;
use lattice::cli::task_args::{CreateArgs, MvArgs};
use lattice::document::document_reader;
use lattice::document::frontmatter_schema::TaskType;
use lattice::error::error_types::LatticeError;
use lattice::git::client_config::FakeClientIdStore;
use lattice::index::document_queries;
use lattice::test::test_environment::TestEnv;

fn create_context_from_env(env: &TestEnv, global: &GlobalOptions) -> CommandContext {
    let mut context = lattice::cli::command_dispatch::create_context(env.repo_root(), global)
        .expect("Create context");
    context.client_id_store = Box::new(FakeClientIdStore::new("WQN"));
    context
}

fn create_task(env: &TestEnv, parent: &str, description: &str) -> String {
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
    let ctx = create_context_from_env(env, &global);

    create_command::execute(ctx, args).expect("Create task");

    let docs = document_queries::all_ids(env.conn()).expect("Query IDs");
    docs.into_iter().last().expect("Should have created a document")
}

fn create_kb_doc(env: &TestEnv, parent: &str, description: &str) -> String {
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
    let ctx = create_context_from_env(env, &global);

    create_command::execute(ctx, args).expect("Create KB doc");

    let docs = document_queries::all_ids(env.conn()).expect("Query IDs");
    docs.into_iter().last().expect("Should have created a document")
}

fn mv_args(id: &str, new_path: &str) -> MvArgs {
    MvArgs { id: id.to_string(), new_path: new_path.to_string(), dry_run: false }
}

// ============================================================================
// Basic Move Tests
// ============================================================================

#[test]
fn mv_moves_document_to_new_location() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");
    env.create_dir("api/docs");

    let doc_id = create_kb_doc(&env, "api/", "Design document");

    let doc_row = document_queries::lookup_by_id(env.conn(), &doc_id).expect("Query").unwrap();
    let original_path = doc_row.path.clone();
    assert!(original_path.contains("api/docs/"), "Should be in api/docs/: {}", original_path);

    let args = mv_args(&doc_id, "api/docs/new_design.md");

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = mv_command::execute(ctx, args);
    assert!(result.is_ok(), "Mv should succeed: {:?}", result);

    let doc_row = document_queries::lookup_by_id(env.conn(), &doc_id).expect("Query").unwrap();
    assert_eq!(doc_row.path, "api/docs/new_design.md", "Path should be updated in index");
    assert_eq!(doc_row.name, "new-design", "Name should be derived from new filename");

    let old_file = env.repo_root().join(&original_path);
    let new_file = env.repo_root().join("api/docs/new_design.md");
    assert!(!old_file.exists(), "Old file should not exist");
    assert!(new_file.exists(), "New file should exist");
}

#[test]
fn mv_updates_name_field_from_new_filename() {
    let env = TestEnv::new();
    env.create_dir("api/docs");

    let doc_id = create_kb_doc(&env, "api/", "Original name");

    let doc_row = document_queries::lookup_by_id(env.conn(), &doc_id).expect("Query").unwrap();
    assert_eq!(doc_row.name, "original-name", "Initial name should be derived from description");

    let args = mv_args(&doc_id, "api/docs/renamed_document.md");

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    mv_command::execute(ctx, args).expect("Mv should succeed");

    let doc_row = document_queries::lookup_by_id(env.conn(), &doc_id).expect("Query").unwrap();
    assert_eq!(doc_row.name, "renamed-document", "Name should be updated to match new filename");

    let doc_path = env.repo_root().join(&doc_row.path);
    let document = document_reader::read(&doc_path).expect("Read document");
    assert_eq!(
        document.frontmatter.name, "renamed-document",
        "Name in frontmatter should match index"
    );
}

#[test]
fn mv_to_different_directory_updates_parent_id() {
    let env = TestEnv::new();
    env.create_dir("api/docs");
    env.create_dir("database/docs");

    env.create_document("database/database.md", "LROOTDB", "database", "Database module");

    let db_root_insert = lattice::index::document_types::InsertDocument::new(
        "LROOTDB".to_string(),
        None,
        "database/database.md".to_string(),
        "database".to_string(),
        "Database module".to_string(),
        None,
        None,
        None,
        None,
        None,
        "hash_db".to_string(),
        50,
    );
    lattice::index::document_queries::insert(env.conn(), &db_root_insert).expect("Insert db root");

    let db_root = lattice::index::directory_roots::DirectoryRoot {
        directory_path: "database".to_string(),
        root_id: "LROOTDB".to_string(),
        parent_path: None,
        depth: 0,
    };
    lattice::index::directory_roots::upsert(env.conn(), &db_root).expect("Upsert db root");

    let doc_id = create_kb_doc(&env, "api/", "Design document");

    let doc_row_before =
        document_queries::lookup_by_id(env.conn(), &doc_id).expect("Query").unwrap();
    let initial_parent_id = doc_row_before.parent_id.clone();

    let args = mv_args(&doc_id, "database/docs/design_document.md");

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    mv_command::execute(ctx, args).expect("Mv should succeed");

    let doc_row_after =
        document_queries::lookup_by_id(env.conn(), &doc_id).expect("Query").unwrap();
    assert_eq!(
        doc_row_after.parent_id,
        Some("LROOTDB".to_string()),
        "Parent-id should be updated to database root"
    );
    assert_ne!(
        doc_row_after.parent_id, initial_parent_id,
        "Parent-id should have changed after moving to different directory"
    );
}

// ============================================================================
// Link Rewriting Tests
// ============================================================================

#[test]
fn mv_rewrites_incoming_links() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");
    env.create_dir("api/docs");

    let task_id = create_task(&env, "api/", "Fix login bug");

    let task_row = document_queries::lookup_by_id(env.conn(), &task_id).expect("Query").unwrap();
    let task_filename =
        std::path::Path::new(&task_row.path).file_name().unwrap().to_string_lossy().to_string();

    let linking_doc_content = format!(
        r#"---
lattice-id: LDOCABC
name: design-doc
description: Design document
created-at: 2026-01-01T00:00:00Z
updated-at: 2026-01-01T00:00:00Z
---

See the [fix login bug](../tasks/{task_filename}#{task_id}) task for details.
"#
    );

    let doc_path = env.repo_root().join("api/docs/design_doc.md");
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
    lattice::index::document_queries::insert(env.conn(), &insert_doc).expect("Insert doc");

    let insert_link = lattice::index::link_queries::InsertLink {
        source_id: "LDOCABC",
        target_id: &task_id,
        link_type: lattice::index::link_queries::LinkType::Body,
        position: 0,
    };
    lattice::index::link_queries::insert_for_document(env.conn(), &[insert_link])
        .expect("Insert link");

    let args = mv_args(&task_id, "api/tasks/renamed_task.md");

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    mv_command::execute(ctx, args).expect("Mv should succeed");

    let updated_content = fs::read_to_string(&doc_path).expect("Read updated doc");
    assert!(
        updated_content.contains("renamed_task.md"),
        "Link should be rewritten to new filename: {}",
        updated_content
    );
}

// ============================================================================
// Dry Run Tests
// ============================================================================

#[test]
fn mv_dry_run_does_not_move_file() {
    let env = TestEnv::new();
    env.create_dir("api/docs");

    let doc_id = create_kb_doc(&env, "api/", "Design document");

    let doc_row = document_queries::lookup_by_id(env.conn(), &doc_id).expect("Query").unwrap();
    let original_path = doc_row.path.clone();
    let original_name = doc_row.name.clone();

    let args =
        MvArgs { id: doc_id.clone(), new_path: "api/docs/new_name.md".to_string(), dry_run: true };

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = mv_command::execute(ctx, args);
    assert!(result.is_ok(), "Dry run should succeed: {:?}", result);

    let doc_row = document_queries::lookup_by_id(env.conn(), &doc_id).expect("Query").unwrap();
    assert_eq!(doc_row.path, original_path, "Path should not change in dry run");
    assert_eq!(doc_row.name, original_name, "Name should not change in dry run");

    let original_file = env.repo_root().join(&original_path);
    let new_file = env.repo_root().join("api/docs/new_name.md");
    assert!(original_file.exists(), "Original file should still exist");
    assert!(!new_file.exists(), "New file should not exist in dry run");
}

// ============================================================================
// Error Case Tests
// ============================================================================

#[test]
fn mv_fails_when_target_already_exists() {
    let env = TestEnv::new();
    env.create_dir("api/docs");

    let doc_id = create_kb_doc(&env, "api/", "First document");
    let _doc2_id = create_kb_doc(&env, "api/", "Second document");

    let doc2_row = document_queries::lookup_by_id(env.conn(), &_doc2_id).expect("Query").unwrap();

    let args = mv_args(&doc_id, &doc2_row.path);

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = mv_command::execute(ctx, args);
    assert!(result.is_err(), "Mv should fail when target exists");

    let err = result.unwrap_err();
    assert!(
        matches!(err, LatticeError::PathAlreadyExists { .. }),
        "Error should be PathAlreadyExists: {:?}",
        err
    );
}

#[test]
fn mv_fails_for_nonexistent_id() {
    let env = TestEnv::new();
    env.create_dir("api/docs");

    let args = mv_args("LNONEXIST", "api/docs/some_doc.md");

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = mv_command::execute(ctx, args);
    assert!(result.is_err(), "Mv should fail for nonexistent ID");

    let err = result.unwrap_err();
    assert!(
        matches!(err, LatticeError::DocumentNotFound { .. }),
        "Error should be DocumentNotFound: {:?}",
        err
    );
}

#[test]
fn mv_fails_when_moving_into_closed_directory() {
    let env = TestEnv::new();
    env.create_dir("api/docs");
    env.create_dir("api/tasks/.closed");

    let doc_id = create_kb_doc(&env, "api/", "Design document");

    let args = mv_args(&doc_id, "api/tasks/.closed/design_document.md");

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = mv_command::execute(ctx, args);
    assert!(result.is_err(), "Mv into .closed/ should fail");

    let err = result.unwrap_err();
    assert!(
        matches!(err, LatticeError::OperationNotAllowed { .. }),
        "Error should be OperationNotAllowed: {:?}",
        err
    );
}

#[test]
fn mv_fails_when_source_and_dest_are_same() {
    let env = TestEnv::new();
    env.create_dir("api/docs");

    let doc_id = create_kb_doc(&env, "api/", "Design document");

    let doc_row = document_queries::lookup_by_id(env.conn(), &doc_id).expect("Query").unwrap();

    let args = mv_args(&doc_id, &doc_row.path);

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = mv_command::execute(ctx, args);
    assert!(result.is_err(), "Mv to same path should fail");

    let err = result.unwrap_err();
    assert!(
        matches!(err, LatticeError::OperationNotAllowed { .. }),
        "Error should be OperationNotAllowed: {:?}",
        err
    );
}

#[test]
fn mv_fails_when_target_missing_md_extension() {
    let env = TestEnv::new();
    env.create_dir("api/docs");

    let doc_id = create_kb_doc(&env, "api/", "Design document");

    let args = mv_args(&doc_id, "api/docs/no_extension");

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = mv_command::execute(ctx, args);
    assert!(result.is_err(), "Mv to path without .md should fail");

    let err = result.unwrap_err();
    assert!(
        matches!(err, LatticeError::InvalidArgument { .. }),
        "Error should be InvalidArgument: {:?}",
        err
    );
}
