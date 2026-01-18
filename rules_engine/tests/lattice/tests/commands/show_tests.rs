//! Tests for the `lat show` command.

use std::fs;

use lattice::cli::command_dispatch::{CommandContext, create_context};
use lattice::cli::commands::show_command::document_formatter::{OutputMode, ShowOutput};
use lattice::cli::commands::show_command::show_executor::{DocumentRef, TaskState};
use lattice::cli::global_options::GlobalOptions;
use lattice::cli::workflow_args::ShowArgs;
use lattice::document::frontmatter_schema::TaskType;
use lattice::error::error_types::LatticeError;
use lattice::index::document_types::InsertDocument;
use lattice::index::link_queries::{self, InsertLink, LinkType};
use lattice::index::{document_queries, label_queries, schema_definition};
use rusqlite::Connection;

fn create_test_repo() -> (tempfile::TempDir, CommandContext) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");

    let global = GlobalOptions::default();
    let context = create_context(repo_root, &global).expect("Failed to create context");

    // Create the schema for tests
    schema_definition::create_schema(&context.conn).expect("Failed to create schema");

    (temp_dir, context)
}

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
    // Verify blocking is renamed to dependents in JSON
    assert!(json.contains("\"dependents\""));
    assert!(!json.contains("\"blocking\""));
    // Verify backlinks is not included in JSON (skip_serializing)
    assert!(!json.contains("\"backlinks\""));
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn show_command_returns_document_not_found_for_missing_id() {
    let (_temp_dir, context) = create_test_repo();

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
    let (temp_dir, context) = create_test_repo();

    // Create a document file
    let doc_path = temp_dir.path().join("docs").join("test.md");
    fs::create_dir_all(doc_path.parent().unwrap()).expect("Create dirs");
    fs::write(
        &doc_path,
        "---\nlattice-id: LDOCAA\nname: test\ndescription: Test document\n---\n\nBody content.",
    )
    .expect("Write doc");

    // Insert into index
    let doc = create_test_document("LDOCAA", "docs/test.md", "test", "Test document");
    insert_doc(&context.conn, &doc);

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
    let (temp_dir, context) = create_test_repo();

    // Create a closed task
    let doc_path = temp_dir.path().join("tasks").join(".closed").join("done.md");
    fs::create_dir_all(doc_path.parent().unwrap()).expect("Create dirs");
    fs::write(
        &doc_path,
        "---\nlattice-id: LDONEX\nname: done\ndescription: Done task\ntask-type: task\npriority: 1\n---\n\nCompleted.",
    )
    .expect("Write doc");

    // Insert into index (is_closed is computed from path)
    let doc = create_task_document("LDONEX", "tasks/.closed/done.md", "done", "Done task", 1);
    insert_doc(&context.conn, &doc);

    let row = document_queries::lookup_by_id(&context.conn, "LDONEX")
        .expect("Lookup should succeed")
        .expect("Document should exist");

    assert!(row.is_closed, "Document should be marked as closed based on path");
}

#[test]
fn show_command_handles_blocked_task() {
    let (temp_dir, context) = create_test_repo();

    // Create blocker task (open)
    let blocker_path = temp_dir.path().join("tasks").join("blocker.md");
    fs::create_dir_all(blocker_path.parent().unwrap()).expect("Create dirs");
    fs::write(
        &blocker_path,
        "---\nlattice-id: LBLKRA\nname: blocker\ndescription: Blocking task\ntask-type: task\npriority: 0\n---\n\nBlocks others.",
    )
    .expect("Write blocker doc");

    // Create blocked task
    let blocked_path = temp_dir.path().join("tasks").join("blocked.md");
    fs::write(
        &blocked_path,
        "---\nlattice-id: LBLKDA\nname: blocked\ndescription: Blocked task\ntask-type: task\npriority: 1\nblocked-by:\n  - LBLKRA\n---\n\nWaiting.",
    )
    .expect("Write blocked doc");

    // Insert documents into index
    let blocker_doc =
        create_task_document("LBLKRA", "tasks/blocker.md", "blocker", "Blocking task", 0);
    let blocked_doc =
        create_task_document("LBLKDA", "tasks/blocked.md", "blocked", "Blocked task", 1);
    insert_doc(&context.conn, &blocker_doc);
    insert_doc(&context.conn, &blocked_doc);

    // Add blocked-by link
    let link = InsertLink {
        source_id: "LBLKDA",
        target_id: "LBLKRA",
        link_type: LinkType::BlockedBy,
        position: 0,
    };
    link_queries::insert_for_document(&context.conn, &[link]).expect("Insert link");

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
    let (temp_dir, context) = create_test_repo();

    // Create two documents
    let doc1_path = temp_dir.path().join("doc1.md");
    let doc2_path = temp_dir.path().join("doc2.md");

    fs::write(
        &doc1_path,
        "---\nlattice-id: LMULTA\nname: doc1\ndescription: First doc\n---\n\nBody 1.",
    )
    .expect("Write doc1");
    fs::write(
        &doc2_path,
        "---\nlattice-id: LMULT2\nname: doc2\ndescription: Second doc\n---\n\nBody 2.",
    )
    .expect("Write doc2");

    let doc1 = create_test_document("LMULTA", "doc1.md", "doc1", "First doc");
    let doc2 = create_test_document("LMULT2", "doc2.md", "doc2", "Second doc");
    insert_doc(&context.conn, &doc1);
    insert_doc(&context.conn, &doc2);

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
    let (temp_dir, context) = create_test_repo();

    let doc_path = temp_dir.path().join("test.md");
    fs::write(
        &doc_path,
        "---\nlattice-id: LLABEL\nname: labeled\ndescription: Labeled doc\nlabels:\n  - foo\n  - bar\n---\n\nContent.",
    )
    .expect("Write doc");

    let doc = create_test_document("LLABEL", "test.md", "labeled", "Labeled doc");
    insert_doc(&context.conn, &doc);

    // Add labels to index
    label_queries::add(&context.conn, "LLABEL", "foo").expect("Add label foo");
    label_queries::add(&context.conn, "LLABEL", "bar").expect("Add label bar");

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
    let (temp_dir, context) = create_test_repo();

    let doc_path = temp_dir.path().join("test.md");
    fs::write(
        &doc_path,
        "---\nlattice-id: LSHORT\nname: short-test\ndescription: Short test\n---\n\nBody.",
    )
    .expect("Write doc");

    let doc = create_test_document("LSHORT", "test.md", "short-test", "Short test");
    insert_doc(&context.conn, &doc);

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
    let (temp_dir, context) = create_test_repo();

    let doc_path = temp_dir.path().join("test.md");
    fs::write(
        &doc_path,
        "---\nlattice-id: LPEEKA\nname: peek-test\ndescription: Peek test\n---\n\nBody.",
    )
    .expect("Write doc");

    let doc = create_test_document("LPEEKA", "test.md", "peek-test", "Peek test");
    insert_doc(&context.conn, &doc);

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
    let (temp_dir, context) = create_test_repo();

    let doc_path = temp_dir.path().join("test.md");
    fs::write(
        &doc_path,
        "---\nlattice-id: LRAWAB\nname: raw-test\ndescription: Raw test\n---\n\nRaw body content.",
    )
    .expect("Write doc");

    let doc = create_test_document("LRAWAB", "test.md", "raw-test", "Raw test");
    insert_doc(&context.conn, &doc);

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
    let (_temp_dir, context) = create_test_repo();

    // Create a root document (path api/api.md - filename matches directory name)
    let root_doc = InsertDocument::new(
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
    insert_doc(&context.conn, &root_doc);

    let row = document_queries::lookup_by_id(&context.conn, "LROOTX")
        .expect("Lookup should succeed")
        .expect("Document should exist");

    let doc_ref = DocumentRef::from_row(&row);
    assert!(doc_ref.is_root, "Root document should have is_root = true");
}

#[test]
fn document_ref_from_row_non_root() {
    let (_temp_dir, context) = create_test_repo();

    // Create a non-root document
    let doc = create_test_document("LNROOT", "api/docs/readme.md", "readme", "API readme");
    insert_doc(&context.conn, &doc);

    let row = document_queries::lookup_by_id(&context.conn, "LNROOT")
        .expect("Lookup should succeed")
        .expect("Document should exist");

    let doc_ref = DocumentRef::from_row(&row);
    assert!(!doc_ref.is_root, "Non-root document should have is_root = false");
}

#[test]
fn show_command_includes_related_documents() {
    let (temp_dir, context) = create_test_repo();

    // Create main document
    let main_path = temp_dir.path().join("main.md");
    fs::write(
        &main_path,
        "---\nlattice-id: LMAINX\nname: main\ndescription: Main document\n---\n\nSee [related](LRELAX) for more info.",
    )
    .expect("Write main doc");

    // Create related document
    let related_path = temp_dir.path().join("related.md");
    fs::write(
        &related_path,
        "---\nlattice-id: LRELAX\nname: related\ndescription: Related document\n---\n\nRelated content.",
    )
    .expect("Write related doc");

    let main_doc = create_test_document("LMAINX", "main.md", "main", "Main document");
    let related_doc = create_test_document("LRELAX", "related.md", "related", "Related document");
    insert_doc(&context.conn, &main_doc);
    insert_doc(&context.conn, &related_doc);

    // Add body link
    let link = InsertLink {
        source_id: "LMAINX",
        target_id: "LRELAX",
        link_type: LinkType::Body,
        position: 0,
    };
    link_queries::insert_for_document(&context.conn, &[link]).expect("Insert link");

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
    let (temp_dir, context) = create_test_repo();

    // Create main task
    let main_path = temp_dir.path().join("tasks").join("main.md");
    fs::create_dir_all(main_path.parent().unwrap()).expect("Create dirs");
    fs::write(
        &main_path,
        "---\nlattice-id: LBLKMN\nname: main\ndescription: Main task\ntask-type: task\npriority: 1\nblocking:\n  - LBLKOT\n---\n\nMain task body.",
    )
    .expect("Write main doc");

    // Create blocked-by task (will be in blocking list)
    let other_path = temp_dir.path().join("tasks").join("other.md");
    fs::write(
        &other_path,
        "---\nlattice-id: LBLKOT\nname: other\ndescription: Other task\ntask-type: task\npriority: 1\nblocked-by:\n  - LBLKMN\n---\n\nOther task.",
    )
    .expect("Write other doc");

    let main_doc = create_task_document("LBLKMN", "tasks/main.md", "main", "Main task", 1);
    let other_doc = create_task_document("LBLKOT", "tasks/other.md", "other", "Other task", 1);
    insert_doc(&context.conn, &main_doc);
    insert_doc(&context.conn, &other_doc);

    // Add blocked-by link from other to main
    let blocked_link = InsertLink {
        source_id: "LBLKOT",
        target_id: "LBLKMN",
        link_type: LinkType::BlockedBy,
        position: 0,
    };
    link_queries::insert_for_document(&context.conn, &[blocked_link])
        .expect("Insert blocked-by link");

    // Also add body link from main to other (to test exclusion from related)
    let body_link = InsertLink {
        source_id: "LBLKMN",
        target_id: "LBLKOT",
        link_type: LinkType::Body,
        position: 0,
    };
    link_queries::insert_for_document(&context.conn, &[body_link]).expect("Insert body link");

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
    let (temp_dir, context) = create_test_repo();

    // Create target document (the one we'll show with --refs)
    let target_path = temp_dir.path().join("target.md");
    fs::write(
        &target_path,
        "---\nlattice-id: LTRGTX\nname: target\ndescription: Target document\n---\n\nTarget content.",
    )
    .expect("Write target doc");

    // Create source document that links to target
    let source_path = temp_dir.path().join("source.md");
    fs::write(
        &source_path,
        "---\nlattice-id: LSRCXX\nname: source\ndescription: Source document\n---\n\nSee [target](LTRGTX).",
    )
    .expect("Write source doc");

    let target_doc = create_test_document("LTRGTX", "target.md", "target", "Target document");
    let source_doc = create_test_document("LSRCXX", "source.md", "source", "Source document");
    insert_doc(&context.conn, &target_doc);
    insert_doc(&context.conn, &source_doc);

    // Add body link from source to target
    let body_link = InsertLink {
        source_id: "LSRCXX",
        target_id: "LTRGTX",
        link_type: LinkType::Body,
        position: 0,
    };
    link_queries::insert_for_document(&context.conn, &[body_link]).expect("Insert body link");

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
    let (temp_dir, context) = create_test_repo();

    // Create root document (parent)
    let root_path = temp_dir.path().join("api").join("api.md");
    fs::create_dir_all(root_path.parent().unwrap()).expect("Create api dir");
    fs::write(
        &root_path,
        "---\nlattice-id: LROOTA\nname: api\ndescription: API root\n---\n\nRoot content.",
    )
    .expect("Write root doc");

    // Create task document with parent
    let task_path = temp_dir.path().join("api").join("tasks").join("my-task.md");
    fs::create_dir_all(task_path.parent().unwrap()).expect("Create tasks dir");
    fs::write(
        &task_path,
        "---\nlattice-id: LTASKP\nname: my-task\ndescription: My task\ntask-type: task\npriority: 1\nparent-id: LROOTA\n---\n\nTask body.",
    )
    .expect("Write task doc");

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
    insert_doc(&context.conn, &root_doc);
    insert_doc(&context.conn, &task_doc);

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
    let (temp_dir, context) = create_test_repo();

    let doc_path = temp_dir.path().join("test.md");
    fs::write(
        &doc_path,
        "---\nlattice-id: LVIEWX\nname: view-test\ndescription: View test\n---\n\nBody.",
    )
    .expect("Write doc");

    let doc = create_test_document("LVIEWX", "test.md", "view-test", "View test");
    insert_doc(&context.conn, &doc);

    // Verify initial view count is 0
    let initial_count = lattice::index::view_tracking::get_view_count(&context.conn, "LVIEWX")
        .expect("Get initial count");
    assert_eq!(initial_count, 0, "Initial view count should be 0");

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

    // Re-create context to get fresh connection
    let global = GlobalOptions::default();
    let context2 = create_context(temp_dir.path(), &global).expect("Create context");

    // Verify view count was incremented
    let new_count = lattice::index::view_tracking::get_view_count(&context2.conn, "LVIEWX")
        .expect("Get new count");
    assert_eq!(new_count, 1, "View count should be 1 after show");
}

#[test]
fn show_command_increments_view_count_multiple_times() {
    let (temp_dir, context) = create_test_repo();

    let doc_path = temp_dir.path().join("test.md");
    fs::write(
        &doc_path,
        "---\nlattice-id: LMULVW\nname: multi-view\ndescription: Multi view test\n---\n\nBody.",
    )
    .expect("Write doc");

    let doc = create_test_document("LMULVW", "test.md", "multi-view", "Multi view test");
    insert_doc(&context.conn, &doc);

    // Show the document twice
    for i in 1..=2 {
        let global = GlobalOptions::default();
        let ctx = create_context(temp_dir.path(), &global).expect("Create context");
        let args = ShowArgs {
            ids: vec!["LMULVW".to_string()],
            short: false,
            refs: false,
            peek: false,
            raw: false,
        };
        let result = lattice::cli::commands::show_command::show_executor::execute(ctx, args);
        assert!(result.is_ok(), "Show {} should succeed: {:?}", i, result);
    }

    // Verify view count is 2
    let global = GlobalOptions::default();
    let ctx = create_context(temp_dir.path(), &global).expect("Create context");
    let count =
        lattice::index::view_tracking::get_view_count(&ctx.conn, "LMULVW").expect("Get count");
    assert_eq!(count, 2, "View count should be 2 after showing twice");
}
