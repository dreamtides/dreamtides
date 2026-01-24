use std::fs;
use std::io::Write;

use llmc::auto_mode::heartbeat_thread::DaemonRegistration;
use llmc::overseer_mode::health_monitor::{ExpectedDaemon, HealthStatus, LogTailer};
use tempfile::TempDir;

#[test]
fn healthy_status_returns_true_for_is_healthy() {
    let status = HealthStatus::Healthy;

    assert!(status.is_healthy(), "Healthy status should return true for is_healthy()");
}

#[test]
fn non_healthy_statuses_return_false_for_is_healthy() {
    let statuses = [
        HealthStatus::ProcessGone,
        HealthStatus::HeartbeatStale { age_secs: 60 },
        HealthStatus::LogError { message: "test error".to_string() },
        HealthStatus::Stalled { stall_secs: 3600 },
        HealthStatus::IdentityMismatch { reason: "PID changed".to_string() },
    ];

    for status in statuses {
        assert!(!status.is_healthy(), "Status {:?} should not be healthy", status);
    }
}

#[test]
fn healthy_status_describes_as_healthy() {
    let status = HealthStatus::Healthy;

    assert_eq!(status.describe(), "Daemon is healthy", "Healthy status should describe as healthy");
}

#[test]
fn process_gone_status_describes_correctly() {
    let status = HealthStatus::ProcessGone;

    assert_eq!(
        status.describe(),
        "Daemon process is no longer running",
        "ProcessGone should describe that process is gone"
    );
}

#[test]
fn heartbeat_stale_status_includes_age_in_description() {
    let status = HealthStatus::HeartbeatStale { age_secs: 45 };
    let description = status.describe();

    assert!(
        description.contains("45"),
        "HeartbeatStale description should include age, got: {}",
        description
    );
    assert!(
        description.contains("stale"),
        "HeartbeatStale description should mention 'stale', got: {}",
        description
    );
}

#[test]
fn log_error_status_includes_message_in_description() {
    let status = HealthStatus::LogError { message: "PANIC: unexpected error".to_string() };
    let description = status.describe();

    assert!(
        description.contains("PANIC: unexpected error"),
        "LogError description should include the error message, got: {}",
        description
    );
}

#[test]
fn stalled_status_includes_duration_in_description() {
    let status = HealthStatus::Stalled { stall_secs: 7200 };
    let description = status.describe();

    assert!(
        description.contains("7200"),
        "Stalled description should include stall duration, got: {}",
        description
    );
}

#[test]
fn identity_mismatch_includes_reason_in_description() {
    let status =
        HealthStatus::IdentityMismatch { reason: "PID changed from 123 to 456".to_string() };
    let description = status.describe();

    assert!(
        description.contains("PID changed from 123 to 456"),
        "IdentityMismatch description should include reason, got: {}",
        description
    );
}

#[test]
fn expected_daemon_from_registration_copies_all_fields() {
    let registration = DaemonRegistration {
        pid: 12345,
        start_time_unix: 1700000000,
        instance_id: "test-instance-uuid".to_string(),
        log_file: "/var/log/daemon.log".to_string(),
    };

    let expected = ExpectedDaemon::from_registration(&registration);

    assert_eq!(expected.pid, 12345, "PID should be copied from registration");
    assert_eq!(expected.start_time_unix, 1700000000, "Start time should be copied");
    assert_eq!(expected.instance_id, "test-instance-uuid", "Instance ID should be copied");
}

#[test]
fn log_tailer_returns_empty_for_nonexistent_file() {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let nonexistent_path = dir.path().join("nonexistent.log");

    let mut tailer = LogTailer::new(nonexistent_path);
    let entries = tailer.read_new_entries();

    assert!(entries.is_empty(), "LogTailer should return empty for nonexistent file");
}

#[test]
fn log_tailer_starts_at_end_of_existing_file() {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let log_path = dir.path().join("test.log");

    fs::write(&log_path, "existing line 1\nexisting line 2\n").expect("Failed to write log");

    let mut tailer = LogTailer::new(log_path);
    let entries = tailer.read_new_entries();

    assert!(
        entries.is_empty(),
        "LogTailer should start at end, ignoring existing content, got: {:?}",
        entries
    );
}

#[test]
fn log_tailer_reads_new_entries_after_creation() {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let log_path = dir.path().join("test.log");

    fs::write(&log_path, "").expect("Failed to create empty log");

    let mut tailer = LogTailer::new(log_path.clone());

    let mut file =
        fs::OpenOptions::new().append(true).open(&log_path).expect("Failed to open log for append");
    writeln!(file, "new line 1").expect("Failed to write line 1");
    writeln!(file, "new line 2").expect("Failed to write line 2");

    let entries = tailer.read_new_entries();

    assert_eq!(entries.len(), 2, "Should read 2 new lines, got: {:?}", entries);
    assert_eq!(entries[0], "new line 1", "First line should match");
    assert_eq!(entries[1], "new line 2", "Second line should match");
}

#[test]
fn log_tailer_tracks_position_between_reads() {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let log_path = dir.path().join("test.log");

    fs::write(&log_path, "").expect("Failed to create empty log");

    let mut tailer = LogTailer::new(log_path.clone());

    {
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&log_path)
            .expect("Failed to open log for append");
        writeln!(file, "batch 1 line 1").expect("Failed to write");
    }

    let batch1 = tailer.read_new_entries();
    assert_eq!(batch1.len(), 1, "First batch should have 1 line");
    assert_eq!(batch1[0], "batch 1 line 1", "First batch line should match");

    {
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&log_path)
            .expect("Failed to open log for append");
        writeln!(file, "batch 2 line 1").expect("Failed to write");
        writeln!(file, "batch 2 line 2").expect("Failed to write");
    }

    let batch2 = tailer.read_new_entries();
    assert_eq!(batch2.len(), 2, "Second batch should have 2 lines");
    assert_eq!(batch2[0], "batch 2 line 1", "Second batch first line should match");
    assert_eq!(batch2[1], "batch 2 line 2", "Second batch second line should match");
}

#[test]
fn log_tailer_handles_file_truncation() {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let log_path = dir.path().join("test.log");

    fs::write(&log_path, "old content line 1\nold content line 2\nold content line 3\n")
        .expect("Failed to write initial content");

    let mut tailer = LogTailer::new(log_path.clone());
    let _ = tailer.read_new_entries();

    fs::write(&log_path, "new\n").expect("Failed to truncate and write new content");

    let entries = tailer.read_new_entries();

    assert_eq!(entries.len(), 1, "Should read new content after truncation");
    assert_eq!(entries[0], "new", "Should read the new content");
}

#[test]
fn log_tailer_check_for_errors_detects_error_level() {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let log_path = dir.path().join("test.log");

    fs::write(&log_path, "").expect("Failed to create empty log");
    let mut tailer = LogTailer::new(log_path.clone());

    let error_log = r#"{"timestamp":"2024-01-01T00:00:00Z","level":"ERROR","event":{"error":"Connection failed"}}"#;
    fs::write(&log_path, format!("{}\n", error_log)).expect("Failed to write error log");

    let error = tailer.check_for_errors();

    assert!(error.is_some(), "Should detect ERROR level log");
    assert!(
        error.as_ref().unwrap().contains("Connection failed"),
        "Error should contain the error message, got: {:?}",
        error
    );
}

#[test]
fn log_tailer_check_for_errors_ignores_warn_level() {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let log_path = dir.path().join("test.log");

    fs::write(&log_path, "").expect("Failed to create empty log");
    let mut tailer = LogTailer::new(log_path.clone());

    let warn_log =
        r#"{"timestamp":"2024-01-01T00:00:00Z","level":"WARN","message":"Slow query detected"}"#;
    fs::write(&log_path, format!("{}\n", warn_log)).expect("Failed to write warn log");

    let error = tailer.check_for_errors();

    assert!(error.is_none(), "Should ignore WARN level log (only ERROR triggers remediation)");
}

#[test]
fn log_tailer_check_for_errors_ignores_info_and_debug() {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let log_path = dir.path().join("test.log");

    fs::write(&log_path, "").expect("Failed to create empty log");
    let mut tailer = LogTailer::new(log_path.clone());

    let logs = r#"{"timestamp":"2024-01-01T00:00:00Z","level":"INFO","message":"Normal operation"}
{"timestamp":"2024-01-01T00:00:01Z","level":"DEBUG","message":"Debug info"}"#;
    fs::write(&log_path, format!("{}\n", logs)).expect("Failed to write logs");

    let error = tailer.check_for_errors();

    assert!(error.is_none(), "Should not detect INFO or DEBUG level logs as errors");
}

#[test]
fn log_tailer_check_for_errors_returns_first_error() {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let log_path = dir.path().join("test.log");

    fs::write(&log_path, "").expect("Failed to create empty log");
    let mut tailer = LogTailer::new(log_path.clone());

    let logs = r#"{"timestamp":"2024-01-01T00:00:00Z","level":"INFO","message":"Normal"}
{"timestamp":"2024-01-01T00:00:01Z","level":"ERROR","event":{"error":"First error"}}
{"timestamp":"2024-01-01T00:00:02Z","level":"ERROR","event":{"error":"Second error"}}"#;
    fs::write(&log_path, format!("{}\n", logs)).expect("Failed to write logs");

    let error = tailer.check_for_errors();

    assert!(error.is_some(), "Should detect ERROR level log");
    assert!(
        error.as_ref().unwrap().contains("First error"),
        "Should return first error, got: {:?}",
        error
    );
}

#[test]
fn log_tailer_handles_malformed_json_lines() {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let log_path = dir.path().join("test.log");

    fs::write(&log_path, "").expect("Failed to create empty log");
    let mut tailer = LogTailer::new(log_path.clone());

    let logs =
        "not valid json\n{\"level\":\"ERROR\",\"event\":{\"error\":\"Real error\"}}\nalso invalid";
    fs::write(&log_path, format!("{}\n", logs)).expect("Failed to write logs");

    let error = tailer.check_for_errors();

    assert!(error.is_some(), "Should still detect valid ERROR log among malformed lines");
    assert!(error.unwrap().contains("Real error"), "Should return the valid error message");
}
