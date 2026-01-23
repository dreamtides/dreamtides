use llmc::state::{WorkerRecord, WorkerStatus};
use llmc::worker::{WorkerTransition, apply_transition, can_transition};

#[test]
fn test_can_transition_idle_to_working() {
    assert!(can_transition(&WorkerStatus::Idle, &WorkerStatus::Working));
}

#[test]
fn test_can_transition_working_to_needs_review() {
    assert!(can_transition(&WorkerStatus::Working, &WorkerStatus::NeedsReview));
}

#[test]
fn test_can_transition_needs_review_to_idle() {
    assert!(can_transition(&WorkerStatus::NeedsReview, &WorkerStatus::Idle));
}

#[test]
fn test_can_transition_needs_review_to_rejected() {
    assert!(can_transition(&WorkerStatus::NeedsReview, &WorkerStatus::Rejected));
}

#[test]
fn test_can_transition_rejected_to_needs_review() {
    assert!(can_transition(&WorkerStatus::Rejected, &WorkerStatus::NeedsReview));
}

#[test]
fn test_can_transition_rejected_to_idle() {
    assert!(can_transition(&WorkerStatus::Rejected, &WorkerStatus::Idle));
}

#[test]
fn test_can_transition_any_to_rebasing() {
    assert!(can_transition(&WorkerStatus::Idle, &WorkerStatus::Rebasing));
    assert!(can_transition(&WorkerStatus::Working, &WorkerStatus::Rebasing));
    assert!(can_transition(&WorkerStatus::NeedsReview, &WorkerStatus::Rebasing));
}

#[test]
fn test_can_transition_any_to_error() {
    assert!(can_transition(&WorkerStatus::Idle, &WorkerStatus::Error));
    assert!(can_transition(&WorkerStatus::Working, &WorkerStatus::Error));
    assert!(can_transition(&WorkerStatus::Rebasing, &WorkerStatus::Error));
}

#[test]
fn test_can_transition_any_to_offline() {
    assert!(can_transition(&WorkerStatus::Idle, &WorkerStatus::Offline));
    assert!(can_transition(&WorkerStatus::Working, &WorkerStatus::Offline));
    assert!(can_transition(&WorkerStatus::Error, &WorkerStatus::Offline));
}

#[test]
fn test_can_transition_idle_to_needs_review() {
    assert!(can_transition(&WorkerStatus::Idle, &WorkerStatus::NeedsReview));
}

#[test]
fn test_cannot_transition_idle_to_rejected() {
    assert!(!can_transition(&WorkerStatus::Idle, &WorkerStatus::Rejected));
}

#[test]
fn test_can_transition_working_to_idle() {
    assert!(can_transition(&WorkerStatus::Working, &WorkerStatus::Idle));
}

#[test]
fn test_apply_transition_to_idle_clears_state() {
    let mut worker = WorkerRecord {
        name: "test".to_string(),
        worktree_path: "/tmp/test".to_string(),
        branch: "llmc/test".to_string(),
        status: WorkerStatus::NeedsReview,
        current_prompt: "Some prompt".to_string(),
        prompt_cmd: None,
        created_at_unix: 1000000000,
        last_activity_unix: 1000000000,
        commit_sha: Some("abc123".to_string()),
        session_id: "llmc-test".to_string(),
        crash_count: 0,
        last_crash_unix: None,
        on_complete_sent_unix: None,
        self_review: false,
        pending_self_review: false,
        commits_first_detected_unix: None,
        pending_rebase_prompt: false,
        error_reason: None,
        auto_retry_count: 0,
        api_error_count: 0,
        last_api_error_unix: None,
        pending_task_prompt: None,
        transcript_session_id: None,
        transcript_path: None,
        active_task_id: None,
    };
    apply_transition(&mut worker, WorkerTransition::ToIdle).unwrap();
    assert_eq!(worker.status, WorkerStatus::Idle);
    assert_eq!(worker.current_prompt, "");
    assert_eq!(worker.commit_sha, None);
}

#[test]
fn test_apply_transition_to_working_sets_prompt() {
    let mut worker = WorkerRecord {
        name: "test".to_string(),
        worktree_path: "/tmp/test".to_string(),
        branch: "llmc/test".to_string(),
        status: WorkerStatus::Idle,
        current_prompt: String::new(),
        prompt_cmd: None,
        created_at_unix: 1000000000,
        last_activity_unix: 1000000000,
        commit_sha: None,
        session_id: "llmc-test".to_string(),
        crash_count: 0,
        last_crash_unix: None,
        on_complete_sent_unix: None,
        self_review: false,
        pending_self_review: false,
        commits_first_detected_unix: None,
        pending_rebase_prompt: false,
        error_reason: None,
        auto_retry_count: 0,
        api_error_count: 0,
        last_api_error_unix: None,
        pending_task_prompt: None,
        transcript_session_id: None,
        transcript_path: None,
        active_task_id: None,
    };
    apply_transition(&mut worker, WorkerTransition::ToWorking {
        prompt: "Test prompt".to_string(),
        prompt_cmd: None,
    })
    .unwrap();
    assert_eq!(worker.status, WorkerStatus::Working);
    assert_eq!(worker.current_prompt, "Test prompt");
}

#[test]
fn test_apply_transition_to_needs_review_sets_commit_sha() {
    let mut worker = WorkerRecord {
        name: "test".to_string(),
        worktree_path: "/tmp/test".to_string(),
        branch: "llmc/test".to_string(),
        status: WorkerStatus::Working,
        current_prompt: "Test prompt".to_string(),
        prompt_cmd: None,
        created_at_unix: 1000000000,
        last_activity_unix: 1000000000,
        commit_sha: None,
        session_id: "llmc-test".to_string(),
        crash_count: 0,
        last_crash_unix: None,
        on_complete_sent_unix: None,
        self_review: false,
        pending_self_review: false,
        commits_first_detected_unix: None,
        pending_rebase_prompt: false,
        error_reason: None,
        auto_retry_count: 0,
        api_error_count: 0,
        last_api_error_unix: None,
        pending_task_prompt: None,
        transcript_session_id: None,
        transcript_path: None,
        active_task_id: None,
    };
    apply_transition(&mut worker, WorkerTransition::ToNeedsReview {
        commit_sha: "abc123".to_string(),
    })
    .unwrap();
    assert_eq!(worker.status, WorkerStatus::NeedsReview);
    assert_eq!(worker.commit_sha, Some("abc123".to_string()));
}

#[test]
fn test_apply_transition_idle_to_needs_review() {
    let mut worker = WorkerRecord {
        name: "test".to_string(),
        worktree_path: "/tmp/test".to_string(),
        branch: "llmc/test".to_string(),
        status: WorkerStatus::Idle,
        current_prompt: String::new(),
        prompt_cmd: None,
        created_at_unix: 1000000000,
        last_activity_unix: 1000000000,
        commit_sha: None,
        session_id: "llmc-test".to_string(),
        crash_count: 0,
        last_crash_unix: None,
        on_complete_sent_unix: None,
        self_review: false,
        pending_self_review: false,
        commits_first_detected_unix: None,
        pending_rebase_prompt: false,
        error_reason: None,
        auto_retry_count: 0,
        api_error_count: 0,
        last_api_error_unix: None,
        pending_task_prompt: None,
        transcript_session_id: None,
        transcript_path: None,
        active_task_id: None,
    };
    apply_transition(&mut worker, WorkerTransition::ToNeedsReview {
        commit_sha: "abc123".to_string(),
    })
    .unwrap();
    assert_eq!(worker.status, WorkerStatus::NeedsReview);
    assert_eq!(worker.commit_sha, Some("abc123".to_string()));
}

#[test]
fn test_apply_transition_none_does_nothing() {
    let mut worker = WorkerRecord {
        name: "test".to_string(),
        worktree_path: "/tmp/test".to_string(),
        branch: "llmc/test".to_string(),
        status: WorkerStatus::Working,
        current_prompt: "Test".to_string(),
        prompt_cmd: None,
        created_at_unix: 1000000000,
        last_activity_unix: 1000000000,
        commit_sha: None,
        session_id: "llmc-test".to_string(),
        crash_count: 0,
        last_crash_unix: None,
        on_complete_sent_unix: None,
        self_review: false,
        pending_self_review: false,
        commits_first_detected_unix: None,
        pending_rebase_prompt: false,
        error_reason: None,
        auto_retry_count: 0,
        api_error_count: 0,
        last_api_error_unix: None,
        pending_task_prompt: None,
        transcript_session_id: None,
        transcript_path: None,
        active_task_id: None,
    };
    let old_status = worker.status;
    apply_transition(&mut worker, WorkerTransition::None).unwrap();
    assert_eq!(worker.status, old_status);
}

#[test]
fn test_can_transition_needs_review_to_reviewing() {
    assert!(can_transition(&WorkerStatus::NeedsReview, &WorkerStatus::Reviewing));
}

#[test]
fn test_can_transition_reviewing_to_needs_review() {
    assert!(can_transition(&WorkerStatus::Reviewing, &WorkerStatus::NeedsReview));
}

#[test]
fn test_can_transition_reviewing_to_idle() {
    assert!(can_transition(&WorkerStatus::Reviewing, &WorkerStatus::Idle));
}

#[test]
fn test_can_transition_reviewing_to_rejected() {
    assert!(can_transition(&WorkerStatus::Reviewing, &WorkerStatus::Rejected));
}

#[test]
fn test_apply_transition_to_reviewing() {
    let mut worker = WorkerRecord {
        name: "test".to_string(),
        worktree_path: "/tmp/test".to_string(),
        branch: "llmc/test".to_string(),
        status: WorkerStatus::NeedsReview,
        current_prompt: "Test prompt".to_string(),
        prompt_cmd: None,
        created_at_unix: 1000000000,
        last_activity_unix: 1000000000,
        commit_sha: Some("abc123".to_string()),
        session_id: "llmc-test".to_string(),
        crash_count: 0,
        last_crash_unix: None,
        on_complete_sent_unix: None,
        self_review: false,
        pending_self_review: false,
        commits_first_detected_unix: None,
        pending_rebase_prompt: false,
        error_reason: None,
        auto_retry_count: 0,
        api_error_count: 0,
        last_api_error_unix: None,
        pending_task_prompt: None,
        transcript_session_id: None,
        transcript_path: None,
        active_task_id: None,
    };
    apply_transition(&mut worker, WorkerTransition::ToReviewing).unwrap();
    assert_eq!(worker.status, WorkerStatus::Reviewing);
    assert_eq!(worker.commit_sha, Some("abc123".to_string()));
}
