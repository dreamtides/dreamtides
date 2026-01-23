use std::io::Write;
use std::time::Duration;

use llmc::auto_mode::auto_config::{AutoConfig, ResolvedAutoConfig};
use llmc::config::{Config, get_config_path, get_llmc_root, validate_model};
use llmc::overseer_mode::overseer_config::OverseerConfig;
use tempfile::NamedTempFile;

#[test]
fn test_defaults() {
    let defaults = llmc::config::DefaultsConfig::default();
    assert_eq!(defaults.model, "sonnet");
    assert!(defaults.skip_permissions);
    assert_eq!(defaults.allowed_tools.len(), 6);
    assert_eq!(defaults.patrol_interval_secs, 30);
    assert!(defaults.sound_on_review);
}

#[test]
fn test_minimal_config() {
    let toml = r#"
        [repo]
        source = "/path/to/repo"
    "#;
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(toml.as_bytes()).unwrap();
    let config = Config::load(file.path()).unwrap();
    assert_eq!(config.repo.source, "/path/to/repo");
    assert_eq!(config.defaults.model, "sonnet");
    assert!(config.workers.is_empty());
}

#[test]
fn test_full_config() {
    let toml = r#"
        [defaults]
        model = "sonnet"
        skip_permissions = false
        allowed_tools = ["Bash", "Read"]
        patrol_interval_secs = 120
        sound_on_review = false

        [repo]
        source = "/path/to/repo"

        [workers.adam]
        model = "opus"
        role_prompt = "You are Adam"
        excluded_from_pool = true
        self_review = true

        [workers.baker]
        role_prompt = "You are Baker"
    "#;
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(toml.as_bytes()).unwrap();
    let config = Config::load(file.path()).unwrap();
    assert_eq!(config.defaults.model, "sonnet");
    assert!(!config.defaults.skip_permissions);
    assert_eq!(config.defaults.allowed_tools, vec!["Bash", "Read"]);
    assert_eq!(config.defaults.patrol_interval_secs, 120);
    assert!(!config.defaults.sound_on_review);
    assert_eq!(config.repo.source, "/path/to/repo");
    let adam = config.get_worker("adam").unwrap();
    assert_eq!(adam.model.as_deref(), Some("opus"));
    assert_eq!(adam.role_prompt.as_deref(), Some("You are Adam"));
    assert!(adam.excluded_from_pool);
    assert_eq!(adam.self_review, Some(true));
    let baker = config.get_worker("baker").unwrap();
    assert_eq!(baker.model, None);
    assert_eq!(baker.role_prompt.as_deref(), Some("You are Baker"));
    assert!(!baker.excluded_from_pool);
    assert_eq!(baker.self_review, None);
}

#[test]
fn test_missing_repo_source() {
    let toml = r#"
        [defaults]
        model = "opus"
    "#;
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(toml.as_bytes()).unwrap();
    let result = Config::load(file.path());
    assert!(result.is_err());
}

#[test]
fn test_invalid_patrol_interval() {
    let toml = r#"
        [defaults]
        patrol_interval_secs = 0

        [repo]
        source = "/path/to/repo"
    "#;
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(toml.as_bytes()).unwrap();
    let result = Config::load(file.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("patrol_interval_secs"));
}

#[test]
fn test_get_llmc_root() {
    let root = get_llmc_root();
    assert!(root.ends_with("llmc"));
}

#[test]
fn test_get_config_path() {
    let config_path = get_config_path();
    assert!(config_path.ends_with("llmc/config.toml"));
}

#[test]
fn test_invalid_default_model() {
    let toml = r#"
        [defaults]
        model = "gpt4"

        [repo]
        source = "/path/to/repo"
    "#;
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(toml.as_bytes()).unwrap();
    let result = Config::load(file.path());
    assert!(result.is_err());
}

#[test]
fn test_invalid_worker_model() {
    let toml = r#"
        [repo]
        source = "/path/to/repo"

        [workers.adam]
        model = "invalid-model"
    "#;
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(toml.as_bytes()).unwrap();
    let result = Config::load(file.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid"));
}

#[test]
fn test_valid_models() {
    validate_model("haiku").unwrap();
    validate_model("sonnet").unwrap();
    validate_model("opus").unwrap();
}

#[test]
fn test_invalid_model_validation() {
    assert!(validate_model("gpt4").is_err());
    assert!(validate_model("invalid").is_err());
    assert!(validate_model("").is_err());
}

#[test]
fn test_auto_config_parsing() {
    let toml = r#"
        [repo]
        source = "/path/to/repo"

        [auto]
        task_list_id = "my-project"
        concurrency = 3
        post_accept_command = "just test"
    "#;
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(toml.as_bytes()).unwrap();
    let config = Config::load(file.path()).unwrap();
    let auto = config.auto.as_ref().unwrap();
    assert_eq!(auto.task_list_id.as_deref(), Some("my-project"));
    assert_eq!(auto.concurrency, 3);
    assert_eq!(auto.post_accept_command.as_deref(), Some("just test"));
}

#[test]
fn test_auto_config_defaults() {
    let toml = r#"
        [repo]
        source = "/path/to/repo"

        [auto]
        task_list_id = "my-project"
    "#;
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(toml.as_bytes()).unwrap();
    let config = Config::load(file.path()).unwrap();
    let auto = config.auto.as_ref().unwrap();
    assert_eq!(auto.task_list_id.as_deref(), Some("my-project"));
    assert_eq!(auto.concurrency, 1);
    assert_eq!(auto.post_accept_command, None);
}

#[test]
fn test_auto_config_not_present() {
    let toml = r#"
        [repo]
        source = "/path/to/repo"
    "#;
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(toml.as_bytes()).unwrap();
    let config = Config::load(file.path()).unwrap();
    assert!(config.auto.is_none());
}

#[test]
fn test_auto_config_empty_section() {
    let toml = r#"
        [repo]
        source = "/path/to/repo"

        [auto]
    "#;
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(toml.as_bytes()).unwrap();
    let config = Config::load(file.path()).unwrap();
    let auto = config.auto.as_ref().unwrap();
    assert_eq!(auto.task_list_id, None);
    assert_eq!(auto.concurrency, 1);
}

#[test]
fn test_resolved_auto_config_from_toml() {
    let toml_config = AutoConfig {
        task_list_id: Some("my-project".to_string()),
        concurrency: 3,
        post_accept_command: Some("just test".to_string()),
        ..Default::default()
    };
    let resolved =
        ResolvedAutoConfig::resolve(Some(&toml_config), "/path/to/repo", None, None).unwrap();
    assert_eq!(resolved.task_list_id, "my-project");
    assert_eq!(resolved.concurrency, 3);
    assert_eq!(resolved.post_accept_command, Some("just test".to_string()));
}

#[test]
fn test_resolved_auto_config_cli_overrides() {
    let toml_config = AutoConfig {
        task_list_id: Some("my-project".to_string()),
        concurrency: 3,
        post_accept_command: Some("just test".to_string()),
        ..Default::default()
    };
    let resolved = ResolvedAutoConfig::resolve(
        Some(&toml_config),
        "/path/to/repo",
        Some(5),
        Some("custom post"),
    )
    .unwrap();
    assert_eq!(resolved.task_list_id, "my-project");
    assert_eq!(resolved.concurrency, 5);
    assert_eq!(resolved.post_accept_command, Some("custom post".to_string()));
}

#[test]
fn test_resolved_auto_config_missing_task_list_id() {
    let toml_config = AutoConfig::default();
    let resolved = ResolvedAutoConfig::resolve(Some(&toml_config), "/path/to/repo", None, None);
    assert!(resolved.is_none());
}

#[test]
fn test_resolved_auto_config_no_config() {
    let resolved = ResolvedAutoConfig::resolve(None, "/path/to/repo", None, None);
    assert!(resolved.is_none());
}

#[test]
fn test_resolved_auto_config_partial_cli_override() {
    let toml_config = AutoConfig {
        task_list_id: Some("my-project".to_string()),
        concurrency: 3,
        post_accept_command: Some("just test".to_string()),
        ..Default::default()
    };
    let resolved =
        ResolvedAutoConfig::resolve(Some(&toml_config), "/path/to/repo", Some(10), None).unwrap();
    assert_eq!(resolved.task_list_id, "my-project");
    assert_eq!(resolved.concurrency, 10);
    assert_eq!(resolved.post_accept_command, Some("just test".to_string()));
}

#[test]
fn test_auto_config_default() {
    let config = AutoConfig::default();
    assert_eq!(config.task_list_id, None);
    assert_eq!(config.concurrency, 1);
    assert_eq!(config.post_accept_command, None);
}

#[test]
fn test_overseer_config_parsing() {
    let toml = r#"
        [repo]
        source = "/path/to/repo"

        [overseer]
        remediation_prompt = "Fix any issues and restart"
        heartbeat_timeout_secs = 60
        stall_timeout_secs = 7200
        restart_cooldown_secs = 120
    "#;
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(toml.as_bytes()).unwrap();
    let config = Config::load(file.path()).unwrap();
    let overseer = config.overseer.as_ref().unwrap();
    assert_eq!(overseer.remediation_prompt.as_deref(), Some("Fix any issues and restart"));
    assert_eq!(overseer.heartbeat_timeout_secs, 60);
    assert_eq!(overseer.stall_timeout_secs, 7200);
    assert_eq!(overseer.restart_cooldown_secs, 120);
}

#[test]
fn test_overseer_config_defaults() {
    let toml = r#"
        [repo]
        source = "/path/to/repo"

        [overseer]
        remediation_prompt = "Fix issues"
    "#;
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(toml.as_bytes()).unwrap();
    let config = Config::load(file.path()).unwrap();
    let overseer = config.overseer.as_ref().unwrap();
    assert_eq!(overseer.remediation_prompt.as_deref(), Some("Fix issues"));
    assert_eq!(overseer.heartbeat_timeout_secs, 30, "heartbeat_timeout_secs should default to 30");
    assert_eq!(overseer.stall_timeout_secs, 3600, "stall_timeout_secs should default to 3600");
    assert_eq!(overseer.restart_cooldown_secs, 60, "restart_cooldown_secs should default to 60");
}

#[test]
fn test_overseer_config_not_present() {
    let toml = r#"
        [repo]
        source = "/path/to/repo"
    "#;
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(toml.as_bytes()).unwrap();
    let config = Config::load(file.path()).unwrap();
    assert!(config.overseer.is_none());
}

#[test]
fn test_overseer_config_empty_section() {
    let toml = r#"
        [repo]
        source = "/path/to/repo"

        [overseer]
    "#;
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(toml.as_bytes()).unwrap();
    let config = Config::load(file.path()).unwrap();
    let overseer = config.overseer.as_ref().unwrap();
    assert_eq!(overseer.remediation_prompt, None);
    assert_eq!(overseer.heartbeat_timeout_secs, 30);
    assert_eq!(overseer.stall_timeout_secs, 3600);
    assert_eq!(overseer.restart_cooldown_secs, 60);
}

#[test]
fn test_overseer_config_default_struct() {
    let config = OverseerConfig::default();
    assert_eq!(config.remediation_prompt, None);
    assert_eq!(config.heartbeat_timeout_secs, 30);
    assert_eq!(config.stall_timeout_secs, 3600);
    assert_eq!(config.restart_cooldown_secs, 60);
}

#[test]
fn test_overseer_config_accessors() {
    let config = OverseerConfig {
        remediation_prompt: Some("Test prompt".to_string()),
        heartbeat_timeout_secs: 45,
        stall_timeout_secs: 1800,
        restart_cooldown_secs: 90,
    };
    assert_eq!(config.get_remediation_prompt(), Some("Test prompt"));
    assert_eq!(config.get_heartbeat_timeout(), Duration::from_secs(45));
    assert_eq!(config.get_stall_timeout(), Duration::from_secs(1800));
    assert_eq!(config.get_restart_cooldown(), Duration::from_secs(90));
}

#[test]
fn test_config_overseer_accessors_with_overseer() {
    let toml = r#"
        [repo]
        source = "/path/to/repo"

        [overseer]
        remediation_prompt = "Test remediation"
        heartbeat_timeout_secs = 45
        stall_timeout_secs = 1800
        restart_cooldown_secs = 90
    "#;
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(toml.as_bytes()).unwrap();
    let config = Config::load(file.path()).unwrap();
    assert_eq!(config.get_remediation_prompt(), Some("Test remediation"));
    assert_eq!(config.get_heartbeat_timeout(), Duration::from_secs(45));
    assert_eq!(config.get_stall_timeout(), Duration::from_secs(1800));
    assert_eq!(config.get_restart_cooldown(), Duration::from_secs(90));
}

#[test]
fn test_config_overseer_accessors_without_overseer() {
    let toml = r#"
        [repo]
        source = "/path/to/repo"
    "#;
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(toml.as_bytes()).unwrap();
    let config = Config::load(file.path()).unwrap();
    assert_eq!(config.get_remediation_prompt(), None);
    assert_eq!(config.get_heartbeat_timeout(), Duration::from_secs(30));
    assert_eq!(config.get_stall_timeout(), Duration::from_secs(3600));
    assert_eq!(config.get_restart_cooldown(), Duration::from_secs(60));
}
