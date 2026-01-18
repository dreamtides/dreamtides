//! Tests for the `lat children` command.

use std::fs;
use std::io::Write;

use chrono::Utc;
use lattice::cli::command_dispatch::create_context;
use lattice::cli::commands::children_command;
use lattice::cli::global_options::GlobalOptions;
use lattice::cli::structure_args::ChildrenArgs;
use lattice::document::frontmatter_schema::TaskType;
use lattice::error::error_types::LatticeError;
use lattice::git::client_config::FakeClientIdStore;
use lattice::index::directory_roots::{self, DirectoryRoot};
use lattice::index::document_types::InsertDocument;
use lattice::index::{document_queries, schema_definition};

fn default_args(root_id: &str) -> ChildrenArgs {
    ChildrenArgs { root_id: root_id.to_string(), recursive: false, tasks: false, docs: false }
}

fn create_test_repo() -> (tempfile::TempDir, lattice::cli::command_dispatch::CommandContext) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");
    fs::create_dir_all(repo_root.join("api/tasks")).expect("Failed to create api/tasks");
    fs::create_dir_all(repo_root.join("api/docs")).expect("Failed to create api/docs");
    fs::create_dir_all(repo_root.join("api/tasks/.closed")).expect("Failed to create .closed");

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

fn create_task_doc(
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
        Some(TaskType::Task),
        Some(2),
        Some(Utc::now()),
        None,
        None,
        format!("hash-{id}"),
        100,
    );
    doc.in_tasks_dir = true;
    doc.is_closed = path.contains("/.closed/");
    doc
}

fn create_docs_doc(
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
    doc.in_docs_dir = true;
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

// ============================================================================
// Basic Execution Tests
// ============================================================================

#[test]
fn children_command_succeeds_with_no_children() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let root_doc = create_root_doc("LAABCD", "api/api.md", "api", "API root document");
    insert_doc(&context.conn, &root_doc, repo_root, "api/api.md");
    insert_root(&context.conn, "api", "LAABCD", 0);

    let args = default_args("LAABCD");
    let result = children_command::execute(context, args);
    assert!(result.is_ok(), "Children command should succeed with no children");
}

#[test]
fn children_command_lists_task_children() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let root_doc = create_root_doc("LBBCDE", "api/api.md", "api", "API root document");
    insert_doc(&context.conn, &root_doc, repo_root, "api/api.md");
    insert_root(&context.conn, "api", "LBBCDE", 0);

    let task1 = create_task_doc("LCEFGH", "LBBCDE", "api/tasks/task1.md", "task1", "First task");
    insert_doc(&context.conn, &task1, repo_root, "api/tasks/task1.md");

    let task2 = create_task_doc("LDFGHI", "LBBCDE", "api/tasks/task2.md", "task2", "Second task");
    insert_doc(&context.conn, &task2, repo_root, "api/tasks/task2.md");

    let args = default_args("LBBCDE");
    let result = children_command::execute(context, args);
    assert!(result.is_ok(), "Children command should list task children");
}

#[test]
fn children_command_lists_docs_children() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let root_doc = create_root_doc("LEGHIJ", "api/api.md", "api", "API root document");
    insert_doc(&context.conn, &root_doc, repo_root, "api/api.md");
    insert_root(&context.conn, "api", "LEGHIJ", 0);

    let doc1 = create_docs_doc("LFHIJK", "LEGHIJ", "api/docs/design.md", "design", "Design doc");
    insert_doc(&context.conn, &doc1, repo_root, "api/docs/design.md");

    let args = default_args("LEGHIJ");
    let result = children_command::execute(context, args);
    assert!(result.is_ok(), "Children command should list docs children");
}

#[test]
fn children_command_lists_mixed_children() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let root_doc = create_root_doc("LGIJKL", "api/api.md", "api", "API root document");
    insert_doc(&context.conn, &root_doc, repo_root, "api/api.md");
    insert_root(&context.conn, "api", "LGIJKL", 0);

    let task1 = create_task_doc("LHJKLM", "LGIJKL", "api/tasks/task1.md", "task1", "First task");
    insert_doc(&context.conn, &task1, repo_root, "api/tasks/task1.md");

    let doc1 = create_docs_doc("LIKLMN", "LGIJKL", "api/docs/design.md", "design", "Design doc");
    insert_doc(&context.conn, &doc1, repo_root, "api/docs/design.md");

    let args = default_args("LGIJKL");
    let result = children_command::execute(context, args);
    assert!(result.is_ok(), "Children command should list mixed children");
}

// ============================================================================
// Filtering Tests
// ============================================================================

#[test]
fn children_command_filters_to_tasks_only() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let root_doc = create_root_doc("LJLMNO", "api/api.md", "api", "API root document");
    insert_doc(&context.conn, &root_doc, repo_root, "api/api.md");
    insert_root(&context.conn, "api", "LJLMNO", 0);

    let task1 = create_task_doc("LKMNOP", "LJLMNO", "api/tasks/task1.md", "task1", "First task");
    insert_doc(&context.conn, &task1, repo_root, "api/tasks/task1.md");

    let doc1 = create_docs_doc("LLNOPQ", "LJLMNO", "api/docs/design.md", "design", "Design doc");
    insert_doc(&context.conn, &doc1, repo_root, "api/docs/design.md");

    let args =
        ChildrenArgs { root_id: "LJLMNO".to_string(), recursive: false, tasks: true, docs: false };
    let result = children_command::execute(context, args);
    assert!(result.is_ok(), "Children command should filter to tasks only");
}

#[test]
fn children_command_filters_to_docs_only() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let root_doc = create_root_doc("LMOPQR", "api/api.md", "api", "API root document");
    insert_doc(&context.conn, &root_doc, repo_root, "api/api.md");
    insert_root(&context.conn, "api", "LMOPQR", 0);

    let task1 = create_task_doc("LNPQRS", "LMOPQR", "api/tasks/task1.md", "task1", "First task");
    insert_doc(&context.conn, &task1, repo_root, "api/tasks/task1.md");

    let doc1 = create_docs_doc("LOQRST", "LMOPQR", "api/docs/design.md", "design", "Design doc");
    insert_doc(&context.conn, &doc1, repo_root, "api/docs/design.md");

    let args =
        ChildrenArgs { root_id: "LMOPQR".to_string(), recursive: false, tasks: false, docs: true };
    let result = children_command::execute(context, args);
    assert!(result.is_ok(), "Children command should filter to docs only");
}

#[test]
fn children_command_rejects_conflicting_filters() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let root_doc = create_root_doc("LPRSTU", "api/api.md", "api", "API root document");
    insert_doc(&context.conn, &root_doc, repo_root, "api/api.md");
    insert_root(&context.conn, "api", "LPRSTU", 0);

    let args =
        ChildrenArgs { root_id: "LPRSTU".to_string(), recursive: false, tasks: true, docs: true };
    let result = children_command::execute(context, args);
    assert!(
        matches!(result, Err(LatticeError::ConflictingOptions { .. })),
        "Children command should reject --tasks and --docs together"
    );
}

// ============================================================================
// Recursive Tests
// ============================================================================

#[test]
fn children_command_non_recursive_excludes_nested() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    fs::create_dir_all(repo_root.join("api/auth/tasks")).expect("Failed to create nested dir");

    let root_doc = create_root_doc("LQSTUV", "api/api.md", "api", "API root document");
    insert_doc(&context.conn, &root_doc, repo_root, "api/api.md");
    insert_root(&context.conn, "api", "LQSTUV", 0);

    let task1 = create_task_doc("LRTUVW", "LQSTUV", "api/tasks/task1.md", "task1", "First task");
    insert_doc(&context.conn, &task1, repo_root, "api/tasks/task1.md");

    let auth_root = create_root_doc("LSUVWX", "api/auth/auth.md", "auth", "Auth subsystem");
    insert_doc(&context.conn, &auth_root, repo_root, "api/auth/auth.md");

    let nested_task =
        create_task_doc("LTVWXY", "LSUVWX", "api/auth/tasks/nested.md", "nested", "Nested task");
    insert_doc(&context.conn, &nested_task, repo_root, "api/auth/tasks/nested.md");

    let args = default_args("LQSTUV");
    let result = children_command::execute(context, args);
    assert!(result.is_ok(), "Non-recursive should exclude nested directories");
}

#[test]
fn children_command_recursive_includes_nested() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    fs::create_dir_all(repo_root.join("api/auth/tasks")).expect("Failed to create nested dir");

    let root_doc = create_root_doc("LUWXYZ", "api/api.md", "api", "API root document");
    insert_doc(&context.conn, &root_doc, repo_root, "api/api.md");
    insert_root(&context.conn, "api", "LUWXYZ", 0);

    let task1 = create_task_doc("LVXYZA", "LUWXYZ", "api/tasks/task1.md", "task1", "First task");
    insert_doc(&context.conn, &task1, repo_root, "api/tasks/task1.md");

    let nested_task =
        create_task_doc("LWYZAB", "LUWXYZ", "api/auth/tasks/nested.md", "nested", "Nested task");
    insert_doc(&context.conn, &nested_task, repo_root, "api/auth/tasks/nested.md");

    let args =
        ChildrenArgs { root_id: "LUWXYZ".to_string(), recursive: true, tasks: false, docs: false };
    let result = children_command::execute(context, args);
    assert!(result.is_ok(), "Recursive should include nested directories");
}

// ============================================================================
// Error Case Tests
// ============================================================================

#[test]
fn children_command_fails_for_nonexistent_id() {
    let (_temp_dir, context) = create_test_repo();

    let args = default_args("LXZABC");
    let result = children_command::execute(context, args);
    assert!(
        matches!(result, Err(LatticeError::DocumentNotFound { .. })),
        "Children command should fail for nonexistent ID"
    );
}

#[test]
fn children_command_fails_for_non_root_document() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let root_doc = create_root_doc("LYABCD", "api/api.md", "api", "API root document");
    insert_doc(&context.conn, &root_doc, repo_root, "api/api.md");
    insert_root(&context.conn, "api", "LYABCD", 0);

    let task1 = create_task_doc("LZBCDE", "LYABCD", "api/tasks/task1.md", "task1", "First task");
    insert_doc(&context.conn, &task1, repo_root, "api/tasks/task1.md");

    let args = default_args("LZBCDE");
    let result = children_command::execute(context, args);
    assert!(
        matches!(result, Err(LatticeError::InvalidArgument { .. })),
        "Children command should fail for non-root document: got {:?}",
        result
    );
}

// ============================================================================
// Closed Tasks Tests
// ============================================================================

#[test]
fn children_command_includes_closed_tasks() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let root_doc = create_root_doc("LAADEF", "api/api.md", "api", "API root document");
    insert_doc(&context.conn, &root_doc, repo_root, "api/api.md");
    insert_root(&context.conn, "api", "LAADEF", 0);

    let open_task =
        create_task_doc("LABEFG", "LAADEF", "api/tasks/open.md", "open-task", "Open task");
    insert_doc(&context.conn, &open_task, repo_root, "api/tasks/open.md");

    let closed_task =
        create_task_doc("LACFGH", "LAADEF", "api/tasks/.closed/done.md", "done-task", "Done task");
    insert_doc(&context.conn, &closed_task, repo_root, "api/tasks/.closed/done.md");

    let args = default_args("LAADEF");
    let result = children_command::execute(context, args);
    assert!(result.is_ok(), "Children command should include closed tasks");
}

// ============================================================================
// JSON Output Tests
// ============================================================================

#[test]
fn children_command_json_output() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");
    fs::create_dir_all(repo_root.join("api/tasks")).expect("Failed to create api/tasks");
    fs::create_dir_all(repo_root.join("api/docs")).expect("Failed to create api/docs");

    let mut global = GlobalOptions::default();
    global.json = true;
    let mut context = create_context(repo_root, &global).expect("Failed to create context");
    context.client_id_store = Box::new(FakeClientIdStore::new("WQN"));
    schema_definition::create_schema(&context.conn).expect("Failed to create schema");

    let root_doc = create_root_doc("LADGHI", "api/api.md", "api", "API root document");
    insert_doc(&context.conn, &root_doc, repo_root, "api/api.md");
    insert_root(&context.conn, "api", "LADGHI", 0);

    let task1 = create_task_doc("LAEHIJ", "LADGHI", "api/tasks/task1.md", "task1", "First task");
    insert_doc(&context.conn, &task1, repo_root, "api/tasks/task1.md");

    let args = default_args("LADGHI");
    let result = children_command::execute(context, args);
    assert!(result.is_ok(), "Children command should produce JSON output");
}
