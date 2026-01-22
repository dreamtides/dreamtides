use std::fs;
use std::time::Duration;

use llmc::auto_mode::auto_logging::{
    AutoEvent, AutoLogEntry, AutoLogger, CommandResult, LogLevel, PostAcceptLogEntry,
    TaskPoolLogEntry, TaskResult,
};
use llmc::config::LlmcPaths;
use tempfile::TempDir;

#[test]
fn log_level_info_serializes_to_uppercase() {
    let entry = AutoLogEntry {
        timestamp: "2024-01-01T00:00:00.000Z".to_string(),
        level: LogLevel::Info,
        event: AutoEvent::DaemonStartup { instance_id: "test".to_string(), concurrency: 1 },
    };
    let json = serde_json::to_string(&entry).expect("Serialization should succeed");
    assert!(json.contains("\"INFO\""), "LogLevel::Info should serialize as INFO");
}

#[test]
fn log_level_error_serializes_to_uppercase() {
    let entry = AutoLogEntry {
        timestamp: "2024-01-01T00:00:00.000Z".to_string(),
        level: LogLevel::Error,
        event: AutoEvent::Error { context: "test".to_string(), error: "test error".to_string() },
    };
    let json = serde_json::to_string(&entry).expect("Serialization should succeed");
    assert!(json.contains("\"ERROR\""), "LogLevel::Error should serialize as ERROR");
}

#[test]
fn task_result_needs_review_serializes_to_snake_case() {
    let entry = AutoLogEntry {
        timestamp: "2024-01-01T00:00:00.000Z".to_string(),
        level: LogLevel::Info,
        event: AutoEvent::TaskCompleted {
            worker_name: "auto-1".to_string(),
            result: TaskResult::NeedsReview,
        },
    };
    let json = serde_json::to_string(&entry).expect("Serialization should succeed");
    assert!(
        json.contains("needs_review"),
        "TaskResult::NeedsReview should serialize to snake_case"
    );
}

#[test]
fn task_result_no_changes_serializes_to_snake_case() {
    let entry = AutoLogEntry {
        timestamp: "2024-01-01T00:00:00.000Z".to_string(),
        level: LogLevel::Info,
        event: AutoEvent::TaskCompleted {
            worker_name: "auto-1".to_string(),
            result: TaskResult::NoChanges,
        },
    };
    let json = serde_json::to_string(&entry).expect("Serialization should succeed");
    assert!(json.contains("no_changes"), "TaskResult::NoChanges should serialize to snake_case");
}

#[test]
fn auto_event_daemon_startup_serializes() {
    let entry = AutoLogEntry {
        timestamp: "2024-01-01T00:00:00.000Z".to_string(),
        level: LogLevel::Info,
        event: AutoEvent::DaemonStartup { instance_id: "abc123".to_string(), concurrency: 4 },
    };
    let json = serde_json::to_string(&entry).expect("Serialization should succeed");
    assert!(json.contains("daemon_startup"), "Event type should be serialized");
    assert!(json.contains("abc123"), "Should contain instance_id");
    assert!(json.contains("4"), "Should contain concurrency");
}

#[test]
fn auto_event_daemon_shutdown_serializes() {
    let entry = AutoLogEntry {
        timestamp: "2024-01-01T00:00:00.000Z".to_string(),
        level: LogLevel::Info,
        event: AutoEvent::DaemonShutdown {
            instance_id: "abc123".to_string(),
            reason: "Normal shutdown".to_string(),
        },
    };
    let json = serde_json::to_string(&entry).expect("Serialization should succeed");
    assert!(json.contains("daemon_shutdown"), "Event type should be serialized");
    assert!(json.contains("Normal shutdown"), "Should contain reason");
}

#[test]
fn auto_event_task_assigned_serializes() {
    let entry = AutoLogEntry {
        timestamp: "2024-01-01T00:00:00.000Z".to_string(),
        level: LogLevel::Info,
        event: AutoEvent::TaskAssigned {
            worker_name: "auto-1".to_string(),
            task_excerpt: "Fix the bug in...".to_string(),
        },
    };
    let json = serde_json::to_string(&entry).expect("Serialization should succeed");
    assert!(json.contains("task_assigned"), "Event type should be serialized");
    assert!(json.contains("auto-1"), "Should contain worker_name");
    assert!(json.contains("task_excerpt"), "Should contain task_excerpt field");
}

#[test]
fn auto_event_accept_success_serializes() {
    let entry = AutoLogEntry {
        timestamp: "2024-01-01T00:00:00.000Z".to_string(),
        level: LogLevel::Info,
        event: AutoEvent::AcceptSuccess {
            worker_name: "auto-1".to_string(),
            commit_sha: "abc123def456".to_string(),
        },
    };
    let json = serde_json::to_string(&entry).expect("Serialization should succeed");
    assert!(json.contains("accept_success"), "Event type should be serialized");
    assert!(json.contains("abc123def456"), "Should contain commit_sha");
}

#[test]
fn auto_event_accept_failure_serializes() {
    let entry = AutoLogEntry {
        timestamp: "2024-01-01T00:00:00.000Z".to_string(),
        level: LogLevel::Error,
        event: AutoEvent::AcceptFailure {
            worker_name: "auto-1".to_string(),
            error: "Rebase conflict".to_string(),
        },
    };
    let json = serde_json::to_string(&entry).expect("Serialization should succeed");
    assert!(json.contains("accept_failure"), "Event type should be serialized");
    assert!(json.contains("Rebase conflict"), "Should contain error");
}

#[test]
fn auto_event_error_serializes() {
    let entry = AutoLogEntry {
        timestamp: "2024-01-01T00:00:00.000Z".to_string(),
        level: LogLevel::Error,
        event: AutoEvent::Error {
            context: "process_idle_workers".to_string(),
            error: "Task pool command failed".to_string(),
        },
    };
    let json = serde_json::to_string(&entry).expect("Serialization should succeed");
    assert!(json.contains("\"error\""), "Event type should be serialized");
    assert!(json.contains("process_idle_workers"), "Should contain context");
}

#[test]
fn task_pool_log_entry_serialization() {
    let entry = TaskPoolLogEntry {
        timestamp: "2024-01-01T00:00:00.000Z".to_string(),
        level: LogLevel::Info,
        command: "lat dispatch --limit 1".to_string(),
        exit_code: 0,
        duration_ms: 150,
        stdout: "LXXWQN: test task".to_string(),
        stderr: "".to_string(),
    };
    let json = serde_json::to_string(&entry).expect("Serialization should succeed");
    assert!(json.contains("lat dispatch"), "Should contain command");
    assert!(json.contains("LXXWQN"), "Should contain task ID from stdout");
    assert!(json.contains("150"), "Should contain duration_ms");
}

#[test]
fn task_pool_log_entry_with_error() {
    let entry = TaskPoolLogEntry {
        timestamp: "2024-01-01T00:00:00.000Z".to_string(),
        level: LogLevel::Error,
        command: "lat dispatch --limit 1".to_string(),
        exit_code: 1,
        duration_ms: 50,
        stdout: "".to_string(),
        stderr: "Database connection failed".to_string(),
    };
    let json = serde_json::to_string(&entry).expect("Serialization should succeed");
    assert!(json.contains("\"ERROR\""), "Should have error level");
    assert!(json.contains("Database connection failed"), "Should contain stderr");
}

#[test]
fn post_accept_log_entry_serialization() {
    let entry = PostAcceptLogEntry {
        timestamp: "2024-01-01T00:00:00.000Z".to_string(),
        level: LogLevel::Info,
        worker_name: "auto-1".to_string(),
        commit_sha: "abc123def456".to_string(),
        command: "just review".to_string(),
        exit_code: 0,
        duration_ms: 5000,
        stdout: "All checks passed".to_string(),
        stderr: "".to_string(),
    };
    let json = serde_json::to_string(&entry).expect("Serialization should succeed");
    assert!(json.contains("abc123def456"), "Should contain commit SHA");
    assert!(json.contains("5000"), "Should contain duration_ms");
    assert!(json.contains("auto-1"), "Should contain worker_name");
}

#[test]
fn post_accept_log_entry_with_failure() {
    let entry = PostAcceptLogEntry {
        timestamp: "2024-01-01T00:00:00.000Z".to_string(),
        level: LogLevel::Error,
        worker_name: "auto-2".to_string(),
        commit_sha: "xyz789".to_string(),
        command: "just review".to_string(),
        exit_code: 1,
        duration_ms: 2000,
        stdout: "".to_string(),
        stderr: "Test failed".to_string(),
    };
    let json = serde_json::to_string(&entry).expect("Serialization should succeed");
    assert!(json.contains("\"ERROR\""), "Should have error level");
    assert!(json.contains("Test failed"), "Should contain stderr");
}

#[test]
fn command_result_stores_all_fields() {
    let result = CommandResult {
        command: "echo hello".to_string(),
        exit_code: 0,
        duration: Duration::from_millis(100),
        stdout: "hello".to_string(),
        stderr: "".to_string(),
    };
    assert_eq!(result.command, "echo hello", "Command should be stored");
    assert_eq!(result.exit_code, 0, "Exit code should be stored");
    assert_eq!(result.duration.as_millis(), 100, "Duration should be stored");
    assert_eq!(result.stdout, "hello", "Stdout should be stored");
    assert_eq!(result.stderr, "", "Stderr should be stored");
}

#[test]
fn task_pool_log_writes_to_disk_immediately() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let paths = LlmcPaths::new(temp_dir.path().to_path_buf());
    let logger = AutoLogger::new_with_paths(&paths).expect("Failed to create logger");
    let log_path = paths.task_pool_log_path();
    let cmd_result = CommandResult {
        command: "test command".to_string(),
        exit_code: 0,
        duration: Duration::from_millis(100),
        stdout: "test output".to_string(),
        stderr: "".to_string(),
    };
    logger.log_task_pool(&cmd_result);
    let content = fs::read_to_string(&log_path).unwrap_or_else(|e| {
        panic!(
            "Failed to read {}: {}. Log entries should be written immediately.",
            log_path.display(),
            e
        )
    });
    assert!(
        content.contains("test command"),
        "task_pool.log should contain the logged command immediately after log call. File content: '{}'",
        content
    );
    assert!(
        content.contains("test output"),
        "task_pool.log should contain stdout from logged entry. File content: '{}'",
        content
    );
}

#[test]
fn post_accept_log_writes_to_disk_immediately() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let paths = LlmcPaths::new(temp_dir.path().to_path_buf());
    let logger = AutoLogger::new_with_paths(&paths).expect("Failed to create logger");
    let log_path = paths.post_accept_log_path();
    let cmd_result = CommandResult {
        command: "just review".to_string(),
        exit_code: 0,
        duration: Duration::from_millis(5000),
        stdout: "All tests passed".to_string(),
        stderr: "".to_string(),
    };
    logger.log_post_accept("auto-1", "abc123def", &cmd_result);
    let content = fs::read_to_string(&log_path).unwrap_or_else(|e| {
        panic!(
            "Failed to read {}: {}. Log entries should be written immediately.",
            log_path.display(),
            e
        )
    });
    assert!(
        content.contains("just review"),
        "post_accept.log should contain the logged command immediately after log call. File content: '{}'",
        content
    );
    assert!(
        content.contains("abc123def"),
        "post_accept.log should contain commit SHA from logged entry. File content: '{}'",
        content
    );
    assert!(
        content.contains("All tests passed"),
        "post_accept.log should contain stdout from logged entry. File content: '{}'",
        content
    );
}
