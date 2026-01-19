use std::path::PathBuf;

use lattice::git::git_ops::{FileChange, FileStatus};

#[test]
fn file_status_is_staged_returns_true_for_modified_in_index() {
    let status =
        FileStatus { path: PathBuf::from("file.md"), index_status: 'M', worktree_status: ' ' };

    assert!(status.is_staged(), "File modified in index should be considered staged");
}

#[test]
fn file_status_is_staged_returns_true_for_added_in_index() {
    let status =
        FileStatus { path: PathBuf::from("new_file.md"), index_status: 'A', worktree_status: ' ' };

    assert!(status.is_staged(), "File added to index should be considered staged");
}

#[test]
fn file_status_is_staged_returns_false_for_untracked() {
    let status =
        FileStatus { path: PathBuf::from("untracked.md"), index_status: '?', worktree_status: '?' };

    assert!(!status.is_staged(), "Untracked file should not be considered staged");
}

#[test]
fn file_status_is_staged_returns_false_for_only_worktree_changes() {
    let status = FileStatus {
        path: PathBuf::from("worktree_only.md"),
        index_status: ' ',
        worktree_status: 'M',
    };

    assert!(!status.is_staged(), "File with only worktree changes should not be staged");
}

#[test]
fn file_status_is_modified_returns_true_for_worktree_changes() {
    let status =
        FileStatus { path: PathBuf::from("modified.md"), index_status: ' ', worktree_status: 'M' };

    assert!(status.is_modified(), "File modified in worktree should return true for is_modified");
}

#[test]
fn file_status_is_modified_returns_false_for_clean_worktree() {
    let status = FileStatus {
        path: PathBuf::from("staged_only.md"),
        index_status: 'M',
        worktree_status: ' ',
    };

    assert!(!status.is_modified(), "File with clean worktree should return false for is_modified");
}

#[test]
fn file_status_is_untracked_returns_true_for_question_marks() {
    let status = FileStatus {
        path: PathBuf::from("new_untracked.md"),
        index_status: '?',
        worktree_status: '?',
    };

    assert!(status.is_untracked(), "File with ?? status should be considered untracked");
}

#[test]
fn file_status_is_untracked_returns_false_for_tracked_files() {
    let status =
        FileStatus { path: PathBuf::from("tracked.md"), index_status: 'M', worktree_status: ' ' };

    assert!(!status.is_untracked(), "Modified tracked file should not be considered untracked");
}

#[test]
fn file_status_handles_both_staged_and_modified() {
    let status =
        FileStatus { path: PathBuf::from("both.md"), index_status: 'M', worktree_status: 'M' };

    assert!(
        status.is_staged() && status.is_modified(),
        "File modified in both index and worktree should be both staged and modified"
    );
}

#[test]
fn file_status_is_deleted_returns_true_for_worktree_deletion() {
    let status =
        FileStatus { path: PathBuf::from("deleted.md"), index_status: ' ', worktree_status: 'D' };

    assert!(status.is_deleted(), "File deleted in worktree should return true for is_deleted");
}

#[test]
fn file_status_is_deleted_returns_true_for_staged_deletion() {
    let status =
        FileStatus { path: PathBuf::from("deleted.md"), index_status: 'D', worktree_status: ' ' };

    assert!(
        status.is_deleted(),
        "File deleted in index (staged) should return true for is_deleted"
    );
}

#[test]
fn file_status_is_deleted_returns_false_for_modified_file() {
    let status =
        FileStatus { path: PathBuf::from("modified.md"), index_status: 'M', worktree_status: 'M' };

    assert!(!status.is_deleted(), "Modified file should return false for is_deleted");
}

#[test]
fn file_status_is_deleted_returns_false_for_untracked_file() {
    let status =
        FileStatus { path: PathBuf::from("untracked.md"), index_status: '?', worktree_status: '?' };

    assert!(!status.is_deleted(), "Untracked file should return false for is_deleted");
}

// ============================================================================
// FileChange tests
// ============================================================================

#[test]
fn file_change_stores_added_status() {
    let change = FileChange { status: 'A', path: PathBuf::from("new_file.md") };
    assert_eq!(change.status, 'A', "Status should be 'A' for added files");
    assert_eq!(change.path, PathBuf::from("new_file.md"));
}

#[test]
fn file_change_stores_modified_status() {
    let change = FileChange { status: 'M', path: PathBuf::from("changed_file.md") };
    assert_eq!(change.status, 'M', "Status should be 'M' for modified files");
}

#[test]
fn file_change_stores_deleted_status() {
    let change = FileChange { status: 'D', path: PathBuf::from("removed_file.md") };
    assert_eq!(change.status, 'D', "Status should be 'D' for deleted files");
}

#[test]
fn file_change_stores_renamed_status() {
    let change = FileChange { status: 'R', path: PathBuf::from("renamed_file.md") };
    assert_eq!(change.status, 'R', "Status should be 'R' for renamed files");
}

#[test]
fn file_change_stores_copied_status() {
    let change = FileChange { status: 'C', path: PathBuf::from("copied_file.md") };
    assert_eq!(change.status, 'C', "Status should be 'C' for copied files");
}

#[test]
fn file_change_equality() {
    let change1 = FileChange { status: 'M', path: PathBuf::from("file.md") };
    let change2 = FileChange { status: 'M', path: PathBuf::from("file.md") };
    assert_eq!(change1, change2, "FileChanges with same status and path should be equal");
}

#[test]
fn file_change_inequality_by_status() {
    let change1 = FileChange { status: 'A', path: PathBuf::from("file.md") };
    let change2 = FileChange { status: 'M', path: PathBuf::from("file.md") };
    assert_ne!(change1, change2, "FileChanges with different status should not be equal");
}

#[test]
fn file_change_inequality_by_path() {
    let change1 = FileChange { status: 'M', path: PathBuf::from("file1.md") };
    let change2 = FileChange { status: 'M', path: PathBuf::from("file2.md") };
    assert_ne!(change1, change2, "FileChanges with different paths should not be equal");
}
