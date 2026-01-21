use std::io::Write;

use llmc::config::Config;
use llmc::overseer_mode::health_monitor::HealthStatus;
use llmc::overseer_mode::remediation_prompt;
use tempfile::NamedTempFile;

fn create_test_config(remediation_prompt: Option<&str>) -> Config {
    let overseer_section = match remediation_prompt {
        Some(prompt) => format!(
            r#"
            [overseer]
            remediation_prompt = """{}"""
            "#,
            prompt
        ),
        None => String::new(),
    };
    let toml = format!(
        r#"
        [repo]
        source = "/tmp/test-repo"
        {}
        "#,
        overseer_section
    );
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(toml.as_bytes()).expect("Failed to write config");
    Config::load(file.path()).expect("Failed to load config")
}

#[test]
fn build_prompt_contains_all_sections() {
    let config = create_test_config(Some("Custom remediation instructions here."));
    let failure = HealthStatus::ProcessGone;
    let prompt = remediation_prompt::build_remediation_prompt(&failure, &config);
    assert!(
        prompt.contains("# Remediation Instructions"),
        "Prompt should contain remediation instructions section"
    );
    assert!(prompt.contains("# Error Context"), "Prompt should contain error context section");
    assert!(
        prompt.contains("# Recovery Instructions"),
        "Prompt should contain recovery instructions section"
    );
}

#[test]
fn build_prompt_includes_user_instructions() {
    let custom_prompt = "You are debugging the LLMC daemon. Check logs carefully.";
    let config = create_test_config(Some(custom_prompt));
    let failure = HealthStatus::Healthy;
    let prompt = remediation_prompt::build_remediation_prompt(&failure, &config);
    assert!(
        prompt.contains(custom_prompt),
        "Prompt should include user-provided remediation instructions"
    );
}

#[test]
fn build_prompt_handles_missing_overseer_config() {
    let config = create_test_config(None);
    let failure = HealthStatus::ProcessGone;
    let prompt = remediation_prompt::build_remediation_prompt(&failure, &config);
    assert!(
        prompt.contains("No overseer configuration found")
            || prompt.contains("No custom remediation instructions"),
        "Prompt should indicate missing overseer configuration"
    );
}

#[test]
fn build_prompt_formats_process_gone_failure() {
    let config = create_test_config(Some("Test"));
    let failure = HealthStatus::ProcessGone;
    let prompt = remediation_prompt::build_remediation_prompt(&failure, &config);
    assert!(prompt.contains("Process Gone"), "Prompt should describe ProcessGone failure");
    assert!(
        prompt.contains("no longer running"),
        "Prompt should explain the daemon is not running"
    );
}

#[test]
fn build_prompt_formats_heartbeat_stale_failure() {
    let config = create_test_config(Some("Test"));
    let failure = HealthStatus::HeartbeatStale { age_secs: 120 };
    let prompt = remediation_prompt::build_remediation_prompt(&failure, &config);
    assert!(prompt.contains("Heartbeat Stale"), "Prompt should describe HeartbeatStale failure");
    assert!(prompt.contains("120 seconds"), "Prompt should include the age in seconds");
}

#[test]
fn build_prompt_formats_log_error_failure() {
    let config = create_test_config(Some("Test"));
    let error_message = "Worker entered error state: git rebase failed";
    let failure = HealthStatus::LogError { message: error_message.to_string() };
    let prompt = remediation_prompt::build_remediation_prompt(&failure, &config);
    assert!(prompt.contains("Error/Warning in Logs"), "Prompt should describe LogError failure");
    assert!(prompt.contains(error_message), "Prompt should include the error message");
}

#[test]
fn build_prompt_formats_stalled_failure() {
    let config = create_test_config(Some("Test"));
    let failure = HealthStatus::Stalled { stall_secs: 3600 };
    let prompt = remediation_prompt::build_remediation_prompt(&failure, &config);
    assert!(prompt.contains("Stalled Progress"), "Prompt should describe Stalled failure");
    assert!(prompt.contains("3600 seconds"), "Prompt should include the stall duration");
}

#[test]
fn build_prompt_formats_identity_mismatch_failure() {
    let config = create_test_config(Some("Test"));
    let reason = "PID changed from 1234 to 5678";
    let failure = HealthStatus::IdentityMismatch { reason: reason.to_string() };
    let prompt = remediation_prompt::build_remediation_prompt(&failure, &config);
    assert!(
        prompt.contains("Identity Mismatch"),
        "Prompt should describe IdentityMismatch failure"
    );
    assert!(prompt.contains(reason), "Prompt should include the mismatch reason");
}

#[test]
fn build_prompt_includes_recovery_instructions() {
    let config = create_test_config(Some("Test"));
    let failure = HealthStatus::ProcessGone;
    let prompt = remediation_prompt::build_remediation_prompt(&failure, &config);
    assert!(
        prompt.contains("Exit normally"),
        "Recovery instructions should mention exiting normally"
    );
    assert!(
        prompt.contains("manual_intervention_needed"),
        "Recovery instructions should mention manual intervention file"
    );
    assert!(
        prompt.contains("overseer will automatically restart"),
        "Recovery instructions should explain automatic restart"
    );
}

#[test]
fn build_prompt_includes_context_subsections() {
    let config = create_test_config(Some("Test"));
    let failure = HealthStatus::ProcessGone;
    let prompt = remediation_prompt::build_remediation_prompt(&failure, &config);
    assert!(prompt.contains("## Failure Type"), "Prompt should have Failure Type subsection");
    assert!(
        prompt.contains("## Daemon Registration"),
        "Prompt should have Daemon Registration subsection"
    );
    assert!(prompt.contains("## Worker States"), "Prompt should have Worker States subsection");
    assert!(prompt.contains("## Git Status"), "Prompt should have Git Status subsection");
    assert!(prompt.contains("## Log Excerpts"), "Prompt should have Log Excerpts subsection");
}

#[test]
fn build_prompt_handles_healthy_status() {
    let config = create_test_config(Some("Test"));
    let failure = HealthStatus::Healthy;
    let prompt = remediation_prompt::build_remediation_prompt(&failure, &config);
    assert!(
        prompt.contains("unexpected remediation trigger"),
        "Healthy status should be marked as unexpected"
    );
}
