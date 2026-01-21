use llmc::auto_mode::auto_config::{AutoConfig, ResolvedAutoConfig};

#[test]
fn resolve_with_cli_task_pool_command() {
    let result =
        ResolvedAutoConfig::resolve(None, Some("echo task"), None, None).expect("Should resolve");
    assert_eq!(result.task_pool_command, "echo task", "CLI command should be used");
    assert_eq!(result.concurrency, 1, "Default concurrency should be 1");
    assert!(result.post_accept_command.is_none(), "No post_accept_command expected");
}

#[test]
fn resolve_with_toml_config() {
    let config = AutoConfig {
        task_pool_command: Some("lat pop".to_string()),
        concurrency: 3,
        post_accept_command: Some("just test".to_string()),
    };
    let result =
        ResolvedAutoConfig::resolve(Some(&config), None, None, None).expect("Should resolve");
    assert_eq!(result.task_pool_command, "lat pop", "TOML command should be used");
    assert_eq!(result.concurrency, 3, "TOML concurrency should be used");
    assert_eq!(
        result.post_accept_command,
        Some("just test".to_string()),
        "TOML post_accept_command should be used"
    );
}

#[test]
fn resolve_cli_overrides_toml() {
    let config = AutoConfig {
        task_pool_command: Some("toml command".to_string()),
        concurrency: 2,
        post_accept_command: Some("toml post".to_string()),
    };
    let result =
        ResolvedAutoConfig::resolve(Some(&config), Some("cli command"), Some(5), Some("cli post"))
            .expect("Should resolve");
    assert_eq!(result.task_pool_command, "cli command", "CLI should override TOML command");
    assert_eq!(result.concurrency, 5, "CLI should override TOML concurrency");
    assert_eq!(
        result.post_accept_command,
        Some("cli post".to_string()),
        "CLI should override TOML post_accept_command"
    );
}

#[test]
fn resolve_returns_none_without_task_pool_command() {
    let result = ResolvedAutoConfig::resolve(None, None, Some(3), None);
    assert!(result.is_none(), "Should return None when no task_pool_command is provided");
}

#[test]
fn resolve_returns_none_with_empty_toml_config() {
    let config = AutoConfig { task_pool_command: None, concurrency: 1, post_accept_command: None };
    let result = ResolvedAutoConfig::resolve(Some(&config), None, None, None);
    assert!(
        result.is_none(),
        "Should return None when TOML has no task_pool_command and no CLI override"
    );
}

#[test]
fn auto_config_default_concurrency() {
    let config = AutoConfig::default();
    assert_eq!(config.concurrency, 1, "Default concurrency should be 1");
    assert!(config.task_pool_command.is_none(), "Default task_pool_command should be None");
    assert!(config.post_accept_command.is_none(), "Default post_accept_command should be None");
}

#[test]
fn resolve_partial_cli_overrides() {
    let config = AutoConfig {
        task_pool_command: Some("toml command".to_string()),
        concurrency: 4,
        post_accept_command: Some("toml post".to_string()),
    };
    // Only override concurrency via CLI
    let result =
        ResolvedAutoConfig::resolve(Some(&config), None, Some(8), None).expect("Should resolve");
    assert_eq!(result.task_pool_command, "toml command", "TOML command should be used");
    assert_eq!(result.concurrency, 8, "CLI concurrency should override TOML");
    assert_eq!(
        result.post_accept_command,
        Some("toml post".to_string()),
        "TOML post_accept_command should be used"
    );
}
