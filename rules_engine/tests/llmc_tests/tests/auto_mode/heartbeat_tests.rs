use std::time::Duration;

use llmc::auto_mode::heartbeat_thread::{
    DaemonRegistration, Heartbeat, generate_instance_id, is_heartbeat_stale,
};

#[test]
fn generate_instance_id_is_unique() {
    let id1 = generate_instance_id();
    let id2 = generate_instance_id();
    assert_ne!(id1, id2, "Generated instance IDs should be unique");
    assert!(!id1.is_empty(), "Instance ID should not be empty");
}

#[test]
fn generate_instance_id_is_uuid_format() {
    let id = generate_instance_id();
    // UUID format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx (36 chars with dashes)
    assert_eq!(id.len(), 36, "Instance ID should be 36 characters (UUID format)");
    assert_eq!(id.matches('-').count(), 4, "UUID should have 4 dashes");
}

#[test]
fn heartbeat_new_has_current_timestamp() {
    let instance_id = "test-instance";
    let heartbeat = Heartbeat::new(instance_id);
    assert_eq!(heartbeat.instance_id, instance_id, "Instance ID should match");
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time error")
        .as_secs();
    let age = now.saturating_sub(heartbeat.timestamp_unix);
    assert!(age < 5, "Heartbeat timestamp should be within 5 seconds of now, age={}", age);
}

#[test]
fn heartbeat_stores_instance_id() {
    let id = "my-test-instance-123";
    let heartbeat = Heartbeat::new(id);
    assert_eq!(heartbeat.instance_id, id, "Heartbeat should store the instance ID");
}

#[test]
fn is_heartbeat_stale_returns_false_for_fresh() {
    let heartbeat = Heartbeat::new("fresh-test");
    let is_stale = is_heartbeat_stale(&heartbeat, Duration::from_secs(30));
    assert!(!is_stale, "Fresh heartbeat should not be stale");
}

#[test]
fn is_heartbeat_stale_returns_true_for_old() {
    let mut heartbeat = Heartbeat::new("old-test");
    // Set timestamp to 60 seconds ago
    heartbeat.timestamp_unix = heartbeat.timestamp_unix.saturating_sub(60);
    let is_stale = is_heartbeat_stale(&heartbeat, Duration::from_secs(30));
    assert!(is_stale, "Heartbeat older than timeout should be stale");
}

#[test]
fn is_heartbeat_stale_boundary_not_stale() {
    let mut heartbeat = Heartbeat::new("boundary-test");
    // Set timestamp to exactly the timeout ago - should NOT be stale
    heartbeat.timestamp_unix = heartbeat.timestamp_unix.saturating_sub(30);
    let is_stale = is_heartbeat_stale(&heartbeat, Duration::from_secs(30));
    assert!(!is_stale, "Heartbeat exactly at timeout should not be stale");
}

#[test]
fn is_heartbeat_stale_boundary_is_stale() {
    let mut heartbeat = Heartbeat::new("boundary-test");
    // Set timestamp to just over the timeout ago - should be stale
    heartbeat.timestamp_unix = heartbeat.timestamp_unix.saturating_sub(31);
    let is_stale = is_heartbeat_stale(&heartbeat, Duration::from_secs(30));
    assert!(is_stale, "Heartbeat just over timeout should be stale");
}

#[test]
fn is_heartbeat_stale_handles_zero_timeout() {
    let heartbeat = Heartbeat::new("zero-timeout-test");
    let is_stale = is_heartbeat_stale(&heartbeat, Duration::from_secs(0));
    // A fresh heartbeat with 0 timeout should not be stale (age is 0, threshold is
    // 0)
    assert!(!is_stale, "Heartbeat with zero timeout and zero age should not be stale");
}

#[test]
fn daemon_registration_new_has_correct_fields() {
    let instance_id = "reg-test";
    let log_file = "/path/to/auto.log";
    let registration = DaemonRegistration::new(instance_id, log_file);
    assert_eq!(registration.instance_id, instance_id, "Instance ID should match");
    assert_eq!(registration.log_file, log_file, "Log file should match");
    assert!(registration.pid > 0, "PID should be positive");
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time error")
        .as_secs();
    let age = now.saturating_sub(registration.start_time_unix);
    assert!(age < 5, "Start time should be within 5 seconds of now, age={}", age);
}

#[test]
fn daemon_registration_stores_current_pid() {
    let registration = DaemonRegistration::new("test", "/log");
    assert_eq!(
        registration.pid,
        std::process::id(),
        "Registration should store current process ID"
    );
}

#[test]
fn daemon_registration_stores_log_path() {
    let log_path = "/custom/path/to/daemon.log";
    let registration = DaemonRegistration::new("test", log_path);
    assert_eq!(registration.log_file, log_path, "Registration should store exact log path");
}

#[test]
fn heartbeat_serialization_roundtrip() {
    let heartbeat = Heartbeat { timestamp_unix: 1234567890, instance_id: "test-id".to_string() };
    let json = serde_json::to_string(&heartbeat).expect("Serialization should succeed");
    let deserialized: Heartbeat =
        serde_json::from_str(&json).expect("Deserialization should succeed");
    assert_eq!(heartbeat.timestamp_unix, deserialized.timestamp_unix, "Timestamp should match");
    assert_eq!(heartbeat.instance_id, deserialized.instance_id, "Instance ID should match");
}

#[test]
fn daemon_registration_serialization_roundtrip() {
    let registration = DaemonRegistration {
        pid: 12345,
        start_time_unix: 9876543210,
        instance_id: "my-instance".to_string(),
        log_file: "/var/log/daemon.log".to_string(),
    };
    let json = serde_json::to_string(&registration).expect("Serialization should succeed");
    let deserialized: DaemonRegistration =
        serde_json::from_str(&json).expect("Deserialization should succeed");
    assert_eq!(registration.pid, deserialized.pid, "PID should match");
    assert_eq!(
        registration.start_time_unix, deserialized.start_time_unix,
        "Start time should match"
    );
    assert_eq!(registration.instance_id, deserialized.instance_id, "Instance ID should match");
    assert_eq!(registration.log_file, deserialized.log_file, "Log file should match");
}
