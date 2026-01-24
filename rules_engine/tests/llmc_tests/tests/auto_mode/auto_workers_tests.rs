use llmc::auto_mode::auto_workers::{
    AUTO_WORKER_PREFIX, auto_worker_name, generate_auto_worker_names, is_auto_worker,
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
        commit_sha: None,
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
fn auto_worker_prefix_is_correct() {
    assert_eq!(AUTO_WORKER_PREFIX, "auto-", "Auto worker prefix should be 'auto-'");
}

#[test]
fn auto_worker_name_format() {
    assert_eq!(auto_worker_name(1), "auto-1", "Worker 1 name");
    assert_eq!(auto_worker_name(2), "auto-2", "Worker 2 name");
    assert_eq!(auto_worker_name(10), "auto-10", "Worker 10 name");
    assert_eq!(auto_worker_name(100), "auto-100", "Worker 100 name");
}

#[test]
fn is_auto_worker_true_for_auto_workers() {
    assert!(is_auto_worker("auto-1"), "auto-1 should be an auto worker");
    assert!(is_auto_worker("auto-2"), "auto-2 should be an auto worker");
    assert!(is_auto_worker("auto-10"), "auto-10 should be an auto worker");
    assert!(is_auto_worker("auto-foo"), "auto-foo should be an auto worker (prefix match)");
}

#[test]
fn is_auto_worker_false_for_non_auto_workers() {
    assert!(!is_auto_worker("worker-1"), "worker-1 should not be an auto worker");
    assert!(!is_auto_worker("main"), "main should not be an auto worker");
    assert!(!is_auto_worker("feature"), "feature should not be an auto worker");
    assert!(!is_auto_worker("auto1"), "auto1 should not be an auto worker (no dash)");
    assert!(!is_auto_worker("xauto-1"), "xauto-1 should not be an auto worker");
    assert!(!is_auto_worker(""), "empty string should not be an auto worker");
}

#[test]
fn generate_auto_worker_names_single() {
    let names = generate_auto_worker_names(1);
    assert_eq!(names, vec!["auto-1"], "Concurrency 1 should produce ['auto-1']");
}

#[test]
fn generate_auto_worker_names_multiple() {
    let names = generate_auto_worker_names(3);
    assert_eq!(
        names,
        vec!["auto-1", "auto-2", "auto-3"],
        "Concurrency 3 should produce ['auto-1', 'auto-2', 'auto-3']"
    );
}

#[test]
fn generate_auto_worker_names_zero() {
    let names = generate_auto_worker_names(0);
    assert!(names.is_empty(), "Concurrency 0 should produce empty list");
}

#[test]
fn get_idle_auto_workers_filters_correctly() {
    let mut state = State::new();
    state.auto_workers = vec!["auto-1".to_string(), "auto-2".to_string(), "auto-3".to_string()];
    state.add_worker(create_worker_record("auto-1", WorkerStatus::Idle));
    state.add_worker(create_worker_record("auto-2", WorkerStatus::Working));
    state.add_worker(create_worker_record("auto-3", WorkerStatus::Idle));
    state.add_worker(create_worker_record("manual-1", WorkerStatus::Idle));
    let idle_auto = llmc::auto_mode::auto_workers::get_idle_auto_workers(&state);
    let names: Vec<&str> = idle_auto.iter().map(|w| w.name.as_str()).collect();
    assert!(names.contains(&"auto-1"), "Should include idle auto-1");
    assert!(names.contains(&"auto-3"), "Should include idle auto-3");
    assert!(!names.contains(&"auto-2"), "Should not include working auto-2");
    assert!(!names.contains(&"manual-1"), "Should not include manual worker");
    assert_eq!(names.len(), 2, "Should have exactly 2 idle auto workers");
}

#[test]
fn set_auto_mode_active() {
    let mut state = State::new();
    assert!(!state.auto_mode, "Auto mode should be false initially");
    assert!(state.auto_workers.is_empty(), "Auto workers should be empty initially");
    let workers = vec!["auto-1".to_string(), "auto-2".to_string()];
    llmc::auto_mode::auto_workers::set_auto_mode_active(&mut state, workers.clone());
    assert!(state.auto_mode, "Auto mode should be true after set");
    assert_eq!(state.auto_workers, workers, "Auto workers should be set");
}

#[test]
fn clear_auto_mode_state() {
    let mut state = State::new();
    state.auto_mode = true;
    state.auto_workers = vec!["auto-1".to_string(), "auto-2".to_string()];
    llmc::auto_mode::auto_workers::clear_auto_mode_state(&mut state);
    assert!(!state.auto_mode, "Auto mode should be false after clear");
    assert!(state.auto_workers.is_empty(), "Auto workers should be empty after clear");
}

#[test]
fn record_task_completion_updates_timestamp() {
    let mut state = State::new();
    assert!(state.last_task_completion_unix.is_none(), "Last completion should be None initially");
    llmc::auto_mode::auto_workers::record_task_completion(&mut state);
    let completion_time =
        state.last_task_completion_unix.expect("Should have completion timestamp");
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time error")
        .as_secs();
    let age = now.saturating_sub(completion_time);
    assert!(age < 5, "Completion timestamp should be within 5 seconds of now, age={}", age);
}

#[test]
fn get_idle_auto_workers_excludes_pending_task_prompt() {
    let mut state = State::new();
    state.auto_workers = vec!["auto-1".to_string(), "auto-2".to_string(), "auto-3".to_string()];

    let mut worker1 = create_worker_record("auto-1", WorkerStatus::Idle);
    worker1.pending_task_prompt = None;
    state.add_worker(worker1);

    let mut worker2 = create_worker_record("auto-2", WorkerStatus::Idle);
    worker2.pending_task_prompt = Some("Pending task".to_string());
    state.add_worker(worker2);

    let mut worker3 = create_worker_record("auto-3", WorkerStatus::Idle);
    worker3.pending_task_prompt = None;
    state.add_worker(worker3);

    let idle_auto = llmc::auto_mode::auto_workers::get_idle_auto_workers(&state);
    let names: Vec<&str> = idle_auto.iter().map(|w| w.name.as_str()).collect();

    assert!(names.contains(&"auto-1"), "Should include idle auto-1 without pending prompt");
    assert!(names.contains(&"auto-3"), "Should include idle auto-3 without pending prompt");
    assert!(
        !names.contains(&"auto-2"),
        "Should NOT include auto-2 with pending_task_prompt - prevents race condition"
    );
    assert_eq!(
        names.len(),
        2,
        "Should have exactly 2 idle auto workers (excluding one with pending prompt)"
    );
}
