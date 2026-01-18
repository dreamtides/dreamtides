//! Tests for the `lat tree` command.

use std::fs;
use std::io::Write;

use chrono::Utc;
use lattice::cli::command_dispatch::create_context;
use lattice::cli::commands::tree_command;
use lattice::cli::global_options::GlobalOptions;
use lattice::cli::structure_args::TreeArgs;
use lattice::document::frontmatter_schema::TaskType;
use lattice::index::document_types::InsertDocument;
use lattice::index::{document_queries, schema_definition};

fn default_args() -> TreeArgs {
    TreeArgs { path: None, depth: None, counts: false, tasks_only: false, docs_only: false }
}

fn create_test_repo() -> (tempfile::TempDir, lattice::cli::command_dispatch::CommandContext) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");
    fs::create_dir_all(repo_root.join("api/tasks")).expect("Failed to create api/tasks");
    fs::create_dir_all(repo_root.join("api/tasks/.closed")).expect("Failed to create .closed");
    fs::create_dir_all(repo_root.join("api/docs")).expect("Failed to create api/docs");
    fs::create_dir_all(repo_root.join("database/tasks")).expect("Failed to create database/tasks");
    fs::create_dir_all(repo_root.join("database/docs")).expect("Failed to create database/docs");

    let global = GlobalOptions::default();
    let context = create_context(repo_root, &global).expect("Failed to create context");
    schema_definition::create_schema(&context.conn).expect("Failed to create schema");

    (temp_dir, context)
}

fn create_task_doc(
    id: &str,
    path: &str,
    name: &str,
    description: &str,
    priority: u8,
    task_type: TaskType,
) -> InsertDocument {
    let mut doc = InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        name.to_string(),
        description.to_string(),
        Some(task_type),
        Some(priority),
        Some(Utc::now()),
        None,
        None,
        format!("hash-{id}"),
        100,
    );
    doc.in_tasks_dir = path.contains("/tasks/");
    doc
}

fn create_kb_doc(id: &str, path: &str, name: &str, description: &str) -> InsertDocument {
    let mut doc = InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        name.to_string(),
        description.to_string(),
        None,
        None,
        Some(Utc::now()),
        None,
        None,
        format!("hash-{id}"),
        100,
    );
    doc.in_docs_dir = path.contains("/docs/");
    doc
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
// Basic Execution Tests
// ============================================================================

#[test]
fn tree_command_succeeds_with_no_documents() {
    let (_temp_dir, context) = create_test_repo();

    let args = default_args();
    let result = tree_command::execute(context, args);
    assert!(result.is_ok(), "Tree command should succeed with no documents");
}

#[test]
fn tree_command_succeeds_with_documents() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let task = create_task_doc(
        "LAABCD",
        "api/tasks/task1.md",
        "task-one",
        "First task",
        2,
        TaskType::Task,
    );
    insert_doc(&context.conn, &task, repo_root, "api/tasks/task1.md");

    let kb = create_kb_doc("LBBCDE", "api/docs/design.md", "design-doc", "API design");
    insert_doc(&context.conn, &kb, repo_root, "api/docs/design.md");

    let args = default_args();
    let result = tree_command::execute(context, args);
    assert!(result.is_ok(), "Tree command should succeed with documents");
}

#[test]
fn tree_command_succeeds_with_json_output() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");
    fs::create_dir_all(repo_root.join("api/tasks")).expect("Failed to create api/tasks");

    let mut global = GlobalOptions::default();
    global.json = true;
    let context = create_context(repo_root, &global).expect("Failed to create context");
    schema_definition::create_schema(&context.conn).expect("Failed to create schema");

    let task =
        create_task_doc("LCCDFF", "api/tasks/task1.md", "task-one", "First task", 1, TaskType::Bug);
    insert_doc(&context.conn, &task, repo_root, "api/tasks/task1.md");

    let args = default_args();
    let result = tree_command::execute(context, args);
    assert!(result.is_ok(), "Tree command should succeed with JSON output");
}

// ============================================================================
// Path Filter Tests
// ============================================================================

#[test]
fn tree_command_filters_by_path() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let api_task = create_task_doc(
        "LDDEFG",
        "api/tasks/api_task.md",
        "api-task",
        "API task",
        2,
        TaskType::Task,
    );
    insert_doc(&context.conn, &api_task, repo_root, "api/tasks/api_task.md");

    let db_task = create_task_doc(
        "LEEFGH",
        "database/tasks/db_task.md",
        "db-task",
        "DB task",
        2,
        TaskType::Task,
    );
    insert_doc(&context.conn, &db_task, repo_root, "database/tasks/db_task.md");

    let args = TreeArgs { path: Some("api/".to_string()), ..default_args() };
    let result = tree_command::execute(context, args);
    assert!(result.is_ok(), "Tree command should filter by path");
}

// ============================================================================
// Depth Limit Tests
// ============================================================================

#[test]
fn tree_command_respects_depth_limit() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let task = create_task_doc(
        "LFFGHI",
        "api/tasks/task1.md",
        "task-one",
        "First task",
        2,
        TaskType::Task,
    );
    insert_doc(&context.conn, &task, repo_root, "api/tasks/task1.md");

    let args = TreeArgs { depth: Some(1), ..default_args() };
    let result = tree_command::execute(context, args);
    assert!(result.is_ok(), "Tree command should respect depth limit");
}

#[test]
fn tree_command_depth_zero_shows_only_top_level() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let task = create_task_doc(
        "LGGHIJ",
        "api/tasks/task1.md",
        "task-one",
        "First task",
        2,
        TaskType::Task,
    );
    insert_doc(&context.conn, &task, repo_root, "api/tasks/task1.md");

    let args = TreeArgs { depth: Some(0), ..default_args() };
    let result = tree_command::execute(context, args);
    assert!(result.is_ok(), "Tree command should handle depth 0");
}

// ============================================================================
// Filter Option Tests
// ============================================================================

#[test]
fn tree_command_tasks_only_filter() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let task = create_task_doc(
        "LHHIJK",
        "api/tasks/task1.md",
        "task-one",
        "First task",
        2,
        TaskType::Task,
    );
    insert_doc(&context.conn, &task, repo_root, "api/tasks/task1.md");

    let kb = create_kb_doc("LIIJKL", "api/docs/design.md", "design-doc", "API design");
    insert_doc(&context.conn, &kb, repo_root, "api/docs/design.md");

    let args = TreeArgs { tasks_only: true, ..default_args() };
    let result = tree_command::execute(context, args);
    assert!(result.is_ok(), "Tree command should filter to tasks only");
}

#[test]
fn tree_command_docs_only_filter() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let task = create_task_doc(
        "LJJKLM",
        "api/tasks/task1.md",
        "task-one",
        "First task",
        2,
        TaskType::Task,
    );
    insert_doc(&context.conn, &task, repo_root, "api/tasks/task1.md");

    let kb = create_kb_doc("LKKLMN", "api/docs/design.md", "design-doc", "API design");
    insert_doc(&context.conn, &kb, repo_root, "api/docs/design.md");

    let args = TreeArgs { docs_only: true, ..default_args() };
    let result = tree_command::execute(context, args);
    assert!(result.is_ok(), "Tree command should filter to docs only");
}

#[test]
fn tree_command_conflicting_filters_returns_error() {
    let (_temp_dir, context) = create_test_repo();

    let args = TreeArgs { tasks_only: true, docs_only: true, ..default_args() };
    let result = tree_command::execute(context, args);
    assert!(result.is_err(), "Tree command should error on conflicting filters");
}

// ============================================================================
// Counts Option Tests
// ============================================================================

#[test]
fn tree_command_with_counts_option() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let task1 = create_task_doc(
        "LLLMNO",
        "api/tasks/task1.md",
        "task-one",
        "First task",
        2,
        TaskType::Task,
    );
    insert_doc(&context.conn, &task1, repo_root, "api/tasks/task1.md");

    let task2 = create_task_doc(
        "LMMNOP",
        "api/tasks/task2.md",
        "task-two",
        "Second task",
        1,
        TaskType::Bug,
    );
    insert_doc(&context.conn, &task2, repo_root, "api/tasks/task2.md");

    let args = TreeArgs { counts: true, ..default_args() };
    let result = tree_command::execute(context, args);
    assert!(result.is_ok(), "Tree command should succeed with counts option");
}

// ============================================================================
// Closed Document Tests
// ============================================================================

#[test]
fn tree_command_shows_closed_documents() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let open =
        create_task_doc("LNNOPQ", "api/tasks/open.md", "open-task", "Open task", 2, TaskType::Task);
    insert_doc(&context.conn, &open, repo_root, "api/tasks/open.md");

    let mut closed = create_task_doc(
        "LOOPQR",
        "api/tasks/.closed/done.md",
        "done-task",
        "Done task",
        2,
        TaskType::Task,
    );
    closed.is_closed = true;
    insert_doc(&context.conn, &closed, repo_root, "api/tasks/.closed/done.md");

    let args = default_args();
    let result = tree_command::execute(context, args);
    assert!(result.is_ok(), "Tree command should show closed documents");
}

// ============================================================================
// Multiple Directory Tests
// ============================================================================

#[test]
fn tree_command_shows_multiple_directories() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let api_task = create_task_doc(
        "LPPQRS",
        "api/tasks/api_task.md",
        "api-task",
        "API task",
        2,
        TaskType::Task,
    );
    insert_doc(&context.conn, &api_task, repo_root, "api/tasks/api_task.md");

    let api_doc = create_kb_doc("LQQRST", "api/docs/api_design.md", "api-design", "API design");
    insert_doc(&context.conn, &api_doc, repo_root, "api/docs/api_design.md");

    let db_task = create_task_doc(
        "LRRSTU",
        "database/tasks/db_task.md",
        "db-task",
        "DB task",
        1,
        TaskType::Feature,
    );
    insert_doc(&context.conn, &db_task, repo_root, "database/tasks/db_task.md");

    let db_doc = create_kb_doc("LSSTUV", "database/docs/schema.md", "schema-doc", "DB schema");
    insert_doc(&context.conn, &db_doc, repo_root, "database/docs/schema.md");

    let args = default_args();
    let result = tree_command::execute(context, args);
    assert!(result.is_ok(), "Tree command should show multiple directories");
}
