use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env, fs};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;

use crate::cli::color_theme;
use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::maintenance_args::{SetupArgs, SetupCommand};
use crate::error::error_types::LatticeError;

/// Name of the MCP server entry in Claude settings.
const MCP_SERVER_NAME: &str = "lattice";

/// Executes the `lat setup` command.
pub fn execute(context: CommandContext, args: SetupArgs) -> LatticeResult<()> {
    match args.command {
        SetupCommand::Claude { check, remove, project } => {
            execute_claude_setup(&context, check, remove, project)
        }
    }
}

/// MCP server configuration entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct McpServerConfig {
    #[serde(rename = "type", default = "default_server_type")]
    server_type: String,
    command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    args: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    env: HashMap<String, String>,
}

fn default_server_type() -> String {
    "stdio".to_string()
}

/// Project-level MCP configuration (`.mcp.json`).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ProjectMcpConfig {
    #[serde(default, rename = "mcpServers", skip_serializing_if = "HashMap::is_empty")]
    mcp_servers: HashMap<String, McpServerConfig>,
    #[serde(flatten)]
    other: HashMap<String, Value>,
}

/// Executes the `lat setup claude` subcommand.
fn execute_claude_setup(
    context: &CommandContext,
    check: bool,
    remove: bool,
    project: bool,
) -> LatticeResult<()> {
    info!(check, remove, project, "Executing setup claude command");

    if check && remove {
        return Err(LatticeError::ConflictingOptions {
            option1: "--check".to_string(),
            option2: "--remove".to_string(),
        });
    }

    if project {
        let mcp_json_path = context.repo_root.join(".mcp.json");
        if check {
            check_project_installation(context, &mcp_json_path)
        } else if remove {
            remove_project_installation(context, &mcp_json_path)
        } else {
            install_project(context, &mcp_json_path)
        }
    } else {
        let claude_json_path = get_claude_json_path()?;
        if check {
            check_user_installation(context, &claude_json_path)
        } else if remove {
            remove_user_installation(context, &claude_json_path)
        } else {
            install_user(context, &claude_json_path)
        }
    }
}

/// Returns the path to ~/.claude.json.
fn get_claude_json_path() -> LatticeResult<PathBuf> {
    let home = env::var("HOME").map_err(|_| LatticeError::ReadError {
        path: PathBuf::from("~"),
        reason: "HOME environment variable not set".to_string(),
    })?;
    Ok(PathBuf::from(home).join(".claude.json"))
}

/// Returns the absolute path to the lat binary by finding it in PATH.
fn get_lat_binary_path() -> LatticeResult<PathBuf> {
    let output = std::process::Command::new("which").arg("lat").output().map_err(|e| {
        LatticeError::OperationNotAllowed { reason: format!("Failed to run 'which lat': {e}") }
    })?;

    if !output.status.success() {
        return Err(LatticeError::OperationNotAllowed {
            reason:
                "'lat' command not found in PATH. Please ensure lat is installed and in your PATH."
                    .to_string(),
        });
    }

    let path_str = String::from_utf8_lossy(&output.stdout);
    Ok(PathBuf::from(path_str.trim()))
}

/// Creates the MCP server configuration for lattice.
fn create_mcp_config(lat_binary: &Path) -> McpServerConfig {
    McpServerConfig {
        server_type: "stdio".to_string(),
        command: lat_binary.display().to_string(),
        args: vec!["mcp".to_string()],
        env: HashMap::new(),
    }
}

/// Checks the user-level installation status in ~/.claude.json.
fn check_user_installation(context: &CommandContext, path: &Path) -> LatticeResult<()> {
    if !path.exists() {
        return print_not_installed(context, path, "settings_file_missing");
    }

    let content = fs::read_to_string(path)
        .map_err(|e| LatticeError::ReadError { path: path.to_path_buf(), reason: e.to_string() })?;

    let json: Value =
        serde_json::from_str(&content).map_err(|e| LatticeError::ConfigParseError {
            path: path.to_path_buf(),
            reason: format!("Invalid JSON: {e}"),
        })?;

    let Some(mcp_servers) = json.get("mcpServers") else {
        return print_not_installed(context, path, "mcpServers_missing");
    };

    let Some(config) = mcp_servers.get(MCP_SERVER_NAME) else {
        return print_not_installed(context, path, "entry_missing");
    };

    check_config_match(context, path, config)
}

/// Checks the project-level installation status in .mcp.json.
fn check_project_installation(context: &CommandContext, path: &Path) -> LatticeResult<()> {
    if !path.exists() {
        return print_not_installed(context, path, "mcp_json_missing");
    }

    let content = fs::read_to_string(path)
        .map_err(|e| LatticeError::ReadError { path: path.to_path_buf(), reason: e.to_string() })?;

    let config: ProjectMcpConfig =
        serde_json::from_str(&content).map_err(|e| LatticeError::ConfigParseError {
            path: path.to_path_buf(),
            reason: format!("Invalid JSON: {e}"),
        })?;

    let Some(server_config) = config.mcp_servers.get(MCP_SERVER_NAME) else {
        return print_not_installed(context, path, "entry_missing");
    };

    let config_value = serde_json::to_value(server_config).unwrap_or(Value::Null);
    check_config_match(context, path, &config_value)
}

fn print_not_installed(context: &CommandContext, path: &Path, reason: &str) -> LatticeResult<()> {
    if context.global.json {
        println!(
            "{}",
            serde_json::json!({
                "installed": false,
                "reason": reason,
                "settings_path": path.display().to_string(),
            })
        );
    } else {
        println!("{} Lattice MCP not installed ({})", color_theme::error("✗"), path.display());
    }
    Ok(())
}

fn check_config_match(context: &CommandContext, path: &Path, config: &Value) -> LatticeResult<()> {
    let current_binary = get_lat_binary_path()?;
    let configured_command = config.get("command").and_then(|v| v.as_str()).unwrap_or("");
    let configured_binary = PathBuf::from(configured_command);

    let version_match = current_binary == configured_binary;

    if context.global.json {
        println!(
            "{}",
            serde_json::json!({
                "installed": true,
                "settings_path": path.display().to_string(),
                "configured_command": configured_command,
                "current_binary": current_binary.display().to_string(),
                "version_match": version_match,
            })
        );
    } else if version_match {
        println!("{} Lattice MCP installed and up to date", color_theme::success("✓"));
        println!("  Settings: {}", path.display());
        println!("  Command: {}", configured_command);
    } else {
        println!("{} Lattice MCP installed but command path differs", color_theme::warning("!"));
        println!("  Settings: {}", path.display());
        println!("  Configured: {}", configured_command);
        println!("  Current: {}", current_binary.display());
        println!("\nRun 'lat setup claude' to update the configuration.");
    }

    Ok(())
}

/// Removes the user-level Lattice MCP installation from ~/.claude.json.
fn remove_user_installation(context: &CommandContext, path: &Path) -> LatticeResult<()> {
    if !path.exists() {
        return print_not_removed(context, "not_installed");
    }

    let content = fs::read_to_string(path)
        .map_err(|e| LatticeError::ReadError { path: path.to_path_buf(), reason: e.to_string() })?;

    let mut json: Value =
        serde_json::from_str(&content).map_err(|e| LatticeError::ConfigParseError {
            path: path.to_path_buf(),
            reason: format!("Invalid JSON: {e}"),
        })?;

    let Some(mcp_servers) = json.get_mut("mcpServers").and_then(|v| v.as_object_mut()) else {
        return print_not_removed(context, "not_installed");
    };

    if mcp_servers.remove(MCP_SERVER_NAME).is_none() {
        return print_not_removed(context, "not_installed");
    }

    let content = serde_json::to_string_pretty(&json).map_err(|e| LatticeError::WriteError {
        path: path.to_path_buf(),
        reason: format!("Failed to serialize: {e}"),
    })?;

    fs::write(path, content).map_err(|e| LatticeError::WriteError {
        path: path.to_path_buf(),
        reason: e.to_string(),
    })?;

    print_removed(context, path)
}

/// Removes the project-level Lattice MCP installation from .mcp.json.
fn remove_project_installation(context: &CommandContext, path: &Path) -> LatticeResult<()> {
    if !path.exists() {
        return print_not_removed(context, "not_installed");
    }

    let content = fs::read_to_string(path)
        .map_err(|e| LatticeError::ReadError { path: path.to_path_buf(), reason: e.to_string() })?;

    let mut config: ProjectMcpConfig =
        serde_json::from_str(&content).map_err(|e| LatticeError::ConfigParseError {
            path: path.to_path_buf(),
            reason: format!("Invalid JSON: {e}"),
        })?;

    if config.mcp_servers.remove(MCP_SERVER_NAME).is_none() {
        return print_not_removed(context, "not_installed");
    }

    let content = serde_json::to_string_pretty(&config).map_err(|e| LatticeError::WriteError {
        path: path.to_path_buf(),
        reason: format!("Failed to serialize: {e}"),
    })?;

    fs::write(path, content).map_err(|e| LatticeError::WriteError {
        path: path.to_path_buf(),
        reason: e.to_string(),
    })?;

    print_removed(context, path)
}

fn print_not_removed(context: &CommandContext, reason: &str) -> LatticeResult<()> {
    if context.global.json {
        println!(
            "{}",
            serde_json::json!({
                "removed": false,
                "reason": reason,
            })
        );
    } else {
        println!("{} Lattice MCP was not installed", color_theme::muted("·"));
    }
    Ok(())
}

fn print_removed(context: &CommandContext, path: &Path) -> LatticeResult<()> {
    info!(path = %path.display(), "Removed Lattice MCP configuration");

    if context.global.json {
        println!(
            "{}",
            serde_json::json!({
                "removed": true,
                "settings_path": path.display().to_string(),
            })
        );
    } else {
        println!("{} Removed Lattice MCP from {}", color_theme::success("✓"), path.display());
    }
    Ok(())
}

/// Installs Lattice as a user-level MCP server in ~/.claude.json.
fn install_user(context: &CommandContext, path: &Path) -> LatticeResult<()> {
    let lat_binary = get_lat_binary_path()?;
    let config = create_mcp_config(&lat_binary);

    let mut json: Value = if path.exists() {
        let content = fs::read_to_string(path).map_err(|e| LatticeError::ReadError {
            path: path.to_path_buf(),
            reason: e.to_string(),
        })?;
        serde_json::from_str(&content).map_err(|e| LatticeError::ConfigParseError {
            path: path.to_path_buf(),
            reason: format!("Invalid JSON: {e}"),
        })?
    } else {
        serde_json::json!({})
    };

    let mcp_servers = json
        .as_object_mut()
        .ok_or_else(|| LatticeError::ConfigParseError {
            path: path.to_path_buf(),
            reason: "Expected JSON object".to_string(),
        })?
        .entry("mcpServers")
        .or_insert_with(|| serde_json::json!({}));

    let was_installed = mcp_servers.get(MCP_SERVER_NAME).is_some();

    let config_value = serde_json::to_value(&config).map_err(|e| LatticeError::WriteError {
        path: path.to_path_buf(),
        reason: format!("Failed to serialize config: {e}"),
    })?;

    mcp_servers
        .as_object_mut()
        .ok_or_else(|| LatticeError::ConfigParseError {
            path: path.to_path_buf(),
            reason: "mcpServers is not an object".to_string(),
        })?
        .insert(MCP_SERVER_NAME.to_string(), config_value);

    let content = serde_json::to_string_pretty(&json).map_err(|e| LatticeError::WriteError {
        path: path.to_path_buf(),
        reason: format!("Failed to serialize: {e}"),
    })?;

    fs::write(path, content).map_err(|e| LatticeError::WriteError {
        path: path.to_path_buf(),
        reason: e.to_string(),
    })?;

    print_installed(context, path, &lat_binary, was_installed)
}

/// Installs Lattice as a project-level MCP server in .mcp.json.
fn install_project(context: &CommandContext, path: &Path) -> LatticeResult<()> {
    let lat_binary = get_lat_binary_path()?;
    let config = create_mcp_config(&lat_binary);

    let mut mcp_config: ProjectMcpConfig = if path.exists() {
        let content = fs::read_to_string(path).map_err(|e| LatticeError::ReadError {
            path: path.to_path_buf(),
            reason: e.to_string(),
        })?;
        serde_json::from_str(&content).map_err(|e| LatticeError::ConfigParseError {
            path: path.to_path_buf(),
            reason: format!("Invalid JSON: {e}"),
        })?
    } else {
        ProjectMcpConfig::default()
    };

    let was_installed = mcp_config.mcp_servers.contains_key(MCP_SERVER_NAME);
    mcp_config.mcp_servers.insert(MCP_SERVER_NAME.to_string(), config);

    let content =
        serde_json::to_string_pretty(&mcp_config).map_err(|e| LatticeError::WriteError {
            path: path.to_path_buf(),
            reason: format!("Failed to serialize: {e}"),
        })?;

    fs::write(path, content).map_err(|e| LatticeError::WriteError {
        path: path.to_path_buf(),
        reason: e.to_string(),
    })?;

    print_installed(context, path, &lat_binary, was_installed)
}

fn print_installed(
    context: &CommandContext,
    path: &Path,
    lat_binary: &Path,
    was_installed: bool,
) -> LatticeResult<()> {
    info!(
        path = %path.display(),
        binary = %lat_binary.display(),
        "Installed Lattice MCP configuration"
    );

    if context.global.json {
        println!(
            "{}",
            serde_json::json!({
                "installed": true,
                "updated": was_installed,
                "settings_path": path.display().to_string(),
                "command": lat_binary.display().to_string(),
            })
        );
    } else if was_installed {
        println!("{} Updated Lattice MCP in {}", color_theme::success("✓"), path.display());
    } else {
        println!("{} Installed Lattice MCP in {}", color_theme::success("✓"), path.display());
    }

    if !context.global.json {
        println!("\nLattice MCP tools are now available in Claude Code:");
        println!("  • lattice_create_task - Create task documents");
        println!("  • lattice_create_document - Create knowledge base documents");
    }

    Ok(())
}
