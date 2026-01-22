use std::collections::HashMap;

use llmc::config::{Config, DefaultsConfig, RepoConfig};
use llmc::patrol::{PatrolReport, build_conflict_prompt, handle_stop};
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
        repo: RepoConfig { source: "/test".to_string() },
        workers: HashMap::new(),
        auto: None,
        overseer: None,
    }
}

#[test]
fn test_handle_stop_unknown_worker() {
    let mut state = State::new();
    let config = create_test_config();
    let result = handle_stop("unknown_worker", 12345, &mut state, &config);
    assert!(result.is_ok());
}

#[test]
fn test_handle_stop_idle_worker_ignored() {
    let mut state = State::new();
    let worker = create_test_worker("adam");
    state.add_worker(worker);
    let config = create_test_config();
    let result = handle_stop("adam", 12345, &mut state, &config);
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
