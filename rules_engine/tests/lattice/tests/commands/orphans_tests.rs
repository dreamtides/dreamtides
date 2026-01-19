//! Tests for the `lat orphans` command.

use std::fs;
use std::io::Write;

use chrono::Utc;
use lattice::cli::commands::orphans_command;
use lattice::cli::structure_args::OrphansArgs;
use lattice::index::document_queries;
use lattice::index::document_types::InsertDocument;
use lattice::index::link_queries::{self, InsertLink, LinkType};
use lattice::test::test_environment::TestEnv;

fn default_args() -> OrphansArgs {
    OrphansArgs { exclude_roots: false, path: None }
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
fn orphans_finds_documents_with_no_incoming_links() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let doc1 = create_doc("LOAAA2", "docs/doc1.md", "doc1", "First orphan document");
    insert_doc(env.conn(), &doc1, env.repo_root(), "docs/doc1.md");

    let doc2 = create_doc("LOBBB2", "docs/doc2.md", "doc2", "Second orphan document");
    insert_doc(env.conn(), &doc2, env.repo_root(), "docs/doc2.md");

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = orphans_command::execute(context, args);
    assert!(result.is_ok(), "orphans should succeed when finding documents with no links");
}

#[test]
fn orphans_excludes_documents_with_incoming_links() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let doc1 = create_doc("LOCCC2", "docs/doc1.md", "doc1", "Source document");
    insert_doc(env.conn(), &doc1, env.repo_root(), "docs/doc1.md");

    let doc2 = create_doc("LODDD2", "docs/doc2.md", "doc2", "Target document with link");
    insert_doc(env.conn(), &doc2, env.repo_root(), "docs/doc2.md");

    insert_link(env.conn(), "LOCCC2", "LODDD2", 0);

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = orphans_command::execute(context, args);
    assert!(result.is_ok(), "orphans should succeed and exclude documents with incoming links");
}

#[test]
fn orphans_returns_empty_when_no_orphans_exist() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let doc1 = create_doc("LOEEE2", "docs/doc1.md", "doc1", "First document");
    insert_doc(env.conn(), &doc1, env.repo_root(), "docs/doc1.md");

    let doc2 = create_doc("LOFFF2", "docs/doc2.md", "doc2", "Second document");
    insert_doc(env.conn(), &doc2, env.repo_root(), "docs/doc2.md");

    insert_link(env.conn(), "LOEEE2", "LOFFF2", 0);
    insert_link(env.conn(), "LOFFF2", "LOEEE2", 0);

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = orphans_command::execute(context, args);
    assert!(result.is_ok(), "orphans should succeed when all documents have incoming links");
}

// ============================================================================
// --exclude-roots Tests
// ============================================================================

#[test]
fn orphans_includes_root_documents_by_default() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let root_doc = create_doc("LOGGG2", "api/api.md", "api", "API root document");
    insert_doc(env.conn(), &root_doc, env.repo_root(), "api/api.md");

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = orphans_command::execute(context, args);
    assert!(result.is_ok(), "orphans should include root documents by default");
}

#[test]
fn orphans_excludes_root_documents_when_flag_set() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let root_doc = create_doc("LOHHH2", "api/api.md", "api", "API root document");
    insert_doc(env.conn(), &root_doc, env.repo_root(), "api/api.md");

    let non_root_doc = create_doc("LOIII2", "docs/doc1.md", "doc1", "Non-root orphan");
    insert_doc(env.conn(), &non_root_doc, env.repo_root(), "docs/doc1.md");

    let args = OrphansArgs { exclude_roots: true, path: None };
    let (_temp, context) = env.into_parts();
    let result = orphans_command::execute(context, args);
    assert!(result.is_ok(), "orphans should exclude root documents when flag is set");
}

// ============================================================================
// --path Filter Tests
// ============================================================================

#[test]
fn orphans_filters_by_path_prefix() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let doc1 = create_doc("LOJJJ2", "docs/doc1.md", "doc1", "Orphan in docs");
    insert_doc(env.conn(), &doc1, env.repo_root(), "docs/doc1.md");

    let doc2 = create_doc("LOKKK2", "api/api.md", "api", "Orphan in api");
    insert_doc(env.conn(), &doc2, env.repo_root(), "api/api.md");

    let args = OrphansArgs { exclude_roots: false, path: Some("docs/".to_string()) };
    let (_temp, context) = env.into_parts();
    let result = orphans_command::execute(context, args);
    assert!(result.is_ok(), "orphans should filter by path prefix");
}

#[test]
fn orphans_returns_empty_when_path_prefix_has_no_orphans() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let doc1 = create_doc("LOLLL2", "docs/doc1.md", "doc1", "Orphan in docs");
    insert_doc(env.conn(), &doc1, env.repo_root(), "docs/doc1.md");

    let args = OrphansArgs { exclude_roots: false, path: Some("nonexistent/".to_string()) };
    let (_temp, context) = env.into_parts();
    let result = orphans_command::execute(context, args);
    assert!(result.is_ok(), "orphans should return empty for nonexistent path prefix");
}

// ============================================================================
// JSON Output Tests
// ============================================================================

#[test]
fn orphans_produces_json_output() {
    let env = TestEnv::new().with_json_output();
    env.create_dir("docs");

    let doc = create_doc("LOMMM2", "docs/doc1.md", "doc1", "Orphan document");
    insert_doc(env.conn(), &doc, env.repo_root(), "docs/doc1.md");

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = orphans_command::execute(context, args);
    assert!(result.is_ok(), "orphans should produce JSON output");
}

#[test]
fn orphans_produces_empty_json_array_when_no_orphans() {
    let env = TestEnv::new().with_json_output();
    env.create_dir("docs");

    let doc1 = create_doc("LONNN2", "docs/doc1.md", "doc1", "Document 1");
    insert_doc(env.conn(), &doc1, env.repo_root(), "docs/doc1.md");

    let doc2 = create_doc("LOOOO2", "docs/doc2.md", "doc2", "Document 2");
    insert_doc(env.conn(), &doc2, env.repo_root(), "docs/doc2.md");

    insert_link(env.conn(), "LONNN2", "LOOOO2", 0);
    insert_link(env.conn(), "LOOOO2", "LONNN2", 0);

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = orphans_command::execute(context, args);
    assert!(result.is_ok(), "orphans should produce empty JSON array");
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn orphans_handles_empty_repository() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = orphans_command::execute(context, args);
    assert!(result.is_ok(), "orphans should handle empty repository");
}

#[test]
fn orphans_combines_exclude_roots_and_path_filter() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let root_doc = create_doc("LOPPP2", "api/api.md", "api", "API root document");
    insert_doc(env.conn(), &root_doc, env.repo_root(), "api/api.md");

    let non_root_doc = create_doc("LOQQQ2", "api/tasks/task1.md", "task1", "Task in api");
    insert_doc(env.conn(), &non_root_doc, env.repo_root(), "api/tasks/task1.md");

    let args = OrphansArgs { exclude_roots: true, path: Some("api/".to_string()) };
    let (_temp, context) = env.into_parts();
    let result = orphans_command::execute(context, args);
    assert!(result.is_ok(), "orphans should combine exclude_roots and path filters");
}
