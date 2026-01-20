use tracing::{debug, info, warn};

use crate::cli::command_dispatch::CommandContext;
use crate::cli::commands::doctor_command::doctor_types::{CheckCategory, CheckResult};
use crate::config::config_loader;
use crate::error::error_types::LatticeError;

/// Runs all configuration checks.
pub fn run_config_checks(context: &CommandContext) -> Result<Vec<CheckResult>, LatticeError> {
    let mut results = Vec::new();

    // 1. User config check (Warning severity)
    results.push(check_user_config());

    // 2. Repo config check (Warning severity)
    results.push(check_repo_config(&context.repo_root));

    // 3. Client ID check (Warning severity)
    results.push(check_client_id(context));

    // 4. Config values check (Warning severity)
    results.extend(check_config_values(&context.repo_root)?);

    Ok(results)
}

/// Checks that ~/.lattice.toml is parseable if present.
fn check_user_config() -> CheckResult {
    debug!("Checking user configuration file");

    let Some(path) = config_loader::user_config_path() else {
        info!("Could not determine home directory, skipping user config check");
        return CheckResult::info(
            CheckCategory::Config,
            "User Config",
            "Could not determine home directory",
        );
    };

    if !path.exists() {
        info!(path = %path.display(), "User config file not found");
        return CheckResult::info(
            CheckCategory::Config,
            "User Config",
            "No ~/.lattice.toml (using defaults)",
        );
    }

    match config_loader::load_user_config() {
        Ok(Some(_)) => {
            info!(path = %path.display(), "User config file is valid");
            CheckResult::passed(CheckCategory::Config, "User Config", "~/.lattice.toml valid")
        }
        Ok(None) => CheckResult::info(
            CheckCategory::Config,
            "User Config",
            "No ~/.lattice.toml (using defaults)",
        ),
        Err(LatticeError::ConfigParseError { reason, .. }) => {
            warn!(path = %path.display(), %reason, "User config file has parse errors");
            CheckResult::warning(
                CheckCategory::Config,
                "User Config",
                format!("~/.lattice.toml parse error: {reason}"),
            )
        }
        Err(LatticeError::ReadError { reason, .. }) => {
            warn!(path = %path.display(), %reason, "Could not read user config file");
            CheckResult::warning(
                CheckCategory::Config,
                "User Config",
                format!("Could not read ~/.lattice.toml: {reason}"),
            )
        }
        Err(e) => {
            warn!(path = %path.display(), ?e, "Unexpected error loading user config");
            CheckResult::warning(
                CheckCategory::Config,
                "User Config",
                format!("~/.lattice.toml error: {e}"),
            )
        }
    }
}

/// Checks that .lattice/config.toml is parseable if present.
fn check_repo_config(repo_root: &std::path::Path) -> CheckResult {
    debug!("Checking repository configuration file");

    let path = config_loader::repo_config_path(repo_root);
    if !path.exists() {
        info!(path = %path.display(), "Repository config file not found");
        return CheckResult::info(
            CheckCategory::Config,
            "Repo Config",
            "No .lattice/config.toml (using defaults)",
        );
    }

    match config_loader::load_repo_config(repo_root) {
        Ok(Some(_)) => {
            info!(path = %path.display(), "Repository config file is valid");
            CheckResult::passed(CheckCategory::Config, "Repo Config", ".lattice/config.toml valid")
        }
        Ok(None) => CheckResult::info(
            CheckCategory::Config,
            "Repo Config",
            "No .lattice/config.toml (using defaults)",
        ),
        Err(LatticeError::ConfigParseError { reason, .. }) => {
            warn!(path = %path.display(), %reason, "Repository config file has parse errors");
            CheckResult::warning(
                CheckCategory::Config,
                "Repo Config",
                format!(".lattice/config.toml parse error: {reason}"),
            )
        }
        Err(LatticeError::ReadError { reason, .. }) => {
            warn!(path = %path.display(), %reason, "Could not read repository config file");
            CheckResult::warning(
                CheckCategory::Config,
                "Repo Config",
                format!("Could not read .lattice/config.toml: {reason}"),
            )
        }
        Err(e) => {
            warn!(path = %path.display(), ?e, "Unexpected error loading repository config");
            CheckResult::warning(
                CheckCategory::Config,
                "Repo Config",
                format!(".lattice/config.toml error: {e}"),
            )
        }
    }
}

/// Checks that a client ID is assigned for this repository.
fn check_client_id(context: &CommandContext) -> CheckResult {
    debug!("Checking client ID assignment");

    match context.client_id_store.get(&context.repo_root) {
        Ok(Some(client_id)) => {
            info!(client_id = %client_id, "Client ID is assigned");
            CheckResult::passed(
                CheckCategory::Config,
                "Client ID",
                format!("Assigned: {client_id}"),
            )
        }
        Ok(None) => {
            warn!("No client ID assigned for this repository");
            CheckResult::warning(CheckCategory::Config, "Client ID", "No client ID assigned")
                .with_details(vec![
                    "A client ID is needed for generating unique document IDs".to_string(),
                ])
                .with_fix("lat doctor --fix")
        }
        Err(e) => {
            warn!(?e, "Could not read client ID");
            CheckResult::warning(
                CheckCategory::Config,
                "Client ID",
                format!("Could not read client ID: {e}"),
            )
            .with_fix("lat doctor --fix")
        }
    }
}

/// Checks that all configuration values are within valid ranges.
fn check_config_values(repo_root: &std::path::Path) -> Result<Vec<CheckResult>, LatticeError> {
    debug!("Validating configuration values");

    let config = match config_loader::load_config(Some(repo_root)) {
        Ok(config) => config,
        Err(e) => {
            warn!(?e, "Could not load configuration for validation");
            return Ok(vec![CheckResult::warning(
                CheckCategory::Config,
                "Config Values",
                format!("Could not load configuration: {e}"),
            )]);
        }
    };

    match config_loader::validate_config(&config) {
        Ok(()) => {
            info!("All configuration values are valid");
            Ok(vec![CheckResult::passed(
                CheckCategory::Config,
                "Config Values",
                "All values within valid ranges",
            )])
        }
        Err(LatticeError::ConfigValidationError { field, reason }) => {
            warn!(field = %field, reason = %reason, "Configuration validation failed");
            Ok(vec![CheckResult::warning(
                CheckCategory::Config,
                "Config Values",
                format!("Invalid value for '{field}': {reason}"),
            )])
        }
        Err(e) => {
            warn!(?e, "Unexpected error validating configuration");
            Ok(vec![CheckResult::warning(
                CheckCategory::Config,
                "Config Values",
                format!("Validation error: {e}"),
            )])
        }
    }
}
