//! Tests for the `lat prime` command.

use std::fs;

use lattice::cli::commands::prime_command;
use lattice::cli::workflow_args::PrimeArgs;
use lattice::config::config_schema::PrimeConfig;
use lattice::test::test_environment::TestEnv;

fn default_args() -> PrimeArgs {
    PrimeArgs { full: false, export: None }
}

// ============================================================================
// Default Output Tests
// ============================================================================

#[test]
fn prime_command_succeeds_with_default_args() {
    let env = TestEnv::new();
    let (_temp, context) = env.into_parts();

    let result = prime_command::execute(context, default_args());
    assert!(result.is_ok(), "Prime command should succeed: {:?}", result);
}

#[test]
fn prime_command_json_output_succeeds() {
    let env = TestEnv::new().with_json_output();
    let (_temp, context) = env.into_parts();

    let result = prime_command::execute(context, default_args());
    assert!(result.is_ok(), "Prime command with JSON output should succeed: {:?}", result);
}

// ============================================================================
// Full Mode Tests
// ============================================================================

#[test]
fn prime_command_full_mode_succeeds() {
    let env = TestEnv::new();
    let (_temp, context) = env.into_parts();

    let args = PrimeArgs { full: true, ..default_args() };
    let result = prime_command::execute(context, args);
    assert!(result.is_ok(), "Prime command with --full should succeed: {:?}", result);
}

#[test]
fn prime_command_full_mode_json_succeeds() {
    let env = TestEnv::new().with_json_output();
    let (_temp, context) = env.into_parts();

    let args = PrimeArgs { full: true, ..default_args() };
    let result = prime_command::execute(context, args);
    assert!(result.is_ok(), "Prime command with --full --json should succeed: {:?}", result);
}

// ============================================================================
// Export Mode Tests
// ============================================================================

#[test]
fn prime_command_export_creates_file() {
    let env = TestEnv::new();
    let export_path = env.path("prime_export.md");
    let export_path_str = export_path.to_string_lossy().to_string();

    let (_temp, context) = env.into_parts();

    let args = PrimeArgs { full: false, export: Some(export_path_str.clone()) };
    let result = prime_command::execute(context, args);
    assert!(result.is_ok(), "Prime export should succeed: {:?}", result);

    assert!(export_path.exists(), "Export file should be created");
    let content = fs::read_to_string(&export_path).expect("Read export file");
    assert!(content.contains("Lattice"), "Export should contain Lattice content");
}

#[test]
fn prime_command_export_full_mode() {
    let env = TestEnv::new();
    let export_path = env.path("prime_export_full.md");
    let export_path_str = export_path.to_string_lossy().to_string();

    let (_temp, context) = env.into_parts();

    let args = PrimeArgs { full: true, export: Some(export_path_str.clone()) };
    let result = prime_command::execute(context, args);
    assert!(result.is_ok(), "Prime export with --full should succeed: {:?}", result);

    assert!(export_path.exists(), "Export file should be created");
    let content = fs::read_to_string(&export_path).expect("Read export file");
    assert!(
        content.contains("Additional Commands"),
        "Full export should contain additional commands section"
    );
}

#[test]
fn prime_command_export_creates_parent_directories() {
    let env = TestEnv::new();
    let export_path = env.path("nested/dir/prime_export.md");
    let export_path_str = export_path.to_string_lossy().to_string();

    let (_temp, context) = env.into_parts();

    let args = PrimeArgs { full: false, export: Some(export_path_str) };
    let result = prime_command::execute(context, args);
    assert!(result.is_ok(), "Prime export with nested path should succeed: {:?}", result);

    assert!(export_path.exists(), "Export file should be created in nested directory");
}

#[test]
fn prime_command_export_with_json_suppresses_console_output() {
    let env = TestEnv::new().with_json_output();
    let export_path = env.path("prime_json_export.md");
    let export_path_str = export_path.to_string_lossy().to_string();

    let (_temp, context) = env.into_parts();

    let args = PrimeArgs { full: false, export: Some(export_path_str) };
    let result = prime_command::execute(context, args);
    assert!(result.is_ok(), "Prime export with --json should succeed: {:?}", result);

    assert!(export_path.exists(), "Export file should be created even with JSON mode");
}

// ============================================================================
// Configuration Tests
// ============================================================================

#[test]
fn prime_command_uses_default_checklist() {
    let env = TestEnv::new();
    let (_temp, context) = env.into_parts();

    let default_config = PrimeConfig::default();
    assert!(!default_config.checklist.is_empty(), "Default checklist should not be empty");
    assert!(
        default_config.checklist.iter().any(|s| s.contains("lat check")),
        "Default checklist should contain 'lat check'"
    );

    let result = prime_command::execute(context, default_args());
    assert!(result.is_ok(), "Prime command should succeed with default config: {:?}", result);
}

#[test]
fn prime_command_uses_custom_checklist_from_config() {
    let mut env = TestEnv::new();

    env.config_mut().prime.checklist =
        vec!["Custom step 1".to_string(), "Custom step 2".to_string(), "Custom step 3".to_string()];

    let (_temp, context) = env.into_parts();

    let result = prime_command::execute(context, default_args());
    assert!(result.is_ok(), "Prime command with custom checklist should succeed: {:?}", result);
}

#[test]
fn prime_command_empty_checklist_is_valid() {
    let mut env = TestEnv::new();
    env.config_mut().prime.checklist = Vec::new();

    let (_temp, context) = env.into_parts();

    let result = prime_command::execute(context, default_args());
    assert!(result.is_ok(), "Prime command with empty checklist should succeed: {:?}", result);
}

// ============================================================================
// Content Verification Tests
// ============================================================================

#[test]
fn prime_export_contains_session_protocol_section() {
    let env = TestEnv::new();
    let export_path = env.path("prime_protocol.md");
    let export_path_str = export_path.to_string_lossy().to_string();

    let (_temp, context) = env.into_parts();

    let args = PrimeArgs { full: false, export: Some(export_path_str) };
    let result = prime_command::execute(context, args);
    assert!(result.is_ok(), "Export should succeed");

    let content = fs::read_to_string(&export_path).expect("Read export file");
    assert!(content.contains("Session Protocol"), "Export should contain Session Protocol section");
}

#[test]
fn prime_export_contains_core_commands_section() {
    let env = TestEnv::new();
    let export_path = env.path("prime_commands.md");
    let export_path_str = export_path.to_string_lossy().to_string();

    let (_temp, context) = env.into_parts();

    let args = PrimeArgs { full: false, export: Some(export_path_str) };
    let result = prime_command::execute(context, args);
    assert!(result.is_ok(), "Export should succeed");

    let content = fs::read_to_string(&export_path).expect("Read export file");
    assert!(content.contains("Core Commands"), "Export should contain Core Commands section");
    assert!(content.contains("lat overview"), "Export should contain lat overview command");
    assert!(content.contains("lat ready"), "Export should contain lat ready command");
    assert!(content.contains("lat show"), "Export should contain lat show command");
}

#[test]
fn prime_export_contains_link_authoring_section() {
    let env = TestEnv::new();
    let export_path = env.path("prime_links.md");
    let export_path_str = export_path.to_string_lossy().to_string();

    let (_temp, context) = env.into_parts();

    let args = PrimeArgs { full: false, export: Some(export_path_str) };
    let result = prime_command::execute(context, args);
    assert!(result.is_ok(), "Export should succeed");

    let content = fs::read_to_string(&export_path).expect("Read export file");
    assert!(content.contains("Link Authoring"), "Export should contain Link Authoring section");
    assert!(
        content.contains("lat fmt"),
        "Export should mention lat fmt command for link expansion"
    );
}

#[test]
fn prime_full_export_contains_additional_sections() {
    let env = TestEnv::new();
    let export_path = env.path("prime_full_sections.md");
    let export_path_str = export_path.to_string_lossy().to_string();

    let (_temp, context) = env.into_parts();

    let args = PrimeArgs { full: true, export: Some(export_path_str) };
    let result = prime_command::execute(context, args);
    assert!(result.is_ok(), "Full export should succeed");

    let content = fs::read_to_string(&export_path).expect("Read export file");
    assert!(
        content.contains("Query Commands"),
        "Full export should contain Query Commands section"
    );
    assert!(
        content.contains("Task Management"),
        "Full export should contain Task Management section"
    );
    assert!(
        content.contains("Hierarchy Commands"),
        "Full export should contain Hierarchy Commands section"
    );
    assert!(
        content.contains("Relationship Commands"),
        "Full export should contain Relationship Commands section"
    );
    assert!(
        content.contains("Maintenance Commands"),
        "Full export should contain Maintenance Commands section"
    );
}

#[test]
fn prime_export_is_valid_markdown() {
    let env = TestEnv::new();
    let export_path = env.path("prime_markdown.md");
    let export_path_str = export_path.to_string_lossy().to_string();

    let (_temp, context) = env.into_parts();

    let args = PrimeArgs { full: true, export: Some(export_path_str) };
    let result = prime_command::execute(context, args);
    assert!(result.is_ok(), "Export should succeed");

    let content = fs::read_to_string(&export_path).expect("Read export file");

    assert!(content.starts_with('#'), "Markdown should start with a header");
    assert!(content.contains("\n## "), "Markdown should contain section headers");
    assert!(content.contains("\n- `lat"), "Markdown should contain command list items");
}

#[test]
fn prime_export_includes_checklist_items() {
    let mut env = TestEnv::new();
    env.config_mut().prime.checklist = vec![
        "Run unit tests".to_string(),
        "Check formatting".to_string(),
        "Commit changes".to_string(),
    ];

    let export_path = env.path("prime_checklist.md");
    let export_path_str = export_path.to_string_lossy().to_string();

    let (_temp, context) = env.into_parts();

    let args = PrimeArgs { full: false, export: Some(export_path_str) };
    let result = prime_command::execute(context, args);
    assert!(result.is_ok(), "Export should succeed");

    let content = fs::read_to_string(&export_path).expect("Read export file");
    assert!(content.contains("Run unit tests"), "Export should contain first checklist item");
    assert!(content.contains("Check formatting"), "Export should contain second checklist item");
    assert!(content.contains("Commit changes"), "Export should contain third checklist item");
}
