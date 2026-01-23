use std::time::{SystemTime, UNIX_EPOCH};

use llmc::state::{State, WorkerRecord, WorkerStatus, get_state_path, validate_state};
use tempfile::TempDir;

fn create_test_worker(name: &str) -> WorkerRecord {
    WorkerRecord {
        name: name.to_string(),
        worktree_path: format!("/tmp/llmc/.worktrees/{}", name),
        branch: format!("llmc/{}", name),
        status: WorkerStatus::Idle,
        current_prompt: String::new(),
        prompt_cmd: None,
        created_at_unix: 1000000000,
        last_activity_unix: 1000000000,
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
    }
}

#[test]
fn test_new_state() {
    let state = State::new();
    assert!(state.workers.is_empty());
}

#[test]
fn test_add_and_get_worker() {
    let mut state = State::new();
    let worker = create_test_worker("adam");
    state.add_worker(worker.clone());
    assert_eq!(state.workers.len(), 1);
    let retrieved = state.get_worker("adam").unwrap();
    assert_eq!(retrieved.name, "adam");
    assert_eq!(retrieved.status, WorkerStatus::Idle);
}

#[test]
fn test_get_worker_mut() {
    let mut state = State::new();
    state.add_worker(create_test_worker("adam"));
    if let Some(worker) = state.get_worker_mut("adam") {
        worker.status = WorkerStatus::Working;
    }
    assert_eq!(state.get_worker("adam").unwrap().status, WorkerStatus::Working);
}

#[test]
fn test_remove_worker() {
    let mut state = State::new();
    state.add_worker(create_test_worker("adam"));
    assert_eq!(state.workers.len(), 1);
    state.remove_worker("adam");
    assert_eq!(state.workers.len(), 0);
    assert!(state.get_worker("adam").is_none());
}

#[test]
fn test_get_idle_workers() {
    let mut state = State::new();
    let mut worker1 = create_test_worker("adam");
    worker1.status = WorkerStatus::Idle;
    state.add_worker(worker1);
    let mut worker2 = create_test_worker("baker");
    worker2.status = WorkerStatus::Working;
    state.add_worker(worker2);
    let mut worker3 = create_test_worker("charlie");
    worker3.status = WorkerStatus::Idle;
    state.add_worker(worker3);
    let idle = state.get_idle_workers();
    assert_eq!(idle.len(), 2);
    let names: Vec<&str> = idle.iter().map(|w| w.name.as_str()).collect();
    assert!(names.contains(&"adam"));
    assert!(names.contains(&"charlie"));
}

#[test]
fn test_save_and_load() {
    let dir = TempDir::new().unwrap();
    let state_path = dir.path().join("state.json");
    let mut state = State::new();
    state.add_worker(create_test_worker("adam"));
    state.save(&state_path).unwrap();
    assert!(state_path.exists());
    let loaded = State::load(&state_path).unwrap();
    assert_eq!(loaded.workers.len(), 1);
    assert!(loaded.get_worker("adam").is_some());
}

#[test]
fn test_load_nonexistent_file() {
    let dir = TempDir::new().unwrap();
    let state_path = dir.path().join("state.json");
    let state = State::load(&state_path).unwrap();
    assert!(state.workers.is_empty());
}

#[test]
fn test_atomic_write_creates_backup() {
    let dir = TempDir::new().unwrap();
    let state_path = dir.path().join("state.json");
    let backup_path = dir.path().join("state.json.bak");
    let mut state = State::new();
    state.add_worker(create_test_worker("adam"));
    state.save(&state_path).unwrap();
    assert!(state_path.exists());
    assert!(!backup_path.exists());
    let mut state2 = State::new();
    state2.add_worker(create_test_worker("baker"));
    state2.save(&state_path).unwrap();
    assert!(state_path.exists());
    assert!(backup_path.exists());
    let backup_state = State::load(&backup_path).unwrap();
    assert!(backup_state.get_worker("adam").is_some());
    assert!(backup_state.get_worker("baker").is_none());
}

#[test]
fn test_validate_duplicate_names() {
    let mut state = State::new();
    let worker = create_test_worker("adam");
    state.workers.insert("adam".to_string(), worker.clone());
    state.workers.insert("adam_duplicate".to_string(), worker);
    let result = validate_state(&state);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Duplicate worker name"));
}

#[test]
fn test_validate_needs_review_requires_commit_sha() {
    let mut state = State::new();
    let mut worker = create_test_worker("adam");
    worker.status = WorkerStatus::NeedsReview;
    worker.commit_sha = None;
    state.add_worker(worker);
    let result = validate_state(&state);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("needs_review but no commit_sha"));
}

#[test]
fn test_validate_future_timestamps() {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let future = now + 10000;
    let mut state = State::new();
    let mut worker = create_test_worker("adam");
    worker.created_at_unix = future;
    state.add_worker(worker);
    let result = validate_state(&state);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("timestamp in the future"));
}

#[test]
fn test_worker_status_serialization() {
    let statuses = vec![
        (WorkerStatus::Idle, "\"idle\""),
        (WorkerStatus::Working, "\"working\""),
        (WorkerStatus::NeedsReview, "\"needs_review\""),
        (WorkerStatus::Reviewing, "\"reviewing\""),
        (WorkerStatus::Rejected, "\"rejected\""),
        (WorkerStatus::Rebasing, "\"rebasing\""),
        (WorkerStatus::Error, "\"error\""),
        (WorkerStatus::Offline, "\"offline\""),
    ];
    for (status, expected_json) in statuses {
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, expected_json);
        let deserialized: WorkerStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, status);
    }
}

#[test]
fn test_worker_record_serialization() {
    let worker = create_test_worker("adam");
    let json = serde_json::to_string(&worker).unwrap();
    let deserialized: WorkerRecord = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, worker.name);
    assert_eq!(deserialized.worktree_path, worker.worktree_path);
    assert_eq!(deserialized.branch, worker.branch);
    assert_eq!(deserialized.status, worker.status);
}

#[test]
fn test_get_state_path() {
    let path = get_state_path();
    assert!(path.ends_with("llmc/state.json"));
}

#[test]
fn test_key_name_mismatch() {
    let mut state = State::new();
    let worker = create_test_worker("adam");
    state.workers.insert("wrong_key".to_string(), worker);
    let result = validate_state(&state);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("doesn't match worker name"));
}
