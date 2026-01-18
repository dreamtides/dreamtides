use std::collections::HashMap;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::config::config_loader;
use crate::error::error_types::LatticeError;

/// Returns the client ID for the given repository path.
///
/// Looks up the client ID in ~/.lattice.toml. If the exact path is not found,
/// also checks for the git toplevel directory as a key.
pub fn get_client_id(repo_root: &Path) -> Result<Option<String>, LatticeError> {
    let Some(user_config) = config_loader::load_user_config()? else {
        debug!("No user config found, cannot determine client ID");
        return Ok(None);
    };
    let canonical = repo_root.canonicalize().map_err(|e| LatticeError::ReadError {
        path: repo_root.to_path_buf(),
        reason: format!("Failed to canonicalize path: {e}"),
    })?;
    if let Some(client_id) = user_config.clients.get(&canonical) {
        debug!(path = %canonical.display(), client_id, "Found client ID for exact path");
        return Ok(Some(client_id.clone()));
    }
    let mut sorted_paths: Vec<_> = user_config.clients.keys().collect();
    sorted_paths.sort();
    for configured_path in sorted_paths {
        let client_id = &user_config.clients[configured_path];
        if let Ok(configured_canonical) = configured_path.canonicalize()
            && configured_canonical == canonical
        {
            debug!(
                path = %canonical.display(),
                client_id,
                "Found client ID via canonical path match"
            );
            return Ok(Some(client_id.clone()));
        }
    }
    debug!(path = %canonical.display(), "No client ID configured for repository");
    Ok(None)
}

/// Sets the client ID for the given repository path.
///
/// Updates ~/.lattice.toml with the new client ID mapping.
pub fn set_client_id(repo_root: &Path, client_id: &str) -> Result<(), LatticeError> {
    validate_client_id(client_id)?;
    let config_path =
        config_loader::user_config_path().ok_or_else(|| LatticeError::WriteError {
            path: PathBuf::from("~/.lattice.toml"),
            reason: "Could not determine home directory".to_string(),
        })?;
    let mut user_config = config_loader::load_user_config()?.unwrap_or_default();
    let canonical = repo_root.canonicalize().map_err(|e| LatticeError::ReadError {
        path: repo_root.to_path_buf(),
        reason: format!("Failed to canonicalize path: {e}"),
    })?;
    user_config.clients.insert(canonical.clone(), client_id.to_string());
    let toml_content =
        toml::to_string_pretty(&user_config).map_err(|e| LatticeError::WriteError {
            path: config_path.clone(),
            reason: format!("Failed to serialize config: {e}"),
        })?;
    std::fs::write(&config_path, toml_content).map_err(|e| LatticeError::WriteError {
        path: config_path.clone(),
        reason: e.to_string(),
    })?;
    info!(path = %canonical.display(), client_id, "Saved client ID to user config");
    Ok(())
}

/// Validates that a client ID is well-formed.
///
/// Client IDs must be 3-6 uppercase Base32 characters (A-Z, 2-7).
pub fn validate_client_id(client_id: &str) -> Result<(), LatticeError> {
    if client_id.len() < 3 || client_id.len() > 6 {
        return Err(LatticeError::ConfigValidationError {
            field: "client_id".to_string(),
            reason: format!("Client ID must be 3-6 characters, got {} characters", client_id.len()),
        });
    }
    if !client_id.chars().all(|c| c.is_ascii_uppercase() || ('2'..='7').contains(&c)) {
        return Err(LatticeError::ConfigValidationError {
            field: "client_id".to_string(),
            reason: "Client ID must contain only uppercase letters A-Z and digits 2-7".to_string(),
        });
    }
    Ok(())
}

/// Returns all configured client IDs from the user config.
pub fn list_client_ids() -> Result<HashMap<PathBuf, String>, LatticeError> {
    let user_config = config_loader::load_user_config()?.unwrap_or_default();
    Ok(user_config.clients)
}

/// Removes the client ID configuration for the given repository path.
pub fn remove_client_id(repo_root: &Path) -> Result<bool, LatticeError> {
    let config_path =
        config_loader::user_config_path().ok_or_else(|| LatticeError::WriteError {
            path: PathBuf::from("~/.lattice.toml"),
            reason: "Could not determine home directory".to_string(),
        })?;
    let Some(mut user_config) = config_loader::load_user_config()? else {
        return Ok(false);
    };
    let canonical = repo_root.canonicalize().map_err(|e| LatticeError::ReadError {
        path: repo_root.to_path_buf(),
        reason: format!("Failed to canonicalize path: {e}"),
    })?;
    let removed = user_config.clients.remove(&canonical).is_some();
    if removed {
        let toml_content =
            toml::to_string_pretty(&user_config).map_err(|e| LatticeError::WriteError {
                path: config_path.clone(),
                reason: format!("Failed to serialize config: {e}"),
            })?;
        std::fs::write(&config_path, toml_content).map_err(|e| LatticeError::WriteError {
            path: config_path.clone(),
            reason: e.to_string(),
        })?;
        info!(path = %canonical.display(), "Removed client ID from user config");
    } else {
        warn!(path = %canonical.display(), "No client ID found to remove");
    }
    Ok(removed)
}

/// Generates a unique client ID based on the machine and user.
///
/// Uses a hash of hostname and username, encoded as Base32.
/// Returns a 3-character client ID (the minimum length per the ID system spec).
pub fn generate_client_id() -> String {
    let hostname = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    let username = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| Uuid::new_v4().to_string()[..8].to_string());
    let mut hasher = Sha256::new();
    hasher.update(hostname.as_bytes());
    hasher.update(username.as_bytes());
    let hash = hasher.finalize();
    let base32_chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut result = String::with_capacity(3);
    for &byte in hash.iter().take(3) {
        let index = (byte as usize) % 32;
        result.push(base32_chars.chars().nth(index).unwrap_or('A'));
    }
    result
}
