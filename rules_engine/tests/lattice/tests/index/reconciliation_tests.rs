use std::path::PathBuf;
use std::sync::Mutex;

use lattice::error::error_types::LatticeError;
use lattice::git::git_ops::{FileStatus, GitOps};
use lattice::index::reconciliation::change_detection::{self, ChangeInfo, ChangeStatus};
use lattice::index::reconciliation::reconciliation_coordinator::{self, ReconciliationResult};
use lattice::index::reconciliation::sync_strategies;
use lattice::index::{connection_pool, schema_definition};
use tempfile::TempDir;

/// A test double for GitOps that returns configured responses.
/// Unlike some other test doubles, this one returns the same result on each
/// call (cloning the stored value) to support code paths that call methods
/// multiple times.
struct FakeGit {
    ls_files_paths: Mutex<Vec<PathBuf>>,
    diff_paths: Mutex<Vec<PathBuf>>,
    status_files: Mutex<Vec<FileStatus>>,
    rev_parse_result: Mutex<String>,
}

impl FakeGit {
    fn new() -> Self {
        Self {
            ls_files_paths: Mutex::new(Vec::new()),
            diff_paths: Mutex::new(Vec::new()),
            status_files: Mutex::new(Vec::new()),
            rev_parse_result: Mutex::new("abc123def456".to_string()),
        }
    }

    fn with_ls_files(paths: Vec<PathBuf>) -> Self {
        Self {
            ls_files_paths: Mutex::new(paths),
            diff_paths: Mutex::new(Vec::new()),
            status_files: Mutex::new(Vec::new()),
            rev_parse_result: Mutex::new("abc123def456".to_string()),
        }
    }

    fn with_status(statuses: Vec<FileStatus>) -> Self {
        Self {
            ls_files_paths: Mutex::new(Vec::new()),
            diff_paths: Mutex::new(Vec::new()),
            status_files: Mutex::new(statuses),
            rev_parse_result: Mutex::new("abc123def456".to_string()),
        }
    }

    fn with_diff(paths: Vec<PathBuf>) -> Self {
        Self {
            ls_files_paths: Mutex::new(Vec::new()),
            diff_paths: Mutex::new(paths),
            status_files: Mutex::new(Vec::new()),
            rev_parse_result: Mutex::new("abc123def456".to_string()),
        }
    }
}

impl GitOps for FakeGit {
    fn ls_files(&self, _pattern: &str) -> Result<Vec<PathBuf>, LatticeError> {
        Ok(self.ls_files_paths.lock().unwrap().clone())
    }

    fn diff(
        &self,
        _from_commit: &str,
        _to_commit: &str,
        _pattern: &str,
    ) -> Result<Vec<PathBuf>, LatticeError> {
        Ok(self.diff_paths.lock().unwrap().clone())
    }

    fn status(&self, _pattern: &str) -> Result<Vec<FileStatus>, LatticeError> {
        Ok(self.status_files.lock().unwrap().clone())
    }

    fn rev_parse(&self, _git_ref: &str) -> Result<String, LatticeError> {
        Ok(self.rev_parse_result.lock().unwrap().clone())
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
}

// ============================================================================
// ReconciliationResult tests
// ============================================================================

#[test]
fn reconciliation_result_skipped_equality() {
    let result1 = ReconciliationResult::Skipped;
    let result2 = ReconciliationResult::Skipped;
    assert_eq!(result1, result2, "Skipped variants should be equal");
}

#[test]
fn reconciliation_result_incremental_equality() {
    let result1 = ReconciliationResult::Incremental { files_updated: 5, files_removed: 2 };
    let result2 = ReconciliationResult::Incremental { files_updated: 5, files_removed: 2 };
    assert_eq!(result1, result2, "Incremental variants with same values should be equal");
}

#[test]
fn reconciliation_result_full_rebuild_equality() {
    let result1 = ReconciliationResult::FullRebuild { documents_indexed: 100 };
    let result2 = ReconciliationResult::FullRebuild { documents_indexed: 100 };
    assert_eq!(result1, result2, "FullRebuild variants with same values should be equal");
}

#[test]
fn reconciliation_result_different_variants_not_equal() {
    let skipped = ReconciliationResult::Skipped;
    let incremental = ReconciliationResult::Incremental { files_updated: 0, files_removed: 0 };
    assert_ne!(skipped, incremental, "Different variants should not be equal");
}

// ============================================================================
// ChangeInfo tests
// ============================================================================

#[test]
fn change_info_is_fast_path_when_all_empty_and_commits_match() {
    let info = ChangeInfo {
        modified_files: Vec::new(),
        deleted_files: Vec::new(),
        uncommitted_files: Vec::new(),
        current_head: Some("abc123".to_string()),
        last_indexed_commit: Some("abc123".to_string()),
    };
    assert!(info.is_fast_path(), "Should be fast path when no changes and commits match");
}

#[test]
fn change_info_is_not_fast_path_when_modified_files_present() {
    let info = ChangeInfo {
        modified_files: vec![PathBuf::from("doc.md")],
        deleted_files: Vec::new(),
        uncommitted_files: Vec::new(),
        current_head: Some("abc123".to_string()),
        last_indexed_commit: Some("abc123".to_string()),
    };
    assert!(!info.is_fast_path(), "Should not be fast path when modified files present");
}

#[test]
fn change_info_is_not_fast_path_when_deleted_files_present() {
    let info = ChangeInfo {
        modified_files: Vec::new(),
        deleted_files: vec![PathBuf::from("removed.md")],
        uncommitted_files: Vec::new(),
        current_head: Some("abc123".to_string()),
        last_indexed_commit: Some("abc123".to_string()),
    };
    assert!(!info.is_fast_path(), "Should not be fast path when deleted files present");
}

#[test]
fn change_info_is_not_fast_path_when_uncommitted_files_present() {
    let info = ChangeInfo {
        modified_files: Vec::new(),
        deleted_files: Vec::new(),
        uncommitted_files: vec![PathBuf::from("wip.md")],
        current_head: Some("abc123".to_string()),
        last_indexed_commit: Some("abc123".to_string()),
    };
    assert!(!info.is_fast_path(), "Should not be fast path when uncommitted files present");
}

#[test]
fn change_info_is_not_fast_path_when_commits_differ() {
    let info = ChangeInfo {
        modified_files: Vec::new(),
        deleted_files: Vec::new(),
        uncommitted_files: Vec::new(),
        current_head: Some("abc123".to_string()),
        last_indexed_commit: Some("def456".to_string()),
    };
    assert!(!info.is_fast_path(), "Should not be fast path when commits differ");
}

#[test]
fn change_info_is_not_fast_path_when_no_last_indexed_commit() {
    let info = ChangeInfo {
        modified_files: Vec::new(),
        deleted_files: Vec::new(),
        uncommitted_files: Vec::new(),
        current_head: Some("abc123".to_string()),
        last_indexed_commit: None,
    };
    assert!(!info.is_fast_path(), "Should not be fast path when no last indexed commit");
}

#[test]
fn change_info_default_is_empty() {
    let info = ChangeInfo::default();
    assert!(info.modified_files.is_empty(), "Default modified_files should be empty");
    assert!(info.deleted_files.is_empty(), "Default deleted_files should be empty");
    assert!(info.uncommitted_files.is_empty(), "Default uncommitted_files should be empty");
    assert!(info.current_head.is_none(), "Default current_head should be None");
    assert!(info.last_indexed_commit.is_none(), "Default last_indexed_commit should be None");
}

// ============================================================================
// ChangeStatus tests
// ============================================================================

#[test]
fn change_status_returns_no_changes_when_all_empty_and_commits_match() {
    let info = ChangeInfo {
        modified_files: vec![],
        deleted_files: vec![],
        uncommitted_files: vec![],
        current_head: Some("abc123".to_string()),
        last_indexed_commit: Some("abc123".to_string()),
    };
    assert_eq!(
        info.status(),
        ChangeStatus::NoChanges,
        "Status should be NoChanges when all empty and commits match"
    );
}

#[test]
fn change_status_returns_committed_changes_when_only_committed_changes() {
    let info = ChangeInfo {
        modified_files: vec![PathBuf::from("doc.md")],
        deleted_files: vec![],
        uncommitted_files: vec![],
        current_head: Some("def456".to_string()),
        last_indexed_commit: Some("abc123".to_string()),
    };
    assert_eq!(
        info.status(),
        ChangeStatus::CommittedChanges,
        "Status should be CommittedChanges when only committed changes exist"
    );
}

#[test]
fn change_status_returns_uncommitted_changes_when_only_uncommitted() {
    let info = ChangeInfo {
        modified_files: vec![],
        deleted_files: vec![],
        uncommitted_files: vec![PathBuf::from("wip.md")],
        current_head: Some("abc123".to_string()),
        last_indexed_commit: Some("abc123".to_string()),
    };
    assert_eq!(
        info.status(),
        ChangeStatus::UncommittedChanges,
        "Status should be UncommittedChanges when only uncommitted changes exist"
    );
}

#[test]
fn change_status_returns_both_when_committed_and_uncommitted() {
    let info = ChangeInfo {
        modified_files: vec![PathBuf::from("doc.md")],
        deleted_files: vec![],
        uncommitted_files: vec![PathBuf::from("wip.md")],
        current_head: Some("def456".to_string()),
        last_indexed_commit: Some("abc123".to_string()),
    };
    assert_eq!(
        info.status(),
        ChangeStatus::Both,
        "Status should be Both when both committed and uncommitted changes exist"
    );
}

#[test]
fn change_info_total_changes_sums_all_file_lists() {
    let info = ChangeInfo {
        modified_files: vec![PathBuf::from("a.md"), PathBuf::from("b.md")],
        deleted_files: vec![PathBuf::from("c.md")],
        uncommitted_files: vec![PathBuf::from("d.md")],
        current_head: Some("def456".to_string()),
        last_indexed_commit: Some("abc123".to_string()),
    };
    assert_eq!(info.total_changes(), 4, "total_changes should sum all three file lists");
}

// ============================================================================
// change_detection::detect_changes tests
// ============================================================================

#[test]
fn detect_changes_returns_fast_path_when_head_matches_last_indexed_and_clean() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");
    schema_definition::create_schema(&conn).expect("should create schema");

    // Set the last indexed commit to match what FakeGit will return
    conn.execute("UPDATE index_metadata SET last_commit = 'abc123def456' WHERE id = 1", [])
        .expect("should update last_commit");

    let git = FakeGit::new();
    let info = change_detection::detect_changes(&git, &conn, temp_dir.path())
        .expect("detect_changes should succeed");

    assert!(info.is_fast_path(), "Should detect fast path when HEAD matches last indexed commit");
}

#[test]
fn detect_changes_returns_uncommitted_files_when_present() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");
    schema_definition::create_schema(&conn).expect("should create schema");

    // Set the last indexed commit to match what FakeGit will return
    conn.execute("UPDATE index_metadata SET last_commit = 'abc123def456' WHERE id = 1", [])
        .expect("should update last_commit");

    let git = FakeGit::with_status(vec![FileStatus {
        path: PathBuf::from("wip.md"),
        index_status: 'M',
        worktree_status: ' ',
    }]);

    let info = change_detection::detect_changes(&git, &conn, temp_dir.path())
        .expect("detect_changes should succeed");

    assert!(!info.is_fast_path(), "Should not be fast path when uncommitted files present");
    assert_eq!(info.uncommitted_files.len(), 1, "Should have one uncommitted file");
}

// ============================================================================
// sync_strategies tests
// ============================================================================

#[test]
fn incremental_sync_returns_correct_counts() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let conn = connection_pool::open_memory_connection().expect("should open in-memory connection");

    let change_info = ChangeInfo {
        modified_files: vec![PathBuf::from("a.md"), PathBuf::from("b.md")],
        deleted_files: vec![PathBuf::from("removed.md")],
        uncommitted_files: vec![PathBuf::from("wip.md")],
        current_head: Some("abc123".to_string()),
        last_indexed_commit: Some("def456".to_string()),
    };

    let git = FakeGit::new();
    let result = sync_strategies::incremental_sync(temp_dir.path(), &git, &conn, &change_info)
        .expect("incremental_sync should succeed");

    // files_updated = modified_files + uncommitted_files = 2 + 1 = 3
    assert_eq!(result.files_updated, 3, "Should count modified and uncommitted files");
    // files_removed = deleted_files = 1
    assert_eq!(result.files_removed, 1, "Should count deleted files");
}

#[test]
fn full_rebuild_returns_document_count() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let conn = connection_pool::open_memory_connection().expect("should open in-memory connection");

    let git = FakeGit::with_ls_files(vec![
        PathBuf::from("docs/readme.md"),
        PathBuf::from("tasks/task1.md"),
        PathBuf::from("api/api.md"),
    ]);

    let result = sync_strategies::full_rebuild(temp_dir.path(), &git, &conn)
        .expect("full_rebuild should succeed");

    assert_eq!(result.documents_indexed, 3, "Should count all markdown files");
}

// ============================================================================
// reconciliation_coordinator::reconcile tests
// ============================================================================

#[test]
fn reconcile_returns_full_rebuild_when_no_schema() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");
    // Note: Not creating schema - this should trigger full rebuild

    let git = FakeGit::with_ls_files(vec![PathBuf::from("doc.md")]);

    let result = reconciliation_coordinator::reconcile(temp_dir.path(), &git, &conn)
        .expect("reconcile should succeed");

    assert!(
        matches!(result, ReconciliationResult::FullRebuild { .. }),
        "Should return FullRebuild when no schema exists, got: {result:?}"
    );
}

#[test]
fn reconcile_returns_skipped_when_no_changes() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");
    schema_definition::create_schema(&conn).expect("should create schema");

    // Set last_commit to match rev_parse result
    conn.execute("UPDATE index_metadata SET last_commit = 'abc123def456' WHERE id = 1", [])
        .expect("should update last_commit");

    let git = FakeGit::new();

    let result = reconciliation_coordinator::reconcile(temp_dir.path(), &git, &conn)
        .expect("reconcile should succeed");

    assert_eq!(
        result,
        ReconciliationResult::Skipped,
        "Should return Skipped when no changes detected"
    );
}

#[test]
fn reconcile_returns_incremental_when_changes_detected() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");
    schema_definition::create_schema(&conn).expect("should create schema");

    // Set a different last_commit to trigger change detection
    conn.execute("UPDATE index_metadata SET last_commit = 'old_commit_hash' WHERE id = 1", [])
        .expect("should update last_commit");

    let git = FakeGit::with_diff(vec![PathBuf::from("changed.md")]);

    let result = reconciliation_coordinator::reconcile(temp_dir.path(), &git, &conn)
        .expect("reconcile should succeed");

    assert!(
        matches!(result, ReconciliationResult::Incremental { .. }),
        "Should return Incremental when changes detected, got: {result:?}"
    );
}

#[test]
fn reconcile_returns_incremental_when_uncommitted_changes_present() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");
    schema_definition::create_schema(&conn).expect("should create schema");

    // Set last_commit to match rev_parse, but have uncommitted changes
    conn.execute("UPDATE index_metadata SET last_commit = 'abc123def456' WHERE id = 1", [])
        .expect("should update last_commit");

    let git = FakeGit::with_status(vec![FileStatus {
        path: PathBuf::from("uncommitted.md"),
        index_status: 'M',
        worktree_status: ' ',
    }]);

    let result = reconciliation_coordinator::reconcile(temp_dir.path(), &git, &conn)
        .expect("reconcile should succeed");

    assert!(
        matches!(result, ReconciliationResult::Incremental { .. }),
        "Should return Incremental when uncommitted changes present, got: {result:?}"
    );
}
