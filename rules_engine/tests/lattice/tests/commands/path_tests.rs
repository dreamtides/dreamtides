//! Tests for the `lat path` command.

use std::fs;
use std::io::Write;

use chrono::Utc;
use lattice::cli::commands::path_command;
use lattice::cli::structure_args::PathArgs;
use lattice::error::error_types::LatticeError;
use lattice::index::document_queries;
use lattice::index::document_types::InsertDocument;
use lattice::index::link_queries::{self, InsertLink, LinkType};
use lattice::test::test_environment::TestEnv;

fn default_args(id1: &str, id2: &str) -> PathArgs {
    PathArgs { id1: id1.to_string(), id2: id2.to_string() }
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
    let env = TestEnv::new();
    env.create_dir("docs");

    let doc = create_doc("LPAAAA", "docs/doc1.md", "doc1", "First document");
    insert_doc(env.conn(), &doc, env.repo_root(), "docs/doc1.md");

    let args = default_args("LPAAAA", "LPAAAA");
    let (_temp, context) = env.into_parts();
    let result = path_command::execute(context, args);
    assert!(result.is_ok(), "path should succeed when source equals target");
}

// ============================================================================
// Direct Path Tests
// ============================================================================

#[test]
fn path_direct_link() {
    let env = TestEnv::new();
    env.create_dir("docs");

    let doc1 = create_doc("LPBBBB", "docs/doc1.md", "doc1", "First document");
    insert_doc(env.conn(), &doc1, env.repo_root(), "docs/doc1.md");

    let doc2 = create_doc("LPCCCC", "docs/doc2.md", "doc2", "Second document");
    insert_doc(env.conn(), &doc2, env.repo_root(), "docs/doc2.md");

    insert_link(env.conn(), "LPBBBB", "LPCCCC", 0);

    let args = default_args("LPBBBB", "LPCCCC");
    let (_temp, context) = env.into_parts();
    let result = path_command::execute(context, args);
    assert!(result.is_ok(), "path should find direct link");
}

// ============================================================================
// Multi-Hop Path Tests
// ============================================================================

#[test]
fn path_two_hops() {
    let env = TestEnv::new();
    env.create_dir("docs");

    let doc1 = create_doc("LPDDDD", "docs/doc1.md", "doc1", "First document");
    insert_doc(env.conn(), &doc1, env.repo_root(), "docs/doc1.md");

    let doc2 = create_doc("LPEEEE", "docs/doc2.md", "doc2", "Second document");
    insert_doc(env.conn(), &doc2, env.repo_root(), "docs/doc2.md");

    let doc3 = create_doc("LPFFFF", "docs/doc3.md", "doc3", "Third document");
    insert_doc(env.conn(), &doc3, env.repo_root(), "docs/doc3.md");

    insert_link(env.conn(), "LPDDDD", "LPEEEE", 0);
    insert_link(env.conn(), "LPEEEE", "LPFFFF", 0);

    let args = default_args("LPDDDD", "LPFFFF");
    let (_temp, context) = env.into_parts();
    let result = path_command::execute(context, args);
    assert!(result.is_ok(), "path should find two-hop path");
}

#[test]
fn path_three_hops() {
    let env = TestEnv::new();
    env.create_dir("docs");

    let doc1 = create_doc("LPGGGG", "docs/doc1.md", "doc1", "First document");
    insert_doc(env.conn(), &doc1, env.repo_root(), "docs/doc1.md");

    let doc2 = create_doc("LPHHHH", "docs/doc2.md", "doc2", "Second document");
    insert_doc(env.conn(), &doc2, env.repo_root(), "docs/doc2.md");

    let doc3 = create_doc("LPIIII", "docs/doc3.md", "doc3", "Third document");
    insert_doc(env.conn(), &doc3, env.repo_root(), "docs/doc3.md");

    let doc4 = create_doc("LPJJJJ", "docs/doc4.md", "doc4", "Fourth document");
    insert_doc(env.conn(), &doc4, env.repo_root(), "docs/doc4.md");

    insert_link(env.conn(), "LPGGGG", "LPHHHH", 0);
    insert_link(env.conn(), "LPHHHH", "LPIIII", 0);
    insert_link(env.conn(), "LPIIII", "LPJJJJ", 0);

    let args = default_args("LPGGGG", "LPJJJJ");
    let (_temp, context) = env.into_parts();
    let result = path_command::execute(context, args);
    assert!(result.is_ok(), "path should find three-hop path");
}

// ============================================================================
// No Path Tests
// ============================================================================

#[test]
fn path_no_connection() {
    let env = TestEnv::new();
    env.create_dir("docs");

    let doc1 = create_doc("LPKKKK", "docs/doc1.md", "doc1", "First document");
    insert_doc(env.conn(), &doc1, env.repo_root(), "docs/doc1.md");

    let doc2 = create_doc("LPLLLL", "docs/doc2.md", "doc2", "Second document");
    insert_doc(env.conn(), &doc2, env.repo_root(), "docs/doc2.md");

    let args = default_args("LPKKKK", "LPLLLL");
    let (_temp, context) = env.into_parts();
    let result = path_command::execute(context, args);
    assert!(result.is_ok(), "path command should succeed even when no path exists");
}

#[test]
fn path_reverse_direction_not_found() {
    let env = TestEnv::new();
    env.create_dir("docs");

    let doc1 = create_doc("LPMMMM", "docs/doc1.md", "doc1", "First document");
    insert_doc(env.conn(), &doc1, env.repo_root(), "docs/doc1.md");

    let doc2 = create_doc("LPNNNN", "docs/doc2.md", "doc2", "Second document");
    insert_doc(env.conn(), &doc2, env.repo_root(), "docs/doc2.md");

    insert_link(env.conn(), "LPMMMM", "LPNNNN", 0);

    let args = default_args("LPNNNN", "LPMMMM");
    let (_temp, context) = env.into_parts();
    let result = path_command::execute(context, args);
    assert!(result.is_ok(), "path command should succeed when reverse path doesn't exist");
}

// ============================================================================
// Error Case Tests
// ============================================================================

#[test]
fn path_source_not_found() {
    let env = TestEnv::new();
    env.create_dir("docs");

    let doc = create_doc("LPOOOO", "docs/doc1.md", "doc1", "First document");
    insert_doc(env.conn(), &doc, env.repo_root(), "docs/doc1.md");

    let args = default_args("LNONEX", "LPOOOO");
    let (_temp, context) = env.into_parts();
    let result = path_command::execute(context, args);
    assert!(
        matches!(result, Err(LatticeError::DocumentNotFound { .. })),
        "path should fail when source document doesn't exist"
    );
}

#[test]
fn path_target_not_found() {
    let env = TestEnv::new();
    env.create_dir("docs");

    let doc = create_doc("LPPPPP", "docs/doc1.md", "doc1", "First document");
    insert_doc(env.conn(), &doc, env.repo_root(), "docs/doc1.md");

    let args = default_args("LPPPPP", "LNONEX");
    let (_temp, context) = env.into_parts();
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
    let env = TestEnv::new();
    env.create_dir("docs");

    let doc1 = create_doc("LPQQQQ", "docs/doc1.md", "doc1", "First document");
    insert_doc(env.conn(), &doc1, env.repo_root(), "docs/doc1.md");

    let doc2 = create_doc("LPRRRR", "docs/doc2.md", "doc2", "Second document");
    insert_doc(env.conn(), &doc2, env.repo_root(), "docs/doc2.md");

    let doc3 = create_doc("LPSSSS", "docs/doc3.md", "doc3", "Third document");
    insert_doc(env.conn(), &doc3, env.repo_root(), "docs/doc3.md");

    let doc4 = create_doc("LPTTTT", "docs/doc4.md", "doc4", "Fourth document");
    insert_doc(env.conn(), &doc4, env.repo_root(), "docs/doc4.md");

    insert_link(env.conn(), "LPQQQQ", "LPTTTT", 0);
    insert_link(env.conn(), "LPQQQQ", "LPRRRR", 1);
    insert_link(env.conn(), "LPRRRR", "LPSSSS", 0);
    insert_link(env.conn(), "LPSSSS", "LPTTTT", 0);

    let args = default_args("LPQQQQ", "LPTTTT");
    let (_temp, context) = env.into_parts();
    let result = path_command::execute(context, args);
    assert!(result.is_ok(), "path should find shortest path when multiple paths exist");
}

// ============================================================================
// JSON Output Tests
// ============================================================================

#[test]
fn path_json_output_with_path() {
    let env = TestEnv::new().with_json_output();
    env.create_dir("docs");

    let doc1 = create_doc("LPUUUU", "docs/doc1.md", "doc1", "First document");
    insert_doc(env.conn(), &doc1, env.repo_root(), "docs/doc1.md");

    let doc2 = create_doc("LPVVVV", "docs/doc2.md", "doc2", "Second document");
    insert_doc(env.conn(), &doc2, env.repo_root(), "docs/doc2.md");

    insert_link(env.conn(), "LPUUUU", "LPVVVV", 0);

    let args = default_args("LPUUUU", "LPVVVV");
    let (_temp, context) = env.into_parts();
    let result = path_command::execute(context, args);
    assert!(result.is_ok(), "path should produce JSON output");
}

#[test]
fn path_json_output_no_path() {
    let env = TestEnv::new().with_json_output();
    env.create_dir("docs");

    let doc1 = create_doc("LPWWWW", "docs/doc1.md", "doc1", "First document");
    insert_doc(env.conn(), &doc1, env.repo_root(), "docs/doc1.md");

    let doc2 = create_doc("LPXXXX", "docs/doc2.md", "doc2", "Second document");
    insert_doc(env.conn(), &doc2, env.repo_root(), "docs/doc2.md");

    let args = default_args("LPWWWW", "LPXXXX");
    let (_temp, context) = env.into_parts();
    let result = path_command::execute(context, args);
    assert!(result.is_ok(), "path should produce JSON output even when no path exists");
}
