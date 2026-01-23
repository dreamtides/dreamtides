use std::collections::HashMap;

use llmc::config::{Config, DefaultsConfig, RepoConfig};
use llmc::sound::play_bell;

#[test]
fn test_play_bell_enabled() {
    let config = Config {
        defaults: DefaultsConfig {
            model: "opus".to_string(),
            skip_permissions: true,
            allowed_tools: vec![],
            patrol_interval_secs: 60,
            sound_on_review: true,
            self_review: None,
        },
        repo: RepoConfig { source: "/test".to_string(), default_branch: "master".to_string() },
        workers: HashMap::new(),
        auto: None,
        overseer: None,
    };
    let result = play_bell(&config);
    assert!(result.is_ok());
}

#[test]
fn test_play_bell_disabled() {
    let config = Config {
        defaults: DefaultsConfig {
            model: "opus".to_string(),
            skip_permissions: true,
            allowed_tools: vec![],
            patrol_interval_secs: 60,
            sound_on_review: false,
            self_review: None,
        },
        repo: RepoConfig { source: "/test".to_string(), default_branch: "master".to_string() },
        workers: HashMap::new(),
        auto: None,
        overseer: None,
    };
    let result = play_bell(&config);
    assert!(result.is_ok());
}
