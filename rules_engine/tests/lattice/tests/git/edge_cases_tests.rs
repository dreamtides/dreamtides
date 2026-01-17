use std::fs;
use std::path::PathBuf;

use lattice::error::error_types::LatticeError;
use lattice::git::edge_cases::{
    effective_git_dir, repo_state_message, should_attempt_incremental_reconciliation,
    sparse_checkout_guidance, validate_repo_state,
};
use lattice::git::repo_detection::{InProgressOp, RepoConfig};

fn make_config() -> RepoConfig {
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

#[test]
fn validate_repo_state_succeeds_for_standard_repo() {
    let config = make_config();

    let result = validate_repo_state(&config);

    assert!(result.is_ok(), "Standard repo should pass validation");
}

#[test]
fn validate_repo_state_fails_for_bare_repo() {
    let mut config = make_config();
    config.is_bare = true;

    let result = validate_repo_state(&config);

    assert!(result.is_err(), "Bare repository should fail validation");
    let err = result.unwrap_err();
    assert!(
        matches!(err, LatticeError::OperationNotAllowed { .. }),
        "Error should be OperationNotAllowed, got: {err:?}"
    );
}

#[test]
fn validate_repo_state_succeeds_for_shallow_clone() {
    let mut config = make_config();
    config.is_shallow = true;

    let result = validate_repo_state(&config);

    assert!(result.is_ok(), "Shallow clone should pass validation (with warning)");
}

#[test]
fn validate_repo_state_succeeds_for_in_progress_operation() {
    let mut config = make_config();
    config.in_progress_op = Some(InProgressOp::Merge);

    let result = validate_repo_state(&config);

    assert!(result.is_ok(), "In-progress operation should pass validation (with warning)");
}

#[test]
fn repo_state_message_returns_none_for_standard_repo() {
    let config = make_config();

    let message = repo_state_message(&config);

    assert!(message.is_none(), "Standard repo should have no state message");
}

#[test]
fn repo_state_message_includes_shallow_warning() {
    let mut config = make_config();
    config.is_shallow = true;

    let message = repo_state_message(&config).expect("Should have message for shallow clone");

    assert!(message.contains("shallow"), "Message should mention shallow clone");
    assert!(message.contains("git fetch --unshallow"), "Message should include remediation");
}

#[test]
fn repo_state_message_includes_sparse_note() {
    let mut config = make_config();
    config.is_sparse = true;

    let message = repo_state_message(&config).expect("Should have message for sparse checkout");

    assert!(message.contains("Sparse checkout"), "Message should mention sparse checkout");
    assert!(message.contains("git sparse-checkout add"), "Message should include remediation");
}

#[test]
fn repo_state_message_includes_treeless_warning() {
    let mut config = make_config();
    config.is_partial = true;
    config.partial_filter = Some("tree:0".to_string());

    let message = repo_state_message(&config).expect("Should have message for treeless clone");

    assert!(message.contains("treeless"), "Message should mention treeless clone");
    assert!(message.contains("blob:none"), "Message should suggest blob:none alternative");
}

#[test]
fn repo_state_message_includes_in_progress_warning() {
    let mut config = make_config();
    config.in_progress_op = Some(InProgressOp::Rebase);

    let message = repo_state_message(&config).expect("Should have message for in-progress op");

    assert!(message.contains("rebase"), "Message should mention rebase");
    assert!(message.contains("git status"), "Message should suggest git status");
}

#[test]
fn repo_state_message_includes_submodule_note() {
    let mut config = make_config();
    config.has_submodules = true;

    let message = repo_state_message(&config).expect("Should have message for submodules");

    assert!(message.contains("Submodules"), "Message should mention submodules");
}

#[test]
fn repo_state_message_combines_multiple_observations() {
    let mut config = make_config();
    config.is_shallow = true;
    config.is_sparse = true;
    config.has_submodules = true;

    let message = repo_state_message(&config).expect("Should have combined message");

    assert!(message.contains("shallow"), "Message should mention shallow");
    assert!(message.contains("Sparse"), "Message should mention sparse");
    assert!(message.contains("Submodules"), "Message should mention submodules");
}

#[test]
fn should_attempt_incremental_reconciliation_returns_true_for_standard() {
    let config = make_config();

    assert!(
        should_attempt_incremental_reconciliation(&config),
        "Standard repo should attempt incremental reconciliation"
    );
}

#[test]
fn should_attempt_incremental_reconciliation_returns_true_for_shallow() {
    let mut config = make_config();
    config.is_shallow = true;

    assert!(
        should_attempt_incremental_reconciliation(&config),
        "Shallow clone should still attempt incremental reconciliation"
    );
}

#[test]
fn should_attempt_incremental_reconciliation_returns_false_for_bare() {
    let mut config = make_config();
    config.is_bare = true;

    assert!(
        !should_attempt_incremental_reconciliation(&config),
        "Bare repo should not attempt incremental reconciliation"
    );
}

#[test]
fn effective_git_dir_returns_git_for_standard_repo() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();
    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");

    let config = make_config();

    let git_dir = effective_git_dir(repo_root, &config);

    assert_eq!(git_dir, repo_root.join(".git"), "Should return .git for standard repo");
}

#[test]
fn effective_git_dir_returns_worktree_dir_for_worktree() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    let mut config = make_config();
    config.is_worktree = true;
    config.main_git_dir = PathBuf::from("/main/repo/.git");
    config.worktree_git_dir = Some(PathBuf::from("/main/repo/.git/worktrees/my-worktree"));

    let git_dir = effective_git_dir(repo_root, &config);

    assert_eq!(
        git_dir,
        PathBuf::from("/main/repo/.git/worktrees/my-worktree"),
        "Should return worktree-specific .git dir for worktree"
    );
}

#[test]
fn effective_git_dir_falls_back_to_repo_root_for_worktree_without_dir() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    let mut config = make_config();
    config.is_worktree = true;
    config.main_git_dir = PathBuf::from("/main/repo/.git");
    config.worktree_git_dir = None; // Fallback case

    let git_dir = effective_git_dir(repo_root, &config);

    assert_eq!(
        git_dir,
        repo_root.join(".git"),
        "Should fall back to repo_root/.git when worktree_git_dir is None"
    );
}

#[test]
fn sparse_checkout_guidance_includes_path_and_command() {
    let config = make_config();
    let doc_path = PathBuf::from("docs/api/design.md");

    let guidance = sparse_checkout_guidance(&doc_path, &config);

    assert!(guidance.contains("docs/api/design.md"), "Guidance should include document path");
    assert!(guidance.contains("git sparse-checkout add"), "Guidance should include command");
}
