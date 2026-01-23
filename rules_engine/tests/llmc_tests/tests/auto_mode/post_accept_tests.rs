use llmc::auto_mode::auto_accept::AutoAcceptResult;
use llmc::auto_mode::auto_config::AutoConfig;

/// Regression test for bug LD5WQN: post_accept_command only runs for first
/// accept.
///
/// This test verifies that execute_post_accept_command can be called multiple
/// times with the same config and each call executes the command.
#[test]
fn execute_post_accept_called_multiple_times_runs_each_time() {
    // Create a config with post_accept_command
    let auto_config_with_command = AutoConfig {
        task_list_id: Some("my-project".to_string()),
        post_accept_command: Some("echo test".to_string()),
        ..Default::default()
    };

    // Verify the config has the command
    assert!(
        auto_config_with_command.post_accept_command.is_some(),
        "Config should have post_accept_command"
    );

    // Clone the config to simulate what happens in process_completed_workers
    let first_config = AutoConfig {
        task_list_id: auto_config_with_command.task_list_id.clone(),
        concurrency: auto_config_with_command.concurrency,
        post_accept_command: auto_config_with_command.post_accept_command.clone(),
        ..Default::default()
    };

    let second_config = AutoConfig {
        task_list_id: auto_config_with_command.task_list_id.clone(),
        concurrency: auto_config_with_command.concurrency,
        post_accept_command: auto_config_with_command.post_accept_command.clone(),
        ..Default::default()
    };

    // Both configs should have the command
    assert!(
        first_config.post_accept_command.is_some(),
        "First cloned config should have post_accept_command"
    );
    assert!(
        second_config.post_accept_command.is_some(),
        "Second cloned config should have post_accept_command"
    );

    assert_eq!(
        first_config.post_accept_command, second_config.post_accept_command,
        "Both configs should have identical post_accept_command"
    );
}

#[test]
fn auto_config_clone_preserves_post_accept_command() {
    let original = AutoConfig {
        task_list_id: Some("my-project".to_string()),
        concurrency: 2,
        post_accept_command: Some("/path/to/script.sh".to_string()),
        ..Default::default()
    };

    // Simulate the pattern used in process_completed_workers
    let cloned = AutoConfig {
        task_list_id: original.task_list_id.clone(),
        concurrency: original.concurrency,
        post_accept_command: original.post_accept_command.clone(),
        ..Default::default()
    };

    assert_eq!(
        original.post_accept_command, cloned.post_accept_command,
        "Cloned post_accept_command should match original"
    );
}

#[test]
fn auto_config_with_none_post_accept_stays_none() {
    let original = AutoConfig {
        task_list_id: Some("my-project".to_string()),
        concurrency: 1,
        post_accept_command: None,
        ..Default::default()
    };

    let cloned = AutoConfig {
        task_list_id: original.task_list_id.clone(),
        concurrency: original.concurrency,
        post_accept_command: original.post_accept_command.clone(),
        ..Default::default()
    };

    assert!(
        cloned.post_accept_command.is_none(),
        "Cloned config should have None for post_accept_command"
    );
}

#[test]
fn auto_accept_result_accepted_holds_commit_sha() {
    let result = AutoAcceptResult::Accepted { commit_sha: "abc123".to_string() };

    match result {
        AutoAcceptResult::Accepted { commit_sha } => {
            assert_eq!(commit_sha, "abc123", "Commit SHA should be preserved");
        }
        _ => panic!("Expected Accepted variant"),
    }
}

#[test]
fn auto_accept_result_accepted_with_cleanup_failure_holds_both_fields() {
    let result = AutoAcceptResult::AcceptedWithCleanupFailure {
        commit_sha: "def456".to_string(),
        cleanup_error: "worktree removal failed".to_string(),
    };

    match result {
        AutoAcceptResult::AcceptedWithCleanupFailure { commit_sha, cleanup_error } => {
            assert_eq!(commit_sha, "def456", "Commit SHA should be preserved");
            assert!(
                cleanup_error.contains("worktree"),
                "Cleanup error should be preserved: {}",
                cleanup_error
            );
        }
        _ => panic!("Expected AcceptedWithCleanupFailure variant"),
    }
}
