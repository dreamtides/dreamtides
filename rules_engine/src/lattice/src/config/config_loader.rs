use std::path::{Path, PathBuf};

use tracing::{debug, info, warn};

use crate::config::config_schema::{Config, RepoConfig, UserConfig};
use crate::error::error_types::LatticeError;

/// Loads configuration from all sources in precedence order.
///
/// Sources (later wins):
/// 1. Built-in defaults
/// 2. ~/.lattice.toml (user config)
/// 3. .lattice/config.toml (repo config)
/// 4. Environment variables
/// 5. CLI flags (handled by caller)
pub fn load_config(repo_root: Option<&Path>) -> Result<Config, LatticeError> {
    let mut config = Config::default();
    debug!("Starting config load with built-in defaults");

    if let Some(user_config) = load_user_config()? {
        debug!("Merging user configuration from ~/.lattice.toml");
        config.merge_user_config(&user_config);
    }

    if let Some(root) = repo_root
        && let Some(repo_config) = load_repo_config(root)?
    {
        debug!(path = %root.display(), "Merging repository configuration");
        config.merge_repo_config(&repo_config);
    }

    config.apply_env_overrides();
    debug!("Applied environment variable overrides");

    info!(log_level = %config.logging.level, "Configuration loaded successfully");
    Ok(config)
}

/// Loads user configuration from ~/.lattice.toml if it exists.
pub fn load_user_config() -> Result<Option<UserConfig>, LatticeError> {
    let Some(home) = home_dir() else {
        debug!("Could not determine home directory, skipping user config");
        return Ok(None);
    };

    let user_config_path = home.join(".lattice.toml");
    if !user_config_path.exists() {
        debug!(path = %user_config_path.display(), "User config file not found");
        return Ok(None);
    }

    let content = std::fs::read_to_string(&user_config_path).map_err(|e| {
        LatticeError::ReadError { path: user_config_path.clone(), reason: e.to_string() }
    })?;

    let user_config: UserConfig = toml::from_str(&content).map_err(|e| {
        LatticeError::ConfigParseError { path: user_config_path.clone(), reason: e.to_string() }
    })?;

    info!(path = %user_config_path.display(), "Loaded user configuration");
    Ok(Some(user_config))
}

/// Loads repository configuration from .lattice/config.toml if it exists.
pub fn load_repo_config(repo_root: &Path) -> Result<Option<RepoConfig>, LatticeError> {
    let repo_config_path = repo_root.join(".lattice").join("config.toml");
    if !repo_config_path.exists() {
        debug!(path = %repo_config_path.display(), "Repository config file not found");
        return Ok(None);
    }

    let content = std::fs::read_to_string(&repo_config_path).map_err(|e| {
        LatticeError::ReadError { path: repo_config_path.clone(), reason: e.to_string() }
    })?;

    let repo_config: RepoConfig = toml::from_str(&content).map_err(|e| {
        LatticeError::ConfigParseError { path: repo_config_path.clone(), reason: e.to_string() }
    })?;

    info!(path = %repo_config_path.display(), "Loaded repository configuration");
    Ok(Some(repo_config))
}

/// Returns the path to the user configuration file.
pub fn user_config_path() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".lattice.toml"))
}

/// Returns the path to the repository configuration file.
pub fn repo_config_path(repo_root: &Path) -> PathBuf {
    repo_root.join(".lattice").join("config.toml")
}

/// Validates configuration values are within acceptable ranges.
pub fn validate_config(config: &Config) -> Result<(), LatticeError> {
    if config.overview.limit == 0 {
        warn!("overview.limit is 0, no documents will be shown");
    }

    if config.overview.view_weight < 0.0
        || config.overview.recency_weight < 0.0
        || config.overview.root_weight < 0.0
    {
        return Err(LatticeError::ConfigValidationError {
            field: "overview weights".to_string(),
            reason: "Weights must be non-negative".to_string(),
        });
    }

    let weight_sum =
        config.overview.view_weight + config.overview.recency_weight + config.overview.root_weight;
    if (weight_sum - 1.0).abs() > 0.001 && weight_sum > 0.0 {
        debug!(weight_sum, "Overview weights do not sum to 1.0, will be normalized");
    }

    if config.defaults.priority > 4 {
        return Err(LatticeError::ConfigValidationError {
            field: "defaults.priority".to_string(),
            reason: "Priority must be 0-4".to_string(),
        });
    }

    if config.check.max_lines == 0 {
        warn!("check.max_lines is 0, all documents will trigger line count warning");
    }

    if config.format.line_width < 40 {
        warn!(line_width = config.format.line_width, "Line width is very narrow");
    }

    if !["error", "warn", "info", "debug", "trace"].contains(&config.logging.level.as_str()) {
        return Err(LatticeError::ConfigValidationError {
            field: "logging.level".to_string(),
            reason: format!(
                "Invalid log level '{}', expected one of: error, warn, info, debug, trace",
                config.logging.level
            ),
        });
    }

    Ok(())
}

/// Returns the user's home directory.
fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from).or({
        #[cfg(target_os = "windows")]
        {
            std::env::var_os("USERPROFILE").map(PathBuf::from)
        }
        #[cfg(not(target_os = "windows"))]
        {
            None
        }
    })
}
