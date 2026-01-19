use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env, fs};

use serde::{Deserialize, Serialize};
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
    command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    args: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    env: HashMap<String, String>,
}

/// Claude Code settings structure (partial, only what we need).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ClaudeSettings {
    #[serde(default, rename = "mcpServers", skip_serializing_if = "HashMap::is_empty")]
    mcp_servers: HashMap<String, McpServerConfig>,
    #[serde(flatten)]
    other: HashMap<String, serde_json::Value>,
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

    let settings_path = get_settings_path(&context.repo_root, project)?;

    if check {
        check_installation(context, &settings_path)
    } else if remove {
        remove_installation(context, &settings_path)
    } else {
        install(context, &settings_path, project)
    }
}

/// Returns the path to the Claude settings file.
fn get_settings_path(repo_root: &Path, project: bool) -> LatticeResult<PathBuf> {
    if project {
        Ok(repo_root.join(".claude").join("settings.local.json"))
    } else {
        let home = env::var("HOME").map_err(|_| LatticeError::ReadError {
            path: PathBuf::from("~"),
            reason: "HOME environment variable not set".to_string(),
        })?;
        Ok(PathBuf::from(home).join(".claude").join("settings.json"))
    }
}

/// Returns the path to the lat binary.
fn get_lat_binary_path() -> LatticeResult<PathBuf> {
    env::current_exe().map_err(|e| LatticeError::ReadError {
        path: PathBuf::from("lat"),
        reason: format!("Failed to determine lat binary path: {e}"),
    })
}

/// Checks the installation status.
fn check_installation(context: &CommandContext, settings_path: &Path) -> LatticeResult<()> {
    if !settings_path.exists() {
        if context.global.json {
            println!(
                "{}",
                serde_json::json!({
                    "installed": false,
                    "reason": "settings_file_missing",
                    "settings_path": settings_path.display().to_string(),
                })
            );
        } else {
            println!(
                "{} Lattice MCP not installed (settings file not found: {})",
                color_theme::error("✗"),
                settings_path.display()
            );
        }
        return Ok(());
    }

    let settings = read_settings(settings_path)?;

    let Some(config) = settings.mcp_servers.get(MCP_SERVER_NAME) else {
        if context.global.json {
            println!(
                "{}",
                serde_json::json!({
                    "installed": false,
                    "reason": "entry_missing",
                    "settings_path": settings_path.display().to_string(),
                })
            );
        } else {
            println!(
                "{} Lattice MCP not installed in {}",
                color_theme::error("✗"),
                settings_path.display()
            );
        }
        return Ok(());
    };

    let current_binary = get_lat_binary_path()?;
    let configured_binary = PathBuf::from(&config.command);

    let version_match = current_binary == configured_binary;

    if context.global.json {
        println!(
            "{}",
            serde_json::json!({
                "installed": true,
                "settings_path": settings_path.display().to_string(),
                "configured_command": config.command,
                "current_binary": current_binary.display().to_string(),
                "version_match": version_match,
            })
        );
    } else if version_match {
        println!("{} Lattice MCP installed and up to date", color_theme::success("✓"));
        println!("  Settings: {}", settings_path.display());
        println!("  Command: {}", config.command);
    } else {
        println!("{} Lattice MCP installed but command path differs", color_theme::warning("!"));
        println!("  Settings: {}", settings_path.display());
        println!("  Configured: {}", config.command);
        println!("  Current: {}", current_binary.display());
        println!("\nRun 'lat setup claude' to update the configuration.");
    }

    Ok(())
}

/// Removes the Lattice MCP installation.
fn remove_installation(context: &CommandContext, settings_path: &Path) -> LatticeResult<()> {
    if !settings_path.exists() {
        if context.global.json {
            println!(
                "{}",
                serde_json::json!({
                    "removed": false,
                    "reason": "not_installed",
                })
            );
        } else {
            println!("{} Lattice MCP was not installed", color_theme::muted("·"));
        }
        return Ok(());
    }

    let mut settings = read_settings(settings_path)?;

    if settings.mcp_servers.remove(MCP_SERVER_NAME).is_none() {
        if context.global.json {
            println!(
                "{}",
                serde_json::json!({
                    "removed": false,
                    "reason": "not_installed",
                })
            );
        } else {
            println!("{} Lattice MCP was not installed", color_theme::muted("·"));
        }
        return Ok(());
    }

    write_settings(settings_path, &settings)?;

    info!(path = %settings_path.display(), "Removed Lattice MCP configuration");

    if context.global.json {
        println!(
            "{}",
            serde_json::json!({
                "removed": true,
                "settings_path": settings_path.display().to_string(),
            })
        );
    } else {
        println!(
            "{} Removed Lattice MCP from {}",
            color_theme::success("✓"),
            settings_path.display()
        );
    }

    Ok(())
}

/// Installs Lattice as an MCP server.
fn install(context: &CommandContext, settings_path: &Path, project: bool) -> LatticeResult<()> {
    let lat_binary = get_lat_binary_path()?;

    let mut settings = if settings_path.exists() {
        read_settings(settings_path)?
    } else {
        ClaudeSettings::default()
    };

    let was_installed = settings.mcp_servers.contains_key(MCP_SERVER_NAME);

    let mut args = vec!["mcp".to_string()];
    if project {
        args.push("--repo".to_string());
        args.push(context.repo_root.display().to_string());
    }

    let config =
        McpServerConfig { command: lat_binary.display().to_string(), args, env: HashMap::new() };

    settings.mcp_servers.insert(MCP_SERVER_NAME.to_string(), config);

    if let Some(parent) = settings_path.parent() {
        fs::create_dir_all(parent).map_err(|e| LatticeError::WriteError {
            path: parent.to_path_buf(),
            reason: format!("Failed to create directory: {e}"),
        })?;
    }

    write_settings(settings_path, &settings)?;

    info!(
        path = %settings_path.display(),
        binary = %lat_binary.display(),
        "Installed Lattice MCP configuration"
    );

    if context.global.json {
        println!(
            "{}",
            serde_json::json!({
                "installed": true,
                "updated": was_installed,
                "settings_path": settings_path.display().to_string(),
                "command": lat_binary.display().to_string(),
            })
        );
    } else if was_installed {
        println!(
            "{} Updated Lattice MCP in {}",
            color_theme::success("✓"),
            settings_path.display()
        );
    } else {
        println!(
            "{} Installed Lattice MCP in {}",
            color_theme::success("✓"),
            settings_path.display()
        );
    }

    if !context.global.json {
        println!("\nLattice MCP tools are now available in Claude Code:");
        println!("  • lattice_create_task - Create task documents");
        println!("  • lattice_create_document - Create knowledge base documents");
    }

    Ok(())
}

/// Reads and parses the Claude settings file.
fn read_settings(path: &Path) -> LatticeResult<ClaudeSettings> {
    let content = fs::read_to_string(path)
        .map_err(|e| LatticeError::ReadError { path: path.to_path_buf(), reason: e.to_string() })?;

    serde_json::from_str(&content).map_err(|e| LatticeError::ConfigParseError {
        path: path.to_path_buf(),
        reason: format!("Invalid JSON: {e}"),
    })
}

/// Writes the Claude settings file.
fn write_settings(path: &Path, settings: &ClaudeSettings) -> LatticeResult<()> {
    let content = serde_json::to_string_pretty(settings).map_err(|e| LatticeError::WriteError {
        path: path.to_path_buf(),
        reason: format!("Failed to serialize settings: {e}"),
    })?;

    fs::write(path, content)
        .map_err(|e| LatticeError::WriteError { path: path.to_path_buf(), reason: e.to_string() })
}
