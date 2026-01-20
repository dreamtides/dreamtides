use std::fs;
use std::path::Path;

use lattice::task::root_detection::{
    compute_parent_id, find_ancestors, find_root_for, is_root_document, root_document_path_for,
    root_document_paths_for, validate_hierarchy,
};
use tempfile::TempDir;

// ============================================================================
// is_root_document tests
// ============================================================================

#[test]
fn is_root_document_returns_true_for_matching_filename_and_directory() {
    assert!(
        is_root_document(Path::new("api/api.md")),
        "api/api.md should be detected as root document"
    );
}

#[test]
fn is_root_document_returns_true_for_auth_root() {
    assert!(
        is_root_document(Path::new("auth/auth.md")),
        "auth/auth.md should be detected as root document"
    );
}

#[test]
fn is_root_document_returns_true_for_nested_root() {
    assert!(
        is_root_document(Path::new("project/api/v2/v2.md")),
        "Nested root document v2/v2.md should be detected as root"
    );
}

#[test]
fn is_root_document_returns_false_for_task() {
    assert!(
        !is_root_document(Path::new("auth/tasks/login.md")),
        "auth/tasks/login.md should not be detected as root document"
    );
}

#[test]
fn is_root_document_returns_false_for_docs() {
    assert!(
        !is_root_document(Path::new("api/docs/api_design.md")),
        "Document in docs/ should not be root"
    );
}

#[test]
fn is_root_document_returns_false_for_non_md_file() {
    assert!(!is_root_document(Path::new("api/api.txt")), "Non-markdown file should not be root");
}

#[test]
fn is_root_document_returns_false_for_closed_task() {
    assert!(
        !is_root_document(Path::new("auth/tasks/.closed/login.md")),
        "Closed task should not be root"
    );
}

#[test]
fn is_root_document_returns_false_for_single_file_path() {
    assert!(
        !is_root_document(Path::new("api.md")),
        "Single file without parent directory should not be root"
    );
}

#[test]
fn is_root_document_returns_false_for_mismatched_names() {
    assert!(
        !is_root_document(Path::new("api/auth.md")),
        "api/auth.md should not be root - names don't match"
    );
}

// ============================================================================
// root_document_path_for tests
// ============================================================================

#[test]
fn root_document_path_for_computes_correct_path() {
    let result = root_document_path_for(Path::new("api"));
    assert_eq!(
        result.expect("Should compute root path").as_path(),
        Path::new("api/api.md"),
        "Should return api/api.md for api/ directory"
    );
}

#[test]
fn root_document_path_for_handles_nested_directory() {
    let result = root_document_path_for(Path::new("project/api/v2"));
    assert_eq!(
        result.expect("Should compute root path").as_path(),
        Path::new("project/api/v2/v2.md"),
        "Should return correct path for nested directory"
    );
}

#[test]
fn root_document_path_for_returns_none_for_empty_path() {
    let result = root_document_path_for(Path::new(""));
    assert!(result.is_none(), "Empty path should return None");
}

// ============================================================================
// find_root_for tests (with filesystem)
// ============================================================================

fn create_test_hierarchy(temp_dir: &TempDir) {
    let root = temp_dir.path();

    fs::create_dir_all(root.join("api/tasks")).expect("Create api/tasks");
    fs::create_dir_all(root.join("api/docs")).expect("Create api/docs");
    fs::create_dir_all(root.join("api/v2/tasks")).expect("Create api/v2/tasks");

    fs::write(
        root.join("api/api.md"),
        "---\nlattice-id: LAPIXX\nname: api\ndescription: API module\n---\n",
    )
    .expect("Write api/api.md");

    fs::write(
        root.join("api/v2/v2.md"),
        "---\nlattice-id: LV2YYY\nname: v2\ndescription: API v2\n---\n",
    )
    .expect("Write api/v2/v2.md");

    fs::write(
        root.join("api/tasks/fix_bug.md"),
        "---\nlattice-id: LBUGZZ\nname: fix-bug\ndescription: Fix bug\ntask-type: bug\npriority: 1\n---\n",
    )
    .expect("Write fix_bug.md");

    fs::write(
        root.join("api/docs/design.md"),
        "---\nlattice-id: LDOCWW\nname: design\ndescription: Design doc\n---\n",
    )
    .expect("Write design.md");

    fs::write(
        root.join("api/v2/tasks/v2_task.md"),
        "---\nlattice-id: LV2TKK\nname: v2-task\ndescription: V2 task\ntask-type: task\npriority: 2\n---\n",
    )
    .expect("Write v2_task.md");
}

#[test]
fn find_root_for_returns_parent_root_for_task() {
    let temp_dir = TempDir::new().expect("Create temp dir");
    create_test_hierarchy(&temp_dir);

    let result = find_root_for(Path::new("api/tasks/fix_bug.md"), temp_dir.path());

    assert_eq!(
        result.expect("Should find root").as_path(),
        Path::new("api/api.md"),
        "Task should find parent directory's root document"
    );
}

#[test]
fn find_root_for_returns_parent_root_for_docs() {
    let temp_dir = TempDir::new().expect("Create temp dir");
    create_test_hierarchy(&temp_dir);

    let result = find_root_for(Path::new("api/docs/design.md"), temp_dir.path());

    assert_eq!(
        result.expect("Should find root").as_path(),
        Path::new("api/api.md"),
        "Document in docs/ should find parent directory's root"
    );
}

#[test]
fn find_root_for_returns_immediate_root_for_nested() {
    let temp_dir = TempDir::new().expect("Create temp dir");
    create_test_hierarchy(&temp_dir);

    let result = find_root_for(Path::new("api/v2/tasks/v2_task.md"), temp_dir.path());

    assert_eq!(
        result.expect("Should find root").as_path(),
        Path::new("api/v2/v2.md"),
        "Nested task should find the immediate enclosing root (v2/v2.md)"
    );
}

#[test]
fn find_root_for_returns_none_when_no_root_exists() {
    let temp_dir = TempDir::new().expect("Create temp dir");
    fs::create_dir_all(temp_dir.path().join("orphan/tasks")).expect("Create orphan/tasks");
    fs::write(
        temp_dir.path().join("orphan/tasks/task.md"),
        "---\nlattice-id: LORPHN\nname: task\ndescription: Orphan task\ntask-type: task\npriority: 2\n---\n",
    )
    .expect("Write orphan task");

    let result = find_root_for(Path::new("orphan/tasks/task.md"), temp_dir.path());

    assert!(result.is_none(), "Should return None when no root exists");
}

// ============================================================================
// compute_parent_id tests
// ============================================================================

#[test]
fn compute_parent_id_returns_root_id_for_task() {
    let temp_dir = TempDir::new().expect("Create temp dir");
    create_test_hierarchy(&temp_dir);

    let parent_id = compute_parent_id(Path::new("api/tasks/fix_bug.md"), temp_dir.path());

    assert_eq!(
        parent_id.expect("Should compute parent ID").as_str(),
        "LAPIXX",
        "Task's parent should be the root document's ID"
    );
}

#[test]
fn compute_parent_id_returns_parent_root_for_root_document() {
    let temp_dir = TempDir::new().expect("Create temp dir");
    create_test_hierarchy(&temp_dir);

    let parent_id = compute_parent_id(Path::new("api/v2/v2.md"), temp_dir.path());

    assert_eq!(
        parent_id.expect("Should compute parent ID").as_str(),
        "LAPIXX",
        "Nested root's parent should be the parent directory's root"
    );
}

#[test]
fn compute_parent_id_returns_error_for_top_level_root() {
    let temp_dir = TempDir::new().expect("Create temp dir");
    create_test_hierarchy(&temp_dir);

    let result = compute_parent_id(Path::new("api/api.md"), temp_dir.path());

    assert!(result.is_err(), "Top-level root should have no parent and return error");
}

#[test]
fn compute_parent_id_returns_error_when_no_root_exists() {
    let temp_dir = TempDir::new().expect("Create temp dir");
    fs::create_dir_all(temp_dir.path().join("orphan")).expect("Create orphan");
    fs::write(
        temp_dir.path().join("orphan/task.md"),
        "---\nlattice-id: LORPHN\nname: task\ndescription: Orphan\n---\n",
    )
    .expect("Write orphan");

    let result = compute_parent_id(Path::new("orphan/task.md"), temp_dir.path());

    let err = result.expect_err("Should return error for orphan document");
    let debug_str = format!("{:?}", err);
    assert!(
        debug_str.contains("RootDocumentNotFound"),
        "Error should be RootDocumentNotFound: {:?}",
        err
    );
}

// ============================================================================
// find_ancestors tests
// ============================================================================

#[test]
fn find_ancestors_returns_single_ancestor_for_simple_task() {
    let temp_dir = TempDir::new().expect("Create temp dir");
    create_test_hierarchy(&temp_dir);

    let ancestors = find_ancestors(Path::new("api/tasks/fix_bug.md"), temp_dir.path());

    assert_eq!(ancestors.len(), 1, "Should find exactly one ancestor");
    assert_eq!(ancestors[0].as_path(), Path::new("api/api.md"), "Ancestor should be api/api.md");
}

#[test]
fn find_ancestors_returns_chain_for_nested_task() {
    let temp_dir = TempDir::new().expect("Create temp dir");
    create_test_hierarchy(&temp_dir);

    let ancestors = find_ancestors(Path::new("api/v2/tasks/v2_task.md"), temp_dir.path());

    assert_eq!(ancestors.len(), 2, "Should find two ancestors");
    assert_eq!(
        ancestors[0].as_path(),
        Path::new("api/v2/v2.md"),
        "First ancestor should be immediate root"
    );
    assert_eq!(
        ancestors[1].as_path(),
        Path::new("api/api.md"),
        "Second ancestor should be parent root"
    );
}

#[test]
fn find_ancestors_excludes_self_for_root_document() {
    let temp_dir = TempDir::new().expect("Create temp dir");
    create_test_hierarchy(&temp_dir);

    let ancestors = find_ancestors(Path::new("api/v2/v2.md"), temp_dir.path());

    assert_eq!(ancestors.len(), 1, "Root should find one ancestor");
    assert_eq!(
        ancestors[0].as_path(),
        Path::new("api/api.md"),
        "Root's ancestor should be parent directory's root"
    );
}

#[test]
fn find_ancestors_returns_empty_for_top_level_root() {
    let temp_dir = TempDir::new().expect("Create temp dir");
    create_test_hierarchy(&temp_dir);

    let ancestors = find_ancestors(Path::new("api/api.md"), temp_dir.path());

    assert!(ancestors.is_empty(), "Top-level root should have no ancestors");
}

#[test]
fn find_ancestors_returns_empty_for_orphan() {
    let temp_dir = TempDir::new().expect("Create temp dir");
    fs::create_dir_all(temp_dir.path().join("orphan")).expect("Create orphan");
    fs::write(
        temp_dir.path().join("orphan/task.md"),
        "---\nlattice-id: LORPHN\nname: task\ndescription: Orphan\n---\n",
    )
    .expect("Write orphan");

    let ancestors = find_ancestors(Path::new("orphan/task.md"), temp_dir.path());

    assert!(ancestors.is_empty(), "Orphan document should have no ancestors");
}

// ============================================================================
// validate_hierarchy tests
// ============================================================================

#[test]
fn validate_hierarchy_accepts_task_with_root() {
    let temp_dir = TempDir::new().expect("Create temp dir");
    create_test_hierarchy(&temp_dir);

    let result = validate_hierarchy(Path::new("api/tasks/fix_bug.md"), temp_dir.path());

    assert!(result.is_ok(), "Task with root should pass validation");
}

#[test]
fn validate_hierarchy_accepts_root_document() {
    let temp_dir = TempDir::new().expect("Create temp dir");
    create_test_hierarchy(&temp_dir);

    let result = validate_hierarchy(Path::new("api/api.md"), temp_dir.path());

    assert!(result.is_ok(), "Root document should pass validation");
}

#[test]
fn validate_hierarchy_rejects_orphan_document() {
    let temp_dir = TempDir::new().expect("Create temp dir");
    fs::create_dir_all(temp_dir.path().join("orphan")).expect("Create orphan");
    fs::write(
        temp_dir.path().join("orphan/task.md"),
        "---\nlattice-id: LORPHN\nname: task\ndescription: Orphan\n---\n",
    )
    .expect("Write orphan");

    let result = validate_hierarchy(Path::new("orphan/task.md"), temp_dir.path());

    let err = result.expect_err("Orphan should fail validation");
    let debug_str = format!("{:?}", err);
    assert!(
        debug_str.contains("RootDocumentNotFound"),
        "Error should mention missing root: {:?}",
        err
    );
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn is_root_document_handles_dots_in_directory_names() {
    assert!(
        !is_root_document(Path::new("api.v2/api.md")),
        "api.v2/api.md should not be root - names don't match"
    );
    assert!(
        is_root_document(Path::new("api.v2/api.v2.md")),
        "api.v2/api.v2.md should be root - names match"
    );
}

#[test]
fn is_root_document_is_case_sensitive() {
    assert!(
        !is_root_document(Path::new("API/api.md")),
        "API/api.md should not be root - case doesn't match"
    );
}

#[test]
fn find_ancestors_preserves_order() {
    let temp_dir = TempDir::new().expect("Create temp dir");
    let root = temp_dir.path();

    fs::create_dir_all(root.join("a/b/c/tasks")).expect("Create nested dirs");
    fs::write(root.join("a/a.md"), "---\nlattice-id: LAXXXX\nname: a\ndescription: A\n---\n")
        .expect("Write a.md");
    fs::write(root.join("a/b/b.md"), "---\nlattice-id: LBXXXX\nname: b\ndescription: B\n---\n")
        .expect("Write b.md");
    fs::write(root.join("a/b/c/c.md"), "---\nlattice-id: LCXXXX\nname: c\ndescription: C\n---\n")
        .expect("Write c.md");
    fs::write(
        root.join("a/b/c/tasks/task.md"),
        "---\nlattice-id: LTXXXX\nname: task\ndescription: Task\ntask-type: task\npriority: 2\n---\n",
    )
    .expect("Write task.md");

    let ancestors = find_ancestors(Path::new("a/b/c/tasks/task.md"), root);

    assert_eq!(ancestors.len(), 3, "Should find three ancestors");
    assert_eq!(ancestors[0].as_path(), Path::new("a/b/c/c.md"));
    assert_eq!(ancestors[1].as_path(), Path::new("a/b/b.md"));
    assert_eq!(ancestors[2].as_path(), Path::new("a/a.md"));
}

// ============================================================================
// Underscore-prefixed root document tests
// ============================================================================

#[test]
fn is_root_document_returns_true_for_underscore_prefixed_root() {
    assert!(
        is_root_document(Path::new("api/_api.md")),
        "api/_api.md should be detected as root document"
    );
}

#[test]
fn is_root_document_returns_true_for_nested_underscore_prefixed_root() {
    assert!(
        is_root_document(Path::new("project/api/v2/_v2.md")),
        "project/api/v2/_v2.md should be detected as root document"
    );
}

#[test]
fn is_root_document_returns_false_for_arbitrary_underscore_prefix() {
    assert!(
        !is_root_document(Path::new("api/_other.md")),
        "api/_other.md should NOT be root - filename doesn't match directory"
    );
}

#[test]
fn is_root_document_returns_false_for_double_underscore() {
    assert!(
        !is_root_document(Path::new("api/__api.md")),
        "api/__api.md should NOT be root - double underscore"
    );
}

#[test]
fn root_document_paths_for_returns_both_variants() {
    let paths = root_document_paths_for(Path::new("api"));
    assert_eq!(paths.len(), 2, "Should return both variants");
    assert_eq!(paths[0].as_path(), Path::new("api/api.md"), "First should be standard form");
    assert_eq!(paths[1].as_path(), Path::new("api/_api.md"), "Second should be underscore form");
}

#[test]
fn find_root_for_finds_underscore_prefixed_root() {
    let temp_dir = TempDir::new().expect("Create temp dir");
    let root = temp_dir.path();

    fs::create_dir_all(root.join("api/tasks")).expect("Create api/tasks");
    fs::write(
        root.join("api/_api.md"),
        "---\nlattice-id: LAPIXX\nname: api\ndescription: API module\n---\n",
    )
    .expect("Write _api.md");
    fs::write(
        root.join("api/tasks/fix_bug.md"),
        "---\nlattice-id: LBUGZZ\nname: fix-bug\ndescription: Fix bug\ntask-type: bug\npriority: 1\n---\n",
    )
    .expect("Write fix_bug.md");

    let result = find_root_for(Path::new("api/tasks/fix_bug.md"), root);

    assert_eq!(
        result.expect("Should find root").as_path(),
        Path::new("api/_api.md"),
        "Task should find underscore-prefixed root document"
    );
}

#[test]
fn compute_parent_id_works_with_underscore_prefixed_root() {
    let temp_dir = TempDir::new().expect("Create temp dir");
    let root = temp_dir.path();

    fs::create_dir_all(root.join("api/tasks")).expect("Create api/tasks");
    fs::write(
        root.join("api/_api.md"),
        "---\nlattice-id: LAPIXX\nname: api\ndescription: API module\n---\n",
    )
    .expect("Write _api.md");
    fs::write(
        root.join("api/tasks/fix_bug.md"),
        "---\nlattice-id: LBUGZZ\nname: fix-bug\ndescription: Fix bug\ntask-type: bug\npriority: 1\n---\n",
    )
    .expect("Write fix_bug.md");

    let result = compute_parent_id(Path::new("api/tasks/fix_bug.md"), root);

    assert_eq!(
        result.expect("Should compute parent ID").as_str(),
        "LAPIXX",
        "Parent ID should be from underscore-prefixed root"
    );
}

#[test]
fn find_ancestors_finds_underscore_prefixed_root() {
    let temp_dir = TempDir::new().expect("Create temp dir");
    let root = temp_dir.path();

    fs::create_dir_all(root.join("a/b/tasks")).expect("Create nested dirs");
    fs::write(root.join("a/_a.md"), "---\nlattice-id: LAXXXX\nname: a\ndescription: A\n---\n")
        .expect("Write _a.md");
    fs::write(root.join("a/b/_b.md"), "---\nlattice-id: LBXXXX\nname: b\ndescription: B\n---\n")
        .expect("Write _b.md");
    fs::write(
        root.join("a/b/tasks/task.md"),
        "---\nlattice-id: LTXXXX\nname: task\ndescription: Task\ntask-type: task\npriority: 2\n---\n",
    )
    .expect("Write task.md");

    let ancestors = find_ancestors(Path::new("a/b/tasks/task.md"), root);

    assert_eq!(ancestors.len(), 2, "Should find two ancestors");
    assert_eq!(ancestors[0].as_path(), Path::new("a/b/_b.md"));
    assert_eq!(ancestors[1].as_path(), Path::new("a/_a.md"));
}

#[test]
fn find_root_for_prefers_standard_form_over_underscore() {
    let temp_dir = TempDir::new().expect("Create temp dir");
    let root = temp_dir.path();

    fs::create_dir_all(root.join("api/tasks")).expect("Create api/tasks");
    // Create both forms
    fs::write(
        root.join("api/api.md"),
        "---\nlattice-id: LAPIST\nname: api\ndescription: API standard\n---\n",
    )
    .expect("Write api.md");
    fs::write(
        root.join("api/_api.md"),
        "---\nlattice-id: LAPIUN\nname: api\ndescription: API underscore\n---\n",
    )
    .expect("Write _api.md");
    fs::write(
        root.join("api/tasks/fix_bug.md"),
        "---\nlattice-id: LBUGZZ\nname: fix-bug\ndescription: Fix bug\ntask-type: bug\npriority: 1\n---\n",
    )
    .expect("Write fix_bug.md");

    let result = find_root_for(Path::new("api/tasks/fix_bug.md"), root);

    assert_eq!(
        result.expect("Should find root").as_path(),
        Path::new("api/api.md"),
        "Standard form should take priority when both exist"
    );
}
