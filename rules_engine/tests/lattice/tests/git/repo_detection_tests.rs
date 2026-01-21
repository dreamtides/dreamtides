use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use lattice::error::error_types::LatticeError;
use lattice::git::git_ops::{FileChange, FileStatus, GitOps};
use lattice::git::repo_detection::{InProgressOp, RepoConfig};

/// A test double for GitOps that returns configured responses for repo
/// detection.
struct FakeGit {
    config_values: Mutex<HashMap<String, Option<String>>>,
    rev_parse_values: Mutex<HashMap<String, String>>,
}

impl FakeGit {
    fn new() -> Self {
        Self {
            config_values: Mutex::new(HashMap::new()),
            rev_parse_values: Mutex::new(HashMap::new()),
        }
    }

    fn with_config(self, key: &str, value: Option<&str>) -> Self {
        self.config_values.lock().unwrap().insert(key.to_string(), value.map(String::from));
        self
    }

    fn with_rev_parse(self, key: &str, value: &str) -> Self {
        self.rev_parse_values.lock().unwrap().insert(key.to_string(), value.to_string());
        self
    }
}

impl GitOps for FakeGit {
    fn ls_files(&self, _pattern: &str) -> Result<Vec<PathBuf>, LatticeError> {
        Ok(Vec::new())
    }

    fn diff(
        &self,
        _from_commit: &str,
        _to_commit: &str,
        _pattern: &str,
    ) -> Result<Vec<PathBuf>, LatticeError> {
        Ok(Vec::new())
    }

    fn status(&self, _pattern: &str) -> Result<Vec<FileStatus>, LatticeError> {
        Ok(Vec::new())
    }

    fn rev_parse(&self, git_ref: &str) -> Result<String, LatticeError> {
        let values = self.rev_parse_values.lock().unwrap();
        if let Some(value) = values.get(git_ref) {
            Ok(value.clone())
        } else if git_ref == "--is-bare-repository" {
            Ok("false".to_string())
        } else {
            Ok("abc123".to_string())
        }
    }

    fn log(
        &self,
        _path: Option<&str>,
        _format: &str,
        _limit: usize,
    ) -> Result<Vec<String>, LatticeError> {
        Ok(Vec::new())
    }

    fn config_get(&self, key: &str) -> Result<Option<String>, LatticeError> {
        let values = self.config_values.lock().unwrap();
        Ok(values.get(key).cloned().flatten())
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
fn detect_reports_standard_repo_when_no_edge_cases() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    // Create minimal .git directory
    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");

    let git = FakeGit::new();
    let config = RepoConfig::detect(repo_root, &git).expect("Detection should succeed");

    assert!(!config.is_shallow, "Should not detect shallow clone");
    assert!(!config.is_partial, "Should not detect partial clone");
    assert!(!config.is_sparse, "Should not detect sparse checkout");
    assert!(!config.is_worktree, "Should not detect worktree");
    assert!(!config.has_submodules, "Should not detect submodules");
    assert!(!config.is_bare, "Should not detect bare repository");
    assert!(config.in_progress_op.is_none(), "Should not detect in-progress operation");
}

#[test]
fn detect_identifies_shallow_clone() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    // Create .git directory with shallow file
    let git_dir = repo_root.join(".git");
    fs::create_dir(&git_dir).expect("Failed to create .git");
    fs::write(git_dir.join("shallow"), "abc123\n").expect("Failed to create shallow file");

    let git = FakeGit::new();
    let config = RepoConfig::detect(repo_root, &git).expect("Detection should succeed");

    assert!(config.is_shallow, "Should detect shallow clone when .git/shallow exists");
}

#[test]
fn detect_identifies_partial_clone() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();
    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");

    let git = FakeGit::new()
        .with_config("remote.origin.promisor", Some("true"))
        .with_config("remote.origin.partialclonefilter", Some("blob:none"));

    let config = RepoConfig::detect(repo_root, &git).expect("Detection should succeed");

    assert!(config.is_partial, "Should detect partial clone");
    assert_eq!(
        config.partial_filter,
        Some("blob:none".to_string()),
        "Should capture filter specification"
    );
}

#[test]
fn detect_identifies_sparse_checkout() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();
    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");

    let git = FakeGit::new().with_config("core.sparseCheckout", Some("true"));

    let config = RepoConfig::detect(repo_root, &git).expect("Detection should succeed");

    assert!(config.is_sparse, "Should detect sparse checkout");
}

#[test]
fn detect_sparse_checkout_case_insensitive() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();
    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");

    let git = FakeGit::new().with_config("core.sparseCheckout", Some("TRUE"));

    let config = RepoConfig::detect(repo_root, &git).expect("Detection should succeed");

    assert!(config.is_sparse, "Should detect sparse checkout with uppercase TRUE");
}

#[test]
fn detect_identifies_submodules() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");
    fs::write(repo_root.join(".gitmodules"), "[submodule \"lib\"]\n")
        .expect("Failed to create .gitmodules");

    let git = FakeGit::new();
    let config = RepoConfig::detect(repo_root, &git).expect("Detection should succeed");

    assert!(config.has_submodules, "Should detect submodules when .gitmodules exists");
}

#[test]
fn detect_identifies_bare_repository() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();
    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");

    let git = FakeGit::new().with_rev_parse("--is-bare-repository", "true");

    let config = RepoConfig::detect(repo_root, &git).expect("Detection should succeed");

    assert!(config.is_bare, "Should detect bare repository");
}

#[test]
fn detect_identifies_rebase_in_progress() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    let git_dir = repo_root.join(".git");
    fs::create_dir(&git_dir).expect("Failed to create .git");
    fs::create_dir(git_dir.join("rebase-merge")).expect("Failed to create rebase-merge");

    let git = FakeGit::new();
    let config = RepoConfig::detect(repo_root, &git).expect("Detection should succeed");

    assert_eq!(
        config.in_progress_op,
        Some(InProgressOp::Rebase),
        "Should detect interactive rebase via rebase-merge"
    );
}

#[test]
fn detect_identifies_rebase_apply_in_progress() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    let git_dir = repo_root.join(".git");
    fs::create_dir(&git_dir).expect("Failed to create .git");
    fs::create_dir(git_dir.join("rebase-apply")).expect("Failed to create rebase-apply");

    let git = FakeGit::new();
    let config = RepoConfig::detect(repo_root, &git).expect("Detection should succeed");

    assert_eq!(
        config.in_progress_op,
        Some(InProgressOp::Rebase),
        "Should detect rebase via rebase-apply"
    );
}

#[test]
fn detect_identifies_merge_in_progress() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    let git_dir = repo_root.join(".git");
    fs::create_dir(&git_dir).expect("Failed to create .git");
    fs::write(git_dir.join("MERGE_HEAD"), "abc123\n").expect("Failed to create MERGE_HEAD");

    let git = FakeGit::new();
    let config = RepoConfig::detect(repo_root, &git).expect("Detection should succeed");

    assert_eq!(config.in_progress_op, Some(InProgressOp::Merge), "Should detect merge in progress");
}

#[test]
fn detect_identifies_cherry_pick_in_progress() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    let git_dir = repo_root.join(".git");
    fs::create_dir(&git_dir).expect("Failed to create .git");
    fs::write(git_dir.join("CHERRY_PICK_HEAD"), "abc123\n")
        .expect("Failed to create CHERRY_PICK_HEAD");

    let git = FakeGit::new();
    let config = RepoConfig::detect(repo_root, &git).expect("Detection should succeed");

    assert_eq!(
        config.in_progress_op,
        Some(InProgressOp::CherryPick),
        "Should detect cherry-pick in progress"
    );
}

#[test]
fn detect_identifies_revert_in_progress() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    let git_dir = repo_root.join(".git");
    fs::create_dir(&git_dir).expect("Failed to create .git");
    fs::write(git_dir.join("REVERT_HEAD"), "abc123\n").expect("Failed to create REVERT_HEAD");

    let git = FakeGit::new();
    let config = RepoConfig::detect(repo_root, &git).expect("Detection should succeed");

    assert_eq!(
        config.in_progress_op,
        Some(InProgressOp::Revert),
        "Should detect revert in progress"
    );
}

#[test]
fn cache_save_and_load_round_trips() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    // Create .git directory
    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");

    let git = FakeGit::new()
        .with_config("core.sparseCheckout", Some("true"))
        .with_config("remote.origin.promisor", Some("true"))
        .with_config("remote.origin.partialclonefilter", Some("blob:none"));

    let original = RepoConfig::detect(repo_root, &git).expect("Detection should succeed");
    original.save_cache(repo_root).expect("Save should succeed");

    let loaded = RepoConfig::load_cached(repo_root)
        .expect("Load should succeed")
        .expect("Cache should exist");

    assert_eq!(original.is_sparse, loaded.is_sparse, "Sparse checkout should round-trip");
    assert_eq!(original.is_partial, loaded.is_partial, "Partial clone should round-trip");
    assert_eq!(original.partial_filter, loaded.partial_filter, "Filter should round-trip");
}

#[test]
fn cache_invalidates_when_git_mtime_differs() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    let git_dir = repo_root.join(".git");
    fs::create_dir(&git_dir).expect("Failed to create .git");

    let git = FakeGit::new();
    let config = RepoConfig::detect(repo_root, &git).expect("Detection should succeed");
    config.save_cache(repo_root).expect("Save should succeed");

    // Manually modify the cached config to have a different git_mtime
    let cache_path = RepoConfig::cache_path(repo_root);
    let content = fs::read_to_string(&cache_path).expect("Failed to read cache");
    let modified_content =
        content.replace(&format!("\"git_mtime\": {}", config.git_mtime), "\"git_mtime\": 12345");
    fs::write(&cache_path, modified_content).expect("Failed to modify cache");

    let loaded = RepoConfig::load_cached(repo_root).expect("Load should succeed");

    assert!(loaded.is_none(), "Cache should be invalidated when git_mtime differs from current");
}

#[test]
fn load_or_detect_uses_cache_when_valid() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");

    let git = FakeGit::new();

    // First call detects and caches
    let first = RepoConfig::load_or_detect(repo_root, &git).expect("First load should succeed");

    // Second call should use cache
    let second = RepoConfig::load_or_detect(repo_root, &git).expect("Second load should succeed");

    assert_eq!(
        first.detected_at, second.detected_at,
        "Second call should use cached config with same timestamp"
    );
}
