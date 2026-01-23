use llmc::auto_mode::auto_config::{AutoConfig, ResolvedAutoConfig};

#[test]
fn resolve_with_task_list_id() {
    let config = AutoConfig { task_list_id: Some("dreamtides".to_string()), ..Default::default() };
    let result = ResolvedAutoConfig::resolve(Some(&config), "/home/user/repo", None, None)
        .expect("Should resolve");
    assert_eq!(result.task_list_id, "dreamtides", "Task list ID should be used");
    assert_eq!(result.concurrency, 1, "Default concurrency should be 1");
    assert!(result.post_accept_command.is_none(), "No post_accept_command expected");
}

#[test]
fn resolve_with_full_toml_config() {
    let config = AutoConfig {
        task_list_id: Some("my-project".to_string()),
        tasks_root: Some("/custom/tasks".to_string()),
        context_config_path: Some("/custom/context.toml".to_string()),
        concurrency: 3,
        post_accept_command: Some("just test".to_string()),
    };
    let result = ResolvedAutoConfig::resolve(Some(&config), "/home/user/repo", None, None)
        .expect("Should resolve");
    assert_eq!(result.task_list_id, "my-project", "TOML task list ID should be used");
    assert_eq!(result.concurrency, 3, "TOML concurrency should be used");
    assert_eq!(
        result.post_accept_command,
        Some("just test".to_string()),
        "TOML post_accept_command should be used"
    );
    assert_eq!(
        result.tasks_root.to_string_lossy(),
        "/custom/tasks",
        "Custom tasks_root should be used"
    );
}

#[test]
fn resolve_cli_overrides_toml() {
    let config = AutoConfig {
        task_list_id: Some("toml-project".to_string()),
        concurrency: 2,
        post_accept_command: Some("toml post".to_string()),
        ..Default::default()
    };
    let result =
        ResolvedAutoConfig::resolve(Some(&config), "/home/user/repo", Some(5), Some("cli post"))
            .expect("Should resolve");
    assert_eq!(result.concurrency, 5, "CLI should override TOML concurrency");
    assert_eq!(
        result.post_accept_command,
        Some("cli post".to_string()),
        "CLI should override TOML post_accept_command"
    );
}

#[test]
fn resolve_returns_none_without_task_list_id() {
    let result = ResolvedAutoConfig::resolve(None, "/home/user/repo", Some(3), None);
    assert!(result.is_none(), "Should return None when no task_list_id is provided");
}

#[test]
fn resolve_returns_none_with_empty_toml_config() {
    let config = AutoConfig::default();
    let result = ResolvedAutoConfig::resolve(Some(&config), "/home/user/repo", None, None);
    assert!(result.is_none(), "Should return None when TOML has no task_list_id");
}

#[test]
fn auto_config_default_values() {
    let config = AutoConfig::default();
    assert_eq!(config.concurrency, 1, "Default concurrency should be 1");
    assert!(config.task_list_id.is_none(), "Default task_list_id should be None");
    assert!(config.tasks_root.is_none(), "Default tasks_root should be None");
    assert!(config.context_config_path.is_none(), "Default context_config_path should be None");
    assert!(config.post_accept_command.is_none(), "Default post_accept_command should be None");
}

#[test]
fn resolve_partial_cli_overrides() {
    let config = AutoConfig {
        task_list_id: Some("toml-project".to_string()),
        concurrency: 4,
        post_accept_command: Some("toml post".to_string()),
        ..Default::default()
    };
    // Only override concurrency via CLI
    let result = ResolvedAutoConfig::resolve(Some(&config), "/home/user/repo", Some(8), None)
        .expect("Should resolve");
    assert_eq!(result.task_list_id, "toml-project", "TOML task_list_id should be used");
    assert_eq!(result.concurrency, 8, "CLI concurrency should override TOML");
    assert_eq!(
        result.post_accept_command,
        Some("toml post".to_string()),
        "TOML post_accept_command should be used"
    );
}

#[test]
fn get_task_directory_combines_root_and_id() {
    let config = AutoConfig { task_list_id: Some("my-tasks".to_string()), ..Default::default() };
    let resolved = ResolvedAutoConfig::resolve(Some(&config), "/home/user/repo", None, None)
        .expect("Should resolve");
    let task_dir = resolved.get_task_directory();
    assert!(
        task_dir.ends_with("my-tasks"),
        "Task directory should end with task_list_id, got: {:?}",
        task_dir
    );
}

#[test]
fn default_context_config_path_uses_repo_source() {
    let config = AutoConfig { task_list_id: Some("my-tasks".to_string()), ..Default::default() };
    let resolved = ResolvedAutoConfig::resolve(Some(&config), "/home/user/repo", None, None)
        .expect("Should resolve");
    let context_path = resolved.context_config_path.expect("Should have context config path");
    assert!(
        context_path.to_string_lossy().contains("llmc_task_context.toml"),
        "Context config path should contain default filename, got: {:?}",
        context_path
    );
}
