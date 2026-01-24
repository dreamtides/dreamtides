use llmc::auto_mode::auto_failure::{
    AutoFailureKind, HardFailure, TransientFailure, check_for_hard_failures, reset_retry_count,
};
use llmc::state::{State, WorkerRecord, WorkerStatus};

fn create_worker_record(name: &str, status: WorkerStatus) -> WorkerRecord {
    WorkerRecord {
        name: name.to_string(),
        worktree_path: format!("/path/to/{}", name),
        branch: format!("llmc/{}", name),
        status,
        current_prompt: String::new(),
        prompt_cmd: None,
        created_at_unix: 1000,
        last_activity_unix: 1000,
        commit_sha: if status == WorkerStatus::NeedsReview {
            Some("abc123".to_string())
        } else {
            None
        },
        session_id: format!("llmc-{}", name),
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
        pending_task_prompt_since_unix: None,
        pending_clear_retry_count: 0,
        pending_prompt_cmd: None,
        transcript_session_id: None,
        transcript_path: None,
        active_task_id: None,
    }
}

#[test]
fn transient_failure_session_crash_display() {
    let failure = TransientFailure::SessionCrash { worker_name: "auto-1".to_string() };
    let display = format!("{}", failure);
    assert!(display.contains("auto-1"), "Display should mention worker name: {}", display);
    assert!(
        display.to_lowercase().contains("crash") || display.to_lowercase().contains("session"),
        "Display should indicate session crash: {}",
        display
    );
}

#[test]
fn transient_failure_tmux_missing_display() {
    let failure = TransientFailure::TmuxSessionMissing { worker_name: "auto-2".to_string() };
    let display = format!("{}", failure);
    assert!(display.contains("auto-2"), "Display should mention worker name: {}", display);
    assert!(
        display.to_lowercase().contains("tmux") || display.to_lowercase().contains("missing"),
        "Display should indicate TMUX missing: {}",
        display
    );
}

#[test]
fn hard_failure_post_accept_command_failed_display() {
    let failure = HardFailure::PostAcceptCommandFailed { message: "Tests failed".to_string() };
    let display = format!("{}", failure);
    assert!(display.contains("Tests failed"), "Display should include error message: {}", display);
    assert!(
        display.to_lowercase().contains("post-accept")
            || display.to_lowercase().contains("post accept"),
        "Display should mention post accept: {}",
        display
    );
}

#[test]
fn hard_failure_worker_retries_exhausted_display() {
    let failure =
        HardFailure::WorkerRetriesExhausted { worker_name: "auto-3".to_string(), retry_count: 5 };
    let display = format!("{}", failure);
    assert!(display.contains("auto-3"), "Display should mention worker name: {}", display);
    assert!(display.contains("5"), "Display should mention retry count: {}", display);
}

#[test]
fn hard_failure_rebase_failure_display() {
    let failure = HardFailure::RebaseFailure {
        worker_name: "auto-1".to_string(),
        message: "Conflict in main.rs".to_string(),
    };
    let display = format!("{}", failure);
    assert!(display.contains("auto-1"), "Display should mention worker name: {}", display);
    assert!(
        display.contains("Conflict in main.rs"),
        "Display should include rebase error: {}",
        display
    );
}

#[test]
fn hard_failure_state_corruption_display() {
    let failure = HardFailure::StateCorruption { message: "Invalid JSON".to_string() };
    let display = format!("{}", failure);
    assert!(
        display.contains("Invalid JSON"),
        "Display should include corruption error: {}",
        display
    );
}

#[test]
fn hard_failure_hook_ipc_failure_display() {
    let failure = HardFailure::HookIpcFailure { message: "Socket error".to_string() };
    let display = format!("{}", failure);
    assert!(display.contains("Socket error"), "Display should include IPC error: {}", display);
}

#[test]
fn auto_failure_kind_transient_display() {
    let kind = AutoFailureKind::Transient(TransientFailure::SessionCrash {
        worker_name: "auto-1".to_string(),
    });
    let display = format!("{}", kind);
    assert!(
        display.to_lowercase().contains("transient"),
        "Display should indicate transient: {}",
        display
    );
}

#[test]
fn auto_failure_kind_hard_display() {
    let kind =
        AutoFailureKind::Hard(HardFailure::StateCorruption { message: "Corrupt".to_string() });
    let display = format!("{}", kind);
    assert!(
        display.to_lowercase().contains("hard"),
        "Display should indicate hard failure: {}",
        display
    );
}

#[test]
fn check_for_hard_failures_none_when_no_errors() {
    let mut state = State::new();
    state.auto_workers = vec!["auto-1".to_string(), "auto-2".to_string()];
    state.add_worker(create_worker_record("auto-1", WorkerStatus::Working));
    state.add_worker(create_worker_record("auto-2", WorkerStatus::Idle));
    let result = check_for_hard_failures(&state);
    assert!(result.is_none(), "Should return None when no workers are in Error state");
}

#[test]
fn check_for_hard_failures_detects_error_state() {
    let mut state = State::new();
    state.auto_workers = vec!["auto-1".to_string(), "auto-2".to_string()];
    state.add_worker(create_worker_record("auto-1", WorkerStatus::Working));
    let mut error_worker = create_worker_record("auto-2", WorkerStatus::Error);
    error_worker.auto_retry_count = 3;
    error_worker.error_reason = Some("Test error reason".to_string());
    state.add_worker(error_worker);
    let result = check_for_hard_failures(&state);
    assert!(result.is_some(), "Should return Some when worker is in Error state");
    match result.unwrap() {
        HardFailure::WorkerInErrorState { worker_name, error_reason } => {
            assert_eq!(worker_name, "auto-2", "Should identify the error worker");
            assert_eq!(
                error_reason,
                Some("Test error reason".to_string()),
                "Should include error reason"
            );
        }
        other => panic!("Expected WorkerInErrorState, got {:?}", other),
    }
}

#[test]
fn reset_retry_count_clears_count() {
    let mut state = State::new();
    let mut worker = create_worker_record("auto-1", WorkerStatus::Idle);
    worker.auto_retry_count = 5;
    state.add_worker(worker);
    reset_retry_count(&mut state, "auto-1");
    let updated = state.get_worker("auto-1").expect("Worker should exist");
    assert_eq!(updated.auto_retry_count, 0, "Retry count should be reset to 0");
}

#[test]
fn reset_retry_count_noop_when_zero() {
    let mut state = State::new();
    let worker = create_worker_record("auto-1", WorkerStatus::Idle);
    state.add_worker(worker);
    // Should not error when retry count is already 0
    reset_retry_count(&mut state, "auto-1");
    let updated = state.get_worker("auto-1").expect("Worker should exist");
    assert_eq!(updated.auto_retry_count, 0, "Retry count should remain 0");
}

#[test]
fn reset_retry_count_noop_for_missing_worker() {
    let mut state = State::new();
    // Should not error when worker doesn't exist
    reset_retry_count(&mut state, "nonexistent");
}

#[test]
fn transient_failure_equality() {
    let f1 = TransientFailure::SessionCrash { worker_name: "auto-1".to_string() };
    let f2 = TransientFailure::SessionCrash { worker_name: "auto-1".to_string() };
    let f3 = TransientFailure::SessionCrash { worker_name: "auto-2".to_string() };
    let f4 = TransientFailure::TmuxSessionMissing { worker_name: "auto-1".to_string() };
    assert_eq!(f1, f2, "Same failure type and worker should be equal");
    assert_ne!(f1, f3, "Different worker names should not be equal");
    assert_ne!(f1, f4, "Different failure types should not be equal");
}

#[test]
fn hard_failure_equality() {
    let f1 = HardFailure::PostAcceptCommandFailed { message: "error".to_string() };
    let f2 = HardFailure::PostAcceptCommandFailed { message: "error".to_string() };
    let f3 = HardFailure::PostAcceptCommandFailed { message: "different".to_string() };
    let f4 = HardFailure::StateCorruption { message: "error".to_string() };
    assert_eq!(f1, f2, "Same failure type and message should be equal");
    assert_ne!(f1, f3, "Different messages should not be equal");
    assert_ne!(f1, f4, "Different failure types should not be equal");
}
