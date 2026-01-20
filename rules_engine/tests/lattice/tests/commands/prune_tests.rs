//! Tests for the `lat prune` command.

use std::fs;
use std::path::Path;

use lattice::cli::command_dispatch::{CommandContext, create_context};
use lattice::cli::commands::{close_command, create_command, prune_command};
use lattice::cli::global_options::GlobalOptions;
use lattice::cli::task_args::{CloseArgs, CreateArgs, PruneArgs};
use lattice::document::document_reader;
use lattice::document::frontmatter_schema::TaskType;
use lattice::error::error_types::LatticeError;
use lattice::git::client_config::FakeClientIdStore;
use lattice::index::{document_queries, link_queries};
use lattice::test::test_environment::TestEnv;

fn create_task(context: &CommandContext, parent: &str, description: &str) -> String {
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
    let mut ctx = create_context(&context.repo_root, &global).expect("Create context");
    ctx.client_id_store = Box::new(FakeClientIdStore::new("WQN"));

    create_command::execute(ctx, args).expect("Create task");

    let docs = document_queries::all_ids(&context.conn).expect("Query IDs");
    docs.into_iter().last().expect("Should have created a document")
}

fn close_task(repo_root: &Path, task_id: &str) {
    let args = CloseArgs { ids: vec![task_id.to_string()], reason: None, dry_run: false };
    let global = GlobalOptions::default();
    let mut ctx = create_context(repo_root, &global).expect("Create context");
    ctx.client_id_store = Box::new(FakeClientIdStore::new("WQN"));
    close_command::execute(ctx, args).expect("Close task");
}

fn prune_args_all() -> PruneArgs {
    PruneArgs { path: None, all: true, force: false, dry_run: false }
}

fn prune_args_path(path: &str) -> PruneArgs {
    PruneArgs { path: Some(path.to_string()), all: false, force: false, dry_run: false }
}

// ============================================================================
// Basic Prune Tests
// ============================================================================

#[test]
fn prune_deletes_closed_task() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let (temp_dir, context) = env.into_parts();

    let task_id = create_task(&context, "api/", "Fix login bug");
    close_task(temp_dir.path(), &task_id);

    let doc_before = document_queries::lookup_by_id(&context.conn, &task_id).expect("Query");
    assert!(doc_before.is_some(), "Closed task should exist before prune");

    let args = prune_args_all();
    let global = GlobalOptions::default();
    let mut ctx = create_context(temp_dir.path(), &global).expect("Create context");
    ctx.client_id_store = Box::new(FakeClientIdStore::new("WQN"));

    let result = prune_command::execute(ctx, args);
    assert!(result.is_ok(), "Prune should succeed: {:?}", result);

    let doc_after = document_queries::lookup_by_id(&context.conn, &task_id).expect("Query");
    assert!(doc_after.is_none(), "Task should be deleted after prune");
}

#[test]
fn prune_with_path_only_deletes_under_path() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");
    env.create_dir("db");
    env.create_dir("db/tasks");

    let (temp_dir, context) = env.into_parts();

    let api_task_id = create_task(&context, "api/", "API task");
    let db_task_id = create_task(&context, "db/", "DB task");

    close_task(temp_dir.path(), &api_task_id);
    close_task(temp_dir.path(), &db_task_id);

    let args = prune_args_path("api/");
    let global = GlobalOptions::default();
    let mut ctx = create_context(temp_dir.path(), &global).expect("Create context");
    ctx.client_id_store = Box::new(FakeClientIdStore::new("WQN"));

    prune_command::execute(ctx, args).expect("Prune should succeed");

    let api_doc = document_queries::lookup_by_id(&context.conn, &api_task_id).expect("Query");
    let db_doc = document_queries::lookup_by_id(&context.conn, &db_task_id).expect("Query");

    assert!(api_doc.is_none(), "API task under path should be deleted");
    assert!(db_doc.is_some(), "DB task outside path should remain");
}

#[test]
fn prune_does_not_delete_open_tasks() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let (temp_dir, context) = env.into_parts();

    let task_id = create_task(&context, "api/", "Open task");

    let args = prune_args_all();
    let global = GlobalOptions::default();
    let mut ctx = create_context(temp_dir.path(), &global).expect("Create context");
    ctx.client_id_store = Box::new(FakeClientIdStore::new("WQN"));

    prune_command::execute(ctx, args).expect("Prune should succeed (no closed tasks)");

    let doc_after = document_queries::lookup_by_id(&context.conn, &task_id).expect("Query");
    assert!(doc_after.is_some(), "Open task should not be deleted by prune");
}

// ============================================================================
// Argument Validation Tests
// ============================================================================

#[test]
fn prune_requires_path_or_all() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let (temp_dir, _context) = env.into_parts();

    let args = PruneArgs { path: None, all: false, force: false, dry_run: false };

    let global = GlobalOptions::default();
    let mut ctx = create_context(temp_dir.path(), &global).expect("Create context");
    ctx.client_id_store = Box::new(FakeClientIdStore::new("WQN"));

    let result = prune_command::execute(ctx, args);
    assert!(result.is_err(), "Prune should fail without path or --all");

    let err = result.unwrap_err();
    assert!(
        matches!(err, LatticeError::MissingArgument { .. }),
        "Error should be MissingArgument: {:?}",
        err
    );
}

#[test]
fn prune_rejects_path_and_all_together() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let (temp_dir, _context) = env.into_parts();

    let args =
        PruneArgs { path: Some("api/".to_string()), all: true, force: false, dry_run: false };

    let global = GlobalOptions::default();
    let mut ctx = create_context(temp_dir.path(), &global).expect("Create context");
    ctx.client_id_store = Box::new(FakeClientIdStore::new("WQN"));

    let result = prune_command::execute(ctx, args);
    assert!(result.is_err(), "Prune should fail with both path and --all");

    let err = result.unwrap_err();
    assert!(
        matches!(err, LatticeError::ConflictingOptions { .. }),
        "Error should be ConflictingOptions: {:?}",
        err
    );
}

// ============================================================================
// Dry Run Tests
// ============================================================================

#[test]
fn prune_dry_run_does_not_delete() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let (temp_dir, context) = env.into_parts();

    let task_id = create_task(&context, "api/", "Fix login bug");
    close_task(temp_dir.path(), &task_id);

    let args = PruneArgs { path: None, all: true, force: false, dry_run: true };

    let global = GlobalOptions::default();
    let mut ctx = create_context(temp_dir.path(), &global).expect("Create context");
    ctx.client_id_store = Box::new(FakeClientIdStore::new("WQN"));

    prune_command::execute(ctx, args).expect("Dry run should succeed");

    let doc_after = document_queries::lookup_by_id(&context.conn, &task_id).expect("Query");
    assert!(doc_after.is_some(), "Task should still exist after dry run");

    let doc_row = doc_after.expect("Document");
    let file_path = temp_dir.path().join(&doc_row.path);
    assert!(file_path.exists(), "Task file should still exist after dry run");
}

// ============================================================================
// YAML Reference Cleanup Tests
// ============================================================================

#[test]
fn prune_removes_yaml_blocking_references() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let (temp_dir, context) = env.into_parts();

    let blocker_id = create_task(&context, "api/", "Blocking task");
    let blocked_id = create_task(&context, "api/", "Blocked task");

    let blocked_doc_row = document_queries::lookup_by_id(&context.conn, &blocked_id)
        .expect("Query")
        .expect("Document");
    let blocked_file_path = temp_dir.path().join(&blocked_doc_row.path);
    let document = document_reader::read(&blocked_file_path).expect("Read");

    let mut frontmatter = document.frontmatter.clone();
    frontmatter.blocked_by = vec![blocker_id.parse().expect("Parse ID")];

    let content =
        lattice::document::frontmatter_parser::format_document(&frontmatter, &document.body)
            .expect("Format");
    lattice::document::document_writer::write_raw(
        &blocked_file_path,
        &content,
        &lattice::document::document_writer::WriteOptions::default(),
    )
    .expect("Write");

    let insert_link = link_queries::InsertLink {
        source_id: &blocked_id,
        target_id: &blocker_id,
        link_type: link_queries::LinkType::BlockedBy,
        position: 0,
    };
    link_queries::insert_for_document(&context.conn, &[insert_link]).expect("Insert link");

    close_task(temp_dir.path(), &blocker_id);

    let args = prune_args_all();
    let global = GlobalOptions::default();
    let mut ctx = create_context(temp_dir.path(), &global).expect("Create context");
    ctx.client_id_store = Box::new(FakeClientIdStore::new("WQN"));

    prune_command::execute(ctx, args).expect("Prune should succeed");

    let updated_doc = document_reader::read(&blocked_file_path).expect("Read updated doc");
    assert!(
        updated_doc.frontmatter.blocked_by.is_empty(),
        "blocked-by should be cleared after pruning the blocker"
    );
}

// ============================================================================
// Inline Link Error Tests
// ============================================================================

#[test]
fn prune_errors_on_inline_links_without_force() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");
    env.create_dir("api/docs");

    let (temp_dir, context) = env.into_parts();

    let task_id = create_task(&context, "api/", "Task to prune");
    close_task(temp_dir.path(), &task_id);

    let task_row =
        document_queries::lookup_by_id(&context.conn, &task_id).expect("Query").expect("Task row");
    let task_filename =
        Path::new(&task_row.path).file_name().unwrap().to_string_lossy().to_string();

    let linking_doc_content = format!(
        r#"---
lattice-id: LDOCABC
name: design-doc
description: Design document
created-at: 2026-01-01T00:00:00Z
updated-at: 2026-01-01T00:00:00Z
---

See the [pruned task](../tasks/.closed/{task_filename}#{task_id}) for details.
"#
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
        false,
    );
    document_queries::insert(&context.conn, &insert_doc).expect("Insert doc");

    let insert_link = link_queries::InsertLink {
        source_id: "LDOCABC",
        target_id: &task_id,
        link_type: link_queries::LinkType::Body,
        position: 0,
    };
    link_queries::insert_for_document(&context.conn, &[insert_link]).expect("Insert link");

    let args = prune_args_all();
    let global = GlobalOptions::default();
    let mut ctx = create_context(temp_dir.path(), &global).expect("Create context");
    ctx.client_id_store = Box::new(FakeClientIdStore::new("WQN"));

    let result = prune_command::execute(ctx, args);
    assert!(result.is_err(), "Prune should fail when inline links exist without --force");

    let err = result.unwrap_err();
    assert!(
        matches!(err, LatticeError::OperationNotAllowed { .. }),
        "Error should be OperationNotAllowed: {:?}",
        err
    );
}

#[test]
fn prune_with_force_converts_inline_links_to_plain_text() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");
    env.create_dir("api/docs");

    let (temp_dir, context) = env.into_parts();

    let task_id = create_task(&context, "api/", "Task to prune");
    close_task(temp_dir.path(), &task_id);

    let task_row =
        document_queries::lookup_by_id(&context.conn, &task_id).expect("Query").expect("Task row");
    let task_filename =
        Path::new(&task_row.path).file_name().unwrap().to_string_lossy().to_string();

    let linking_doc_content = format!(
        r#"---
lattice-id: LDOCABC
name: design-doc
description: Design document
created-at: 2026-01-01T00:00:00Z
updated-at: 2026-01-01T00:00:00Z
---

See the [pruned task](../tasks/.closed/{task_filename}#{task_id}) for details.
"#
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
        false,
    );
    document_queries::insert(&context.conn, &insert_doc).expect("Insert doc");

    let insert_link = link_queries::InsertLink {
        source_id: "LDOCABC",
        target_id: &task_id,
        link_type: link_queries::LinkType::Body,
        position: 0,
    };
    link_queries::insert_for_document(&context.conn, &[insert_link]).expect("Insert link");

    let args = PruneArgs { path: None, all: true, force: true, dry_run: false };
    let global = GlobalOptions::default();
    let mut ctx = create_context(temp_dir.path(), &global).expect("Create context");
    ctx.client_id_store = Box::new(FakeClientIdStore::new("WQN"));

    prune_command::execute(ctx, args).expect("Prune with force should succeed");

    let updated_content = fs::read_to_string(&doc_path).expect("Read updated doc");
    assert!(
        !updated_content.contains(&format!("[pruned task]")),
        "Link syntax should be removed: {}",
        updated_content
    );
    assert!(
        updated_content.contains("pruned task"),
        "Link text should remain as plain text: {}",
        updated_content
    );
}

// ============================================================================
// Batch Prune Tests
// ============================================================================

#[test]
fn prune_handles_multiple_closed_tasks() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let (temp_dir, context) = env.into_parts();

    let task1_id = create_task(&context, "api/", "First task");
    let task2_id = create_task(&context, "api/", "Second task");
    let task3_id = create_task(&context, "api/", "Third task");

    close_task(temp_dir.path(), &task1_id);
    close_task(temp_dir.path(), &task2_id);

    let args = prune_args_all();
    let global = GlobalOptions::default();
    let mut ctx = create_context(temp_dir.path(), &global).expect("Create context");
    ctx.client_id_store = Box::new(FakeClientIdStore::new("WQN"));

    prune_command::execute(ctx, args).expect("Prune should succeed");

    let doc1 = document_queries::lookup_by_id(&context.conn, &task1_id).expect("Query");
    let doc2 = document_queries::lookup_by_id(&context.conn, &task2_id).expect("Query");
    let doc3 = document_queries::lookup_by_id(&context.conn, &task3_id).expect("Query");

    assert!(doc1.is_none(), "First closed task should be deleted");
    assert!(doc2.is_none(), "Second closed task should be deleted");
    assert!(doc3.is_some(), "Open task should remain");
}

// ============================================================================
// Index Cleanup Tests
// ============================================================================

#[test]
fn prune_removes_all_link_entries() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let (temp_dir, context) = env.into_parts();

    let task_id = create_task(&context, "api/", "Task with links");
    close_task(temp_dir.path(), &task_id);

    let args = prune_args_all();
    let global = GlobalOptions::default();
    let mut ctx = create_context(temp_dir.path(), &global).expect("Create context");
    ctx.client_id_store = Box::new(FakeClientIdStore::new("WQN"));

    prune_command::execute(ctx, args).expect("Prune should succeed");

    let links_after = link_queries::count_outgoing(&context.conn, &task_id).expect("Count");
    let incoming_after = link_queries::count_incoming(&context.conn, &task_id).expect("Count");

    assert_eq!(links_after, 0, "All outgoing links should be deleted");
    assert_eq!(incoming_after, 0, "All incoming links should be deleted");
}

// ============================================================================
// Empty Result Tests
// ============================================================================

#[test]
fn prune_succeeds_with_no_closed_tasks() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let (temp_dir, context) = env.into_parts();

    let _task_id = create_task(&context, "api/", "Open task");

    let args = prune_args_all();
    let global = GlobalOptions::default();
    let mut ctx = create_context(temp_dir.path(), &global).expect("Create context");
    ctx.client_id_store = Box::new(FakeClientIdStore::new("WQN"));

    let result = prune_command::execute(ctx, args);
    assert!(result.is_ok(), "Prune should succeed with no closed tasks: {:?}", result);
}
