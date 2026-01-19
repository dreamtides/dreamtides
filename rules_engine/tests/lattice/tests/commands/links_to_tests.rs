//! Tests for the `lat links-to` command.

use std::fs;
use std::io::Write;

use chrono::Utc;
use lattice::cli::command_dispatch::create_context;
use lattice::cli::commands::links_to;
use lattice::cli::global_options::GlobalOptions;
use lattice::cli::structure_args::LinksToArgs;
use lattice::error::error_types::LatticeError;
use lattice::git::client_config::FakeClientIdStore;
use lattice::index::document_types::InsertDocument;
use lattice::index::link_queries::{InsertLink, LinkType};
use lattice::index::{document_queries, link_queries, schema_definition};

fn default_args(id: &str) -> LinksToArgs {
    LinksToArgs { id: id.to_string() }
}

fn create_test_repo() -> (tempfile::TempDir, lattice::cli::command_dispatch::CommandContext) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");
    fs::create_dir_all(repo_root.join("docs")).expect("Failed to create docs");

    let global = GlobalOptions::default();
    let mut context = create_context(repo_root, &global).expect("Failed to create context");
    context.client_id_store = Box::new(FakeClientIdStore::new("WQN"));
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
// Basic Execution Tests
// ============================================================================

#[test]
fn links_to_succeeds_with_no_incoming_links() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let doc = create_doc("LTAAA2", "docs/doc1.md", "doc1", "First document");
    insert_doc(&context.conn, &doc, repo_root, "docs/doc1.md");

    let args = default_args("LTAAA2");
    let result = links_to::execute(context, args);
    assert!(result.is_ok(), "links-to should succeed with no incoming links");
}

#[test]
fn links_to_lists_single_incoming_link() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let doc1 = create_doc("LTBBB2", "docs/doc1.md", "doc1", "First document");
    insert_doc(&context.conn, &doc1, repo_root, "docs/doc1.md");

    let doc2 = create_doc("LTCCC2", "docs/doc2.md", "doc2", "Second document");
    insert_doc(&context.conn, &doc2, repo_root, "docs/doc2.md");

    insert_link(&context.conn, "LTBBB2", "LTCCC2", 0);

    let args = default_args("LTCCC2");
    let result = links_to::execute(context, args);
    assert!(result.is_ok(), "links-to should list single incoming link");
}

#[test]
fn links_to_lists_multiple_incoming_links() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let doc1 = create_doc("LTDDD2", "docs/doc1.md", "doc1", "First document");
    insert_doc(&context.conn, &doc1, repo_root, "docs/doc1.md");

    let doc2 = create_doc("LTEEE2", "docs/doc2.md", "doc2", "Second document");
    insert_doc(&context.conn, &doc2, repo_root, "docs/doc2.md");

    let doc3 = create_doc("LTFFF2", "docs/doc3.md", "doc3", "Third document");
    insert_doc(&context.conn, &doc3, repo_root, "docs/doc3.md");

    insert_link(&context.conn, "LTDDD2", "LTFFF2", 0);
    insert_link(&context.conn, "LTEEE2", "LTFFF2", 0);

    let args = default_args("LTFFF2");
    let result = links_to::execute(context, args);
    assert!(result.is_ok(), "links-to should list multiple incoming links");
}

// ============================================================================
// Link Type Tests
// ============================================================================

#[test]
fn links_to_shows_different_link_types() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let doc1 = create_doc("LTGGG2", "docs/doc1.md", "doc1", "First document");
    insert_doc(&context.conn, &doc1, repo_root, "docs/doc1.md");

    let doc2 = create_doc("LTHHH2", "docs/doc2.md", "doc2", "Second document");
    insert_doc(&context.conn, &doc2, repo_root, "docs/doc2.md");

    let doc3 = create_doc("LTIII2", "docs/doc3.md", "doc3", "Third document");
    insert_doc(&context.conn, &doc3, repo_root, "docs/doc3.md");

    let body_link = InsertLink {
        source_id: "LTGGG2",
        target_id: "LTIII2",
        link_type: LinkType::Body,
        position: 0,
    };
    let blocked_by_link = InsertLink {
        source_id: "LTHHH2",
        target_id: "LTIII2",
        link_type: LinkType::BlockedBy,
        position: 0,
    };
    link_queries::insert_for_document(&context.conn, &[body_link, blocked_by_link])
        .expect("Failed to insert links");

    let args = default_args("LTIII2");
    let result = links_to::execute(context, args);
    assert!(result.is_ok(), "links-to should show different link types");
}

// ============================================================================
// Error Case Tests
// ============================================================================

#[test]
fn links_to_fails_for_nonexistent_id() {
    let (_temp_dir, context) = create_test_repo();

    let args = default_args("LTJJJ2");
    let result = links_to::execute(context, args);
    assert!(
        matches!(result, Err(LatticeError::DocumentNotFound { .. })),
        "links-to should fail for nonexistent ID"
    );
}

// ============================================================================
// Dangling Link Tests
// ============================================================================

#[test]
fn links_to_handles_dangling_links_gracefully() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let doc1 = create_doc("LTKKK2", "docs/doc1.md", "doc1", "First document");
    insert_doc(&context.conn, &doc1, repo_root, "docs/doc1.md");

    let doc2 = create_doc("LTLLL2", "docs/doc2.md", "doc2", "Second document");
    insert_doc(&context.conn, &doc2, repo_root, "docs/doc2.md");

    insert_link(&context.conn, "LTKKK2", "LTLLL2", 0);
    insert_link(&context.conn, "LNONEXISTENT", "LTLLL2", 0);

    let args = default_args("LTLLL2");
    let result = links_to::execute(context, args);
    assert!(result.is_ok(), "links-to should handle dangling links gracefully: {:?}", result);
}

// ============================================================================
// JSON Output Tests
// ============================================================================

#[test]
fn links_to_json_output() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");
    fs::create_dir_all(repo_root.join("docs")).expect("Failed to create docs");

    let mut global = GlobalOptions::default();
    global.json = true;
    let mut context = create_context(repo_root, &global).expect("Failed to create context");
    context.client_id_store = Box::new(FakeClientIdStore::new("WQN"));
    schema_definition::create_schema(&context.conn).expect("Failed to create schema");

    let doc1 = create_doc("LTMMM2", "docs/doc1.md", "doc1", "First document");
    insert_doc(&context.conn, &doc1, repo_root, "docs/doc1.md");

    let doc2 = create_doc("LTNNN2", "docs/doc2.md", "doc2", "Second document");
    insert_doc(&context.conn, &doc2, repo_root, "docs/doc2.md");

    insert_link(&context.conn, "LTMMM2", "LTNNN2", 0);

    let args = default_args("LTNNN2");
    let result = links_to::execute(context, args);
    assert!(result.is_ok(), "links-to should produce JSON output");
}

#[test]
fn links_to_json_output_empty() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");
    fs::create_dir_all(repo_root.join("docs")).expect("Failed to create docs");

    let mut global = GlobalOptions::default();
    global.json = true;
    let mut context = create_context(repo_root, &global).expect("Failed to create context");
    context.client_id_store = Box::new(FakeClientIdStore::new("WQN"));
    schema_definition::create_schema(&context.conn).expect("Failed to create schema");

    let doc = create_doc("LTOOO2", "docs/doc1.md", "doc1", "First document");
    insert_doc(&context.conn, &doc, repo_root, "docs/doc1.md");

    let args = default_args("LTOOO2");
    let result = links_to::execute(context, args);
    assert!(result.is_ok(), "links-to should produce empty JSON array");
}
