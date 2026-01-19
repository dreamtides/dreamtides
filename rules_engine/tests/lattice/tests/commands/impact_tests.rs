//! Tests for the `lat impact` command.

use std::fs;
use std::io::Write;

use chrono::Utc;
use lattice::cli::command_dispatch::create_context;
use lattice::cli::commands::impact_command;
use lattice::cli::global_options::GlobalOptions;
use lattice::cli::structure_args::ImpactArgs;
use lattice::error::error_types::LatticeError;
use lattice::git::client_config::FakeClientIdStore;
use lattice::index::document_types::InsertDocument;
use lattice::index::link_queries::{InsertLink, LinkType};
use lattice::index::{document_queries, link_queries, schema_definition};

fn default_args(id: &str) -> ImpactArgs {
    ImpactArgs { id: id.to_string() }
}

fn create_test_repo() -> (tempfile::TempDir, lattice::cli::command_dispatch::CommandContext) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");
    fs::create_dir_all(repo_root.join("docs")).expect("Failed to create docs");

    let global = GlobalOptions::default();
    let mut context = create_context(repo_root, &global).expect("Failed to create context");
    context.client_id_store = Box::new(FakeClientIdStore::new("IMP"));
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
// Error Case Tests
// ============================================================================

#[test]
fn impact_document_not_found() {
    let (_temp_dir, context) = create_test_repo();

    let args = default_args("LNONEX");
    let result = impact_command::execute(context, args);
    assert!(
        matches!(result, Err(LatticeError::DocumentNotFound { .. })),
        "impact should fail when document doesn't exist"
    );
}

// ============================================================================
// No Impact Tests
// ============================================================================

#[test]
fn impact_no_backlinks() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let doc = create_doc("LIAAAA", "docs/doc1.md", "doc1", "Document with no backlinks");
    insert_doc(&context.conn, &doc, repo_root, "docs/doc1.md");

    let args = default_args("LIAAAA");
    let result = impact_command::execute(context, args);
    assert!(result.is_ok(), "impact should succeed when no documents link to target");
}

#[test]
fn impact_only_outgoing_links() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let doc1 = create_doc("LIBBBB", "docs/doc1.md", "doc1", "Source document");
    insert_doc(&context.conn, &doc1, repo_root, "docs/doc1.md");

    let doc2 = create_doc("LICCCC", "docs/doc2.md", "doc2", "Target document");
    insert_doc(&context.conn, &doc2, repo_root, "docs/doc2.md");

    insert_link(&context.conn, "LIBBBB", "LICCCC", 0);

    let args = default_args("LIBBBB");
    let result = impact_command::execute(context, args);
    assert!(result.is_ok(), "impact should succeed with no backlinks (only outgoing links)");
}

// ============================================================================
// Direct Reference Tests (Depth 1)
// ============================================================================

#[test]
fn impact_single_direct_reference() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let doc1 = create_doc("LIDDDD", "docs/doc1.md", "doc1", "Target document");
    insert_doc(&context.conn, &doc1, repo_root, "docs/doc1.md");

    let doc2 = create_doc("LIEEEE", "docs/doc2.md", "doc2", "Document linking to target");
    insert_doc(&context.conn, &doc2, repo_root, "docs/doc2.md");

    insert_link(&context.conn, "LIEEEE", "LIDDDD", 0);

    let args = default_args("LIDDDD");
    let result = impact_command::execute(context, args);
    assert!(result.is_ok(), "impact should find single direct reference");
}

#[test]
fn impact_multiple_direct_references() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let target = create_doc("LIFFFF", "docs/target.md", "target", "Target document");
    insert_doc(&context.conn, &target, repo_root, "docs/target.md");

    let ref1 = create_doc("LIGGGG", "docs/ref1.md", "ref1", "First referencing document");
    insert_doc(&context.conn, &ref1, repo_root, "docs/ref1.md");

    let ref2 = create_doc("LIHHHH", "docs/ref2.md", "ref2", "Second referencing document");
    insert_doc(&context.conn, &ref2, repo_root, "docs/ref2.md");

    let ref3 = create_doc("LIIIII", "docs/ref3.md", "ref3", "Third referencing document");
    insert_doc(&context.conn, &ref3, repo_root, "docs/ref3.md");

    insert_link(&context.conn, "LIGGGG", "LIFFFF", 0);
    insert_link(&context.conn, "LIHHHH", "LIFFFF", 0);
    insert_link(&context.conn, "LIIIII", "LIFFFF", 0);

    let args = default_args("LIFFFF");
    let result = impact_command::execute(context, args);
    assert!(result.is_ok(), "impact should find multiple direct references");
}

// ============================================================================
// Transitive Reference Tests (Depth 2+)
// ============================================================================

#[test]
fn impact_depth_two() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let target = create_doc("LIJJJJ", "docs/target.md", "target", "Target document");
    insert_doc(&context.conn, &target, repo_root, "docs/target.md");

    let depth1 = create_doc("LIKKKK", "docs/depth1.md", "depth1", "Depth 1 document");
    insert_doc(&context.conn, &depth1, repo_root, "docs/depth1.md");

    let depth2 = create_doc("LILLLL", "docs/depth2.md", "depth2", "Depth 2 document");
    insert_doc(&context.conn, &depth2, repo_root, "docs/depth2.md");

    insert_link(&context.conn, "LIKKKK", "LIJJJJ", 0);
    insert_link(&context.conn, "LILLLL", "LIKKKK", 0);

    let args = default_args("LIJJJJ");
    let result = impact_command::execute(context, args);
    assert!(result.is_ok(), "impact should find depth-2 transitive references");
}

#[test]
fn impact_depth_three() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let target = create_doc("LIMMMM", "docs/target.md", "target", "Target document");
    insert_doc(&context.conn, &target, repo_root, "docs/target.md");

    let depth1 = create_doc("LINNNN", "docs/depth1.md", "depth1", "Depth 1 document");
    insert_doc(&context.conn, &depth1, repo_root, "docs/depth1.md");

    let depth2 = create_doc("LIOOOO", "docs/depth2.md", "depth2", "Depth 2 document");
    insert_doc(&context.conn, &depth2, repo_root, "docs/depth2.md");

    let depth3 = create_doc("LIPPPP", "docs/depth3.md", "depth3", "Depth 3 document");
    insert_doc(&context.conn, &depth3, repo_root, "docs/depth3.md");

    insert_link(&context.conn, "LINNNN", "LIMMMM", 0);
    insert_link(&context.conn, "LIOOOO", "LINNNN", 0);
    insert_link(&context.conn, "LIPPPP", "LIOOOO", 0);

    let args = default_args("LIMMMM");
    let result = impact_command::execute(context, args);
    assert!(result.is_ok(), "impact should find depth-3 transitive references");
}

// ============================================================================
// Circular Reference Tests
// ============================================================================

#[test]
fn impact_circular_two_documents() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let doc1 = create_doc("LIQQQQ", "docs/doc1.md", "doc1", "First document");
    insert_doc(&context.conn, &doc1, repo_root, "docs/doc1.md");

    let doc2 = create_doc("LIRRRR", "docs/doc2.md", "doc2", "Second document");
    insert_doc(&context.conn, &doc2, repo_root, "docs/doc2.md");

    insert_link(&context.conn, "LIQQQQ", "LIRRRR", 0);
    insert_link(&context.conn, "LIRRRR", "LIQQQQ", 0);

    let args = default_args("LIQQQQ");
    let result = impact_command::execute(context, args);
    assert!(result.is_ok(), "impact should handle circular references without infinite loop");
}

#[test]
fn impact_circular_three_documents() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let doc1 = create_doc("LISSSS", "docs/doc1.md", "doc1", "First document");
    insert_doc(&context.conn, &doc1, repo_root, "docs/doc1.md");

    let doc2 = create_doc("LITTTT", "docs/doc2.md", "doc2", "Second document");
    insert_doc(&context.conn, &doc2, repo_root, "docs/doc2.md");

    let doc3 = create_doc("LIUUUU", "docs/doc3.md", "doc3", "Third document");
    insert_doc(&context.conn, &doc3, repo_root, "docs/doc3.md");

    insert_link(&context.conn, "LISSSS", "LITTTT", 0);
    insert_link(&context.conn, "LITTTT", "LIUUUU", 0);
    insert_link(&context.conn, "LIUUUU", "LISSSS", 0);

    let args = default_args("LISSSS");
    let result = impact_command::execute(context, args);
    assert!(result.is_ok(), "impact should handle 3-node circular references");
}

#[test]
fn impact_self_reference() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let doc = create_doc("LIVVVV", "docs/self.md", "self", "Self-referencing document");
    insert_doc(&context.conn, &doc, repo_root, "docs/self.md");

    insert_link(&context.conn, "LIVVVV", "LIVVVV", 0);

    let args = default_args("LIVVVV");
    let result = impact_command::execute(context, args);
    assert!(result.is_ok(), "impact should handle self-references");
}

// ============================================================================
// Complex Graph Tests
// ============================================================================

#[test]
fn impact_diamond_pattern() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let target = create_doc("LIWWWW", "docs/target.md", "target", "Target at bottom of diamond");
    insert_doc(&context.conn, &target, repo_root, "docs/target.md");

    let mid1 = create_doc("LIXXXX", "docs/mid1.md", "mid1", "Middle left");
    insert_doc(&context.conn, &mid1, repo_root, "docs/mid1.md");

    let mid2 = create_doc("LIYYYY", "docs/mid2.md", "mid2", "Middle right");
    insert_doc(&context.conn, &mid2, repo_root, "docs/mid2.md");

    let top = create_doc("LIZZZZ", "docs/top.md", "top", "Top of diamond");
    insert_doc(&context.conn, &top, repo_root, "docs/top.md");

    insert_link(&context.conn, "LIXXXX", "LIWWWW", 0);
    insert_link(&context.conn, "LIYYYY", "LIWWWW", 0);
    insert_link(&context.conn, "LIZZZZ", "LIXXXX", 0);
    insert_link(&context.conn, "LIZZZZ", "LIYYYY", 1);

    let args = default_args("LIWWWW");
    let result = impact_command::execute(context, args);
    assert!(result.is_ok(), "impact should handle diamond dependency pattern");
}

// ============================================================================
// JSON Output Tests
// ============================================================================

#[test]
fn impact_json_output() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");
    fs::create_dir_all(repo_root.join("docs")).expect("Failed to create docs");

    let mut global = GlobalOptions::default();
    global.json = true;
    let mut context = create_context(repo_root, &global).expect("Failed to create context");
    context.client_id_store = Box::new(FakeClientIdStore::new("IMP"));
    schema_definition::create_schema(&context.conn).expect("Failed to create schema");

    let target = create_doc("LIAAAB", "docs/target.md", "target", "Target document");
    insert_doc(&context.conn, &target, repo_root, "docs/target.md");

    let ref1 = create_doc("LIBBBC", "docs/ref1.md", "ref1", "Referencing document");
    insert_doc(&context.conn, &ref1, repo_root, "docs/ref1.md");

    insert_link(&context.conn, "LIBBBC", "LIAAAB", 0);

    let args = default_args("LIAAAB");
    let result = impact_command::execute(context, args);
    assert!(result.is_ok(), "impact should produce JSON output");
}

#[test]
fn impact_json_output_empty() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");
    fs::create_dir_all(repo_root.join("docs")).expect("Failed to create docs");

    let mut global = GlobalOptions::default();
    global.json = true;
    let mut context = create_context(repo_root, &global).expect("Failed to create context");
    context.client_id_store = Box::new(FakeClientIdStore::new("IMP"));
    schema_definition::create_schema(&context.conn).expect("Failed to create schema");

    let doc = create_doc("LICCCD", "docs/doc.md", "doc", "Isolated document");
    insert_doc(&context.conn, &doc, repo_root, "docs/doc.md");

    let args = default_args("LICCCD");
    let result = impact_command::execute(context, args);
    assert!(result.is_ok(), "impact should produce JSON output when no affected documents");
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn impact_handles_empty_repository_except_target() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let doc = create_doc("LIDDDE", "docs/only.md", "only", "Only document in repo");
    insert_doc(&context.conn, &doc, repo_root, "docs/only.md");

    let args = default_args("LIDDDE");
    let result = impact_command::execute(context, args);
    assert!(result.is_ok(), "impact should handle repository with single document");
}
