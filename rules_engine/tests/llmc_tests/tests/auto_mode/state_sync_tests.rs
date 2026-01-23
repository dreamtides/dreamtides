use std::sync::Arc;
/// Tests for daemon state synchronization with state.json changes from external
/// commands.
///
/// These tests verify that when state.json is modified by external commands
/// (like `llmc reset`), the daemon correctly picks up those changes on
/// subsequent iterations.
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use llmc::lock::StateLock;
use llmc::state::{State, WorkerRecord, WorkerStatus};
use tempfile::TempDir;

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
    }
}

#[test]
fn test_state_changes_visible_after_reload() {
    let dir = TempDir::new().unwrap();
    let state_path = dir.path().join("state.json");

    // Initial state: worker is Working
    let mut state = State::new();
    state.add_worker(create_worker_record("auto-1", WorkerStatus::Working));
    state.save(&state_path).unwrap();

    // Verify initial state
    let loaded = State::load(&state_path).unwrap();
    assert_eq!(
        loaded.get_worker("auto-1").unwrap().status,
        WorkerStatus::Working,
        "Initial state should show Working"
    );

    // External command modifies state to Offline (simulating reset)
    let mut modified_state = State::load(&state_path).unwrap();
    modified_state.get_worker_mut("auto-1").unwrap().status = WorkerStatus::Offline;
    modified_state.save(&state_path).unwrap();

    // Reload state - should see the change
    let reloaded = State::load(&state_path).unwrap();
    assert_eq!(
        reloaded.get_worker("auto-1").unwrap().status,
        WorkerStatus::Offline,
        "Reloaded state should show Offline after external modification"
    );
}

#[test]
fn test_concurrent_state_modification_with_lock() {
    let dir = TempDir::new().unwrap();
    let state_path = dir.path().join("state.json");
    let lock_path = dir.path().join("state.lock");

    // Initial state
    let mut state = State::new();
    state.add_worker(create_worker_record("auto-1", WorkerStatus::Working));
    state.save(&state_path).unwrap();

    // Thread 1 (simulating daemon): acquire lock, load state, hold for a bit
    let state_path_clone = state_path.clone();
    let lock_path_clone = lock_path.clone();
    let daemon_started = Arc::new(AtomicBool::new(false));
    let daemon_started_clone = daemon_started.clone();
    let daemon_released_lock = Arc::new(AtomicBool::new(false));
    let daemon_released_clone = daemon_released_lock.clone();

    let daemon_thread = thread::spawn(move || {
        let _lock = StateLock::try_acquire_lock(&lock_path_clone).unwrap();
        let state = State::load(&state_path_clone).unwrap();
        daemon_started_clone.store(true, Ordering::SeqCst);

        // Hold the lock for a short time (simulating processing)
        thread::sleep(Duration::from_millis(100));

        // The daemon has the old state (Working)
        assert_eq!(
            state.get_worker("auto-1").unwrap().status,
            WorkerStatus::Working,
            "Daemon should see Working state"
        );

        // Release lock by dropping _lock
        daemon_released_clone.store(true, Ordering::SeqCst);
    });

    // Wait for daemon to start
    while !daemon_started.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(10));
    }

    // Thread 2 (simulating reset): try to acquire lock - should fail while daemon
    // holds it
    let lock_result = StateLock::try_acquire_lock(&lock_path);
    assert!(lock_result.is_err(), "Should not be able to acquire lock while daemon holds it");

    // Wait for daemon to release lock
    daemon_thread.join().unwrap();
    assert!(daemon_released_lock.load(Ordering::SeqCst), "Daemon should have released lock");

    // Now reset can acquire lock and modify state
    {
        let _lock = StateLock::try_acquire_lock(&lock_path).unwrap();
        let mut state = State::load(&state_path).unwrap();
        state.get_worker_mut("auto-1").unwrap().status = WorkerStatus::Offline;
        state.save(&state_path).unwrap();
    }

    // Daemon's next iteration should see the change
    {
        let _lock = StateLock::try_acquire_lock(&lock_path).unwrap();
        let state = State::load(&state_path).unwrap();
        assert_eq!(
            state.get_worker("auto-1").unwrap().status,
            WorkerStatus::Offline,
            "Daemon's next iteration should see Offline after reset"
        );
    }
}

#[test]
fn test_stale_state_overwrite_scenario() {
    let dir = TempDir::new().unwrap();
    let state_path = dir.path().join("state.json");

    // Initial state: worker is Working
    let mut state = State::new();
    state.add_worker(create_worker_record("auto-1", WorkerStatus::Working));
    state.save(&state_path).unwrap();

    // Daemon loads state into memory
    let daemon_state = State::load(&state_path).unwrap();
    assert_eq!(
        daemon_state.get_worker("auto-1").unwrap().status,
        WorkerStatus::Working,
        "Daemon sees Working"
    );

    // External command modifies state (this would require acquiring lock in real
    // scenario)
    let mut reset_state = State::load(&state_path).unwrap();
    reset_state.get_worker_mut("auto-1").unwrap().status = WorkerStatus::Offline;
    reset_state.save(&state_path).unwrap();

    // If daemon saves its stale in-memory state, it overwrites the reset changes
    // This demonstrates the race condition if the lock isn't properly managed
    daemon_state.save(&state_path).unwrap();

    let final_state = State::load(&state_path).unwrap();
    assert_eq!(
        final_state.get_worker("auto-1").unwrap().status,
        WorkerStatus::Working,
        "Stale daemon state overwrote reset's changes - this demonstrates the race condition"
    );
}

#[test]
fn test_proper_state_reload_prevents_overwrite() {
    let dir = TempDir::new().unwrap();
    let state_path = dir.path().join("state.json");
    let lock_path = dir.path().join("state.lock");

    // Initial state: worker is Working
    let mut state = State::new();
    state.add_worker(create_worker_record("auto-1", WorkerStatus::Working));
    state.save(&state_path).unwrap();

    // Daemon iteration 1: acquire lock, load, save, release
    {
        let _lock = StateLock::try_acquire_lock(&lock_path).unwrap();
        let state = State::load(&state_path).unwrap();
        assert_eq!(state.get_worker("auto-1").unwrap().status, WorkerStatus::Working);
        state.save(&state_path).unwrap();
    }

    // Reset runs: acquire lock, modify, save, release
    {
        let _lock = StateLock::try_acquire_lock(&lock_path).unwrap();
        let mut state = State::load(&state_path).unwrap();
        state.get_worker_mut("auto-1").unwrap().status = WorkerStatus::Offline;
        state.save(&state_path).unwrap();
    }

    // Daemon iteration 2: acquire lock, load (sees change!), process, save
    {
        let _lock = StateLock::try_acquire_lock(&lock_path).unwrap();
        let state = State::load(&state_path).unwrap();
        assert_eq!(
            state.get_worker("auto-1").unwrap().status,
            WorkerStatus::Offline,
            "Daemon iteration 2 should see reset's Offline state"
        );
        // Daemon can now process based on the new state
    }
}
