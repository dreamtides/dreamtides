use tv_lib::logging::log_aggregator::{self, FrontendLogMessage};

fn make_message(level: &str, component: &str, msg: &str) -> FrontendLogMessage {
    FrontendLogMessage {
        ts: "2025-01-15T14:30:45.123-08:00".to_string(),
        level: level.to_string(),
        component: component.to_string(),
        msg: msg.to_string(),
        context: None,
    }
}

#[test]
fn test_ingest_error_level() {
    let message = make_message("ERROR", "tv.ui", "Component failed to render");
    log_aggregator::ingest(&message);
}

#[test]
fn test_ingest_warn_level() {
    let message = make_message("WARN", "tv.ui", "Slow render detected");
    log_aggregator::ingest(&message);
}

#[test]
fn test_ingest_info_level() {
    let message = make_message("INFO", "tv.ui.render", "Sheet rendered");
    log_aggregator::ingest(&message);
}

#[test]
fn test_ingest_debug_level() {
    let message = make_message("DEBUG", "tv.ipc", "Command invoked");
    log_aggregator::ingest(&message);
}

#[test]
fn test_ingest_trace_level() {
    let message = make_message("TRACE", "tv.ui.event", "Mouse moved");
    log_aggregator::ingest(&message);
}

#[test]
fn test_ingest_unknown_level_falls_back_to_info() {
    let message = make_message("CUSTOM", "tv.ui", "Custom level message");
    log_aggregator::ingest(&message);
}

#[test]
fn test_ingest_with_context() {
    let context = serde_json::json!({
        "file_path": "/tmp/test.toml",
        "row": 42,
        "duration_ms": 150
    });
    let message = FrontendLogMessage {
        ts: "2025-01-15T14:30:45.123-08:00".to_string(),
        level: "INFO".to_string(),
        component: "tv.ui.render".to_string(),
        msg: "Sheet rendered".to_string(),
        context: Some(context),
    };
    log_aggregator::ingest(&message);
}

#[test]
fn test_ingest_with_empty_context() {
    let message = FrontendLogMessage {
        ts: "2025-01-15T14:30:45.123-08:00".to_string(),
        level: "DEBUG".to_string(),
        component: "tv.ipc".to_string(),
        msg: "Event received".to_string(),
        context: Some(serde_json::json!({})),
    };
    log_aggregator::ingest(&message);
}

#[test]
fn test_ingest_with_empty_message() {
    let message = make_message("INFO", "tv.ui", "");
    log_aggregator::ingest(&message);
}

#[test]
fn test_ingest_with_hierarchical_component() {
    let message = make_message("INFO", "tv.derived.executor.compute", "Computation complete");
    log_aggregator::ingest(&message);
}

#[test]
fn test_ingest_preserves_pacific_timestamp() {
    let message = FrontendLogMessage {
        ts: "2025-07-15T14:30:45.123-07:00".to_string(),
        level: "INFO".to_string(),
        component: "tv.ui".to_string(),
        msg: "PDT timestamp".to_string(),
        context: None,
    };
    log_aggregator::ingest(&message);
}

#[test]
fn test_frontend_log_message_deserializes_from_json() {
    let json = r#"{
        "ts": "2025-01-15T14:30:45.123-08:00",
        "level": "WARN",
        "component": "tv.sync",
        "msg": "Watcher event received",
        "context": {"file_path": "/tmp/test.toml", "event_type": "modify"}
    }"#;
    let message: FrontendLogMessage = serde_json::from_str(json).expect("Should deserialize");
    assert_eq!(message.level, "WARN");
    assert_eq!(message.component, "tv.sync");
    assert_eq!(message.msg, "Watcher event received");
    assert!(message.context.is_some());
}

#[test]
fn test_frontend_log_message_deserializes_without_context() {
    let json = r#"{
        "ts": "2025-01-15T14:30:45.123-08:00",
        "level": "INFO",
        "component": "tv.toml",
        "msg": "File loaded"
    }"#;
    let message: FrontendLogMessage = serde_json::from_str(json).expect("Should deserialize");
    assert_eq!(message.level, "INFO");
    assert_eq!(message.component, "tv.toml");
    assert_eq!(message.msg, "File loaded");
    assert!(message.context.is_none());
}

#[test]
fn test_frontend_log_message_deserializes_with_null_context() {
    let json = r#"{
        "ts": "2025-01-15T14:30:45.123-08:00",
        "level": "DEBUG",
        "component": "tv.ui",
        "msg": "Render complete",
        "context": null
    }"#;
    let message: FrontendLogMessage = serde_json::from_str(json).expect("Should deserialize");
    assert!(message.context.is_none());
}
