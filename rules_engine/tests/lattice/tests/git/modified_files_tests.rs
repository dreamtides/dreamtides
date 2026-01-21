use std::path::PathBuf;
use std::sync::Mutex;

use lattice::error::error_types::LatticeError;
use lattice::git::git_ops::{FileChange, FileStatus, GitOps};
use lattice::git::modified_files;

/// A test double for GitOps that returns configured responses.
struct FakeGit {
    ls_files_result: Mutex<Result<Vec<PathBuf>, LatticeError>>,
    diff_result: Mutex<Result<Vec<PathBuf>, LatticeError>>,
    status_result: Mutex<Result<Vec<FileStatus>, LatticeError>>,
}

impl FakeGit {
    fn with_ls_files(paths: Vec<PathBuf>) -> Self {
        Self {
            ls_files_result: Mutex::new(Ok(paths)),
            diff_result: Mutex::new(Ok(Vec::new())),
            status_result: Mutex::new(Ok(Vec::new())),
        }
    }

    fn with_diff(paths: Vec<PathBuf>) -> Self {
        Self {
            ls_files_result: Mutex::new(Ok(Vec::new())),
            diff_result: Mutex::new(Ok(paths)),
            status_result: Mutex::new(Ok(Vec::new())),
        }
    }

    fn with_status(statuses: Vec<FileStatus>) -> Self {
        Self {
            ls_files_result: Mutex::new(Ok(Vec::new())),
            diff_result: Mutex::new(Ok(Vec::new())),
            status_result: Mutex::new(Ok(statuses)),
        }
    }

    fn with_ls_files_error(error: LatticeError) -> Self {
        Self {
            ls_files_result: Mutex::new(Err(error)),
            diff_result: Mutex::new(Ok(Vec::new())),
            status_result: Mutex::new(Ok(Vec::new())),
        }
    }

    fn with_diff_error(error: LatticeError) -> Self {
        Self {
            ls_files_result: Mutex::new(Ok(Vec::new())),
            diff_result: Mutex::new(Err(error)),
            status_result: Mutex::new(Ok(Vec::new())),
        }
    }

    fn with_status_error(error: LatticeError) -> Self {
        Self {
            ls_files_result: Mutex::new(Ok(Vec::new())),
            diff_result: Mutex::new(Ok(Vec::new())),
            status_result: Mutex::new(Err(error)),
        }
    }
}

impl GitOps for FakeGit {
    fn ls_files(&self, _pattern: &str) -> Result<Vec<PathBuf>, LatticeError> {
        let mut result = self.ls_files_result.lock().unwrap();
        std::mem::replace(&mut *result, Ok(Vec::new()))
    }

    fn diff(
        &self,
        _from_commit: &str,
        _to_commit: &str,
        _pattern: &str,
    ) -> Result<Vec<PathBuf>, LatticeError> {
        let mut result = self.diff_result.lock().unwrap();
        std::mem::replace(&mut *result, Ok(Vec::new()))
    }

    fn status(&self, _pattern: &str) -> Result<Vec<FileStatus>, LatticeError> {
        let mut result = self.status_result.lock().unwrap();
        std::mem::replace(&mut *result, Ok(Vec::new()))
    }

    fn rev_parse(&self, _git_ref: &str) -> Result<String, LatticeError> {
        Ok("abc123".to_string())
    }

    fn log(
        &self,
        _path: Option<&str>,
        _format: &str,
        _limit: usize,
    ) -> Result<Vec<String>, LatticeError> {
        Ok(Vec::new())
    }

    fn config_get(&self, _key: &str) -> Result<Option<String>, LatticeError> {
        Ok(None)
    }

    fn diff_name_status(
        &self,
        _from_commit: &str,
        _to_commit: &str,
        _pattern: &str,
    ) -> Result<Vec<FileChange>, LatticeError> {
        Ok(Vec::new())
    }

    fn oldest_commit_since(&self, _date: &str) -> Result<Option<String>, LatticeError> {
        Ok(None)
    }

    fn commit_file(&self, _path: &std::path::Path, _message: &str) -> Result<(), LatticeError> {
        Ok(())
    }
}

#[test]
fn list_all_documents_returns_all_tracked_markdown_files() {
    let paths = vec![
        PathBuf::from("docs/readme.md"),
        PathBuf::from("tasks/feature.md"),
        PathBuf::from("api/api.md"),
    ];
    let git = FakeGit::with_ls_files(paths.clone());

    let result =
        modified_files::list_all_documents(&git).expect("list_all_documents should succeed");

    assert_eq!(result, paths, "Should return all paths from git ls-files");
}

#[test]
fn list_all_documents_returns_empty_vec_when_no_documents() {
    let git = FakeGit::with_ls_files(Vec::new());

    let result =
        modified_files::list_all_documents(&git).expect("list_all_documents should succeed");

    assert!(result.is_empty(), "Should return empty vec when no markdown files exist");
}

#[test]
fn list_all_documents_propagates_git_error() {
    let git = FakeGit::with_ls_files_error(LatticeError::GitError {
        operation: "ls-files".to_string(),
        reason: "repository not found".to_string(),
    });

    let result = modified_files::list_all_documents(&git);

    assert!(result.is_err(), "Should propagate git error");
    let err = result.unwrap_err();
    assert!(matches!(err, LatticeError::GitError { .. }), "Error should be GitError, got: {err:?}");
}

#[test]
fn list_changed_documents_returns_modified_files_since_commit() {
    let changed_paths = vec![PathBuf::from("docs/updated.md"), PathBuf::from("tasks/new_task.md")];
    let git = FakeGit::with_diff(changed_paths.clone());

    let result = modified_files::list_changed_documents(&git, "abc123")
        .expect("list_changed_documents should succeed");

    assert_eq!(result, changed_paths, "Should return paths changed since the specified commit");
}

#[test]
fn list_changed_documents_returns_empty_when_no_changes() {
    let git = FakeGit::with_diff(Vec::new());

    let result = modified_files::list_changed_documents(&git, "head~10")
        .expect("list_changed_documents should succeed");

    assert!(result.is_empty(), "Should return empty vec when no files changed");
}

#[test]
fn list_changed_documents_propagates_git_error() {
    let git = FakeGit::with_diff_error(LatticeError::GitError {
        operation: "diff".to_string(),
        reason: "bad revision".to_string(),
    });

    let result = modified_files::list_changed_documents(&git, "invalid-commit");

    assert!(result.is_err(), "Should propagate git error");
}

#[test]
fn list_uncommitted_changes_returns_paths_from_status() {
    let statuses = vec![
        FileStatus { path: PathBuf::from("staged.md"), index_status: 'M', worktree_status: ' ' },
        FileStatus { path: PathBuf::from("modified.md"), index_status: ' ', worktree_status: 'M' },
        FileStatus { path: PathBuf::from("both.md"), index_status: 'A', worktree_status: 'M' },
    ];
    let git = FakeGit::with_status(statuses);

    let result = modified_files::list_uncommitted_changes(&git)
        .expect("list_uncommitted_changes should succeed");

    assert_eq!(
        result,
        vec![PathBuf::from("staged.md"), PathBuf::from("modified.md"), PathBuf::from("both.md"),],
        "Should extract paths from FileStatus entries"
    );
}

#[test]
fn list_uncommitted_changes_returns_empty_when_clean() {
    let git = FakeGit::with_status(Vec::new());

    let result = modified_files::list_uncommitted_changes(&git)
        .expect("list_uncommitted_changes should succeed");

    assert!(result.is_empty(), "Should return empty vec when working tree is clean");
}

#[test]
fn list_uncommitted_changes_propagates_git_error() {
    let git = FakeGit::with_status_error(LatticeError::GitError {
        operation: "status".to_string(),
        reason: "not a git repository".to_string(),
    });

    let result = modified_files::list_uncommitted_changes(&git);

    assert!(result.is_err(), "Should propagate git error");
}
