use std::time::Duration;

use llmc::overseer_mode::overseer_config::OverseerConfig;

#[test]
fn default_config_has_expected_timeout_values() {
    let config = OverseerConfig::default();

    assert_eq!(config.heartbeat_timeout_secs, 30, "Default heartbeat timeout should be 30 seconds");
    assert_eq!(
        config.stall_timeout_secs, 3600,
        "Default stall timeout should be 3600 seconds (1 hour)"
    );
    assert_eq!(config.restart_cooldown_secs, 60, "Default restart cooldown should be 60 seconds");
}

#[test]
fn default_config_has_no_remediation_prompt() {
    let config = OverseerConfig::default();

    assert!(
        config.remediation_prompt.is_none(),
        "Default config should have no remediation prompt"
    );
    assert!(
        config.get_remediation_prompt().is_none(),
        "get_remediation_prompt should return None for default config"
    );
}

#[test]
fn get_heartbeat_timeout_returns_duration() {
    let config = OverseerConfig::default();
    let timeout = config.get_heartbeat_timeout();

    assert_eq!(
        timeout,
        Duration::from_secs(30),
        "get_heartbeat_timeout should return Duration of 30 seconds"
    );
}

#[test]
fn get_stall_timeout_returns_duration() {
    let config = OverseerConfig::default();
    let timeout = config.get_stall_timeout();

    assert_eq!(
        timeout,
        Duration::from_secs(3600),
        "get_stall_timeout should return Duration of 3600 seconds"
    );
}

#[test]
fn get_restart_cooldown_returns_duration() {
    let config = OverseerConfig::default();
    let cooldown = config.get_restart_cooldown();

    assert_eq!(
        cooldown,
        Duration::from_secs(60),
        "get_restart_cooldown should return Duration of 60 seconds"
    );
}

#[test]
fn get_remediation_prompt_returns_configured_prompt() {
    let mut config = OverseerConfig::default();
    config.remediation_prompt = Some("Test remediation instructions".to_string());
    let prompt = config.get_remediation_prompt();

    assert_eq!(
        prompt,
        Some("Test remediation instructions"),
        "get_remediation_prompt should return the configured prompt"
    );
}

#[test]
fn custom_timeout_values_are_preserved() {
    let config = OverseerConfig {
        remediation_prompt: Some("Custom prompt".to_string()),
        heartbeat_timeout_secs: 60,
        stall_timeout_secs: 7200,
        restart_cooldown_secs: 120,
    };

    assert_eq!(config.get_heartbeat_timeout(), Duration::from_secs(60));
    assert_eq!(config.get_stall_timeout(), Duration::from_secs(7200));
    assert_eq!(config.get_restart_cooldown(), Duration::from_secs(120));
    assert_eq!(config.get_remediation_prompt(), Some("Custom prompt"));
}

#[test]
fn config_serializes_to_json() {
    let config = OverseerConfig {
        remediation_prompt: Some("Test prompt".to_string()),
        heartbeat_timeout_secs: 30,
        stall_timeout_secs: 3600,
        restart_cooldown_secs: 60,
    };

    let json = serde_json::to_string(&config).expect("Should serialize to JSON");

    assert!(json.contains("Test prompt"), "JSON should contain remediation_prompt, got: {}", json);
    assert!(
        json.contains("heartbeat_timeout_secs"),
        "JSON should contain heartbeat_timeout_secs, got: {}",
        json
    );
}

#[test]
fn config_round_trips_through_json() {
    let original = OverseerConfig {
        remediation_prompt: Some("Round trip test".to_string()),
        heartbeat_timeout_secs: 45,
        stall_timeout_secs: 1800,
        restart_cooldown_secs: 90,
    };

    let json = serde_json::to_string(&original).expect("Should serialize to JSON");
    let recovered: OverseerConfig =
        serde_json::from_str(&json).expect("Should deserialize from JSON");

    assert_eq!(
        recovered.remediation_prompt, original.remediation_prompt,
        "Remediation prompt should round-trip"
    );
    assert_eq!(
        recovered.heartbeat_timeout_secs, original.heartbeat_timeout_secs,
        "Heartbeat timeout should round-trip"
    );
    assert_eq!(
        recovered.stall_timeout_secs, original.stall_timeout_secs,
        "Stall timeout should round-trip"
    );
    assert_eq!(
        recovered.restart_cooldown_secs, original.restart_cooldown_secs,
        "Restart cooldown should round-trip"
    );
}
