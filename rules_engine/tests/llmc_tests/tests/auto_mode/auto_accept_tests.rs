use llmc::auto_mode::auto_accept::AutoAcceptResult;
use llmc::auto_mode::auto_config::{AutoConfig, ResolvedAutoConfig};

#[test]
fn auto_config_from_resolved_preserves_post_accept_command() {
    let config = AutoConfig {
        task_list_id: Some("my-project".to_string()),
        post_accept_command: Some("/path/to/post_accept.sh".to_string()),
        ..Default::default()
    };
    let resolved = ResolvedAutoConfig::resolve(Some(&config), "/path/to/repo", Some(2), None)
        .expect("Should resolve config");

    assert_eq!(
        resolved.post_accept_command,
        Some("/path/to/post_accept.sh".to_string()),
        "ResolvedAutoConfig should have post_accept_command"
    );

    let auto_cfg = AutoConfig {
        task_list_id: Some(resolved.task_list_id.clone()),
        tasks_root: Some(resolved.tasks_root.to_string_lossy().to_string()),
        context_config_path: resolved
            .context_config_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string()),
        concurrency: resolved.concurrency,
        post_accept_command: resolved.post_accept_command.clone(),
    };

    assert_eq!(
        auto_cfg.post_accept_command,
        Some("/path/to/post_accept.sh".to_string()),
        "AutoConfig created from ResolvedAutoConfig should preserve post_accept_command"
    );
}

#[test]
fn auto_config_from_resolved_with_none_post_accept_command() {
    let config = AutoConfig { task_list_id: Some("my-project".to_string()), ..Default::default() };
    let resolved = ResolvedAutoConfig::resolve(Some(&config), "/path/to/repo", Some(2), None)
        .expect("Should resolve config");

    assert_eq!(
        resolved.post_accept_command, None,
        "ResolvedAutoConfig should have None for post_accept_command"
    );

    let auto_cfg = AutoConfig {
        task_list_id: Some(resolved.task_list_id.clone()),
        tasks_root: Some(resolved.tasks_root.to_string_lossy().to_string()),
        context_config_path: resolved
            .context_config_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string()),
        concurrency: resolved.concurrency,
        post_accept_command: resolved.post_accept_command.clone(),
    };

    assert_eq!(
        auto_cfg.post_accept_command, None,
        "AutoConfig should have None for post_accept_command when not configured"
    );
}

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
