use std::path::Path;

use lattice::task::directory_structure::{
    LocationWarning, expected_location, is_in_docs_dir, is_in_tasks_dir, target_dir_name,
    validate_location,
};

// ============================================================================
// is_in_tasks_dir tests
// ============================================================================

#[test]
fn is_in_tasks_dir_returns_true_for_tasks_directory() {
    assert!(is_in_tasks_dir("auth/tasks/fix_bug.md"), "Path containing /tasks/ should be detected");
}

#[test]
fn is_in_tasks_dir_returns_true_for_nested_tasks() {
    assert!(
        is_in_tasks_dir("project/api/auth/tasks/feature.md"),
        "Nested path with /tasks/ should be detected"
    );
}

#[test]
fn is_in_tasks_dir_returns_true_for_closed_tasks() {
    assert!(
        is_in_tasks_dir("auth/tasks/.closed/fix_bug.md"),
        "Closed task path should still be in tasks directory"
    );
}

#[test]
fn is_in_tasks_dir_returns_false_for_docs() {
    assert!(!is_in_tasks_dir("auth/docs/design.md"), "Path in docs/ should not be in tasks");
}

#[test]
fn is_in_tasks_dir_returns_false_for_partial_match() {
    assert!(!is_in_tasks_dir("auth/my_tasks_file.md"), "Partial match should not be detected");
}

#[test]
fn is_in_tasks_dir_returns_false_without_slashes() {
    assert!(!is_in_tasks_dir("authtasks/foo.md"), "Path without /tasks/ segment should not match");
}

// ============================================================================
// is_in_docs_dir tests
// ============================================================================

#[test]
fn is_in_docs_dir_returns_true_for_docs_directory() {
    assert!(is_in_docs_dir("api/docs/design.md"), "Path containing /docs/ should be detected");
}

#[test]
fn is_in_docs_dir_returns_true_for_nested_docs() {
    assert!(
        is_in_docs_dir("project/api/auth/docs/security.md"),
        "Nested path with /docs/ should be detected"
    );
}

#[test]
fn is_in_docs_dir_returns_false_for_tasks() {
    assert!(!is_in_docs_dir("auth/tasks/fix_bug.md"), "Path in tasks/ should not be in docs");
}

#[test]
fn is_in_docs_dir_returns_false_for_partial_match() {
    assert!(!is_in_docs_dir("auth/my_docs_file.md"), "Partial match should not be detected");
}

#[test]
fn is_in_docs_dir_returns_false_without_slashes() {
    assert!(!is_in_docs_dir("apidocs/foo.md"), "Path without /docs/ segment should not match");
}

// ============================================================================
// validate_location tests - valid locations
// ============================================================================

#[test]
fn validate_location_accepts_root_document() {
    let result = validate_location(Path::new("api/api.md"), false);
    assert!(result.is_none(), "Root document should be valid regardless of task-type");
}

#[test]
fn validate_location_accepts_root_document_with_task_type() {
    let result = validate_location(Path::new("api/api.md"), true);
    assert!(result.is_none(), "Root document with task-type should still be valid");
}

#[test]
fn validate_location_accepts_task_in_tasks_dir() {
    let result = validate_location(Path::new("api/tasks/fix_bug.md"), true);
    assert!(result.is_none(), "Task in tasks/ directory should be valid");
}

#[test]
fn validate_location_accepts_kb_doc_in_docs_dir() {
    let result = validate_location(Path::new("api/docs/design.md"), false);
    assert!(result.is_none(), "Knowledge base doc in docs/ directory should be valid");
}

#[test]
fn validate_location_accepts_closed_task_in_tasks() {
    let result = validate_location(Path::new("api/tasks/.closed/fix_bug.md"), true);
    assert!(result.is_none(), "Closed task in tasks/.closed/ should be valid");
}

// ============================================================================
// validate_location tests - W017: not in standard location
// ============================================================================

#[test]
fn validate_location_warns_loose_file() {
    let result = validate_location(Path::new("api/loose_file.md"), false);
    assert_eq!(
        result,
        Some(LocationWarning::NotInStandardLocation),
        "File not in tasks/ or docs/ should trigger W017"
    );
}

#[test]
fn validate_location_warns_loose_task() {
    let result = validate_location(Path::new("api/loose_task.md"), true);
    assert_eq!(
        result,
        Some(LocationWarning::NotInStandardLocation),
        "Task not in tasks/ should trigger W017"
    );
}

// ============================================================================
// validate_location tests - W018: task in docs/
// ============================================================================

#[test]
fn validate_location_warns_task_in_docs() {
    let result = validate_location(Path::new("api/docs/bug_fix.md"), true);
    assert_eq!(
        result,
        Some(LocationWarning::TaskInDocsDir),
        "Task in docs/ directory should trigger W018"
    );
}

#[test]
fn validate_location_warns_task_in_nested_docs() {
    let result = validate_location(Path::new("project/api/docs/subdocs/fix.md"), true);
    assert_eq!(
        result,
        Some(LocationWarning::TaskInDocsDir),
        "Task in nested docs/ path should trigger W018"
    );
}

// ============================================================================
// validate_location tests - W019: KB doc in tasks/
// ============================================================================

#[test]
fn validate_location_warns_kb_in_tasks() {
    let result = validate_location(Path::new("api/tasks/design.md"), false);
    assert_eq!(
        result,
        Some(LocationWarning::KnowledgeBaseInTasksDir),
        "Knowledge base doc in tasks/ should trigger W019"
    );
}

#[test]
fn validate_location_allows_kb_in_closed() {
    let result = validate_location(Path::new("api/tasks/.closed/old_doc.md"), false);
    assert!(
        result.is_none(),
        "Closed docs without task-type should be allowed (may have been tasks)"
    );
}

// ============================================================================
// expected_location tests
// ============================================================================

#[test]
fn expected_location_returns_same_for_root_document() {
    let path = Path::new("api/api.md");
    assert_eq!(
        expected_location(path, false).as_path(),
        path,
        "Root document should stay where it is"
    );
}

#[test]
fn expected_location_returns_same_for_task_in_tasks() {
    let path = Path::new("api/tasks/fix_bug.md");
    assert_eq!(
        expected_location(path, true).as_path(),
        path,
        "Task in tasks/ should stay where it is"
    );
}

#[test]
fn expected_location_returns_same_for_kb_in_docs() {
    let path = Path::new("api/docs/design.md");
    assert_eq!(
        expected_location(path, false).as_path(),
        path,
        "KB doc in docs/ should stay where it is"
    );
}

#[test]
fn expected_location_moves_loose_task_to_tasks() {
    let result = expected_location(Path::new("api/fix_bug.md"), true);
    assert_eq!(
        result.as_path(),
        Path::new("api/tasks/fix_bug.md"),
        "Loose task should be moved to tasks/"
    );
}

#[test]
fn expected_location_moves_loose_kb_to_docs() {
    let result = expected_location(Path::new("api/design.md"), false);
    assert_eq!(
        result.as_path(),
        Path::new("api/docs/design.md"),
        "Loose KB doc should be moved to docs/"
    );
}

#[test]
fn expected_location_moves_task_from_docs_to_tasks() {
    let result = expected_location(Path::new("api/docs/fix_bug.md"), true);
    assert_eq!(
        result.as_path(),
        Path::new("api/tasks/fix_bug.md"),
        "Task in docs/ should be moved to tasks/"
    );
}

#[test]
fn expected_location_moves_kb_from_tasks_to_docs() {
    let result = expected_location(Path::new("api/tasks/design.md"), false);
    assert_eq!(
        result.as_path(),
        Path::new("api/docs/design.md"),
        "KB doc in tasks/ should be moved to docs/"
    );
}

#[test]
fn expected_location_handles_nested_path() {
    let result = expected_location(Path::new("project/api/v2/docs/fix_bug.md"), true);
    assert_eq!(
        result.as_path(),
        Path::new("project/api/v2/tasks/fix_bug.md"),
        "Should replace first /docs/ with /tasks/"
    );
}

// ============================================================================
// LocationWarning tests
// ============================================================================

#[test]
fn location_warning_codes_are_correct() {
    assert_eq!(
        LocationWarning::NotInStandardLocation.code(),
        "W017",
        "NotInStandardLocation should be W017"
    );
    assert_eq!(LocationWarning::TaskInDocsDir.code(), "W018", "TaskInDocsDir should be W018");
    assert_eq!(
        LocationWarning::KnowledgeBaseInTasksDir.code(),
        "W019",
        "KnowledgeBaseInTasksDir should be W019"
    );
}

#[test]
fn location_warning_message_includes_path() {
    let path = Path::new("api/loose_file.md");
    let message = LocationWarning::NotInStandardLocation.message(path);
    assert!(message.contains("api/loose_file.md"), "Message should include the path: {}", message);
}

#[test]
fn location_warning_task_in_docs_message() {
    let path = Path::new("api/docs/task.md");
    let message = LocationWarning::TaskInDocsDir.message(path);
    assert!(
        message.contains("task") && message.contains("docs/"),
        "Message should mention task in docs: {}",
        message
    );
}

#[test]
fn location_warning_kb_in_tasks_message() {
    let path = Path::new("api/tasks/kb.md");
    let message = LocationWarning::KnowledgeBaseInTasksDir.message(path);
    assert!(
        message.contains("knowledge base") && message.contains("tasks/"),
        "Message should mention KB in tasks: {}",
        message
    );
}

// ============================================================================
// target_dir_name tests
// ============================================================================

#[test]
fn target_dir_name_returns_tasks_for_task_type() {
    assert_eq!(target_dir_name(true), "tasks", "Should return 'tasks' for task documents");
}

#[test]
fn target_dir_name_returns_docs_for_kb() {
    assert_eq!(target_dir_name(false), "docs", "Should return 'docs' for knowledge base documents");
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn validate_location_handles_deeply_nested_path() {
    let result = validate_location(Path::new("a/b/c/d/tasks/e/fix.md"), true);
    assert!(result.is_none(), "Deeply nested task in tasks/ should be valid");
}

#[test]
fn validate_location_with_both_tasks_and_docs_in_path() {
    let result = validate_location(Path::new("tasks/docs/tasks/file.md"), true);
    assert!(
        result.is_none(),
        "Path with both /tasks/ and /docs/ should check for presence of /tasks/"
    );
}

#[test]
fn expected_location_handles_empty_parent() {
    let result = expected_location(Path::new("fix_bug.md"), true);
    assert_eq!(
        result.as_path(),
        Path::new("tasks/fix_bug.md"),
        "File at root should get tasks/ prefix"
    );
}
