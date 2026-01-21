use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::auto_mode::auto_config::AutoConfig;
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
    /// Configuration for autonomous operation mode
    pub auto: Option<AutoConfig>,
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
    pub self_review: Option<SelfReviewConfig>,
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
    /// If true, enable self-review for this worker using the
    /// defaults.self_review prompt
    #[serde(default)]
    pub self_review: Option<bool>,
}
/// Configuration for the self-review prompt sent when a worker finishes a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfReviewConfig {
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
    30
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
            self_review: None,
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
