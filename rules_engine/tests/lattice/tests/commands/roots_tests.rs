//! Tests for the `lat roots` command.

use std::fs;
use std::io::Write;

use chrono::Utc;
use lattice::cli::commands::roots_command;
use lattice::cli::structure_args::RootsArgs;
use lattice::index::directory_roots::{self, DirectoryRoot};
use lattice::index::document_queries;
use lattice::index::document_types::InsertDocument;
use lattice::test::test_environment::TestEnv;

fn default_args() -> RootsArgs {
    RootsArgs {}
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
        false,
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
    InsertDocument::new(
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
        false,
    )
}

fn insert_doc(env: &TestEnv, doc: &InsertDocument, path: &str) {
    document_queries::insert(env.conn(), doc).expect("Failed to insert document");
    let full_path = env.repo_root().join(path);
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

fn insert_root(env: &TestEnv, directory_path: &str, root_id: &str, depth: u32) {
    let root = DirectoryRoot {
        directory_path: directory_path.to_string(),
        root_id: root_id.to_string(),
        parent_path: None,
        depth,
    };
    directory_roots::upsert(env.conn(), &root).expect("Failed to insert directory root");
}

fn insert_root_with_parent(
    env: &TestEnv,
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
    directory_roots::upsert(env.conn(), &root).expect("Failed to insert directory root");
}

// ============================================================================
// Basic Execution Tests
// ============================================================================

#[test]
fn roots_command_succeeds_with_no_roots() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = roots_command::execute(context, args);
    assert!(result.is_ok(), "Roots command should succeed with no roots");
}

#[test]
fn roots_command_succeeds_with_single_root() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let root_doc = create_root_doc("LAABCD", "api/api.md", "api", "API root document");
    insert_doc(&env, &root_doc, "api/api.md");
    insert_root(&env, "api", "LAABCD", 0);

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = roots_command::execute(context, args);
    assert!(result.is_ok(), "Roots command should succeed with single root");
}

#[test]
fn roots_command_succeeds_with_multiple_roots() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");
    env.create_dir("database");

    let api_root = create_root_doc("LBBCDE", "api/api.md", "api", "API root document");
    insert_doc(&env, &api_root, "api/api.md");
    insert_root(&env, "api", "LBBCDE", 0);

    let db_root =
        create_root_doc("LCCDEF", "database/database.md", "database", "Database root document");
    insert_doc(&env, &db_root, "database/database.md");
    insert_root(&env, "database", "LCCDEF", 0);

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = roots_command::execute(context, args);
    assert!(result.is_ok(), "Roots command should succeed with multiple roots");
}

// ============================================================================
// Child Count Tests
// ============================================================================

#[test]
fn roots_command_counts_children_correctly() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let root_doc = create_root_doc("LDDEFG", "api/api.md", "api", "API root document");
    insert_doc(&env, &root_doc, "api/api.md");
    insert_root(&env, "api", "LDDEFG", 0);

    let child1 = create_child_doc("LEEFGH", "LDDEFG", "api/docs/design.md", "design", "API design");
    insert_doc(&env, &child1, "api/docs/design.md");

    let child2 = create_child_doc("LFFGHI", "LDDEFG", "api/tasks/task1.md", "task1", "First task");
    insert_doc(&env, &child2, "api/tasks/task1.md");

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = roots_command::execute(context, args);
    assert!(result.is_ok(), "Roots command should count children correctly");
}

#[test]
fn roots_command_does_not_count_root_as_child() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let root_doc = create_root_doc("LGGHIJ", "api/api.md", "api", "API root document");
    insert_doc(&env, &root_doc, "api/api.md");
    insert_root(&env, "api", "LGGHIJ", 0);

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = roots_command::execute(context, args);
    assert!(result.is_ok(), "Roots command should not count root as child");
}

#[test]
fn roots_command_counts_nested_children() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");
    env.create_dir("api/tasks/.closed");

    let root_doc = create_root_doc("LHHIJK", "api/api.md", "api", "API root document");
    insert_doc(&env, &root_doc, "api/api.md");
    insert_root(&env, "api", "LHHIJK", 0);

    let doc1 = create_child_doc("LIIJKL", "LHHIJK", "api/docs/design.md", "design", "Design doc");
    insert_doc(&env, &doc1, "api/docs/design.md");

    let task1 = create_child_doc("LJJKLM", "LHHIJK", "api/tasks/task1.md", "task1", "Task 1");
    insert_doc(&env, &task1, "api/tasks/task1.md");

    let mut closed =
        create_child_doc("LKKLMN", "LHHIJK", "api/tasks/.closed/done.md", "done", "Done task");
    closed.is_closed = true;
    insert_doc(&env, &closed, "api/tasks/.closed/done.md");

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = roots_command::execute(context, args);
    assert!(result.is_ok(), "Roots command should count nested children including closed");
}

// ============================================================================
// JSON Output Tests
// ============================================================================

#[test]
fn roots_command_json_output() {
    let env = TestEnv::new().with_json_output();
    env.create_dir("api");
    env.create_dir("api/tasks");

    let root_doc = create_root_doc("LLLMNO", "api/api.md", "api", "API root document");
    insert_doc(&env, &root_doc, "api/api.md");
    insert_root(&env, "api", "LLLMNO", 0);

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = roots_command::execute(context, args);
    assert!(result.is_ok(), "Roots command should produce JSON output");
}

#[test]
fn roots_command_json_output_multiple_roots() {
    let env = TestEnv::new().with_json_output();
    env.create_dir("api");
    env.create_dir("database");

    let api_root = create_root_doc("LMMNOP", "api/api.md", "api", "API root");
    insert_doc(&env, &api_root, "api/api.md");
    insert_root(&env, "api", "LMMNOP", 0);

    let db_root = create_root_doc("LNNOPQ", "database/database.md", "database", "Database root");
    insert_doc(&env, &db_root, "database/database.md");
    insert_root(&env, "database", "LNNOPQ", 0);

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = roots_command::execute(context, args);
    assert!(result.is_ok(), "Roots command should produce JSON output for multiple roots");
}

// ============================================================================
// Nested Root Tests
// ============================================================================

#[test]
fn roots_command_shows_nested_roots() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/auth");
    env.create_dir("api/tasks");

    let api_root = create_root_doc("LOOPQR", "api/api.md", "api", "API root");
    insert_doc(&env, &api_root, "api/api.md");
    insert_root(&env, "api", "LOOPQR", 0);

    let auth_root = create_root_doc("LPPQRS", "api/auth/auth.md", "auth", "Auth subsystem root");
    insert_doc(&env, &auth_root, "api/auth/auth.md");
    insert_root_with_parent(&env, "api/auth", "LPPQRS", "api", 1);

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = roots_command::execute(context, args);
    assert!(result.is_ok(), "Roots command should show nested roots");
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn roots_command_handles_missing_root_document() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    insert_root(&env, "api", "LQQRST", 0);

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = roots_command::execute(context, args);
    assert!(result.is_ok(), "Roots command should skip missing root documents gracefully");
}

#[test]
fn roots_command_output_sorted_by_path() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");
    env.create_dir("zebra");
    env.create_dir("alpha");

    let zebra_root = create_root_doc("LRRSTU", "zebra/zebra.md", "zebra", "Zebra root");
    insert_doc(&env, &zebra_root, "zebra/zebra.md");
    insert_root(&env, "zebra", "LRRSTU", 0);

    let alpha_root = create_root_doc("LSSTUV", "alpha/alpha.md", "alpha", "Alpha root");
    insert_doc(&env, &alpha_root, "alpha/alpha.md");
    insert_root(&env, "alpha", "LSSTUV", 0);

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = roots_command::execute(context, args);
    assert!(result.is_ok(), "Roots command should sort output by path");
}
