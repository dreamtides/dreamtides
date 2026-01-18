use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use lattice::claim::claim_operations::{
    claim_task, get_claim, is_claimed, list_claims, release_claim,
};
use lattice::error::error_types::LatticeError;
use lattice::id::lattice_id::LatticeId;

#[test]
fn claim_and_release_task() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();
    let work_path = PathBuf::from("/test/worktree");
    let id = LatticeId::from_parts(100, "ABC");

    // Initially not claimed
    assert!(!is_claimed(repo_root, &id).expect("is_claimed failed"));

    // Claim the task
    claim_task(repo_root, &id, &work_path).expect("claim_task failed");

    // Now it should be claimed
    assert!(is_claimed(repo_root, &id).expect("is_claimed failed"));

    // Release the claim
    release_claim(repo_root, &id).expect("release_claim failed");

    // No longer claimed
    assert!(!is_claimed(repo_root, &id).expect("is_claimed failed"));
}

#[test]
fn claim_already_claimed_task_fails() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();
    let work_path = PathBuf::from("/test/worktree");
    let id = LatticeId::from_parts(200, "DEF");

    // Claim the task
    claim_task(repo_root, &id, &work_path).expect("First claim should succeed");

    // Try to claim again - should fail
    let result = claim_task(repo_root, &id, &work_path);
    assert!(result.is_err(), "Second claim should fail");
    match result.unwrap_err() {
        LatticeError::OperationNotAllowed { reason } => {
            assert!(
                reason.contains(&id.to_string()),
                "Error message should mention the task ID: {reason}"
            );
        }
        e => panic!("Expected OperationNotAllowed error, got {e:?}"),
    }
}

#[test]
fn release_nonexistent_claim_is_idempotent() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();
    let id = LatticeId::from_parts(300, "GHI");

    // Release a claim that was never created - should succeed
    release_claim(repo_root, &id).expect("Release should be idempotent");

    // Release again - still should succeed
    release_claim(repo_root, &id).expect("Release should remain idempotent");
}

#[test]
fn get_claim_returns_data_when_claimed() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();
    let work_path = PathBuf::from("/test/worktree/feature-branch");
    let id = LatticeId::from_parts(400, "JKL");

    // Initially no claim data
    let data = get_claim(repo_root, &id).expect("get_claim failed");
    assert!(data.is_none(), "Should have no claim data initially");

    // Claim the task
    claim_task(repo_root, &id, &work_path).expect("claim_task failed");

    // Now should have claim data
    let data = get_claim(repo_root, &id)
        .expect("get_claim failed")
        .expect("Should have claim data after claiming");

    assert_eq!(data.work_path, work_path, "Work path should match");
}

#[test]
fn list_claims_returns_empty_for_no_claims() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    let claims = list_claims(repo_root).expect("list_claims failed");
    assert!(claims.is_empty(), "Should have no claims initially");
}

#[test]
fn list_claims_returns_all_claims_sorted_by_time() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();
    let work_path = PathBuf::from("/test/worktree");

    let id1 = LatticeId::from_parts(500, "MNO");
    let id2 = LatticeId::from_parts(501, "MNO");
    let id3 = LatticeId::from_parts(502, "MNO");

    // Claim tasks with small delays to ensure ordering
    claim_task(repo_root, &id1, &work_path).expect("claim_task 1 failed");
    thread::sleep(Duration::from_millis(10));
    claim_task(repo_root, &id2, &work_path).expect("claim_task 2 failed");
    thread::sleep(Duration::from_millis(10));
    claim_task(repo_root, &id3, &work_path).expect("claim_task 3 failed");

    let claims = list_claims(repo_root).expect("list_claims failed");
    assert_eq!(claims.len(), 3, "Should have 3 claims");

    // Verify sorted by timestamp (oldest first)
    assert_eq!(claims[0].id, id1, "First claim should be id1 (oldest)");
    assert_eq!(claims[1].id, id2, "Second claim should be id2");
    assert_eq!(claims[2].id, id3, "Third claim should be id3 (newest)");

    // Verify timestamps are in order
    assert!(
        claims[0].data.claimed_at <= claims[1].data.claimed_at,
        "Claims should be sorted by time"
    );
    assert!(
        claims[1].data.claimed_at <= claims[2].data.claimed_at,
        "Claims should be sorted by time"
    );
}

#[test]
fn list_claims_excludes_released_claims() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();
    let work_path = PathBuf::from("/test/worktree");

    let id1 = LatticeId::from_parts(600, "PQR");
    let id2 = LatticeId::from_parts(601, "PQR");

    claim_task(repo_root, &id1, &work_path).expect("claim_task 1 failed");
    claim_task(repo_root, &id2, &work_path).expect("claim_task 2 failed");

    // Release one claim
    release_claim(repo_root, &id1).expect("release_claim failed");

    let claims = list_claims(repo_root).expect("list_claims failed");
    assert_eq!(claims.len(), 1, "Should have 1 claim after release");
    assert_eq!(claims[0].id, id2, "Remaining claim should be id2");
}

#[test]
fn claim_stores_correct_work_path() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();
    let work_path = PathBuf::from("/path/to/my/project/worktree-feature");
    let id = LatticeId::from_parts(700, "STU");

    claim_task(repo_root, &id, &work_path).expect("claim_task failed");

    let claim_data =
        get_claim(repo_root, &id).expect("get_claim failed").expect("Should have claim data");

    assert_eq!(claim_data.work_path, work_path, "Claim should store the exact work path");
}

#[test]
fn multiple_repos_have_separate_claims() {
    let temp_dir1 = tempfile::tempdir().expect("Failed to create temp dir 1");
    let temp_dir2 = tempfile::tempdir().expect("Failed to create temp dir 2");
    let repo_root1 = temp_dir1.path();
    let repo_root2 = temp_dir2.path();
    let work_path = PathBuf::from("/test/worktree");
    let id = LatticeId::from_parts(800, "VWX");

    // Claim same ID in both repos
    claim_task(repo_root1, &id, &work_path).expect("claim in repo1 failed");
    claim_task(repo_root2, &id, &work_path).expect("claim in repo2 failed");

    // Both should show as claimed (separate claim stores)
    assert!(is_claimed(repo_root1, &id).expect("is_claimed repo1 failed"));
    assert!(is_claimed(repo_root2, &id).expect("is_claimed repo2 failed"));

    // Release in repo1 should not affect repo2
    release_claim(repo_root1, &id).expect("release_claim repo1 failed");
    assert!(!is_claimed(repo_root1, &id).expect("is_claimed repo1 failed"));
    assert!(is_claimed(repo_root2, &id).expect("is_claimed repo2 should still be claimed"));
}
