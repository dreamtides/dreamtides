use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Complete Lattice configuration with all settings from all sources.
#[derive(Debug, Clone, Default)]
pub struct Config {
    pub overview: OverviewConfig,
    pub prime: PrimeConfig,
    pub format: FormatConfig,
    pub check: CheckConfig,
    pub sparse: SparseConfig,
    pub claim: ClaimConfig,
    pub logging: LoggingConfig,
    pub defaults: DefaultsConfig,
}

/// Configuration for `lat overview` command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OverviewConfig {
    pub limit: u32,
    pub view_weight: f64,
    pub recency_weight: f64,
    pub root_weight: f64,
    pub recency_half_life_days: u32,
}

/// Configuration for `lat prime` command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PrimeConfig {
    pub checklist: Vec<String>,
}

/// Configuration for `lat fmt` command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FormatConfig {
    pub line_width: u32,
    pub list_marker: String,
}

/// Configuration for `lat check` command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CheckConfig {
    pub max_lines: u32,
    pub max_name_length: u32,
    pub max_description_length: u32,
}

/// Configuration for sparse checkout handling.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SparseConfig {
    pub warn_sparse_links: bool,
    pub auto_expand: bool,
}

/// Configuration for `lat claim` command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ClaimConfig {
    pub stale_days: u32,
}

/// Configuration for logging.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub level: String,
    pub max_file_size_mb: u32,
}

/// Default values for task creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DefaultsConfig {
    pub priority: u8,
    pub line_width: u32,
}

/// User configuration from ~/.lattice.toml.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct UserConfig {
    pub clients: HashMap<PathBuf, String>,
    #[serde(default)]
    pub defaults: DefaultsConfig,
}

/// Repository configuration from .lattice/config.toml.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RepoConfig {
    #[serde(default)]
    pub overview: OverviewConfig,
    #[serde(default)]
    pub prime: PrimeConfig,
    #[serde(default)]
    pub format: FormatConfig,
    #[serde(default)]
    pub check: CheckConfig,
    #[serde(default)]
    pub sparse: SparseConfig,
    #[serde(default)]
    pub claim: ClaimConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

impl Default for OverviewConfig {
    fn default() -> Self {
        Self {
            limit: 10,
            view_weight: 0.5,
            recency_weight: 0.3,
            root_weight: 0.2,
            recency_half_life_days: 7,
        }
    }
}

impl Default for PrimeConfig {
    fn default() -> Self {
        Self {
            checklist: vec![
                "lat check".to_string(),
                "lat fmt".to_string(),
                "git status".to_string(),
                "git add <files>".to_string(),
                "git commit".to_string(),
            ],
        }
    }
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self { line_width: 80, list_marker: "-".to_string() }
    }
}

impl Default for CheckConfig {
    fn default() -> Self {
        Self { max_lines: 500, max_name_length: 64, max_description_length: 1024 }
    }
}

impl Default for SparseConfig {
    fn default() -> Self {
        Self { warn_sparse_links: true, auto_expand: false }
    }
}

impl Default for ClaimConfig {
    fn default() -> Self {
        Self { stale_days: 7 }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self { level: "info".to_string(), max_file_size_mb: 10 }
    }
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self { priority: 2, line_width: 80 }
    }
}

impl Config {
    /// Merges user-level configuration into this config.
    pub fn merge_user_config(&mut self, user_config: &UserConfig) {
        self.defaults.priority = user_config.defaults.priority;
        if user_config.defaults.line_width != DefaultsConfig::default().line_width {
            self.defaults.line_width = user_config.defaults.line_width;
            self.format.line_width = user_config.defaults.line_width;
        }
    }

    /// Merges repository-level configuration into this config.
    pub fn merge_repo_config(&mut self, repo_config: &RepoConfig) {
        self.overview = repo_config.overview.clone();
        self.prime = repo_config.prime.clone();
        self.format = repo_config.format.clone();
        self.check = repo_config.check.clone();
        self.sparse = repo_config.sparse.clone();
        self.claim = repo_config.claim.clone();
        self.logging = repo_config.logging.clone();
    }

    /// Applies environment variable overrides.
    pub fn apply_env_overrides(&mut self) {
        if let Ok(level) = std::env::var("LATTICE_LOG_LEVEL") {
            self.logging.level = level;
        }
    }
}
