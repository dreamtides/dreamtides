use std::collections::HashMap;

use llmc::config::{Config, DefaultsConfig, RepoConfig};
use llmc::ipc::messages::HookEvent;
use llmc::patrol::{PatrolReport, build_conflict_prompt, handle_hook_event, handle_stop};
use llmc::state::{State, WorkerRecord, WorkerStatus};

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
        pending_task_prompt_since_unix: None,
        pending_prompt_cmd: None,
        transcript_session_id: None,
        transcript_path: None,
        active_task_id: None,
    }
}

fn create_test_config() -> Config {
    Config {
        defaults: DefaultsConfig {
            model: "opus".to_string(),
            skip_permissions: true,
            allowed_tools: vec![],
            patrol_interval_secs: 60,
            sound_on_review: false,
            self_review: None,
        },
        repo: RepoConfig { source: "/test".to_string(), default_branch: "master".to_string() },
        workers: HashMap::new(),
        auto: None,
        overseer: None,
    }
}

#[test]
fn test_handle_stop_unknown_worker() {
    let mut state = State::new();
    let config = create_test_config();
    let result = handle_stop("unknown_worker", 12345, None, &mut state, &config);
    assert!(result.is_ok());
}

#[test]
fn test_handle_stop_idle_worker_ignored() {
    let mut state = State::new();
    let worker = create_test_worker("adam");
    state.add_worker(worker);
    let config = create_test_config();
    let result = handle_stop("adam", 12345, None, &mut state, &config);
    assert!(result.is_ok());
    assert_eq!(state.get_worker("adam").unwrap().status, WorkerStatus::Idle);
}

#[test]
fn test_patrol_report_default() {
    let report = PatrolReport::default();
    assert_eq!(report.sessions_checked, 0);
    assert!(report.transitions_applied.is_empty());
    assert!(report.rebases_triggered.is_empty());
    assert!(report.errors.is_empty());
}

#[test]
fn test_build_conflict_prompt() {
    let conflicts = vec!["src/main.rs".to_string(), "src/lib.rs".to_string()];
    let original_task = "Fix the authentication bug in the login system";
    let prompt = build_conflict_prompt(&conflicts, original_task);
    assert!(prompt.contains("REBASE CONFLICT DETECTED"));
    assert!(prompt.contains("src/main.rs"));
    assert!(prompt.contains("src/lib.rs"));
    assert!(prompt.contains("IMPORTANT - Resolution steps (must complete ALL steps):"));
    assert!(prompt.contains("Your original task:"));
    assert!(prompt.contains("Fix the authentication bug"));
    assert!(prompt.contains("git rebase --continue"));
    assert!(prompt.contains("Successfully rebased and updated"));
}

#[test]
fn test_session_start_offline_worker_transitions_to_idle() {
    let mut state = State::new();
    let mut worker = create_test_worker("auto-1");
    worker.status = WorkerStatus::Offline;
    state.add_worker(worker);
    let config = create_test_config();

    let event = HookEvent::SessionStart {
        worker: "auto-1".to_string(),
        session_id: "test-session".to_string(),
        timestamp: 1234567890,
        transcript_path: None,
    };

    let result = handle_hook_event(&event, &mut state, &config);
    assert!(result.is_ok(), "SessionStart hook should succeed");
    assert_eq!(
        state.get_worker("auto-1").unwrap().status,
        WorkerStatus::Idle,
        "Offline worker should transition to Idle on SessionStart"
    );
}

#[test]
fn test_session_start_with_pending_prompt_state_changes() {
    let mut state = State::new();
    let mut worker = create_test_worker("auto-1");
    worker.status = WorkerStatus::Idle;
    worker.pending_task_prompt = Some("Test task prompt".to_string());
    worker.pending_prompt_cmd = Some("bd show test-123".to_string());
    state.add_worker(worker);
    let config = create_test_config();

    let event = HookEvent::SessionStart {
        worker: "auto-1".to_string(),
        session_id: "llmc-auto-1".to_string(),
        timestamp: 1234567890,
        transcript_path: None,
    };

    let _result = handle_hook_event(&event, &mut state, &config);

    let worker = state.get_worker("auto-1").unwrap();
    assert!(
        worker.pending_task_prompt.is_none(),
        "pending_task_prompt should be cleared after SessionStart"
    );
    assert!(
        worker.pending_prompt_cmd.is_none(),
        "pending_prompt_cmd should be cleared after SessionStart"
    );
    assert_eq!(
        worker.current_prompt, "Test task prompt",
        "current_prompt should be set from pending_task_prompt"
    );
    assert_eq!(
        worker.prompt_cmd,
        Some("bd show test-123".to_string()),
        "prompt_cmd should be set from pending_prompt_cmd"
    );
    assert_eq!(
        worker.status,
        WorkerStatus::Working,
        "Worker should transition to Working after pending prompt is sent"
    );
}

#[test]
fn test_tmux_session_name_format_differs_from_claude_uuid() {
    let worker_name = "auto-1";
    let tmux_session_name = llmc::config::get_worker_session_name(worker_name);
    let claude_uuid = "a3eade4c-f6f1-4a15-a268-e9494d235cf0";
    assert_ne!(
        tmux_session_name, claude_uuid,
        "TMUX session name should not equal Claude session UUID - TMUX session names are \
         derived from worker names (e.g., 'llmc-auto-1'), not from Claude's session UUIDs"
    );
    assert!(
        tmux_session_name.contains(worker_name),
        "TMUX session name '{}' should contain worker name '{}'",
        tmux_session_name,
        worker_name
    );
    assert!(
        !tmux_session_name.contains('-') || !tmux_session_name.contains("4a15"),
        "TMUX session name should not look like a UUID (no segments like '-4a15-')"
    );
}
