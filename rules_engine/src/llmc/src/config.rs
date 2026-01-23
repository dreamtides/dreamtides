use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::auto_mode::auto_config::AutoConfig;
use crate::overseer_mode::overseer_config::OverseerConfig;

/// Valid Claude Code models
const VALID_MODELS: &[&str] = &["haiku", "sonnet", "opus"];

/// Provides access to LLMC paths derived from a root directory.
///
/// Use `LlmcPaths::from_env()` in production code to read from LLMC_ROOT.
/// Use `LlmcPaths::new(root)` in tests with a temporary directory.
#[derive(Debug, Clone)]
pub struct LlmcPaths {
    root: PathBuf,
}

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
    /// Configuration for the overseer supervisor process
    pub overseer: Option<OverseerConfig>,
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
    /// The default branch name (e.g., "main" or "master").
    /// If not specified, defaults to "master" for backwards compatibility.
    #[serde(default = "default_branch")]
    pub default_branch: String,
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

/// Returns the LLMC root directory.
///
/// The root directory is determined in the following order:
/// 1. `LLMC_ROOT` environment variable (if set)
/// 2. Default: `~/llmc`
///
/// This allows multiple LLMC instances to run in different directories
/// simultaneously without interfering with each other.
pub fn get_llmc_root() -> PathBuf {
    if let Ok(llmc_root) = std::env::var("LLMC_ROOT") {
        return PathBuf::from(llmc_root);
    }
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .expect("Could not determine home directory");
    PathBuf::from(home).join("llmc")
}

/// Returns the path to the config file (~/llmc/config.toml)
pub fn get_config_path() -> PathBuf {
    get_llmc_root().join("config.toml")
}

/// Returns the TMUX session prefix for this LLMC instance.
///
/// When using the default `~/llmc` root, returns just "llmc".
/// When using a custom LLMC_ROOT, returns "llmc-<dirname>" where dirname
/// is the last component of the LLMC root path. This ensures multiple
/// LLMC instances don't conflict on TMUX session names.
///
/// Examples:
/// - `~/llmc` -> "llmc" (default, backward compatible)
/// - `/tmp/llmc-test` -> "llmc-llmc-test"
/// - `~/projects/myproject-llmc` -> "llmc-myproject-llmc"
pub fn get_session_prefix() -> String {
    let llmc_root = get_llmc_root();
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).unwrap_or_default();
    let default_root = PathBuf::from(&home).join("llmc");

    if llmc_root == default_root {
        "llmc".to_string()
    } else {
        let dirname = llmc_root.file_name().and_then(|n| n.to_str()).unwrap_or("custom");
        format!("llmc-{}", dirname)
    }
}

/// Returns the TMUX session name for a worker in this LLMC instance.
///
/// Format: `<session_prefix>-<worker_name>`
/// Examples:
/// - Default: `llmc-adam`, `llmc-auto-1`
/// - Custom LLMC_ROOT `/tmp/test-llmc`: `llmc-test-llmc-adam`
pub fn get_worker_session_name(worker_name: &str) -> String {
    format!("{}-{}", get_session_prefix(), worker_name)
}

/// Returns the prefix pattern used to identify all LLMC sessions for this
/// instance.
///
/// This is `<session_prefix>-` and is used in `starts_with()` checks to find
/// all sessions belonging to this LLMC instance.
pub fn get_session_prefix_pattern() -> String {
    format!("{}-", get_session_prefix())
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

fn default_branch() -> String {
    "master".to_string()
}

impl RepoConfig {
    /// Returns the origin reference for the default branch (e.g.,
    /// "origin/main").
    pub fn origin_branch(&self) -> String {
        format!("origin/{}", self.default_branch)
    }
}

impl LlmcPaths {
    /// Creates paths from an explicit root directory.
    ///
    /// Use this in tests to avoid depending on environment variables.
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Creates paths by reading LLMC_ROOT from the environment.
    ///
    /// Falls back to `~/llmc` if LLMC_ROOT is not set.
    pub fn from_env() -> Self {
        Self::new(get_llmc_root())
    }

    /// Returns the path to the logs directory.
    fn logs_dir(&self) -> PathBuf {
        self.root.join("logs")
    }

    /// Returns the path to the task pool log file.
    pub fn task_discovery_log_path(&self) -> PathBuf {
        self.logs_dir().join("task_discovery.log")
    }

    /// Returns the path to the post accept log file.
    pub fn post_accept_log_path(&self) -> PathBuf {
        self.logs_dir().join("post_accept.log")
    }

    /// Returns the path to the auto log file.
    pub fn auto_log_path(&self) -> PathBuf {
        self.logs_dir().join("auto.log")
    }
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
        if self.get_heartbeat_timeout().is_zero() {
            bail!("overseer.heartbeat_timeout_secs must be greater than 0");
        }
        if self.get_stall_timeout().is_zero() {
            bail!("overseer.stall_timeout_secs must be greater than 0");
        }
        if self.get_restart_cooldown().is_zero() {
            bail!("overseer.restart_cooldown_secs must be greater than 0");
        }
        if let Some(prompt) = self.get_remediation_prompt()
            && prompt.is_empty()
        {
            bail!("overseer.remediation_prompt cannot be empty");
        }
        Ok(())
    }

    /// Gets the configuration for a specific worker
    pub fn get_worker(&self, name: &str) -> Option<&WorkerConfig> {
        self.workers.get(name)
    }

    /// Returns the remediation prompt from overseer config, if configured.
    pub fn get_remediation_prompt(&self) -> Option<&str> {
        self.overseer.as_ref().and_then(|o| o.get_remediation_prompt())
    }

    /// Returns the heartbeat timeout from overseer config, or the default
    /// (30s).
    pub fn get_heartbeat_timeout(&self) -> Duration {
        self.overseer
            .as_ref()
            .map(OverseerConfig::get_heartbeat_timeout)
            .unwrap_or_else(|| Duration::from_secs(30))
    }

    /// Returns the stall timeout from overseer config, or the default (3600s).
    pub fn get_stall_timeout(&self) -> Duration {
        self.overseer
            .as_ref()
            .map(OverseerConfig::get_stall_timeout)
            .unwrap_or_else(|| Duration::from_secs(3600))
    }

    /// Returns the restart cooldown from overseer config, or the default (60s).
    pub fn get_restart_cooldown(&self) -> Duration {
        self.overseer
            .as_ref()
            .map(OverseerConfig::get_restart_cooldown)
            .unwrap_or_else(|| Duration::from_secs(60))
    }
}
