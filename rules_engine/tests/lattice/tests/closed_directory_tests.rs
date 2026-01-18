use std::path::Path;

use lattice::task::closed_directory::{
    CLOSED_DIR_NAME, closed_path_for, ensure_closed_dir, is_in_closed, unclosed_path_for,
    validate_closed_path_structure,
};
use tempfile::TempDir;

// ============================================================================
// is_in_closed tests
// ============================================================================

#[test]
fn is_in_closed_returns_true_for_closed_directory() {
    assert!(
        is_in_closed("auth/tasks/.closed/fix_bug.md"),
        "Path containing /.closed/ should be detected as closed"
    );
}

#[test]
fn is_in_closed_returns_true_for_nested_project_path() {
    assert!(
        is_in_closed("project/auth/tasks/.closed/feature.md"),
        "Nested path with /.closed/ should be detected as closed"
    );
}

#[test]
fn is_in_closed_returns_false_for_open_task() {
    assert!(
        !is_in_closed("auth/tasks/fix_bug.md"),
        "Path without /.closed/ should not be detected as closed"
    );
}

#[test]
fn is_in_closed_returns_false_for_partial_closed_match() {
    assert!(
        !is_in_closed("auth/tasks/closed_issue.md"),
        "File named closed_issue should not be detected as closed"
    );
}

#[test]
fn is_in_closed_returns_false_for_closed_without_slashes() {
    assert!(
        !is_in_closed("auth/tasks.closed/foo.md"),
        "Path with .closed not surrounded by slashes should not match"
    );
}

// ============================================================================
// closed_path_for tests
// ============================================================================

#[test]
fn closed_path_for_computes_correct_path() {
    let result = closed_path_for(Path::new("auth/tasks/fix_bug.md"));
    assert_eq!(
        result.expect("Should compute closed path").as_path(),
        Path::new("auth/tasks/.closed/fix_bug.md"),
        "Should insert .closed directory before filename"
    );
}

#[test]
fn closed_path_for_handles_deep_nesting() {
    let result = closed_path_for(Path::new("project/api/auth/tasks/oauth_bug.md"));
    assert_eq!(
        result.expect("Should compute closed path").as_path(),
        Path::new("project/api/auth/tasks/.closed/oauth_bug.md"),
        "Should insert .closed in deeply nested path"
    );
}

#[test]
fn closed_path_for_errors_if_already_closed() {
    let result = closed_path_for(Path::new("auth/tasks/.closed/fix_bug.md"));
    let err = result.expect_err("Should reject already-closed path");
    let debug_str = format!("{:?}", err);
    assert!(
        debug_str.contains("already in a .closed/ directory"),
        "Error should mention already closed: {:?}",
        err
    );
}

#[test]
fn closed_path_for_handles_simple_path() {
    let result = closed_path_for(Path::new("tasks/fix_bug.md"));
    assert_eq!(
        result.expect("Should compute closed path").as_path(),
        Path::new("tasks/.closed/fix_bug.md"),
        "Should handle simple task path"
    );
}

// ============================================================================
// unclosed_path_for tests
// ============================================================================

#[test]
fn unclosed_path_for_computes_correct_path() {
    let result = unclosed_path_for(Path::new("auth/tasks/.closed/fix_bug.md"));
    assert_eq!(
        result.expect("Should compute unclosed path").as_path(),
        Path::new("auth/tasks/fix_bug.md"),
        "Should remove .closed directory from path"
    );
}

#[test]
fn unclosed_path_for_handles_deep_nesting() {
    let result = unclosed_path_for(Path::new("project/api/auth/tasks/.closed/oauth_bug.md"));
    assert_eq!(
        result.expect("Should compute unclosed path").as_path(),
        Path::new("project/api/auth/tasks/oauth_bug.md"),
        "Should handle deeply nested closed path"
    );
}

#[test]
fn unclosed_path_for_errors_if_not_closed() {
    let result = unclosed_path_for(Path::new("auth/tasks/fix_bug.md"));
    let err = result.expect_err("Should reject non-closed path");
    let debug_str = format!("{:?}", err);
    assert!(
        debug_str.contains("not in a .closed/ directory"),
        "Error should mention not closed: {:?}",
        err
    );
}

#[test]
fn unclosed_path_for_handles_simple_path() {
    let result = unclosed_path_for(Path::new("tasks/.closed/fix_bug.md"));
    assert_eq!(
        result.expect("Should compute unclosed path").as_path(),
        Path::new("tasks/fix_bug.md"),
        "Should handle simple closed task path"
    );
}

// ============================================================================
// ensure_closed_dir tests
// ============================================================================

#[test]
fn ensure_closed_dir_creates_directory() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let repo_root = temp_dir.path();
    let tasks_dir = Path::new("auth/tasks");

    std::fs::create_dir_all(repo_root.join(tasks_dir)).expect("Failed to create tasks directory");

    let result = ensure_closed_dir(tasks_dir, repo_root);
    let closed_dir = result.expect("Should create .closed directory");

    assert_eq!(
        closed_dir.as_path(),
        Path::new("auth/tasks/.closed"),
        "Should return relative path to closed directory"
    );
    assert!(repo_root.join(&closed_dir).exists(), "Closed directory should exist on filesystem");
}

#[test]
fn ensure_closed_dir_is_idempotent() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let repo_root = temp_dir.path();
    let tasks_dir = Path::new("api/tasks");

    std::fs::create_dir_all(repo_root.join(tasks_dir)).expect("Failed to create tasks directory");

    let result1 = ensure_closed_dir(tasks_dir, repo_root);
    let result2 = ensure_closed_dir(tasks_dir, repo_root);

    assert!(result1.is_ok(), "First call should succeed");
    assert!(result2.is_ok(), "Second call should also succeed");
    assert_eq!(
        result1.expect("First should succeed"),
        result2.expect("Second should succeed"),
        "Both calls should return the same path"
    );
}

// ============================================================================
// validate_closed_path_structure tests
// ============================================================================

#[test]
fn validate_closed_path_structure_accepts_valid_path() {
    let result = validate_closed_path_structure(Path::new("auth/tasks/.closed/fix_bug.md"));
    assert!(result.is_ok(), "Valid closed path should pass validation");
}

#[test]
fn validate_closed_path_structure_rejects_nested_closed() {
    let result =
        validate_closed_path_structure(Path::new("auth/tasks/.closed/subtasks/.closed/nested.md"));
    let err = result.expect_err("Nested .closed should fail validation");
    assert!(
        err.to_string().contains("nested .closed/"),
        "Error should mention nested closed: {}",
        err
    );
}

#[test]
fn validate_closed_path_structure_accepts_open_path() {
    let result = validate_closed_path_structure(Path::new("auth/tasks/fix_bug.md"));
    assert!(result.is_ok(), "Open task path should pass validation");
}

// ============================================================================
// Round-trip tests
// ============================================================================

#[test]
fn closed_and_unclosed_are_inverse_operations() {
    let original = Path::new("auth/tasks/implement_feature.md");
    let closed = closed_path_for(original).expect("Should compute closed path");
    let unclosed = unclosed_path_for(&closed).expect("Should compute unclosed path");

    assert_eq!(original, unclosed, "unclosed_path_for should reverse closed_path_for");
}

#[test]
fn round_trip_with_deep_nesting() {
    let original = Path::new("project/api/v2/auth/tasks/oauth_refresh.md");
    let closed = closed_path_for(original).expect("Should compute closed path");
    let unclosed = unclosed_path_for(&closed).expect("Should compute unclosed path");

    assert_eq!(original, unclosed, "Round trip should preserve deeply nested path");
}

// ============================================================================
// CLOSED_DIR_NAME constant test
// ============================================================================

#[test]
fn closed_dir_name_constant_is_correct() {
    assert_eq!(CLOSED_DIR_NAME, ".closed", "CLOSED_DIR_NAME should be .closed");
}
