use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use lattice::error::error_types::LatticeError;
use lattice::git::edge_cases::{repo_state_message, validate_repo_state};
use lattice::git::git_ops::{FileChange, FileStatus, GitOps};
use lattice::git::repo_detection::{InProgressOp, RepoConfig};

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
}

#[test]
fn blobless_clone_detected_with_blob_none_filter() {
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
        "Should capture blob:none filter"
    );
}

#[test]
fn treeless_clone_detected_with_tree_zero_filter() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();
    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");

    let git = FakeGit::new()
        .with_config("remote.origin.promisor", Some("true"))
        .with_config("remote.origin.partialclonefilter", Some("tree:0"));

    let config = RepoConfig::detect(repo_root, &git).expect("Detection should succeed");

    assert!(config.is_partial, "Should detect partial clone");
    assert_eq!(config.partial_filter, Some("tree:0".to_string()), "Should capture tree:0 filter");
}

#[test]
fn treeless_clone_triggers_warning_message() {
    let mut config = make_base_config();
    config.is_partial = true;
    config.partial_filter = Some("tree:0".to_string());

    let message = repo_state_message(&config).expect("Should have message");

    assert!(message.contains("treeless"), "Message should warn about treeless clone");
    assert!(message.contains("blob:none"), "Message should suggest blob:none alternative");
}

#[test]
fn blobless_clone_does_not_trigger_treeless_warning() {
    let mut config = make_base_config();
    config.is_partial = true;
    config.partial_filter = Some("blob:none".to_string());

    let message = repo_state_message(&config);

    assert!(
        message.is_none() || !message.as_ref().unwrap().contains("treeless"),
        "Blobless clone should not show treeless warning"
    );
}

#[test]
fn combined_shallow_and_partial_detected() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    let git_dir = repo_root.join(".git");
    fs::create_dir(&git_dir).expect("Failed to create .git");
    fs::write(git_dir.join("shallow"), "abc123\n").expect("Failed to create shallow file");

    let git = FakeGit::new()
        .with_config("remote.origin.promisor", Some("true"))
        .with_config("remote.origin.partialclonefilter", Some("blob:none"));

    let config = RepoConfig::detect(repo_root, &git).expect("Detection should succeed");

    assert!(config.is_shallow, "Should detect shallow clone");
    assert!(config.is_partial, "Should detect partial clone");
}

#[test]
fn combined_sparse_and_submodules_detected() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");
    fs::write(repo_root.join(".gitmodules"), "[submodule]\n")
        .expect("Failed to create .gitmodules");

    let git = FakeGit::new().with_config("core.sparseCheckout", Some("true"));

    let config = RepoConfig::detect(repo_root, &git).expect("Detection should succeed");

    assert!(config.is_sparse, "Should detect sparse checkout");
    assert!(config.has_submodules, "Should detect submodules");
}

#[test]
fn all_edge_cases_combined() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    let git_dir = repo_root.join(".git");
    fs::create_dir(&git_dir).expect("Failed to create .git");
    fs::write(git_dir.join("shallow"), "abc123\n").expect("Failed to create shallow");
    fs::write(git_dir.join("MERGE_HEAD"), "def456\n").expect("Failed to create MERGE_HEAD");
    fs::write(repo_root.join(".gitmodules"), "[submodule]\n")
        .expect("Failed to create .gitmodules");

    let git = FakeGit::new()
        .with_config("remote.origin.promisor", Some("true"))
        .with_config("remote.origin.partialclonefilter", Some("blob:none"))
        .with_config("core.sparseCheckout", Some("true"));

    let config = RepoConfig::detect(repo_root, &git).expect("Detection should succeed");

    assert!(config.is_shallow, "Should detect shallow clone");
    assert!(config.is_partial, "Should detect partial clone");
    assert!(config.is_sparse, "Should detect sparse checkout");
    assert!(config.has_submodules, "Should detect submodules");
    assert_eq!(config.in_progress_op, Some(InProgressOp::Merge), "Should detect merge");
}

#[test]
fn combined_state_message_includes_all_observations() {
    let mut config = make_base_config();
    config.is_shallow = true;
    config.is_sparse = true;
    config.has_submodules = true;
    config.in_progress_op = Some(InProgressOp::Rebase);

    let message = repo_state_message(&config).expect("Should have combined message");

    assert!(message.contains("shallow"), "Should mention shallow");
    assert!(message.contains("Sparse"), "Should mention sparse");
    assert!(message.contains("Submodules"), "Should mention submodules");
    assert!(message.contains("rebase"), "Should mention rebase");
}

#[test]
fn cache_loads_when_git_mtime_unchanged() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();
    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");

    let git = FakeGit::new().with_config("core.sparseCheckout", Some("true"));

    let first = RepoConfig::detect(repo_root, &git).expect("First detection");
    first.save_cache(repo_root).expect("Save cache");

    let cached = RepoConfig::load_cached(repo_root)
        .expect("Load should succeed")
        .expect("Cache should exist");

    assert_eq!(first.detected_at, cached.detected_at, "Should load from cache");
    assert_eq!(first.is_sparse, cached.is_sparse, "Sparse flag should match");
}

#[test]
fn cache_invalidates_on_git_mtime_change() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();
    let git_dir = repo_root.join(".git");
    fs::create_dir(&git_dir).expect("Failed to create .git");

    let git = FakeGit::new();
    let config = RepoConfig::detect(repo_root, &git).expect("Detection");
    config.save_cache(repo_root).expect("Save cache");

    let cache_path = RepoConfig::cache_path(repo_root);
    let content = fs::read_to_string(&cache_path).expect("Read cache");
    let modified =
        content.replace(&format!("\"git_mtime\": {}", config.git_mtime), "\"git_mtime\": 99999");
    fs::write(&cache_path, modified).expect("Modify cache");

    let result = RepoConfig::load_cached(repo_root).expect("Load attempt");
    assert!(result.is_none(), "Cache should be invalidated when mtime differs");
}

#[test]
fn cache_missing_returns_none() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();
    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");

    let result = RepoConfig::load_cached(repo_root).expect("Load should not error");
    assert!(result.is_none(), "Missing cache should return None");
}

#[test]
fn load_or_detect_caches_on_first_call() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();
    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");

    let git = FakeGit::new();
    let _first = RepoConfig::load_or_detect(repo_root, &git).expect("First call");

    let cache_path = RepoConfig::cache_path(repo_root);
    assert!(cache_path.exists(), "Cache file should exist after first call");
}

#[test]
fn validate_repo_state_allows_combined_degraded_states() {
    let mut config = make_base_config();
    config.is_shallow = true;
    config.is_partial = true;
    config.partial_filter = Some("tree:0".to_string());
    config.is_sparse = true;
    config.has_submodules = true;

    let result = validate_repo_state(&config);
    assert!(result.is_ok(), "Combined degraded states should still be valid");
}

#[test]
fn validate_repo_state_rejects_bare_even_with_other_flags() {
    let mut config = make_base_config();
    config.is_bare = true;
    config.is_shallow = true;
    config.is_sparse = true;

    let result = validate_repo_state(&config);
    assert!(result.is_err(), "Bare repository should always fail validation");
}

#[test]
fn worktree_with_in_progress_op_detected() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    let main_git = temp_dir.path().join("main_repo").join(".git");
    fs::create_dir_all(&main_git).expect("Failed to create main .git");

    let worktree_git = main_git.join("worktrees").join("my-worktree");
    fs::create_dir_all(&worktree_git).expect("Failed to create worktree git dir");
    fs::write(worktree_git.join("MERGE_HEAD"), "abc123\n").expect("Failed to create MERGE_HEAD");

    let git_file_content = format!("gitdir: {}", worktree_git.display());
    fs::write(repo_root.join(".git"), git_file_content).expect("Failed to create .git file");

    let git = FakeGit::new();
    let config = RepoConfig::detect(repo_root, &git).expect("Detection should succeed");

    assert!(config.is_worktree, "Should detect worktree");
    assert_eq!(
        config.in_progress_op,
        Some(InProgressOp::Merge),
        "Should detect merge in worktree-specific git dir"
    );
}

fn make_base_config() -> RepoConfig {
    RepoConfig {
        detected_at: "2025-01-16T00:00:00Z".to_string(),
        git_mtime: 0,
        is_shallow: false,
        is_partial: false,
        partial_filter: None,
        is_sparse: false,
        is_worktree: false,
        main_git_dir: PathBuf::from(".git"),
        worktree_git_dir: None,
        has_submodules: false,
        is_bare: false,
        in_progress_op: None,
    }
}
