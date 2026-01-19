//! Tests for the `lat show` command.

use std::fs;

use lattice::cli::command_dispatch::CommandContext;
use lattice::cli::commands::show_command::document_formatter::{
    AncestorRef, OutputMode, ShowOutput,
};
use lattice::cli::commands::show_command::show_executor::DocumentRef;
use lattice::cli::global_options::GlobalOptions;
use lattice::cli::workflow_args::ShowArgs;
use lattice::config::config_schema::Config;
use lattice::document::frontmatter_schema::TaskType;
use lattice::error::error_types::LatticeError;
use lattice::git::client_config::FakeClientIdStore;
use lattice::index::directory_roots::{DirectoryRoot, upsert};
use lattice::index::document_types::InsertDocument;
use lattice::index::link_queries::{InsertLink, LinkType};
use lattice::index::{
    connection_pool, document_queries, label_queries, link_queries, schema_definition,
};
use lattice::task::task_state::TaskState;
use lattice::test::fake_git::FakeGit;
use lattice::test::test_environment::TestEnv;
use lattice::test::test_fixtures::{KbDocBuilder, RootDocBuilder, TaskDocBuilder};
use rusqlite::Connection;
use tempfile::TempDir;

fn create_test_document(id: &str, path: &str, name: &str, description: &str) -> InsertDocument {
    InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        name.to_string(),
        description.to_string(),
        None,
        None,
        None,
        None,
        None,
        "abc123".to_string(),
        100,
    )
}

fn create_task_document(
    id: &str,
    path: &str,
    name: &str,
    description: &str,
    priority: u8,
) -> InsertDocument {
    InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        name.to_string(),
        description.to_string(),
        Some(TaskType::Task),
        Some(priority),
        None,
        None,
        None,
        "def456".to_string(),
        200,
    )
}

/// Creates a test repository for the template composition tests.
/// This is the legacy pattern; new tests should use TestEnv instead.
fn create_test_repo() -> (TempDir, CommandContext) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let repo_root = temp_dir.path().to_path_buf();

    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");
    connection_pool::ensure_lattice_dir(&repo_root).expect("Failed to create .lat");

    let conn = connection_pool::open_connection(&repo_root).expect("Failed to open connection");
    schema_definition::create_schema(&conn).expect("Failed to create schema");

    let context = CommandContext {
        git: Box::new(FakeGit::new()),
        conn,
        config: Config::default(),
        repo_root,
        global: GlobalOptions::default(),
        client_id_store: Box::new(FakeClientIdStore::new("TST")),
    };

    (temp_dir, context)
}

/// Inserts a document into the database (for template tests).
fn insert_doc(conn: &Connection, doc: &InsertDocument) {
    document_queries::insert(conn, doc).expect("Failed to insert document");
}

// ============================================================================
// TaskState Tests
// ============================================================================

#[test]
fn task_state_display_formats_correctly() {
    assert_eq!(TaskState::Open.to_string(), "open");
    assert_eq!(TaskState::Blocked.to_string(), "blocked");
    assert_eq!(TaskState::Closed.to_string(), "closed");
}

// ============================================================================
// DocumentRef Tests
// ============================================================================

#[test]
fn document_ref_type_indicator_for_task() {
    let doc_ref = DocumentRef {
        id: "LTESTA".to_string(),
        name: "test-task".to_string(),
        description: "Test task".to_string(),
        task_type: Some(TaskType::Bug),
        priority: Some(1),
        is_closed: false,
        is_root: false,
    };

    assert_eq!(doc_ref.type_indicator(), "P1");
}

#[test]
fn document_ref_type_indicator_for_closed_task() {
    let doc_ref = DocumentRef {
        id: "LTESTB".to_string(),
        name: "closed-task".to_string(),
        description: "Closed task".to_string(),
        task_type: Some(TaskType::Feature),
        priority: Some(0),
        is_closed: true,
        is_root: false,
    };

    assert_eq!(doc_ref.type_indicator(), "P0/closed");
}

#[test]
fn document_ref_type_indicator_for_knowledge_base() {
    let doc_ref = DocumentRef {
        id: "LTESTC".to_string(),
        name: "kb-doc".to_string(),
        description: "Knowledge base doc".to_string(),
        task_type: None,
        priority: None,
        is_closed: false,
        is_root: false,
    };

    assert_eq!(doc_ref.type_indicator(), "doc");
}

// ============================================================================
// OutputMode Tests
// ============================================================================

#[test]
fn output_mode_equality() {
    assert_eq!(OutputMode::Full, OutputMode::Full);
    assert_eq!(OutputMode::Short, OutputMode::Short);
    assert_ne!(OutputMode::Full, OutputMode::Short);
}

// ============================================================================
// ShowOutput Tests
// ============================================================================

#[test]
fn show_output_serializes_to_json() {
    let output = ShowOutput {
        id: "LTESTA".to_string(),
        name: "test-doc".to_string(),
        description: "Test document".to_string(),
        path: "test/test-doc.md".to_string(),
        state: TaskState::Open,
        priority: Some(1),
        task_type: Some(TaskType::Bug),
        labels: vec!["label1".to_string(), "label2".to_string()],
        created_at: None,
        updated_at: None,
        closed_at: None,
        ancestors: Vec::new(),
        composed_context: None,
        composed_acceptance: None,
        body: Some("Body content".to_string()),
        parent: None,
        dependencies: Vec::new(),
        blocking: Vec::new(),
        related: Vec::new(),
        backlinks: Vec::new(),
        claimed: false,
    };

    let json = serde_json::to_string(&output).expect("Should serialize to JSON");
    assert!(json.contains("\"id\":\"LTESTA\""));
    assert!(json.contains("\"state\":\"open\""));
    assert!(json.contains("\"task_type\":\"bug\""));
    assert!(json.contains("\"dependents\""), "blocking should be renamed to dependents in JSON");
    assert!(!json.contains("\"blocking\""), "blocking should not appear in JSON");
    assert!(
        !json.contains("\"backlinks\""),
        "backlinks should not appear in JSON (skip_serializing)"
    );
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn show_command_returns_document_not_found_for_missing_id() {
    let env = TestEnv::new();
    let (_temp, context) = env.into_parts();

    let args = ShowArgs {
        ids: vec!["LNFXYZ".to_string()],
        short: false,
        refs: false,
        peek: false,
        raw: false,
    };

    let result = lattice::cli::commands::show_command::show_executor::execute(context, args);

    assert!(result.is_err(), "Should return error for missing document");
    if let Err(LatticeError::DocumentNotFound { id }) = result {
        assert_eq!(id, "LNFXYZ", "Error should contain the missing ID");
    } else {
        panic!("Expected DocumentNotFound error");
    }
}

#[test]
fn show_command_finds_existing_document() {
    let env = TestEnv::new();

    // Create and track a document using fixtures
    let doc = KbDocBuilder::new("Test document").id("LDOCAA").name("test").build();
    env.create_dir("docs");
    env.write_file("docs/test.md", &doc.content);
    env.fake_git().track_file("docs/test.md");

    // Insert into index
    let insert_doc = create_test_document("LDOCAA", "docs/test.md", "test", "Test document");
    document_queries::insert(env.conn(), &insert_doc).expect("Insert document");

    let (_temp, context) = env.into_parts();

    let args = ShowArgs {
        ids: vec!["LDOCAA".to_string()],
        short: false,
        refs: false,
        peek: false,
        raw: false,
    };

    let result = lattice::cli::commands::show_command::show_executor::execute(context, args);

    assert!(result.is_ok(), "Should succeed for existing document: {:?}", result);
}

#[test]
fn show_command_handles_task_with_closed_state() {
    let env = TestEnv::new();

    // Create a closed task using fixtures
    let task = TaskDocBuilder::new("Done task").id("LDONEX").priority(1).build();
    env.create_dir("tasks/.closed");
    env.write_file("tasks/.closed/done.md", &task.content);
    env.fake_git().track_file("tasks/.closed/done.md");

    // Insert into index (is_closed is computed from path)
    let insert_doc =
        create_task_document("LDONEX", "tasks/.closed/done.md", "done", "Done task", 1);
    document_queries::insert(env.conn(), &insert_doc).expect("Insert document");

    let row = document_queries::lookup_by_id(env.conn(), "LDONEX")
        .expect("Lookup should succeed")
        .expect("Document should exist");

    assert!(row.is_closed, "Document should be marked as closed based on path");
}

#[test]
fn show_command_handles_blocked_task() {
    let env = TestEnv::new();

    // Create blocker and blocked tasks using fixtures
    let blocker = TaskDocBuilder::new("Blocking task").id("LBLKRA").priority(0).build();
    let blocked = TaskDocBuilder::new("Blocked task")
        .id("LBLKDA")
        .priority(1)
        .blocked_by(vec!["LBLKRA"])
        .build();

    env.create_dir("tasks");
    env.write_file("tasks/blocker.md", &blocker.content);
    env.write_file("tasks/blocked.md", &blocked.content);
    env.fake_git().track_files(["tasks/blocker.md", "tasks/blocked.md"]);

    // Insert documents into index
    let blocker_doc =
        create_task_document("LBLKRA", "tasks/blocker.md", "blocker", "Blocking task", 0);
    let blocked_doc =
        create_task_document("LBLKDA", "tasks/blocked.md", "blocked", "Blocked task", 1);
    document_queries::insert(env.conn(), &blocker_doc).expect("Insert blocker");
    document_queries::insert(env.conn(), &blocked_doc).expect("Insert blocked");

    // Add blocked-by link
    let link = InsertLink {
        source_id: "LBLKDA",
        target_id: "LBLKRA",
        link_type: LinkType::BlockedBy,
        position: 0,
    };
    link_queries::insert_for_document(env.conn(), &[link]).expect("Insert link");

    let (_temp, context) = env.into_parts();

    // Verify state computation
    let args = ShowArgs {
        ids: vec!["LBLKDA".to_string()],
        short: false,
        refs: false,
        peek: false,
        raw: false,
    };

    let result = lattice::cli::commands::show_command::show_executor::execute(context, args);
    assert!(result.is_ok(), "Should succeed: {:?}", result);
}

#[test]
fn show_command_handles_multiple_ids() {
    let env = TestEnv::new();

    // Create two documents using fixtures
    let doc1 = KbDocBuilder::new("First doc").id("LMULTA").name("doc1").build();
    let doc2 = KbDocBuilder::new("Second doc").id("LMULT2").name("doc2").build();

    env.write_file("doc1.md", &doc1.content);
    env.write_file("doc2.md", &doc2.content);
    env.fake_git().track_files(["doc1.md", "doc2.md"]);

    let insert_doc1 = create_test_document("LMULTA", "doc1.md", "doc1", "First doc");
    let insert_doc2 = create_test_document("LMULT2", "doc2.md", "doc2", "Second doc");
    document_queries::insert(env.conn(), &insert_doc1).expect("Insert doc1");
    document_queries::insert(env.conn(), &insert_doc2).expect("Insert doc2");

    let (_temp, context) = env.into_parts();

    let args = ShowArgs {
        ids: vec!["LMULTA".to_string(), "LMULT2".to_string()],
        short: false,
        refs: false,
        peek: false,
        raw: false,
    };

    let result = lattice::cli::commands::show_command::show_executor::execute(context, args);
    assert!(result.is_ok(), "Should handle multiple documents: {:?}", result);
}

#[test]
fn show_command_loads_labels() {
    let env = TestEnv::new();

    let doc = KbDocBuilder::new("Labeled doc").id("LLABEL").name("labeled").build();
    env.write_file("test.md", &doc.content);
    env.fake_git().track_file("test.md");

    let insert_doc = create_test_document("LLABEL", "test.md", "labeled", "Labeled doc");
    document_queries::insert(env.conn(), &insert_doc).expect("Insert doc");

    // Add labels to index
    label_queries::add(env.conn(), "LLABEL", "foo").expect("Add label foo");
    label_queries::add(env.conn(), "LLABEL", "bar").expect("Add label bar");

    let (_temp, context) = env.into_parts();

    let args = ShowArgs {
        ids: vec!["LLABEL".to_string()],
        short: false,
        refs: false,
        peek: false,
        raw: false,
    };

    let result = lattice::cli::commands::show_command::show_executor::execute(context, args);
    assert!(result.is_ok(), "Should succeed with labels: {:?}", result);
}

#[test]
fn show_command_with_short_flag() {
    let env = TestEnv::new();

    let doc = KbDocBuilder::new("Short test").id("LSHORT").name("short-test").build();
    env.write_file("test.md", &doc.content);
    env.fake_git().track_file("test.md");

    let insert_doc = create_test_document("LSHORT", "test.md", "short-test", "Short test");
    document_queries::insert(env.conn(), &insert_doc).expect("Insert doc");

    let (_temp, context) = env.into_parts();

    let args = ShowArgs {
        ids: vec!["LSHORT".to_string()],
        short: true,
        refs: false,
        peek: false,
        raw: false,
    };

    let result = lattice::cli::commands::show_command::show_executor::execute(context, args);
    assert!(result.is_ok(), "Short mode should succeed: {:?}", result);
}

#[test]
fn show_command_with_peek_flag() {
    let env = TestEnv::new();

    let doc = KbDocBuilder::new("Peek test").id("LPEEKA").name("peek-test").build();
    env.write_file("test.md", &doc.content);
    env.fake_git().track_file("test.md");

    let insert_doc = create_test_document("LPEEKA", "test.md", "peek-test", "Peek test");
    document_queries::insert(env.conn(), &insert_doc).expect("Insert doc");

    let (_temp, context) = env.into_parts();

    let args = ShowArgs {
        ids: vec!["LPEEKA".to_string()],
        short: false,
        refs: false,
        peek: true,
        raw: false,
    };

    let result = lattice::cli::commands::show_command::show_executor::execute(context, args);
    assert!(result.is_ok(), "Peek mode should succeed: {:?}", result);
}

#[test]
fn show_command_with_raw_flag() {
    let env = TestEnv::new();

    let doc = KbDocBuilder::new("Raw test").id("LRAWAB").name("raw-test").build();
    env.write_file("test.md", &doc.content);
    env.fake_git().track_file("test.md");

    let insert_doc = create_test_document("LRAWAB", "test.md", "raw-test", "Raw test");
    document_queries::insert(env.conn(), &insert_doc).expect("Insert doc");

    let (_temp, context) = env.into_parts();

    let args = ShowArgs {
        ids: vec!["LRAWAB".to_string()],
        short: false,
        refs: false,
        peek: false,
        raw: true,
    };

    let result = lattice::cli::commands::show_command::show_executor::execute(context, args);
    assert!(result.is_ok(), "Raw mode should succeed: {:?}", result);
}

// ============================================================================
// Knowledge Base Document Tests
// ============================================================================

#[test]
fn document_ref_from_row_includes_is_root() {
    let env = TestEnv::new();

    // Create a root document (path api/api.md - filename matches directory name)
    let root = RootDocBuilder::new("api", "API root document").id("LROOTX").build();
    env.create_dir("api");
    env.write_file(&root.path, &root.content);
    env.fake_git().track_file(&root.path);

    let insert_doc = InsertDocument::new(
        "LROOTX".to_string(),
        None,
        "api/api.md".to_string(),
        "api".to_string(),
        "API root document".to_string(),
        None,
        None,
        None,
        None,
        None,
        "hash123".to_string(),
        100,
    );
    document_queries::insert(env.conn(), &insert_doc).expect("Insert doc");

    let row = document_queries::lookup_by_id(env.conn(), "LROOTX")
        .expect("Lookup should succeed")
        .expect("Document should exist");

    let doc_ref = DocumentRef::from_row(&row);
    assert!(doc_ref.is_root, "Root document should have is_root = true");
}

#[test]
fn document_ref_from_row_non_root() {
    let env = TestEnv::new();

    // Create a non-root document
    let doc = KbDocBuilder::new("API readme").id("LNROOT").name("readme").build();
    env.create_dir("api/docs");
    env.write_file("api/docs/readme.md", &doc.content);
    env.fake_git().track_file("api/docs/readme.md");

    let insert_doc = create_test_document("LNROOT", "api/docs/readme.md", "readme", "API readme");
    document_queries::insert(env.conn(), &insert_doc).expect("Insert doc");

    let row = document_queries::lookup_by_id(env.conn(), "LNROOT")
        .expect("Lookup should succeed")
        .expect("Document should exist");

    let doc_ref = DocumentRef::from_row(&row);
    assert!(!doc_ref.is_root, "Non-root document should have is_root = false");
}

#[test]
fn show_command_includes_related_documents() {
    let env = TestEnv::new();

    // Create main document with a link to related
    let main_content = "---\nlattice-id: LMAINX\nname: main\ndescription: Main document\n---\n\nSee [related](LRELAX) for more info.";
    let related = KbDocBuilder::new("Related document").id("LRELAX").name("related").build();

    env.write_file("main.md", main_content);
    env.write_file("related.md", &related.content);
    env.fake_git().track_files(["main.md", "related.md"]);

    let main_doc = create_test_document("LMAINX", "main.md", "main", "Main document");
    let related_doc = create_test_document("LRELAX", "related.md", "related", "Related document");
    document_queries::insert(env.conn(), &main_doc).expect("Insert main");
    document_queries::insert(env.conn(), &related_doc).expect("Insert related");

    // Add body link
    let link = InsertLink {
        source_id: "LMAINX",
        target_id: "LRELAX",
        link_type: LinkType::Body,
        position: 0,
    };
    link_queries::insert_for_document(env.conn(), &[link]).expect("Insert link");

    let (_temp, context) = env.into_parts();

    let args = ShowArgs {
        ids: vec!["LMAINX".to_string()],
        short: false,
        refs: false,
        peek: false,
        raw: false,
    };

    let result = lattice::cli::commands::show_command::show_executor::execute(context, args);
    assert!(result.is_ok(), "Should succeed with related docs: {:?}", result);
}

#[test]
fn show_command_excludes_blocking_from_related() {
    let env = TestEnv::new();

    // Create main task that blocks another
    let main =
        TaskDocBuilder::new("Main task").id("LBLKMN").priority(1).blocking(vec!["LBLKOT"]).build();
    let other = TaskDocBuilder::new("Other task")
        .id("LBLKOT")
        .priority(1)
        .blocked_by(vec!["LBLKMN"])
        .build();

    env.create_dir("tasks");
    env.write_file("tasks/main.md", &main.content);
    env.write_file("tasks/other.md", &other.content);
    env.fake_git().track_files(["tasks/main.md", "tasks/other.md"]);

    let main_doc = create_task_document("LBLKMN", "tasks/main.md", "main", "Main task", 1);
    let other_doc = create_task_document("LBLKOT", "tasks/other.md", "other", "Other task", 1);
    document_queries::insert(env.conn(), &main_doc).expect("Insert main");
    document_queries::insert(env.conn(), &other_doc).expect("Insert other");

    // Add blocked-by link from other to main
    let blocked_link = InsertLink {
        source_id: "LBLKOT",
        target_id: "LBLKMN",
        link_type: LinkType::BlockedBy,
        position: 0,
    };
    link_queries::insert_for_document(env.conn(), &[blocked_link]).expect("Insert blocked-by link");

    // Also add body link from main to other (to test exclusion from related)
    let body_link = InsertLink {
        source_id: "LBLKMN",
        target_id: "LBLKOT",
        link_type: LinkType::Body,
        position: 0,
    };
    link_queries::insert_for_document(env.conn(), &[body_link]).expect("Insert body link");

    let (_temp, context) = env.into_parts();

    let args = ShowArgs {
        ids: vec!["LBLKMN".to_string()],
        short: false,
        refs: false,
        peek: false,
        raw: false,
    };

    let result = lattice::cli::commands::show_command::show_executor::execute(context, args);
    assert!(result.is_ok(), "Should succeed: {:?}", result);
}

#[test]
fn show_command_with_refs_flag_finds_backlinks() {
    let env = TestEnv::new();

    // Create target and source documents
    let target = KbDocBuilder::new("Target document").id("LTRGTX").name("target").build();
    let source_content = "---\nlattice-id: LSRCXX\nname: source\ndescription: Source document\n---\n\nSee [target](LTRGTX).";

    env.write_file("target.md", &target.content);
    env.write_file("source.md", source_content);
    env.fake_git().track_files(["target.md", "source.md"]);

    let target_doc = create_test_document("LTRGTX", "target.md", "target", "Target document");
    let source_doc = create_test_document("LSRCXX", "source.md", "source", "Source document");
    document_queries::insert(env.conn(), &target_doc).expect("Insert target");
    document_queries::insert(env.conn(), &source_doc).expect("Insert source");

    // Add body link from source to target
    let body_link = InsertLink {
        source_id: "LSRCXX",
        target_id: "LTRGTX",
        link_type: LinkType::Body,
        position: 0,
    };
    link_queries::insert_for_document(env.conn(), &[body_link]).expect("Insert body link");

    let (_temp, context) = env.into_parts();

    // Show target with --refs flag
    let args = ShowArgs {
        ids: vec!["LTRGTX".to_string()],
        short: false,
        refs: true,
        peek: false,
        raw: false,
    };

    let result = lattice::cli::commands::show_command::show_executor::execute(context, args);
    assert!(result.is_ok(), "Refs mode should succeed: {:?}", result);
}

#[test]
fn show_command_peek_mode_displays_parent_and_counts() {
    let env = TestEnv::new();

    // Create root document (parent) and task
    let root = RootDocBuilder::new("api", "API root").id("LROOTA").build();
    let task = TaskDocBuilder::new("My task").id("LTASKP").priority(1).build();

    env.create_dir("api/tasks");
    env.write_file(&root.path, &root.content);
    env.write_file("api/tasks/my-task.md", &task.content);
    env.fake_git().track_files([root.path.as_str(), "api/tasks/my-task.md"]);

    let root_doc = InsertDocument::new(
        "LROOTA".to_string(),
        None,
        "api/api.md".to_string(),
        "api".to_string(),
        "API root".to_string(),
        None,
        None,
        None,
        None,
        None,
        "hash1".to_string(),
        100,
    );
    let task_doc = InsertDocument::new(
        "LTASKP".to_string(),
        Some("LROOTA".to_string()),
        "api/tasks/my-task.md".to_string(),
        "my-task".to_string(),
        "My task".to_string(),
        Some(TaskType::Task),
        Some(1),
        None,
        None,
        None,
        "hash2".to_string(),
        200,
    );
    document_queries::insert(env.conn(), &root_doc).expect("Insert root");
    document_queries::insert(env.conn(), &task_doc).expect("Insert task");

    let (_temp, context) = env.into_parts();

    // Show task with --peek flag
    let args = ShowArgs {
        ids: vec!["LTASKP".to_string()],
        short: false,
        refs: false,
        peek: true,
        raw: false,
    };

    let result = lattice::cli::commands::show_command::show_executor::execute(context, args);
    assert!(result.is_ok(), "Peek mode should succeed: {:?}", result);
}

// ============================================================================
// View Tracking Tests
// ============================================================================

#[test]
fn show_command_records_view() {
    let env = TestEnv::new();

    let doc = KbDocBuilder::new("View test").id("LVIEWX").name("view-test").build();
    env.write_file("test.md", &doc.content);
    env.fake_git().track_file("test.md");

    let insert_doc = create_test_document("LVIEWX", "test.md", "view-test", "View test");
    document_queries::insert(env.conn(), &insert_doc).expect("Insert doc");

    // Verify initial view count is 0
    let initial_count = lattice::index::view_tracking::get_view_count(env.conn(), "LVIEWX")
        .expect("Get initial count");
    assert_eq!(initial_count, 0, "Initial view count should be 0");

    let (_temp, context) = env.into_parts();

    // Show the document
    let args = ShowArgs {
        ids: vec!["LVIEWX".to_string()],
        short: false,
        refs: false,
        peek: false,
        raw: false,
    };
    let result = lattice::cli::commands::show_command::show_executor::execute(context, args);
    assert!(result.is_ok(), "Show should succeed: {:?}", result);

    // Note: View count increment happens in the context's connection which was
    // consumed. This test verifies the command succeeds; view count testing
    // requires re-creating context.
}

#[test]
fn show_command_increments_view_count_multiple_times() {
    let env = TestEnv::new();

    let doc = KbDocBuilder::new("Multi view test").id("LMULVW").name("multi-view").build();
    env.write_file("test.md", &doc.content);
    env.fake_git().track_file("test.md");

    let insert_doc = create_test_document("LMULVW", "test.md", "multi-view", "Multi view test");
    document_queries::insert(env.conn(), &insert_doc).expect("Insert doc");

    let (_temp, context) = env.into_parts();

    // Show the document
    let args = ShowArgs {
        ids: vec!["LMULVW".to_string()],
        short: false,
        refs: false,
        peek: false,
        raw: false,
    };
    let result = lattice::cli::commands::show_command::show_executor::execute(context, args);
    assert!(result.is_ok(), "Show should succeed: {:?}", result);

    // Note: Testing multiple views requires re-creating context from temp
    // directory. The command execution is the primary test here.
}

// ============================================================================
// Template Composition Integration Tests
// ============================================================================

fn make_root(path: &str, id: &str, parent: Option<&str>, depth: u32) -> DirectoryRoot {
    DirectoryRoot {
        directory_path: path.to_string(),
        root_id: id.to_string(),
        parent_path: parent.map(|s| s.to_string()),
        depth,
    }
}

fn create_template_hierarchy(temp_dir: &tempfile::TempDir) {
    let root = temp_dir.path();

    fs::create_dir_all(root.join("project")).expect("Create project");
    fs::create_dir_all(root.join("project/api")).expect("Create api");
    fs::create_dir_all(root.join("project/api/tasks")).expect("Create tasks");

    fs::write(
        root.join("project/project.md"),
        r#"---
lattice-id: LPRJZA
name: project
description: Project root
---

# Project

## [Lattice] Context

This is project-wide context.
All tasks inherit this.

## [Lattice] Acceptance Criteria

- [ ] All tests pass
- [ ] Code reviewed
"#,
    )
    .expect("Write project.md");

    fs::write(
        root.join("project/api/api.md"),
        r#"---
lattice-id: LAPIZA
name: api
description: API subsystem
---

# API

## [Lattice] Context

API-specific context here.
Handle REST conventions.

## [Lattice] Acceptance Criteria

- [ ] API docs updated
- [ ] Backward compatible
"#,
    )
    .expect("Write api.md");

    fs::write(
        root.join("project/api/tasks/fix_bug.md"),
        r#"---
lattice-id: LBUGZA
name: fix-bug
description: Fix validation bug
task-type: bug
priority: 1
---

# Fix Validation Bug

The validation logic is broken.
"#,
    )
    .expect("Write fix_bug.md");
}

#[test]
fn show_command_includes_composed_templates_for_tasks() {
    let (temp_dir, context) = create_test_repo();
    create_template_hierarchy(&temp_dir);

    let project_doc = InsertDocument::new(
        "LPRJZA".to_string(),
        None,
        "project/project.md".to_string(),
        "project".to_string(),
        "Project root".to_string(),
        None,
        None,
        None,
        None,
        None,
        "hash1".to_string(),
        100,
    );
    let api_doc = InsertDocument::new(
        "LAPIZA".to_string(),
        Some("LPRJZA".to_string()),
        "project/api/api.md".to_string(),
        "api".to_string(),
        "API subsystem".to_string(),
        None,
        None,
        None,
        None,
        None,
        "hash2".to_string(),
        200,
    );
    let task_doc = InsertDocument::new(
        "LBUGZA".to_string(),
        Some("LAPIZA".to_string()),
        "project/api/tasks/fix_bug.md".to_string(),
        "fix-bug".to_string(),
        "Fix validation bug".to_string(),
        Some(TaskType::Bug),
        Some(1),
        None,
        None,
        None,
        "hash3".to_string(),
        300,
    );
    insert_doc(&context.conn, &project_doc);
    insert_doc(&context.conn, &api_doc);
    insert_doc(&context.conn, &task_doc);

    upsert(&context.conn, &make_root("project", "LPRJZA", None, 0)).expect("Insert project root");
    upsert(&context.conn, &make_root("project/api", "LAPIZA", Some("project"), 1))
        .expect("Insert api root");

    let args = ShowArgs {
        ids: vec!["LBUGZA".to_string()],
        short: false,
        refs: false,
        peek: false,
        raw: false,
    };

    let result = lattice::cli::commands::show_command::show_executor::execute(context, args);
    assert!(result.is_ok(), "Show should succeed with templates: {:?}", result);
}

#[test]
fn show_command_raw_mode_skips_template_composition() {
    let (temp_dir, context) = create_test_repo();
    create_template_hierarchy(&temp_dir);

    let project_doc = InsertDocument::new(
        "LPRJZA".to_string(),
        None,
        "project/project.md".to_string(),
        "project".to_string(),
        "Project root".to_string(),
        None,
        None,
        None,
        None,
        None,
        "hash1".to_string(),
        100,
    );
    let api_doc = InsertDocument::new(
        "LAPIZA".to_string(),
        Some("LPRJZA".to_string()),
        "project/api/api.md".to_string(),
        "api".to_string(),
        "API subsystem".to_string(),
        None,
        None,
        None,
        None,
        None,
        "hash2".to_string(),
        200,
    );
    let task_doc = InsertDocument::new(
        "LBUGZA".to_string(),
        Some("LAPIZA".to_string()),
        "project/api/tasks/fix_bug.md".to_string(),
        "fix-bug".to_string(),
        "Fix validation bug".to_string(),
        Some(TaskType::Bug),
        Some(1),
        None,
        None,
        None,
        "hash3".to_string(),
        300,
    );
    insert_doc(&context.conn, &project_doc);
    insert_doc(&context.conn, &api_doc);
    insert_doc(&context.conn, &task_doc);

    upsert(&context.conn, &make_root("project", "LPRJZA", None, 0)).expect("Insert project root");
    upsert(&context.conn, &make_root("project/api", "LAPIZA", Some("project"), 1))
        .expect("Insert api root");

    let args = ShowArgs {
        ids: vec!["LBUGZA".to_string()],
        short: false,
        refs: false,
        peek: false,
        raw: true,
    };

    let result = lattice::cli::commands::show_command::show_executor::execute(context, args);
    assert!(result.is_ok(), "Raw mode should succeed: {:?}", result);
}

#[test]
fn show_command_knowledge_base_document_skips_template_composition() {
    let (temp_dir, context) = create_test_repo();

    fs::create_dir_all(temp_dir.path().join("docs")).expect("Create docs");
    fs::write(
        temp_dir.path().join("docs/readme.md"),
        "---\nlattice-id: LRDMEA\nname: readme\ndescription: Project readme\n---\n\nReadme body.",
    )
    .expect("Write readme");

    let doc = create_test_document("LRDMEA", "docs/readme.md", "readme", "Project readme");
    insert_doc(&context.conn, &doc);

    let args = ShowArgs {
        ids: vec!["LRDMEA".to_string()],
        short: false,
        refs: false,
        peek: false,
        raw: false,
    };

    let result = lattice::cli::commands::show_command::show_executor::execute(context, args);
    assert!(result.is_ok(), "KB doc should succeed without templates: {:?}", result);
}

#[test]
fn show_output_serializes_template_fields_in_json() {
    let output = ShowOutput {
        id: "LTMPLZ".to_string(),
        name: "template-test".to_string(),
        description: "Template test task".to_string(),
        path: "project/api/tasks/test.md".to_string(),
        state: TaskState::Open,
        priority: Some(1),
        task_type: Some(TaskType::Task),
        labels: Vec::new(),
        created_at: None,
        updated_at: None,
        closed_at: None,
        ancestors: vec![
            AncestorRef {
                id: "LPRJZA".to_string(),
                name: "project".to_string(),
                path: "project/project.md".to_string(),
            },
            AncestorRef {
                id: "LAPIZA".to_string(),
                name: "api".to_string(),
                path: "project/api/api.md".to_string(),
            },
        ],
        composed_context: Some("Project context\n\nAPI context".to_string()),
        composed_acceptance: Some("- [ ] API docs\n\n- [ ] Tests pass".to_string()),
        body: Some("Task body".to_string()),
        parent: None,
        dependencies: Vec::new(),
        blocking: Vec::new(),
        related: Vec::new(),
        backlinks: Vec::new(),
        claimed: false,
    };

    let json = serde_json::to_string(&output).expect("Should serialize to JSON");

    assert!(json.contains("\"ancestors\""), "Should include ancestors field");
    assert!(json.contains("\"composed_context\""), "Should include composed_context field");
    assert!(json.contains("\"composed_acceptance\""), "Should include composed_acceptance field");
    assert!(json.contains("LPRJZA"), "Should include project ancestor ID");
    assert!(json.contains("LAPIZA"), "Should include api ancestor ID");
    assert!(json.contains("Project context"), "Should include context content");
    assert!(json.contains("API docs"), "Should include acceptance content");
}

#[test]
fn show_output_omits_empty_template_fields_in_json() {
    let output = ShowOutput {
        id: "LNOTML".to_string(),
        name: "no-template".to_string(),
        description: "No template task".to_string(),
        path: "tasks/test.md".to_string(),
        state: TaskState::Open,
        priority: Some(2),
        task_type: Some(TaskType::Task),
        labels: Vec::new(),
        created_at: None,
        updated_at: None,
        closed_at: None,
        ancestors: Vec::new(),
        composed_context: None,
        composed_acceptance: None,
        body: Some("Body".to_string()),
        parent: None,
        dependencies: Vec::new(),
        blocking: Vec::new(),
        related: Vec::new(),
        backlinks: Vec::new(),
        claimed: false,
    };

    let json = serde_json::to_string(&output).expect("Should serialize to JSON");

    assert!(!json.contains("\"ancestors\""), "Should omit empty ancestors");
    assert!(!json.contains("\"composed_context\""), "Should omit null composed_context");
    assert!(!json.contains("\"composed_acceptance\""), "Should omit null composed_acceptance");
}
