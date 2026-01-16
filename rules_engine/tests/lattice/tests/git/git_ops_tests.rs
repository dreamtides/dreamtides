use std::path::PathBuf;

use lattice::git::git_ops::FileStatus;

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
