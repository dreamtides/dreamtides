use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

/// Valid Claude Code models
const VALID_MODELS: &[&str] = &["haiku", "sonnet", "opus"];

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
    pub on_complete: Option<OnCompleteConfig>,
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
    pub on_complete: Option<OnCompleteConfig>,
}

/// Configuration for the "on complete" prompt sent when a worker finishes a
/// task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnCompleteConfig {
    pub prompt: String,
    #[serde(default)]
    pub include_original: bool,
    #[serde(default)]
    pub clear: bool,
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

/// Validates a model string against known Claude Code models
pub fn validate_model(model: &str) -> Result<()> {
    if !VALID_MODELS.contains(&model) {
        bail!(
            "Invalid model: '{}'\n\
             Valid models are: {}",
            model,
            VALID_MODELS.join(", ")
        );
    }
    Ok(())
}

fn default_model() -> String {
    "sonnet".to_string()
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
            on_complete: None,
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

        validate_model(&self.defaults.model)
            .with_context(|| "Invalid default model in [defaults] section")?;

        #[expect(clippy::iter_over_hash_type)]
        for (name, worker_config) in &self.workers {
            if let Some(model) = &worker_config.model {
                validate_model(model)
                    .with_context(|| format!("Invalid model for worker '{}'", name))?;
            }
        }

        Ok(())
    }

    /// Gets the configuration for a specific worker
    pub fn get_worker(&self, name: &str) -> Option<&WorkerConfig> {
        self.workers.get(name)
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
        assert_eq!(defaults.model, "sonnet");
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
}
