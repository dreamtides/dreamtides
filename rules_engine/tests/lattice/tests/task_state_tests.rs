use std::path::Path;

use lattice::document::frontmatter_schema::TaskType;
use lattice::index::document_types::InsertDocument;
use lattice::index::{connection_pool, document_queries, link_queries, schema_definition};
use lattice::task::task_state::{
    TaskState, compute_state, compute_state_with_blockers, is_closed_path, state_from_path,
};

/// Creates an in-memory database with the Lattice schema for testing.
fn create_test_db() -> rusqlite::Connection {
    let conn =
        connection_pool::open_memory_connection().expect("Failed to open in-memory connection");
    schema_definition::create_schema(&conn).expect("Failed to create schema");
    conn
}

/// Creates a test document with minimal required fields.
/// Note: is_closed is automatically computed from path (true if path contains
/// /.closed/).
fn create_test_document(id: &str, path: &str) -> InsertDocument {
    InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        "test-doc".to_string(),
        "Test document".to_string(),
        Some(TaskType::Task),
        Some(2),
        None,
        None,
        None,
        "hash123".to_string(),
        100,
    )
}

// ============================================================================
// is_closed_path tests
// ============================================================================

#[test]
fn is_closed_path_returns_true_for_closed_directory() {
    assert!(
        is_closed_path("auth/tasks/.closed/fix_bug.md"),
        "Path containing /.closed/ should be detected as closed"
    );
}

#[test]
fn is_closed_path_returns_true_for_nested_closed_directory() {
    assert!(
        is_closed_path("project/auth/tasks/.closed/feature.md"),
        "Nested path with /.closed/ should be detected as closed"
    );
}

#[test]
fn is_closed_path_returns_false_for_open_task() {
    assert!(
        !is_closed_path("auth/tasks/fix_bug.md"),
        "Path without /.closed/ should not be detected as closed"
    );
}

#[test]
fn is_closed_path_returns_false_for_docs_directory() {
    assert!(
        !is_closed_path("auth/docs/design.md"),
        "Docs directory path should not be detected as closed"
    );
}

#[test]
fn is_closed_path_returns_false_for_partial_closed_match() {
    // "closed" without the dot prefix and slashes should not match
    assert!(
        !is_closed_path("auth/tasks/closed_issue.md"),
        "File named closed_issue should not be detected as closed"
    );
}

#[test]
fn is_closed_path_handles_root_level_closed_directory() {
    assert!(is_closed_path("/.closed/task.md"), "Root-level .closed directory should be detected");
}

// ============================================================================
// state_from_path tests
// ============================================================================

#[test]
fn state_from_path_returns_closed_for_closed_directory() {
    let state = state_from_path("auth/tasks/.closed/fix_bug.md");
    assert_eq!(state, TaskState::Closed, "Path in .closed directory should return Closed state");
}

#[test]
fn state_from_path_returns_open_for_regular_task() {
    let state = state_from_path("auth/tasks/fix_bug.md");
    assert_eq!(state, TaskState::Open, "Regular task path should return Open state");
}

#[test]
fn state_from_path_returns_open_for_docs() {
    let state = state_from_path("auth/docs/design.md");
    assert_eq!(state, TaskState::Open, "Docs path should return Open state");
}

// ============================================================================
// TaskState enum tests
// ============================================================================

#[test]
fn task_state_as_str_returns_correct_values() {
    assert_eq!(TaskState::Open.as_str(), "open");
    assert_eq!(TaskState::Blocked.as_str(), "blocked");
    assert_eq!(TaskState::Closed.as_str(), "closed");
}

#[test]
fn task_state_display_matches_as_str() {
    assert_eq!(format!("{}", TaskState::Open), "open");
    assert_eq!(format!("{}", TaskState::Blocked), "blocked");
    assert_eq!(format!("{}", TaskState::Closed), "closed");
}

#[test]
fn task_state_is_closed_returns_true_only_for_closed() {
    assert!(!TaskState::Open.is_closed());
    assert!(!TaskState::Blocked.is_closed());
    assert!(TaskState::Closed.is_closed());
}

#[test]
fn task_state_is_blocked_returns_true_only_for_blocked() {
    assert!(!TaskState::Open.is_blocked());
    assert!(TaskState::Blocked.is_blocked());
    assert!(!TaskState::Closed.is_blocked());
}

#[test]
fn task_state_is_open_returns_true_only_for_open() {
    assert!(TaskState::Open.is_open());
    assert!(!TaskState::Blocked.is_open());
    assert!(!TaskState::Closed.is_open());
}

// ============================================================================
// compute_state_with_blockers tests
// ============================================================================

#[test]
fn compute_state_with_blockers_returns_closed_when_is_closed_true() {
    let conn = create_test_db();
    let result = compute_state_with_blockers(&conn, "LTEST1", true);
    assert_eq!(
        result.expect("Should succeed"),
        TaskState::Closed,
        "Should return Closed when is_closed is true"
    );
}

#[test]
fn compute_state_with_blockers_returns_open_when_no_blockers() {
    let conn = create_test_db();

    // Insert a document with no blocked-by links
    let doc = create_test_document("LTEST1", "auth/tasks/task.md");
    document_queries::insert(&conn, &doc).expect("Failed to insert document");

    let result = compute_state_with_blockers(&conn, "LTEST1", false);
    assert_eq!(
        result.expect("Should succeed"),
        TaskState::Open,
        "Should return Open when there are no blockers"
    );
}

#[test]
fn compute_state_with_blockers_returns_blocked_when_blocker_is_open() {
    let conn = create_test_db();

    // Create the blocked task
    let blocked_doc = create_test_document("LBLOCKED", "auth/tasks/blocked.md");
    document_queries::insert(&conn, &blocked_doc).expect("Failed to insert blocked document");

    // Create the blocking task (open, not in .closed/)
    let blocker_doc = create_test_document("LBLOCKER", "auth/tasks/blocker.md");
    document_queries::insert(&conn, &blocker_doc).expect("Failed to insert blocker document");

    // Create the blocked-by link
    let link = link_queries::InsertLink {
        source_id: "LBLOCKED",
        target_id: "LBLOCKER",
        link_type: link_queries::LinkType::BlockedBy,
        position: 0,
    };
    link_queries::insert_for_document(&conn, &[link]).expect("Failed to insert link");

    let result = compute_state_with_blockers(&conn, "LBLOCKED", false);
    assert_eq!(
        result.expect("Should succeed"),
        TaskState::Blocked,
        "Should return Blocked when blocker is open"
    );
}

#[test]
fn compute_state_with_blockers_returns_open_when_blocker_is_closed() {
    let conn = create_test_db();

    // Create the task
    let task_doc = create_test_document("LTASK", "auth/tasks/task.md");
    document_queries::insert(&conn, &task_doc).expect("Failed to insert task document");

    // Create the blocker that is closed (in .closed/ directory)
    // is_closed is automatically set to true by InsertDocument::new because path
    // contains /.closed/
    let blocker_doc = create_test_document("LBLOCKER", "auth/tasks/.closed/blocker.md");
    document_queries::insert(&conn, &blocker_doc).expect("Failed to insert blocker document");

    // Create the blocked-by link
    let link = link_queries::InsertLink {
        source_id: "LTASK",
        target_id: "LBLOCKER",
        link_type: link_queries::LinkType::BlockedBy,
        position: 0,
    };
    link_queries::insert_for_document(&conn, &[link]).expect("Failed to insert link");

    let result = compute_state_with_blockers(&conn, "LTASK", false);
    assert_eq!(
        result.expect("Should succeed"),
        TaskState::Open,
        "Should return Open when all blockers are closed"
    );
}

#[test]
fn compute_state_with_blockers_returns_blocked_when_any_blocker_is_open() {
    let conn = create_test_db();

    // Create the task
    let task_doc = create_test_document("LTASK", "auth/tasks/task.md");
    document_queries::insert(&conn, &task_doc).expect("Failed to insert task document");

    // Create closed blocker (is_closed set automatically from path)
    let closed_blocker = create_test_document("LCLOSED", "auth/tasks/.closed/closed.md");
    document_queries::insert(&conn, &closed_blocker).expect("Failed to insert closed blocker");

    // Create open blocker
    let open_blocker = create_test_document("LOPEN", "auth/tasks/open.md");
    document_queries::insert(&conn, &open_blocker).expect("Failed to insert open blocker");

    // Create blocked-by links to both
    let links = [
        link_queries::InsertLink {
            source_id: "LTASK",
            target_id: "LCLOSED",
            link_type: link_queries::LinkType::BlockedBy,
            position: 0,
        },
        link_queries::InsertLink {
            source_id: "LTASK",
            target_id: "LOPEN",
            link_type: link_queries::LinkType::BlockedBy,
            position: 1,
        },
    ];
    link_queries::insert_for_document(&conn, &links).expect("Failed to insert links");

    let result = compute_state_with_blockers(&conn, "LTASK", false);
    assert_eq!(
        result.expect("Should succeed"),
        TaskState::Blocked,
        "Should return Blocked when any blocker is still open"
    );
}

#[test]
fn compute_state_with_blockers_treats_missing_blocker_as_closed() {
    let conn = create_test_db();

    // Create the task
    let task_doc = create_test_document("LTASK", "auth/tasks/task.md");
    document_queries::insert(&conn, &task_doc).expect("Failed to insert task document");

    // Create blocked-by link to non-existent document
    let link = link_queries::InsertLink {
        source_id: "LTASK",
        target_id: "LMISSING",
        link_type: link_queries::LinkType::BlockedBy,
        position: 0,
    };
    link_queries::insert_for_document(&conn, &[link]).expect("Failed to insert link");

    let result = compute_state_with_blockers(&conn, "LTASK", false);
    assert_eq!(
        result.expect("Should succeed"),
        TaskState::Open,
        "Should return Open when blocker document doesn't exist (treat as closed)"
    );
}

// ============================================================================
// compute_state tests
// ============================================================================

#[test]
fn compute_state_integrates_path_and_blocker_checking() {
    let conn = create_test_db();

    // Create an open task
    let task_doc = create_test_document("LTASK", "auth/tasks/task.md");
    document_queries::insert(&conn, &task_doc).expect("Failed to insert task document");

    // Test with open path
    let result = compute_state(&conn, "LTASK", Path::new("auth/tasks/task.md"));
    assert_eq!(
        result.expect("Should succeed"),
        TaskState::Open,
        "compute_state should return Open for task without blockers"
    );
}

#[test]
fn compute_state_detects_closed_from_path() {
    let conn = create_test_db();

    // Create a closed task (is_closed automatically set from path)
    let task_doc = create_test_document("LTASK", "auth/tasks/.closed/task.md");
    document_queries::insert(&conn, &task_doc).expect("Failed to insert task document");

    let result = compute_state(&conn, "LTASK", Path::new("auth/tasks/.closed/task.md"));
    assert_eq!(
        result.expect("Should succeed"),
        TaskState::Closed,
        "compute_state should return Closed for path in .closed directory"
    );
}

// ============================================================================
// Serialization tests
// ============================================================================

#[test]
fn task_state_serializes_to_lowercase() {
    let open_json = serde_json::to_string(&TaskState::Open).expect("Failed to serialize Open");
    assert_eq!(open_json, "\"open\"");

    let blocked_json =
        serde_json::to_string(&TaskState::Blocked).expect("Failed to serialize Blocked");
    assert_eq!(blocked_json, "\"blocked\"");

    let closed_json =
        serde_json::to_string(&TaskState::Closed).expect("Failed to serialize Closed");
    assert_eq!(closed_json, "\"closed\"");
}
