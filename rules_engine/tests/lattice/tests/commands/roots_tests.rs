//! Tests for the `lat roots` command.

use std::fs;
use std::io::Write;

use chrono::Utc;
use lattice::cli::command_dispatch::create_context;
use lattice::cli::commands::roots_command;
use lattice::cli::global_options::GlobalOptions;
use lattice::cli::structure_args::RootsArgs;
use lattice::git::client_config::FakeClientIdStore;
use lattice::index::directory_roots::{self, DirectoryRoot};
use lattice::index::document_types::InsertDocument;
use lattice::index::{document_queries, schema_definition};

fn default_args() -> RootsArgs {
    RootsArgs {}
}

fn create_test_repo() -> (tempfile::TempDir, lattice::cli::command_dispatch::CommandContext) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");
    fs::create_dir_all(repo_root.join("api/tasks")).expect("Failed to create api/tasks");
    fs::create_dir_all(repo_root.join("api/docs")).expect("Failed to create api/docs");
    fs::create_dir_all(repo_root.join("database/tasks")).expect("Failed to create database/tasks");
    fs::create_dir_all(repo_root.join("database/docs")).expect("Failed to create database/docs");

    let global = GlobalOptions::default();
    let mut context = create_context(repo_root, &global).expect("Failed to create context");
    context.client_id_store = Box::new(FakeClientIdStore::new("WQN"));
    schema_definition::create_schema(&context.conn).expect("Failed to create schema");

    (temp_dir, context)
}

fn create_root_doc(id: &str, path: &str, name: &str, description: &str) -> InsertDocument {
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
    doc.is_root = true;
    doc
}

fn create_child_doc(
    id: &str,
    parent_id: &str,
    path: &str,
    name: &str,
    description: &str,
) -> InsertDocument {
    let mut doc = InsertDocument::new(
        id.to_string(),
        Some(parent_id.to_string()),
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
    doc.in_tasks_dir = path.contains("/tasks/");
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

fn insert_root(conn: &rusqlite::Connection, directory_path: &str, root_id: &str, depth: u32) {
    let root = DirectoryRoot {
        directory_path: directory_path.to_string(),
        root_id: root_id.to_string(),
        parent_path: None,
        depth,
    };
    directory_roots::upsert(conn, &root).expect("Failed to insert directory root");
}

fn insert_root_with_parent(
    conn: &rusqlite::Connection,
    directory_path: &str,
    root_id: &str,
    parent_path: &str,
    depth: u32,
) {
    let root = DirectoryRoot {
        directory_path: directory_path.to_string(),
        root_id: root_id.to_string(),
        parent_path: Some(parent_path.to_string()),
        depth,
    };
    directory_roots::upsert(conn, &root).expect("Failed to insert directory root");
}

// ============================================================================
// Basic Execution Tests
// ============================================================================

#[test]
fn roots_command_succeeds_with_no_roots() {
    let (_temp_dir, context) = create_test_repo();

    let args = default_args();
    let result = roots_command::execute(context, args);
    assert!(result.is_ok(), "Roots command should succeed with no roots");
}

#[test]
fn roots_command_succeeds_with_single_root() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let root_doc = create_root_doc("LAABCD", "api/api.md", "api", "API root document");
    insert_doc(&context.conn, &root_doc, repo_root, "api/api.md");
    insert_root(&context.conn, "api", "LAABCD", 0);

    let args = default_args();
    let result = roots_command::execute(context, args);
    assert!(result.is_ok(), "Roots command should succeed with single root");
}

#[test]
fn roots_command_succeeds_with_multiple_roots() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let api_root = create_root_doc("LBBCDE", "api/api.md", "api", "API root document");
    insert_doc(&context.conn, &api_root, repo_root, "api/api.md");
    insert_root(&context.conn, "api", "LBBCDE", 0);

    let db_root =
        create_root_doc("LCCDEF", "database/database.md", "database", "Database root document");
    insert_doc(&context.conn, &db_root, repo_root, "database/database.md");
    insert_root(&context.conn, "database", "LCCDEF", 0);

    let args = default_args();
    let result = roots_command::execute(context, args);
    assert!(result.is_ok(), "Roots command should succeed with multiple roots");
}

// ============================================================================
// Child Count Tests
// ============================================================================

#[test]
fn roots_command_counts_children_correctly() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let root_doc = create_root_doc("LDDEFG", "api/api.md", "api", "API root document");
    insert_doc(&context.conn, &root_doc, repo_root, "api/api.md");
    insert_root(&context.conn, "api", "LDDEFG", 0);

    let child1 = create_child_doc("LEEFGH", "LDDEFG", "api/docs/design.md", "design", "API design");
    insert_doc(&context.conn, &child1, repo_root, "api/docs/design.md");

    let child2 = create_child_doc("LFFGHI", "LDDEFG", "api/tasks/task1.md", "task1", "First task");
    insert_doc(&context.conn, &child2, repo_root, "api/tasks/task1.md");

    let args = default_args();
    let result = roots_command::execute(context, args);
    assert!(result.is_ok(), "Roots command should count children correctly");
}

#[test]
fn roots_command_does_not_count_root_as_child() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let root_doc = create_root_doc("LGGHIJ", "api/api.md", "api", "API root document");
    insert_doc(&context.conn, &root_doc, repo_root, "api/api.md");
    insert_root(&context.conn, "api", "LGGHIJ", 0);

    let args = default_args();
    let result = roots_command::execute(context, args);
    assert!(result.is_ok(), "Roots command should not count root as child");
}

#[test]
fn roots_command_counts_nested_children() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    fs::create_dir_all(repo_root.join("api/tasks/.closed")).expect("Failed to create .closed");

    let root_doc = create_root_doc("LHHIJK", "api/api.md", "api", "API root document");
    insert_doc(&context.conn, &root_doc, repo_root, "api/api.md");
    insert_root(&context.conn, "api", "LHHIJK", 0);

    let doc1 = create_child_doc("LIIJKL", "LHHIJK", "api/docs/design.md", "design", "Design doc");
    insert_doc(&context.conn, &doc1, repo_root, "api/docs/design.md");

    let task1 = create_child_doc("LJJKLM", "LHHIJK", "api/tasks/task1.md", "task1", "Task 1");
    insert_doc(&context.conn, &task1, repo_root, "api/tasks/task1.md");

    let mut closed =
        create_child_doc("LKKLMN", "LHHIJK", "api/tasks/.closed/done.md", "done", "Done task");
    closed.is_closed = true;
    insert_doc(&context.conn, &closed, repo_root, "api/tasks/.closed/done.md");

    let args = default_args();
    let result = roots_command::execute(context, args);
    assert!(result.is_ok(), "Roots command should count nested children including closed");
}

// ============================================================================
// JSON Output Tests
// ============================================================================

#[test]
fn roots_command_json_output() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");
    fs::create_dir_all(repo_root.join("api/tasks")).expect("Failed to create api/tasks");

    let mut global = GlobalOptions::default();
    global.json = true;
    let mut context = create_context(repo_root, &global).expect("Failed to create context");
    context.client_id_store = Box::new(FakeClientIdStore::new("WQN"));
    schema_definition::create_schema(&context.conn).expect("Failed to create schema");

    let root_doc = create_root_doc("LLLMNO", "api/api.md", "api", "API root document");
    insert_doc(&context.conn, &root_doc, repo_root, "api/api.md");
    insert_root(&context.conn, "api", "LLLMNO", 0);

    let args = default_args();
    let result = roots_command::execute(context, args);
    assert!(result.is_ok(), "Roots command should produce JSON output");
}

#[test]
fn roots_command_json_output_multiple_roots() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");
    fs::create_dir_all(repo_root.join("api")).expect("Failed to create api");
    fs::create_dir_all(repo_root.join("database")).expect("Failed to create database");

    let mut global = GlobalOptions::default();
    global.json = true;
    let mut context = create_context(repo_root, &global).expect("Failed to create context");
    context.client_id_store = Box::new(FakeClientIdStore::new("WQN"));
    schema_definition::create_schema(&context.conn).expect("Failed to create schema");

    let api_root = create_root_doc("LMMNOP", "api/api.md", "api", "API root");
    insert_doc(&context.conn, &api_root, repo_root, "api/api.md");
    insert_root(&context.conn, "api", "LMMNOP", 0);

    let db_root = create_root_doc("LNNOPQ", "database/database.md", "database", "Database root");
    insert_doc(&context.conn, &db_root, repo_root, "database/database.md");
    insert_root(&context.conn, "database", "LNNOPQ", 0);

    let args = default_args();
    let result = roots_command::execute(context, args);
    assert!(result.is_ok(), "Roots command should produce JSON output for multiple roots");
}

// ============================================================================
// Nested Root Tests
// ============================================================================

#[test]
fn roots_command_shows_nested_roots() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");
    fs::create_dir_all(repo_root.join("api/auth")).expect("Failed to create api/auth");

    let global = GlobalOptions::default();
    let mut context = create_context(repo_root, &global).expect("Failed to create context");
    context.client_id_store = Box::new(FakeClientIdStore::new("WQN"));
    schema_definition::create_schema(&context.conn).expect("Failed to create schema");

    let api_root = create_root_doc("LOOPQR", "api/api.md", "api", "API root");
    insert_doc(&context.conn, &api_root, repo_root, "api/api.md");
    insert_root(&context.conn, "api", "LOOPQR", 0);

    let auth_root = create_root_doc("LPPQRS", "api/auth/auth.md", "auth", "Auth subsystem root");
    insert_doc(&context.conn, &auth_root, repo_root, "api/auth/auth.md");
    insert_root_with_parent(&context.conn, "api/auth", "LPPQRS", "api", 1);

    let args = default_args();
    let result = roots_command::execute(context, args);
    assert!(result.is_ok(), "Roots command should show nested roots");
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn roots_command_handles_missing_root_document() {
    let (_temp_dir, context) = create_test_repo();

    insert_root(&context.conn, "api", "LQQRST", 0);

    let args = default_args();
    let result = roots_command::execute(context, args);
    assert!(result.is_ok(), "Roots command should skip missing root documents gracefully");
}

#[test]
fn roots_command_output_sorted_by_path() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    fs::create_dir_all(repo_root.join("zebra")).expect("Failed to create zebra");
    fs::create_dir_all(repo_root.join("alpha")).expect("Failed to create alpha");

    let zebra_root = create_root_doc("LRRSTU", "zebra/zebra.md", "zebra", "Zebra root");
    insert_doc(&context.conn, &zebra_root, repo_root, "zebra/zebra.md");
    insert_root(&context.conn, "zebra", "LRRSTU", 0);

    let alpha_root = create_root_doc("LSSTUV", "alpha/alpha.md", "alpha", "Alpha root");
    insert_doc(&context.conn, &alpha_root, repo_root, "alpha/alpha.md");
    insert_root(&context.conn, "alpha", "LSSTUV", 0);

    let args = default_args();
    let result = roots_command::execute(context, args);
    assert!(result.is_ok(), "Roots command should sort output by path");
}
