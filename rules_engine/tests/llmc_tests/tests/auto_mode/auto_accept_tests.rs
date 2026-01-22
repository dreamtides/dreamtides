use llmc::auto_mode::auto_accept::AutoAcceptResult;

#[test]
fn accepted_with_cleanup_failure_contains_commit_and_error() {
    let result = AutoAcceptResult::AcceptedWithCleanupFailure {
        commit_sha: "abc123def456".to_string(),
        cleanup_error: "Failed to remove worktree".to_string(),
    };

    match result {
        AutoAcceptResult::AcceptedWithCleanupFailure { commit_sha, cleanup_error } => {
            assert_eq!(commit_sha, "abc123def456", "Should contain the commit SHA");
            assert!(
                cleanup_error.contains("worktree"),
                "Should contain the cleanup error: {}",
                cleanup_error
            );
        }
        _ => panic!("Expected AcceptedWithCleanupFailure variant"),
    }
}

#[test]
fn accepted_result_contains_commit_sha() {
    let result = AutoAcceptResult::Accepted { commit_sha: "def789".to_string() };

    match result {
        AutoAcceptResult::Accepted { commit_sha } => {
            assert_eq!(commit_sha, "def789", "Should contain the commit SHA");
        }
        _ => panic!("Expected Accepted variant"),
    }
}

#[test]
fn no_changes_result() {
    let result = AutoAcceptResult::NoChanges;

    assert!(matches!(result, AutoAcceptResult::NoChanges), "Should be NoChanges variant");
}

#[test]
fn source_repo_dirty_result() {
    let result = AutoAcceptResult::SourceRepoDirty;

    assert!(
        matches!(result, AutoAcceptResult::SourceRepoDirty),
        "Should be SourceRepoDirty variant"
    );
}

#[test]
fn rebase_conflict_result_contains_conflicts() {
    let result = AutoAcceptResult::RebaseConflict {
        conflicts: vec!["file1.rs".to_string(), "file2.rs".to_string()],
    };

    match result {
        AutoAcceptResult::RebaseConflict { conflicts } => {
            assert_eq!(conflicts.len(), 2, "Should contain 2 conflicts");
            assert!(conflicts.contains(&"file1.rs".to_string()));
            assert!(conflicts.contains(&"file2.rs".to_string()));
        }
        _ => panic!("Expected RebaseConflict variant"),
    }
}

#[test]
fn accepted_with_cleanup_failure_is_debug_printable() {
    let result = AutoAcceptResult::AcceptedWithCleanupFailure {
        commit_sha: "abc123".to_string(),
        cleanup_error: "Test error".to_string(),
    };

    let debug_str = format!("{:?}", result);
    assert!(
        debug_str.contains("AcceptedWithCleanupFailure"),
        "Debug output should contain variant name: {}",
        debug_str
    );
    assert!(debug_str.contains("abc123"), "Debug output should contain commit SHA: {}", debug_str);
    assert!(
        debug_str.contains("Test error"),
        "Debug output should contain error message: {}",
        debug_str
    );
}
