#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

/// Global LLMC configuration loaded from ~/llmc/config.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub defaults: DefaultsConfig,
    pub repo: RepoConfig,
    #[serde(default)]
    pub workers: HashMap<String, WorkerConfig>,
}

/// Default values for worker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultsConfig {
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default = "default_skip_permissions")]
    pub skip_permissions: bool,
    #[serde(default = "default_allowed_tools")]
    pub allowed_tools: Vec<String>,
    #[serde(default = "default_patrol_interval_secs")]
    pub patrol_interval_secs: u32,
    #[serde(default = "default_sound_on_review")]
    pub sound_on_review: bool,
}

/// Repository configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoConfig {
    pub source: String,
}

/// Per-worker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerConfig {
    pub model: Option<String>,
    pub role_prompt: Option<String>,
    #[serde(default = "default_excluded_from_pool")]
    pub excluded_from_pool: bool,
}

/// Returns the LLMC root directory (~/llmc)
pub fn get_llmc_root() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .expect("Could not determine home directory");
    PathBuf::from(home).join("llmc")
}

/// Returns the path to the config file (~/llmc/config.toml)
pub fn get_config_path() -> PathBuf {
    get_llmc_root().join("config.toml")
}

fn default_model() -> String {
    "opus".to_string()
}

fn default_skip_permissions() -> bool {
    true
}

fn default_allowed_tools() -> Vec<String> {
    vec![
        "Bash".to_string(),
        "Edit".to_string(),
        "Read".to_string(),
        "Write".to_string(),
        "Glob".to_string(),
        "Grep".to_string(),
    ]
}

fn default_patrol_interval_secs() -> u32 {
    60
}

fn default_sound_on_review() -> bool {
    true
}

fn default_excluded_from_pool() -> bool {
    false
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        DefaultsConfig {
            model: default_model(),
            skip_permissions: default_skip_permissions(),
            allowed_tools: default_allowed_tools(),
            patrol_interval_secs: default_patrol_interval_secs(),
            sound_on_review: default_sound_on_review(),
        }
    }
}

impl Config {
    /// Loads configuration from the given path
    pub fn load(path: &Path) -> Result<Config> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        config.validate()?;

        Ok(config)
    }

    /// Validates the configuration
    fn validate(&self) -> Result<()> {
        if self.repo.source.is_empty() {
            bail!("repo.source is required in config.toml");
        }

        if self.defaults.patrol_interval_secs == 0 {
            bail!("defaults.patrol_interval_secs must be greater than 0");
        }

        Ok(())
    }

    /// Gets the configuration for a specific worker
    pub fn get_worker(&self, name: &str) -> Option<&WorkerConfig> {
        self.workers.get(name)
    }

    /// Gets the model to use for a worker (worker-specific or default)
    pub fn get_worker_model(&self, name: &str) -> &str {
        self.workers.get(name).and_then(|w| w.model.as_deref()).unwrap_or(&self.defaults.model)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn test_defaults() {
        let defaults = DefaultsConfig::default();
        assert_eq!(defaults.model, "opus");
        assert!(defaults.skip_permissions);
        assert_eq!(defaults.allowed_tools.len(), 6);
        assert_eq!(defaults.patrol_interval_secs, 60);
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
        assert_eq!(config.defaults.model, "opus");
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

        let baker = config.get_worker("baker").unwrap();
        assert_eq!(baker.model, None);
        assert_eq!(baker.role_prompt.as_deref(), Some("You are Baker"));
        assert!(!baker.excluded_from_pool);

        assert_eq!(config.get_worker_model("adam"), "opus");
        assert_eq!(config.get_worker_model("baker"), "sonnet");
        assert_eq!(config.get_worker_model("unknown"), "sonnet");
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
}
