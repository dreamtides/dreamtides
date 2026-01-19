//! Tests for the `lat links-from` command.

use std::fs;
use std::io::Write;

use chrono::Utc;
use lattice::cli::commands::links_from;
use lattice::cli::structure_args::LinksFromArgs;
use lattice::error::error_types::LatticeError;
use lattice::index::document_queries;
use lattice::index::document_types::InsertDocument;
use lattice::index::link_queries::{self, InsertLink, LinkType};
use lattice::test::test_environment::TestEnv;

fn default_args(id: &str) -> LinksFromArgs {
    LinksFromArgs { id: id.to_string() }
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
// Basic Execution Tests
// ============================================================================

#[test]
fn links_from_succeeds_with_no_outgoing_links() {
    let env = TestEnv::new();
    env.create_dir("docs");

    let doc = create_doc("LAABCD", "docs/doc1.md", "doc1", "First document");
    insert_doc(env.conn(), &doc, env.repo_root(), "docs/doc1.md");

    let args = default_args("LAABCD");
    let (_temp, context) = env.into_parts();
    let result = links_from::execute(context, args);
    assert!(result.is_ok(), "links-from should succeed with no outgoing links");
}

#[test]
fn links_from_lists_single_outgoing_link() {
    let env = TestEnv::new();
    env.create_dir("docs");

    let doc1 = create_doc("LBBCDE", "docs/doc1.md", "doc1", "First document");
    insert_doc(env.conn(), &doc1, env.repo_root(), "docs/doc1.md");

    let doc2 = create_doc("LCCDEF", "docs/doc2.md", "doc2", "Second document");
    insert_doc(env.conn(), &doc2, env.repo_root(), "docs/doc2.md");

    insert_link(env.conn(), "LBBCDE", "LCCDEF", 0);

    let args = default_args("LBBCDE");
    let (_temp, context) = env.into_parts();
    let result = links_from::execute(context, args);
    assert!(result.is_ok(), "links-from should list single outgoing link");
}

#[test]
fn links_from_lists_multiple_outgoing_links() {
    let env = TestEnv::new();
    env.create_dir("docs");

    let doc1 = create_doc("LDDEFG", "docs/doc1.md", "doc1", "First document");
    insert_doc(env.conn(), &doc1, env.repo_root(), "docs/doc1.md");

    let doc2 = create_doc("LEEFGH", "docs/doc2.md", "doc2", "Second document");
    insert_doc(env.conn(), &doc2, env.repo_root(), "docs/doc2.md");

    let doc3 = create_doc("LFFGHI", "docs/doc3.md", "doc3", "Third document");
    insert_doc(env.conn(), &doc3, env.repo_root(), "docs/doc3.md");

    insert_link(env.conn(), "LDDEFG", "LEEFGH", 0);
    insert_link(env.conn(), "LDDEFG", "LFFGHI", 1);

    let args = default_args("LDDEFG");
    let (_temp, context) = env.into_parts();
    let result = links_from::execute(context, args);
    assert!(result.is_ok(), "links-from should list multiple outgoing links");
}

// ============================================================================
// Link Type Tests
// ============================================================================

#[test]
fn links_from_shows_different_link_types() {
    let env = TestEnv::new();
    env.create_dir("docs");

    let doc1 = create_doc("LGGHIJ", "docs/doc1.md", "doc1", "First document");
    insert_doc(env.conn(), &doc1, env.repo_root(), "docs/doc1.md");

    let doc2 = create_doc("LHHIJK", "docs/doc2.md", "doc2", "Second document");
    insert_doc(env.conn(), &doc2, env.repo_root(), "docs/doc2.md");

    let doc3 = create_doc("LIIJKL", "docs/doc3.md", "doc3", "Third document");
    insert_doc(env.conn(), &doc3, env.repo_root(), "docs/doc3.md");

    let body_link = InsertLink {
        source_id: "LGGHIJ",
        target_id: "LHHIJK",
        link_type: LinkType::Body,
        position: 0,
    };
    let blocked_by_link = InsertLink {
        source_id: "LGGHIJ",
        target_id: "LIIJKL",
        link_type: LinkType::BlockedBy,
        position: 1,
    };
    link_queries::insert_for_document(env.conn(), &[body_link, blocked_by_link])
        .expect("Failed to insert links");

    let args = default_args("LGGHIJ");
    let (_temp, context) = env.into_parts();
    let result = links_from::execute(context, args);
    assert!(result.is_ok(), "links-from should show different link types");
}

// ============================================================================
// Error Case Tests
// ============================================================================

#[test]
fn links_from_fails_for_nonexistent_id() {
    let env = TestEnv::new();
    env.create_dir("docs");

    let args = default_args("LJJKLM");
    let (_temp, context) = env.into_parts();
    let result = links_from::execute(context, args);
    assert!(
        matches!(result, Err(LatticeError::DocumentNotFound { .. })),
        "links-from should fail for nonexistent ID"
    );
}

// ============================================================================
// Dangling Link Tests
// ============================================================================

#[test]
fn links_from_handles_dangling_links_gracefully() {
    let env = TestEnv::new();
    env.create_dir("docs");

    let doc1 = create_doc("LKKLMN", "docs/doc1.md", "doc1", "First document");
    insert_doc(env.conn(), &doc1, env.repo_root(), "docs/doc1.md");

    let doc2 = create_doc("LLLMNO", "docs/doc2.md", "doc2", "Second document");
    insert_doc(env.conn(), &doc2, env.repo_root(), "docs/doc2.md");

    insert_link(env.conn(), "LKKLMN", "LLLMNO", 0);
    insert_link(env.conn(), "LKKLMN", "LNONEXISTENT", 1);

    let args = default_args("LKKLMN");
    let (_temp, context) = env.into_parts();
    let result = links_from::execute(context, args);
    assert!(result.is_ok(), "links-from should handle dangling links gracefully: {:?}", result);
}

// ============================================================================
// JSON Output Tests
// ============================================================================

#[test]
fn links_from_json_output() {
    let env = TestEnv::new().with_json_output();
    env.create_dir("docs");

    let doc1 = create_doc("LMMNOP", "docs/doc1.md", "doc1", "First document");
    insert_doc(env.conn(), &doc1, env.repo_root(), "docs/doc1.md");

    let doc2 = create_doc("LNNOPQ", "docs/doc2.md", "doc2", "Second document");
    insert_doc(env.conn(), &doc2, env.repo_root(), "docs/doc2.md");

    insert_link(env.conn(), "LMMNOP", "LNNOPQ", 0);

    let args = default_args("LMMNOP");
    let (_temp, context) = env.into_parts();
    let result = links_from::execute(context, args);
    assert!(result.is_ok(), "links-from should produce JSON output");
}

#[test]
fn links_from_json_output_empty() {
    let env = TestEnv::new().with_json_output();
    env.create_dir("docs");

    let doc = create_doc("LOOPQR", "docs/doc1.md", "doc1", "First document");
    insert_doc(env.conn(), &doc, env.repo_root(), "docs/doc1.md");

    let args = default_args("LOOPQR");
    let (_temp, context) = env.into_parts();
    let result = links_from::execute(context, args);
    assert!(result.is_ok(), "links-from should produce empty JSON array");
}
