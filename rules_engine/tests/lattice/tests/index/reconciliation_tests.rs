use std::path::PathBuf;
use std::sync::Mutex;

use lattice::error::error_types::LatticeError;
use lattice::git::git_ops::{FileChange, FileStatus, GitOps};
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
fn incremental_sync_skips_nonexistent_files() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");
    schema_definition::create_schema(&conn).expect("should create schema");

    // Provide paths to files that don't exist on disk
    let change_info = ChangeInfo {
        modified_files: vec![PathBuf::from("nonexistent1.md"), PathBuf::from("nonexistent2.md")],
        deleted_files: vec![PathBuf::from("removed.md")],
        uncommitted_files: vec![PathBuf::from("nonexistent3.md")],
        current_head: Some("abc123".to_string()),
        last_indexed_commit: Some("def456".to_string()),
    };

    let git = FakeGit::new();
    let result = sync_strategies::incremental_sync(temp_dir.path(), &git, &conn, &change_info)
        .expect("incremental_sync should succeed");

    // Files don't exist, so they're skipped
    assert_eq!(result.files_updated, 0, "Should skip nonexistent modified files");
    // Deleted files are also counted as "not found" since they don't exist in index
    assert_eq!(result.files_removed, 0, "Should report 0 removed for files not in index");
}

#[test]
fn full_rebuild_skips_nonexistent_files() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");
    schema_definition::create_schema(&conn).expect("should create schema");

    // Provide paths to files that don't exist on disk
    let git = FakeGit::with_ls_files(vec![
        PathBuf::from("docs/readme.md"),
        PathBuf::from("tasks/task1.md"),
        PathBuf::from("api/api.md"),
    ]);

    let result = sync_strategies::full_rebuild(temp_dir.path(), &git, &conn)
        .expect("full_rebuild should succeed");

    // Files don't exist on disk, so 0 documents are indexed
    assert_eq!(result.documents_indexed, 0, "Should skip nonexistent files");
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

// ============================================================================
// sync_strategies tests with actual documents
// ============================================================================

/// Helper to create a valid Lattice document in the filesystem.
fn create_lattice_document(dir: &std::path::Path, relative_path: &str, id: &str, name: &str) {
    let full_path = dir.join(relative_path);
    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent).expect("should create parent dirs");
    }
    let content = format!(
        r#"---
lattice-id: {}
name: {}
description: Test document {}
---

# {}

This is the body content.
"#,
        id, name, name, name
    );
    std::fs::write(&full_path, content).expect("should write document");
}

/// Helper to create a document with labels.
fn create_document_with_labels(
    dir: &std::path::Path,
    relative_path: &str,
    id: &str,
    name: &str,
    labels: &[&str],
) {
    let full_path = dir.join(relative_path);
    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent).expect("should create parent dirs");
    }
    let labels_yaml = labels.iter().map(|l| format!("  - {l}")).collect::<Vec<_>>().join("\n");
    let content = format!(
        r#"---
lattice-id: {}
name: {}
description: Test document with labels
labels:
{}
---

# {}

Document body.
"#,
        id, name, labels_yaml, name
    );
    std::fs::write(&full_path, content).expect("should write document");
}

/// Helper to create a document with links.
fn create_document_with_links(
    dir: &std::path::Path,
    relative_path: &str,
    id: &str,
    name: &str,
    link_target: &str,
) {
    let full_path = dir.join(relative_path);
    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent).expect("should create parent dirs");
    }
    let content = format!(
        r#"---
lattice-id: {}
name: {}
description: Test document with links
---

# {}

See the [other document](#{}).
"#,
        id, name, name, link_target
    );
    std::fs::write(&full_path, content).expect("should write document");
}

#[test]
fn full_rebuild_indexes_actual_documents() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    // Create actual Lattice documents
    create_lattice_document(temp_dir.path(), "doc1.md", "LABC23", "doc-one");
    create_lattice_document(temp_dir.path(), "doc2.md", "LDEF45", "doc-two");
    create_lattice_document(temp_dir.path(), "subdir/doc3.md", "LGHI67", "doc-three");

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");
    schema_definition::create_schema(&conn).expect("should create schema");

    let git = FakeGit::with_ls_files(vec![
        PathBuf::from("doc1.md"),
        PathBuf::from("doc2.md"),
        PathBuf::from("subdir/doc3.md"),
    ]);

    let result = sync_strategies::full_rebuild(temp_dir.path(), &git, &conn)
        .expect("full_rebuild should succeed");

    assert_eq!(result.documents_indexed, 3, "Should index all 3 documents");

    // Verify documents are in the database
    let doc1 = lattice::index::document_queries::lookup_by_id(&conn, "LABC23")
        .expect("lookup should succeed");
    assert!(doc1.is_some(), "Document LABC23 should be in index");
    assert_eq!(doc1.unwrap().name, "doc-one", "Document name should match");
}

#[test]
fn full_rebuild_indexes_labels() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    create_document_with_labels(temp_dir.path(), "labeled.md", "LLAB23", "labeled-doc", &[
        "urgent", "backend",
    ]);

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");
    schema_definition::create_schema(&conn).expect("should create schema");

    let git = FakeGit::with_ls_files(vec![PathBuf::from("labeled.md")]);

    sync_strategies::full_rebuild(temp_dir.path(), &git, &conn)
        .expect("full_rebuild should succeed");

    // Verify labels are in the database
    let labels =
        lattice::index::label_queries::get_labels(&conn, "LLAB23").expect("should get labels");
    assert_eq!(labels.len(), 2, "Should have 2 labels");
    assert!(labels.contains(&"urgent".to_string()), "Should have urgent label");
    assert!(labels.contains(&"backend".to_string()), "Should have backend label");
}

#[test]
fn full_rebuild_indexes_body_links() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    create_lattice_document(temp_dir.path(), "target.md", "LTGT23", "target-doc");
    create_document_with_links(temp_dir.path(), "source.md", "LSRC23", "source-doc", "LTGT23");

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");
    schema_definition::create_schema(&conn).expect("should create schema");

    let git = FakeGit::with_ls_files(vec![PathBuf::from("target.md"), PathBuf::from("source.md")]);

    sync_strategies::full_rebuild(temp_dir.path(), &git, &conn)
        .expect("full_rebuild should succeed");

    // Verify links are in the database
    let outgoing =
        lattice::index::link_queries::query_outgoing(&conn, "LSRC23").expect("should get links");
    assert_eq!(outgoing.len(), 1, "Should have 1 outgoing link");
    assert_eq!(outgoing[0].target_id, "LTGT23", "Link target should match");
}

#[test]
fn full_rebuild_skips_files_with_conflict_markers() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    // Create a normal document
    create_lattice_document(temp_dir.path(), "normal.md", "LNRM23", "normal-doc");

    // Create a document with conflict markers
    let conflicted_path = temp_dir.path().join("conflicted.md");
    std::fs::write(
        &conflicted_path,
        r#"---
lattice-id: LCFL23
name: conflicted-doc
description: Document with conflicts
---

<<<<<<< HEAD
This is my version.
=======
This is their version.
>>>>>>> branch
"#,
    )
    .expect("should write conflicted document");

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");
    schema_definition::create_schema(&conn).expect("should create schema");

    let git =
        FakeGit::with_ls_files(vec![PathBuf::from("normal.md"), PathBuf::from("conflicted.md")]);

    let result = sync_strategies::full_rebuild(temp_dir.path(), &git, &conn)
        .expect("full_rebuild should succeed");

    assert_eq!(result.documents_indexed, 1, "Should only index 1 document (skip conflicted)");

    // Verify normal document is indexed
    let normal = lattice::index::document_queries::lookup_by_id(&conn, "LNRM23")
        .expect("lookup should succeed");
    assert!(normal.is_some(), "Normal document should be indexed");

    // Verify conflicted document is NOT indexed
    let conflicted = lattice::index::document_queries::lookup_by_id(&conn, "LCFL23")
        .expect("lookup should succeed");
    assert!(conflicted.is_none(), "Conflicted document should NOT be indexed");
}

#[test]
fn full_rebuild_skips_non_lattice_documents() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    // Create a valid Lattice document
    create_lattice_document(temp_dir.path(), "valid.md", "LVLD23", "valid-doc");

    // Create a plain markdown file (no frontmatter)
    let plain_path = temp_dir.path().join("plain.md");
    std::fs::write(&plain_path, "# Just a plain markdown file\n\nNo frontmatter here.")
        .expect("should write plain markdown");

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");
    schema_definition::create_schema(&conn).expect("should create schema");

    let git = FakeGit::with_ls_files(vec![PathBuf::from("valid.md"), PathBuf::from("plain.md")]);

    let result = sync_strategies::full_rebuild(temp_dir.path(), &git, &conn)
        .expect("full_rebuild should succeed");

    assert_eq!(result.documents_indexed, 1, "Should only index valid Lattice document");
}

#[test]
fn incremental_sync_adds_new_document() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");
    schema_definition::create_schema(&conn).expect("should create schema");

    // Create a new document
    create_lattice_document(temp_dir.path(), "new.md", "LNEW23", "new-doc");

    let change_info = ChangeInfo {
        modified_files: vec![PathBuf::from("new.md")],
        deleted_files: vec![],
        uncommitted_files: vec![],
        current_head: Some("abc123".to_string()),
        last_indexed_commit: Some("def456".to_string()),
    };

    let git = FakeGit::new();
    let result = sync_strategies::incremental_sync(temp_dir.path(), &git, &conn, &change_info)
        .expect("incremental_sync should succeed");

    assert_eq!(result.files_updated, 1, "Should have updated 1 file");
    assert_eq!(result.files_removed, 0, "Should have removed 0 files");

    // Verify document is in the database
    let doc = lattice::index::document_queries::lookup_by_id(&conn, "LNEW23")
        .expect("lookup should succeed");
    assert!(doc.is_some(), "New document should be in index");
}

#[test]
fn incremental_sync_removes_deleted_document() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");
    schema_definition::create_schema(&conn).expect("should create schema");

    // First, add a document to the index
    create_lattice_document(temp_dir.path(), "to-delete.md", "LDEL23", "delete-me");

    let git = FakeGit::with_ls_files(vec![PathBuf::from("to-delete.md")]);
    sync_strategies::full_rebuild(temp_dir.path(), &git, &conn)
        .expect("initial full_rebuild should succeed");

    // Verify document is initially present
    let doc = lattice::index::document_queries::lookup_by_id(&conn, "LDEL23")
        .expect("lookup should succeed");
    assert!(doc.is_some(), "Document should be in index initially");

    // Now simulate deletion (remove from filesystem and mark as deleted in
    // change_info)
    std::fs::remove_file(temp_dir.path().join("to-delete.md"))
        .expect("should delete document file");

    let change_info = ChangeInfo {
        modified_files: vec![],
        deleted_files: vec![PathBuf::from("to-delete.md")],
        uncommitted_files: vec![],
        current_head: Some("new123".to_string()),
        last_indexed_commit: Some("old456".to_string()),
    };

    let git2 = FakeGit::new();
    let result = sync_strategies::incremental_sync(temp_dir.path(), &git2, &conn, &change_info)
        .expect("incremental_sync should succeed");

    assert_eq!(result.files_removed, 1, "Should have removed 1 file");

    // Verify document is no longer in the database
    let doc = lattice::index::document_queries::lookup_by_id(&conn, "LDEL23")
        .expect("lookup should succeed");
    assert!(doc.is_none(), "Deleted document should be removed from index");
}

#[test]
fn incremental_sync_updates_existing_document() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");
    schema_definition::create_schema(&conn).expect("should create schema");

    // Create initial document
    create_lattice_document(temp_dir.path(), "existing.md", "LEXS23", "old-name");

    let git = FakeGit::with_ls_files(vec![PathBuf::from("existing.md")]);
    sync_strategies::full_rebuild(temp_dir.path(), &git, &conn)
        .expect("initial full_rebuild should succeed");

    // Verify initial state
    let doc = lattice::index::document_queries::lookup_by_id(&conn, "LEXS23")
        .expect("lookup should succeed")
        .expect("document should exist");
    assert_eq!(doc.name, "old-name", "Initial name should match");

    // Update the document with new content
    let updated_path = temp_dir.path().join("existing.md");
    std::fs::write(
        &updated_path,
        r#"---
lattice-id: LEXS23
name: new-name
description: Updated description
---

# Updated Content

New body text.
"#,
    )
    .expect("should update document");

    let change_info = ChangeInfo {
        modified_files: vec![PathBuf::from("existing.md")],
        deleted_files: vec![],
        uncommitted_files: vec![],
        current_head: Some("new123".to_string()),
        last_indexed_commit: Some("old456".to_string()),
    };

    let git2 = FakeGit::new();
    sync_strategies::incremental_sync(temp_dir.path(), &git2, &conn, &change_info)
        .expect("incremental_sync should succeed");

    // Verify updated state
    let doc = lattice::index::document_queries::lookup_by_id(&conn, "LEXS23")
        .expect("lookup should succeed")
        .expect("document should still exist");
    assert_eq!(doc.name, "new-name", "Updated name should match");
    assert_eq!(doc.description, "Updated description", "Updated description should match");
}

#[test]
fn incremental_sync_skips_conflicted_files() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");
    schema_definition::create_schema(&conn).expect("should create schema");

    // Create a conflicted document
    let conflicted_path = temp_dir.path().join("conflicted.md");
    std::fs::write(
        &conflicted_path,
        r#"---
lattice-id: LCNF23
name: conflicted
description: Document with merge conflict
---

<<<<<<< HEAD
Local changes
=======
Remote changes
>>>>>>> origin/main
"#,
    )
    .expect("should write conflicted document");

    let change_info = ChangeInfo {
        modified_files: vec![PathBuf::from("conflicted.md")],
        deleted_files: vec![],
        uncommitted_files: vec![],
        current_head: Some("abc123".to_string()),
        last_indexed_commit: Some("def456".to_string()),
    };

    let git = FakeGit::new();
    let result = sync_strategies::incremental_sync(temp_dir.path(), &git, &conn, &change_info)
        .expect("incremental_sync should succeed");

    assert_eq!(result.files_updated, 0, "Should not have updated conflicted file");

    // Verify conflicted document is NOT in the database
    let doc = lattice::index::document_queries::lookup_by_id(&conn, "LCNF23")
        .expect("lookup should succeed");
    assert!(doc.is_none(), "Conflicted document should not be indexed");
}

#[test]
fn full_rebuild_populates_fts_index() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    // Create a document with searchable content
    let doc_path = temp_dir.path().join("searchable.md");
    std::fs::write(
        &doc_path,
        r#"---
lattice-id: LSRCH2
name: searchable-doc
description: A document with unique searchable content
---

# Searchable Document

This document contains the word xylophone which is unique.
"#,
    )
    .expect("should write searchable document");

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");
    schema_definition::create_schema(&conn).expect("should create schema");

    let git = FakeGit::with_ls_files(vec![PathBuf::from("searchable.md")]);

    sync_strategies::full_rebuild(temp_dir.path(), &git, &conn)
        .expect("full_rebuild should succeed");

    // Search for the unique word
    let results =
        lattice::index::fulltext_search::search(&conn, "xylophone").expect("search should succeed");
    assert_eq!(results.len(), 1, "Should find exactly one document with 'xylophone'");
    assert_eq!(results[0].document_id, "LSRCH2", "Found document ID should match");
}

// ============================================================================
// directory_roots incremental sync tests
// ============================================================================

/// Helper to create a root document (filename matches directory).
fn create_root_document(dir: &std::path::Path, subdir: &str, id: &str) {
    let relative_path = format!("{}/{}.md", subdir, subdir);
    create_lattice_document(dir, &relative_path, id, subdir);
}

#[test]
fn incremental_sync_adds_new_root_document_to_directory_roots() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");
    schema_definition::create_schema(&conn).expect("should create schema");

    // Create a root document: api/api.md
    create_root_document(temp_dir.path(), "api", "LAPI23");

    let change_info = ChangeInfo {
        modified_files: vec![PathBuf::from("api/api.md")],
        deleted_files: vec![],
        uncommitted_files: vec![],
        current_head: Some("abc123".to_string()),
        last_indexed_commit: Some("def456".to_string()),
    };

    let git = FakeGit::new();
    let result = sync_strategies::incremental_sync(temp_dir.path(), &git, &conn, &change_info)
        .expect("incremental_sync should succeed");

    assert_eq!(result.files_updated, 1, "Should have updated 1 file");

    // Verify directory_roots entry was created
    let root_id = lattice::index::directory_roots::get_root_id(&conn, "api")
        .expect("get_root_id should succeed");
    assert_eq!(root_id, Some("LAPI23".to_string()), "directory_roots should have the root doc ID");
}

#[test]
fn incremental_sync_removes_deleted_root_from_directory_roots() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");
    schema_definition::create_schema(&conn).expect("should create schema");

    // First, create and index a root document
    create_root_document(temp_dir.path(), "api", "LAPI24");

    let git = FakeGit::with_ls_files(vec![PathBuf::from("api/api.md")]);
    sync_strategies::full_rebuild(temp_dir.path(), &git, &conn)
        .expect("initial full_rebuild should succeed");

    // Verify directory_roots entry exists
    let root_id = lattice::index::directory_roots::get_root_id(&conn, "api")
        .expect("get_root_id should succeed");
    assert_eq!(root_id, Some("LAPI24".to_string()), "Root should be in directory_roots initially");

    // Delete the file
    std::fs::remove_file(temp_dir.path().join("api/api.md")).expect("should delete root doc");

    let change_info = ChangeInfo {
        modified_files: vec![],
        deleted_files: vec![PathBuf::from("api/api.md")],
        uncommitted_files: vec![],
        current_head: Some("new123".to_string()),
        last_indexed_commit: Some("old456".to_string()),
    };

    let git2 = FakeGit::new();
    sync_strategies::incremental_sync(temp_dir.path(), &git2, &conn, &change_info)
        .expect("incremental_sync should succeed");

    // Verify directory_roots entry was removed
    let root_id = lattice::index::directory_roots::get_root_id(&conn, "api")
        .expect("get_root_id should succeed");
    assert_eq!(root_id, None, "directory_roots entry should be removed after deletion");
}

#[test]
fn incremental_sync_does_not_affect_directory_roots_for_non_root_documents() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");
    schema_definition::create_schema(&conn).expect("should create schema");

    // Create a non-root document: api/readme.md (not api/api.md)
    create_lattice_document(temp_dir.path(), "api/readme.md", "LRDM23", "readme");

    let change_info = ChangeInfo {
        modified_files: vec![PathBuf::from("api/readme.md")],
        deleted_files: vec![],
        uncommitted_files: vec![],
        current_head: Some("abc123".to_string()),
        last_indexed_commit: Some("def456".to_string()),
    };

    let git = FakeGit::new();
    sync_strategies::incremental_sync(temp_dir.path(), &git, &conn, &change_info)
        .expect("incremental_sync should succeed");

    // Verify NO directory_roots entry was created for api/
    let root_id = lattice::index::directory_roots::get_root_id(&conn, "api")
        .expect("get_root_id should succeed");
    assert_eq!(root_id, None, "Non-root document should not create directory_roots entry");
}

#[test]
fn incremental_sync_handles_root_document_move() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");
    schema_definition::create_schema(&conn).expect("should create schema");

    // First, create and index a root document at api/api.md
    create_root_document(temp_dir.path(), "api", "LMOV25");

    let git = FakeGit::with_ls_files(vec![PathBuf::from("api/api.md")]);
    sync_strategies::full_rebuild(temp_dir.path(), &git, &conn)
        .expect("initial full_rebuild should succeed");

    // Verify initial state
    let api_root =
        lattice::index::directory_roots::get_root_id(&conn, "api").expect("should succeed");
    assert_eq!(api_root, Some("LMOV25".to_string()), "api should have root initially");

    // Simulate a move: delete old, create at new location
    std::fs::remove_file(temp_dir.path().join("api/api.md")).expect("should delete old");
    create_root_document(temp_dir.path(), "core", "LMOV25");

    let change_info = ChangeInfo {
        modified_files: vec![PathBuf::from("core/core.md")],
        deleted_files: vec![PathBuf::from("api/api.md")],
        uncommitted_files: vec![],
        current_head: Some("new123".to_string()),
        last_indexed_commit: Some("old456".to_string()),
    };

    let git2 = FakeGit::new();
    sync_strategies::incremental_sync(temp_dir.path(), &git2, &conn, &change_info)
        .expect("incremental_sync should succeed");

    // Verify old directory_roots entry was removed
    let api_root =
        lattice::index::directory_roots::get_root_id(&conn, "api").expect("should succeed");
    assert_eq!(api_root, None, "Old api root should be removed");

    // Verify new directory_roots entry was created
    let core_root =
        lattice::index::directory_roots::get_root_id(&conn, "core").expect("should succeed");
    assert_eq!(core_root, Some("LMOV25".to_string()), "New core root should be created");
}

#[test]
fn full_rebuild_populates_directory_roots() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let lattice_dir = temp_dir.path().join(".lattice");
    std::fs::create_dir_all(&lattice_dir).expect("failed to create .lattice dir");

    // Create multiple root documents
    create_root_document(temp_dir.path(), "api", "LAPIDR");
    create_root_document(temp_dir.path(), "core", "LCORDR");
    // Create a non-root document
    create_lattice_document(temp_dir.path(), "docs/readme.md", "LRDMDR", "readme");

    let conn = connection_pool::open_connection(temp_dir.path())
        .expect("should open connection successfully");
    schema_definition::create_schema(&conn).expect("should create schema");

    let git = FakeGit::with_ls_files(vec![
        PathBuf::from("api/api.md"),
        PathBuf::from("core/core.md"),
        PathBuf::from("docs/readme.md"),
    ]);

    sync_strategies::full_rebuild(temp_dir.path(), &git, &conn)
        .expect("full_rebuild should succeed");

    // Verify root documents are in directory_roots
    let api_root =
        lattice::index::directory_roots::get_root_id(&conn, "api").expect("should succeed");
    assert_eq!(api_root, Some("LAPIDR".to_string()), "api should be in directory_roots");

    let core_root =
        lattice::index::directory_roots::get_root_id(&conn, "core").expect("should succeed");
    assert_eq!(core_root, Some("LCORDR".to_string()), "core should be in directory_roots");

    // Verify non-root document is NOT in directory_roots
    let docs_root =
        lattice::index::directory_roots::get_root_id(&conn, "docs").expect("should succeed");
    assert_eq!(docs_root, None, "docs should NOT be in directory_roots (readme.md is not root)");
}
