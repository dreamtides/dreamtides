//! Tests for the `lat path` command.

use std::fs;
use std::io::Write;

use chrono::Utc;
use lattice::cli::command_dispatch::create_context;
use lattice::cli::commands::path_command;
use lattice::cli::global_options::GlobalOptions;
use lattice::cli::structure_args::PathArgs;
use lattice::error::error_types::LatticeError;
use lattice::git::client_config::FakeClientIdStore;
use lattice::index::document_types::InsertDocument;
use lattice::index::link_queries::{InsertLink, LinkType};
use lattice::index::{document_queries, link_queries, schema_definition};

fn default_args(id1: &str, id2: &str) -> PathArgs {
    PathArgs { id1: id1.to_string(), id2: id2.to_string() }
}

fn create_test_repo() -> (tempfile::TempDir, lattice::cli::command_dispatch::CommandContext) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");
    fs::create_dir_all(repo_root.join("docs")).expect("Failed to create docs");

    let global = GlobalOptions::default();
    let mut context = create_context(repo_root, &global).expect("Failed to create context");
    context.client_id_store = Box::new(FakeClientIdStore::new("PTH"));
    schema_definition::create_schema(&context.conn).expect("Failed to create schema");

    (temp_dir, context)
}

fn create_doc(id: &str, path: &str, name: &str, description: &str) -> InsertDocument {
    InsertDocument::new(
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

fn insert_link(conn: &rusqlite::Connection, source_id: &str, target_id: &str, position: u32) {
    let link = InsertLink { source_id, target_id, link_type: LinkType::Body, position };
    link_queries::insert_for_document(conn, &[link]).expect("Failed to insert link");
}

// ============================================================================
// Same Document Tests
// ============================================================================

#[test]
fn path_same_document() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let doc = create_doc("LPAAAA", "docs/doc1.md", "doc1", "First document");
    insert_doc(&context.conn, &doc, repo_root, "docs/doc1.md");

    let args = default_args("LPAAAA", "LPAAAA");
    let result = path_command::execute(context, args);
    assert!(result.is_ok(), "path should succeed when source equals target");
}

// ============================================================================
// Direct Path Tests
// ============================================================================

#[test]
fn path_direct_link() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let doc1 = create_doc("LPBBBB", "docs/doc1.md", "doc1", "First document");
    insert_doc(&context.conn, &doc1, repo_root, "docs/doc1.md");

    let doc2 = create_doc("LPCCCC", "docs/doc2.md", "doc2", "Second document");
    insert_doc(&context.conn, &doc2, repo_root, "docs/doc2.md");

    insert_link(&context.conn, "LPBBBB", "LPCCCC", 0);

    let args = default_args("LPBBBB", "LPCCCC");
    let result = path_command::execute(context, args);
    assert!(result.is_ok(), "path should find direct link");
}

// ============================================================================
// Multi-Hop Path Tests
// ============================================================================

#[test]
fn path_two_hops() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let doc1 = create_doc("LPDDDD", "docs/doc1.md", "doc1", "First document");
    insert_doc(&context.conn, &doc1, repo_root, "docs/doc1.md");

    let doc2 = create_doc("LPEEEE", "docs/doc2.md", "doc2", "Second document");
    insert_doc(&context.conn, &doc2, repo_root, "docs/doc2.md");

    let doc3 = create_doc("LPFFFF", "docs/doc3.md", "doc3", "Third document");
    insert_doc(&context.conn, &doc3, repo_root, "docs/doc3.md");

    insert_link(&context.conn, "LPDDDD", "LPEEEE", 0);
    insert_link(&context.conn, "LPEEEE", "LPFFFF", 0);

    let args = default_args("LPDDDD", "LPFFFF");
    let result = path_command::execute(context, args);
    assert!(result.is_ok(), "path should find two-hop path");
}

#[test]
fn path_three_hops() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let doc1 = create_doc("LPGGGG", "docs/doc1.md", "doc1", "First document");
    insert_doc(&context.conn, &doc1, repo_root, "docs/doc1.md");

    let doc2 = create_doc("LPHHHH", "docs/doc2.md", "doc2", "Second document");
    insert_doc(&context.conn, &doc2, repo_root, "docs/doc2.md");

    let doc3 = create_doc("LPIIII", "docs/doc3.md", "doc3", "Third document");
    insert_doc(&context.conn, &doc3, repo_root, "docs/doc3.md");

    let doc4 = create_doc("LPJJJJ", "docs/doc4.md", "doc4", "Fourth document");
    insert_doc(&context.conn, &doc4, repo_root, "docs/doc4.md");

    insert_link(&context.conn, "LPGGGG", "LPHHHH", 0);
    insert_link(&context.conn, "LPHHHH", "LPIIII", 0);
    insert_link(&context.conn, "LPIIII", "LPJJJJ", 0);

    let args = default_args("LPGGGG", "LPJJJJ");
    let result = path_command::execute(context, args);
    assert!(result.is_ok(), "path should find three-hop path");
}

// ============================================================================
// No Path Tests
// ============================================================================

#[test]
fn path_no_connection() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let doc1 = create_doc("LPKKKK", "docs/doc1.md", "doc1", "First document");
    insert_doc(&context.conn, &doc1, repo_root, "docs/doc1.md");

    let doc2 = create_doc("LPLLLL", "docs/doc2.md", "doc2", "Second document");
    insert_doc(&context.conn, &doc2, repo_root, "docs/doc2.md");

    let args = default_args("LPKKKK", "LPLLLL");
    let result = path_command::execute(context, args);
    assert!(result.is_ok(), "path command should succeed even when no path exists");
}

#[test]
fn path_reverse_direction_not_found() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let doc1 = create_doc("LPMMMM", "docs/doc1.md", "doc1", "First document");
    insert_doc(&context.conn, &doc1, repo_root, "docs/doc1.md");

    let doc2 = create_doc("LPNNNN", "docs/doc2.md", "doc2", "Second document");
    insert_doc(&context.conn, &doc2, repo_root, "docs/doc2.md");

    insert_link(&context.conn, "LPMMMM", "LPNNNN", 0);

    let args = default_args("LPNNNN", "LPMMMM");
    let result = path_command::execute(context, args);
    assert!(result.is_ok(), "path command should succeed when reverse path doesn't exist");
}

// ============================================================================
// Error Case Tests
// ============================================================================

#[test]
fn path_source_not_found() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let doc = create_doc("LPOOOO", "docs/doc1.md", "doc1", "First document");
    insert_doc(&context.conn, &doc, repo_root, "docs/doc1.md");

    let args = default_args("LNONEX", "LPOOOO");
    let result = path_command::execute(context, args);
    assert!(
        matches!(result, Err(LatticeError::DocumentNotFound { .. })),
        "path should fail when source document doesn't exist"
    );
}

#[test]
fn path_target_not_found() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let doc = create_doc("LPPPPP", "docs/doc1.md", "doc1", "First document");
    insert_doc(&context.conn, &doc, repo_root, "docs/doc1.md");

    let args = default_args("LPPPPP", "LNONEX");
    let result = path_command::execute(context, args);
    assert!(
        matches!(result, Err(LatticeError::DocumentNotFound { .. })),
        "path should fail when target document doesn't exist"
    );
}

// ============================================================================
// Shortest Path Tests
// ============================================================================

#[test]
fn path_finds_shortest_when_multiple_paths_exist() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let doc1 = create_doc("LPQQQQ", "docs/doc1.md", "doc1", "First document");
    insert_doc(&context.conn, &doc1, repo_root, "docs/doc1.md");

    let doc2 = create_doc("LPRRRR", "docs/doc2.md", "doc2", "Second document");
    insert_doc(&context.conn, &doc2, repo_root, "docs/doc2.md");

    let doc3 = create_doc("LPSSSS", "docs/doc3.md", "doc3", "Third document");
    insert_doc(&context.conn, &doc3, repo_root, "docs/doc3.md");

    let doc4 = create_doc("LPTTTT", "docs/doc4.md", "doc4", "Fourth document");
    insert_doc(&context.conn, &doc4, repo_root, "docs/doc4.md");

    insert_link(&context.conn, "LPQQQQ", "LPTTTT", 0);
    insert_link(&context.conn, "LPQQQQ", "LPRRRR", 1);
    insert_link(&context.conn, "LPRRRR", "LPSSSS", 0);
    insert_link(&context.conn, "LPSSSS", "LPTTTT", 0);

    let args = default_args("LPQQQQ", "LPTTTT");
    let result = path_command::execute(context, args);
    assert!(result.is_ok(), "path should find shortest path when multiple paths exist");
}

// ============================================================================
// JSON Output Tests
// ============================================================================

#[test]
fn path_json_output_with_path() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");
    fs::create_dir_all(repo_root.join("docs")).expect("Failed to create docs");

    let mut global = GlobalOptions::default();
    global.json = true;
    let mut context = create_context(repo_root, &global).expect("Failed to create context");
    context.client_id_store = Box::new(FakeClientIdStore::new("PTH"));
    schema_definition::create_schema(&context.conn).expect("Failed to create schema");

    let doc1 = create_doc("LPUUUU", "docs/doc1.md", "doc1", "First document");
    insert_doc(&context.conn, &doc1, repo_root, "docs/doc1.md");

    let doc2 = create_doc("LPVVVV", "docs/doc2.md", "doc2", "Second document");
    insert_doc(&context.conn, &doc2, repo_root, "docs/doc2.md");

    insert_link(&context.conn, "LPUUUU", "LPVVVV", 0);

    let args = default_args("LPUUUU", "LPVVVV");
    let result = path_command::execute(context, args);
    assert!(result.is_ok(), "path should produce JSON output");
}

#[test]
fn path_json_output_no_path() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");
    fs::create_dir_all(repo_root.join("docs")).expect("Failed to create docs");

    let mut global = GlobalOptions::default();
    global.json = true;
    let mut context = create_context(repo_root, &global).expect("Failed to create context");
    context.client_id_store = Box::new(FakeClientIdStore::new("PTH"));
    schema_definition::create_schema(&context.conn).expect("Failed to create schema");

    let doc1 = create_doc("LPWWWW", "docs/doc1.md", "doc1", "First document");
    insert_doc(&context.conn, &doc1, repo_root, "docs/doc1.md");

    let doc2 = create_doc("LPXXXX", "docs/doc2.md", "doc2", "Second document");
    insert_doc(&context.conn, &doc2, repo_root, "docs/doc2.md");

    let args = default_args("LPWWWW", "LPXXXX");
    let result = path_command::execute(context, args);
    assert!(result.is_ok(), "path should produce JSON output even when no path exists");
}
