use std::fs;

use lattice::config::config_loader::{load_repo_config, validate_config};
use lattice::config::config_schema::{
    CheckConfig, ClaimConfig, Config, DefaultsConfig, FormatConfig, LoggingConfig, OverviewConfig,
    PrimeConfig, RepoConfig, SparseConfig, UserConfig,
};
use lattice::git::client_config::{generate_client_id, validate_client_id};
use tempfile::TempDir;

#[test]
fn default_overview_config_has_correct_values() {
    let config = OverviewConfig::default();

    assert_eq!(config.limit, 10, "Default limit should be 10");
    assert!((config.view_weight - 0.5).abs() < 0.001, "Default view_weight should be 0.5");
    assert!((config.recency_weight - 0.3).abs() < 0.001, "Default recency_weight should be 0.3");
    assert!((config.root_weight - 0.2).abs() < 0.001, "Default root_weight should be 0.2");
    assert_eq!(config.recency_half_life_days, 7, "Default recency_half_life_days should be 7");
}

#[test]
fn default_prime_config_has_expected_checklist() {
    let config = PrimeConfig::default();

    assert_eq!(config.checklist.len(), 5, "Default checklist should have 5 items");
    assert!(
        config.checklist.contains(&"lat check".to_string()),
        "Checklist should include 'lat check'"
    );
    assert!(
        config.checklist.contains(&"git commit".to_string()),
        "Checklist should include 'git commit'"
    );
}

#[test]
fn default_format_config_values() {
    let config = FormatConfig::default();

    assert_eq!(config.line_width, 80, "Default line_width should be 80");
    assert_eq!(config.list_marker, "-", "Default list_marker should be '-'");
}

#[test]
fn default_check_config_values() {
    let config = CheckConfig::default();

    assert_eq!(config.max_lines, 500, "Default max_lines should be 500");
    assert_eq!(config.max_name_length, 64, "Default max_name_length should be 64");
    assert_eq!(
        config.max_description_length, 1024,
        "Default max_description_length should be 1024"
    );
}

#[test]
fn default_sparse_config_values() {
    let config = SparseConfig::default();

    assert!(config.warn_sparse_links, "Default warn_sparse_links should be true");
    assert!(!config.auto_expand, "Default auto_expand should be false");
}

#[test]
fn default_claim_config_values() {
    let config = ClaimConfig::default();

    assert_eq!(config.stale_days, 7, "Default stale_days should be 7");
}

#[test]
fn default_logging_config_values() {
    let config = LoggingConfig::default();

    assert_eq!(config.level, "info", "Default log level should be 'info'");
    assert_eq!(config.max_file_size_mb, 10, "Default max_file_size_mb should be 10");
}

#[test]
fn default_defaults_config_values() {
    let config = DefaultsConfig::default();

    assert_eq!(config.priority, 2, "Default priority should be 2");
    assert_eq!(config.line_width, 80, "Default line_width should be 80");
}

#[test]
fn config_merges_user_config_priority() {
    let mut config = Config::default();
    let mut user_config = UserConfig::default();
    user_config.defaults.priority = 1;

    config.merge_user_config(&user_config);

    assert_eq!(config.defaults.priority, 1, "Config should use user config priority");
}

#[test]
fn config_merges_user_config_line_width() {
    let mut config = Config::default();
    let mut user_config = UserConfig::default();
    user_config.defaults.line_width = 120;

    config.merge_user_config(&user_config);

    assert_eq!(config.defaults.line_width, 120, "Config should use user config line_width");
    assert_eq!(
        config.format.line_width, 120,
        "Format config should also use user config line_width"
    );
}

#[test]
fn config_merges_repo_config() {
    let mut config = Config::default();
    let mut repo_config = RepoConfig::default();
    repo_config.overview.limit = 20;
    repo_config.format.line_width = 100;
    repo_config.check.max_lines = 1000;

    config.merge_repo_config(&repo_config);

    assert_eq!(config.overview.limit, 20, "Config should use repo config overview.limit");
    assert_eq!(config.format.line_width, 100, "Config should use repo config format.line_width");
    assert_eq!(config.check.max_lines, 1000, "Config should use repo config check.max_lines");
}

#[test]
fn repo_config_parses_from_toml() {
    let toml_content = r#"
[overview]
limit = 15
view_weight = 0.6
recency_weight = 0.2
root_weight = 0.2

[format]
line_width = 100
list_marker = "*"

[check]
max_lines = 750

[claim]
stale_days = 14
"#;

    let repo_config: RepoConfig =
        toml::from_str(toml_content).expect("Should parse valid TOML into RepoConfig");

    assert_eq!(repo_config.overview.limit, 15, "Should parse overview.limit");
    assert!((repo_config.overview.view_weight - 0.6).abs() < 0.001, "Should parse view_weight");
    assert_eq!(repo_config.format.line_width, 100, "Should parse format.line_width");
    assert_eq!(repo_config.format.list_marker, "*", "Should parse format.list_marker");
    assert_eq!(repo_config.check.max_lines, 750, "Should parse check.max_lines");
    assert_eq!(repo_config.claim.stale_days, 14, "Should parse claim.stale_days");
}

#[test]
fn repo_config_uses_defaults_for_missing_sections() {
    let toml_content = r#"
[format]
line_width = 90
"#;

    let repo_config: RepoConfig =
        toml::from_str(toml_content).expect("Should parse partial TOML into RepoConfig");

    assert_eq!(
        repo_config.overview.limit,
        OverviewConfig::default().limit,
        "Should use default for missing overview section"
    );
    assert_eq!(repo_config.format.line_width, 90, "Should use provided format.line_width");
    assert_eq!(
        repo_config.format.list_marker,
        FormatConfig::default().list_marker,
        "Should use default for missing format fields"
    );
}

#[test]
fn user_config_parses_clients_mapping() {
    let toml_content = r#"
[clients]
"/path/to/repo" = "DT"
"/other/repo" = "K2"

[defaults]
priority = 1
"#;

    let user_config: UserConfig =
        toml::from_str(toml_content).expect("Should parse valid TOML into UserConfig");

    assert_eq!(user_config.clients.len(), 2, "Should parse two client mappings");
    assert_eq!(user_config.defaults.priority, 1, "Should parse defaults.priority");
}

#[test]
fn load_repo_config_returns_none_for_missing_file() {
    let dir = TempDir::new().expect("Should create temp dir");

    let result = load_repo_config(dir.path()).expect("Should not error for missing config");

    assert!(result.is_none(), "Should return None when config file doesn't exist");
}

#[test]
fn load_repo_config_parses_existing_file() {
    let dir = TempDir::new().expect("Should create temp dir");
    let lattice_dir = dir.path().join(".lattice");
    fs::create_dir_all(&lattice_dir).expect("Should create .lattice directory");

    let config_content = r#"
[overview]
limit = 25

[logging]
level = "debug"
"#;
    fs::write(lattice_dir.join("config.toml"), config_content).expect("Should write config file");

    let result = load_repo_config(dir.path()).expect("Should load config without error");
    let config = result.expect("Should return Some for existing config");

    assert_eq!(config.overview.limit, 25, "Should parse overview.limit from file");
    assert_eq!(config.logging.level, "debug", "Should parse logging.level from file");
}

#[test]
fn validate_config_rejects_negative_weights() {
    let mut config = Config::default();
    config.overview.view_weight = -0.1;

    let result = validate_config(&config);

    assert!(result.is_err(), "Should reject negative weight");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("non-negative"),
        "Error should mention non-negative requirement"
    );
}

#[test]
fn validate_config_rejects_invalid_priority() {
    let mut config = Config::default();
    config.defaults.priority = 5;

    let result = validate_config(&config);

    assert!(result.is_err(), "Should reject priority > 4");
    let err = result.unwrap_err();
    assert!(err.to_string().contains("0-4"), "Error should mention valid range");
}

#[test]
fn validate_config_rejects_invalid_log_level() {
    let mut config = Config::default();
    config.logging.level = "invalid".to_string();

    let result = validate_config(&config);

    assert!(result.is_err(), "Should reject invalid log level");
    let err = result.unwrap_err();
    assert!(err.to_string().contains("error"), "Error should mention valid log levels");
}

#[test]
fn validate_config_accepts_valid_config() {
    let config = Config::default();

    let result = validate_config(&config);

    assert!(result.is_ok(), "Should accept valid default config");
}

#[test]
fn validate_client_id_accepts_valid_ids() {
    assert!(validate_client_id("K2X").is_ok(), "Should accept 3-char ID");
    assert!(validate_client_id("AB7Z").is_ok(), "Should accept 4-char ID");
    assert!(validate_client_id("234AB").is_ok(), "Should accept 5-char ID");
    assert!(validate_client_id("ABCDEF").is_ok(), "Should accept 6-char ID");
}

#[test]
fn validate_client_id_rejects_too_short() {
    let result = validate_client_id("AB");

    assert!(result.is_err(), "Should reject 2-char ID");
    let err = result.unwrap_err();
    assert!(err.to_string().contains("3-6"), "Error should mention valid length");
}

#[test]
fn validate_client_id_rejects_too_long() {
    let result = validate_client_id("ABCDEFG");

    assert!(result.is_err(), "Should reject 7-char ID");
    let err = result.unwrap_err();
    assert!(err.to_string().contains("3-6"), "Error should mention valid length");
}

#[test]
fn validate_client_id_rejects_invalid_characters() {
    assert!(validate_client_id("abc").is_err(), "Should reject lowercase letters");
    assert!(validate_client_id("012").is_err(), "Should reject digits 0 and 1");
    assert!(validate_client_id("A-B").is_err(), "Should reject special characters");
    assert!(validate_client_id("A89").is_err(), "Should reject digits 8 and 9");
}

#[test]
fn generate_client_id_produces_valid_id() {
    let client_id = generate_client_id();

    assert!(validate_client_id(&client_id).is_ok(), "Generated ID should be valid: {client_id}");
    assert!(client_id.len() >= 3, "Generated ID should be at least 3 chars");
}

#[test]
fn generate_client_id_is_deterministic_for_same_environment() {
    let id1 = generate_client_id();
    let id2 = generate_client_id();

    assert_eq!(id1, id2, "Same environment should produce same client ID");
}

#[test]
fn apply_env_overrides_sets_log_level_from_env() {
    let mut config = Config::default();
    assert_eq!(config.logging.level, "info", "Default should be 'info'");

    // SAFETY: This test runs single-threaded and immediately cleans up the env var
    unsafe {
        std::env::set_var("LATTICE_LOG_LEVEL", "debug");
    }
    config.apply_env_overrides();
    unsafe {
        std::env::remove_var("LATTICE_LOG_LEVEL");
    }

    assert_eq!(
        config.logging.level, "debug",
        "LATTICE_LOG_LEVEL env var should override logging.level"
    );
}
