use std::fs;

use anyhow::{Context, Result, bail};

use crate::config::{self, Config};
pub fn run_config_get(key: &str) -> Result<()> {
    let config_path = config::get_config_path();
    if !config_path.exists() {
        bail!(
            "Configuration file not found: {}\n\
             Run 'llmc init' to create a new workspace.",
            config_path.display()
        );
    }
    let config = Config::load(&config_path)?;
    let value = get_config_value(&config, key)?;
    println!("{}", value);
    Ok(())
}
pub fn run_config_set(key: &str, value: &str) -> Result<()> {
    let config_path = config::get_config_path();
    if !config_path.exists() {
        bail!(
            "Configuration file not found: {}\n\
             Run 'llmc init' to create a new workspace.",
            config_path.display()
        );
    }
    validate_config_key(key)?;
    validate_config_value(key, value)?;
    set_config_value(&config_path, key, value)?;
    println!("✓ Configuration updated: {} = {}", key, value);
    let config = Config::load(&config_path)?;
    let actual_value = get_config_value(&config, key)?;
    println!("Verified: {}", actual_value);
    Ok(())
}
fn get_config_value(config: &Config, key: &str) -> Result<String> {
    let parts: Vec<&str> = key.split('.').collect();
    match parts.as_slice() {
        ["defaults", "model"] => Ok(config.defaults.model.clone()),
        ["defaults", "skip_permissions"] => Ok(config.defaults.skip_permissions.to_string()),
        ["defaults", "allowed_tools"] => Ok(config.defaults.allowed_tools.join(", ")),
        ["defaults", "patrol_interval_secs"] => {
            Ok(config.defaults.patrol_interval_secs.to_string())
        }
        ["defaults", "sound_on_review"] => Ok(config.defaults.sound_on_review.to_string()),
        ["workers", worker_name, "model"] => {
            let worker_config = config.get_worker(worker_name).with_context(|| {
                format!(
                    "Worker '{}' not found in configuration.\n\
                     Available workers: {}",
                    worker_name,
                    get_worker_names(config)
                )
            })?;
            Ok(worker_config
                .model
                .clone()
                .unwrap_or_else(|| format!("{} (from defaults)", config.defaults.model)))
        }
        ["workers", worker_name, "role_prompt"] => {
            let worker_config = config.get_worker(worker_name).with_context(|| {
                format!(
                    "Worker '{}' not found in configuration.\n\
                     Available workers: {}",
                    worker_name,
                    get_worker_names(config)
                )
            })?;
            Ok(worker_config.role_prompt.clone().unwrap_or_else(|| "(not set)".to_string()))
        }
        ["workers", worker_name, "excluded_from_pool"] => {
            let worker_config = config.get_worker(worker_name).with_context(|| {
                format!(
                    "Worker '{}' not found in configuration.\n\
                     Available workers: {}",
                    worker_name,
                    get_worker_names(config)
                )
            })?;
            Ok(worker_config.excluded_from_pool.to_string())
        }
        _ => {
            bail!(
                "Invalid configuration key: '{}'\n\
             \n\
             Valid keys:\n\
             Defaults:\n\
             - defaults.model\n\
             - defaults.skip_permissions\n\
             - defaults.allowed_tools\n\
             - defaults.patrol_interval_secs\n\
             - defaults.sound_on_review\n\
             \n\
             Worker-specific (replace <worker> with worker name):\n\
             - workers.<worker>.model\n\
             - workers.<worker>.role_prompt\n\
             - workers.<worker>.excluded_from_pool",
                key
            )
        }
    }
}
fn set_config_value(config_path: &std::path::Path, key: &str, value: &str) -> Result<()> {
    let config_content = fs::read_to_string(config_path).context("Failed to read config.toml")?;
    let parts: Vec<&str> = key.split('.').collect();
    let new_content = match parts.as_slice() {
        ["defaults", field] => set_defaults_field(&config_content, field, value)?,
        ["workers", worker_name, field] => {
            set_worker_field(&config_content, worker_name, field, value)?
        }
        _ => bail!("Invalid configuration key: '{}'", key),
    };
    fs::write(config_path, new_content).context("Failed to write config.toml")?;
    Ok(())
}
fn set_defaults_field(content: &str, field: &str, value: &str) -> Result<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines: Vec<String> = Vec::new();
    let mut in_defaults_section = false;
    let mut field_updated = false;
    for line in &lines {
        let trimmed = line.trim();
        if trimmed == "[defaults]" {
            in_defaults_section = true;
            new_lines.push(line.to_string());
            continue;
        }
        if in_defaults_section {
            if trimmed.starts_with('[') && trimmed != "[defaults]" {
                if !field_updated {
                    new_lines.push(format_field_line(field, value)?);
                    field_updated = true;
                }
                in_defaults_section = false;
            } else if trimmed.starts_with(&format!("{} =", field)) {
                new_lines.push(format_field_line(field, value)?);
                field_updated = true;
                continue;
            }
        }
        new_lines.push(line.to_string());
    }
    if in_defaults_section && !field_updated {
        new_lines.push(format_field_line(field, value)?);
    } else if !field_updated {
        let mut insert_pos = 0;
        for (i, line) in lines.iter().enumerate() {
            if line.trim() == "[defaults]" {
                insert_pos = i + 1;
                while insert_pos < lines.len() && !lines[insert_pos].trim().starts_with('[') {
                    insert_pos += 1;
                }
                break;
            }
        }
        if insert_pos == 0 {
            new_lines.insert(0, format_field_line(field, value)?);
        } else {
            new_lines.insert(insert_pos, format_field_line(field, value)?);
        }
    }
    Ok(new_lines.join("\n") + "\n")
}
fn set_worker_field(content: &str, worker_name: &str, field: &str, value: &str) -> Result<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines: Vec<String> = Vec::new();
    let section_header = format!("[workers.{}]", worker_name);
    let mut in_worker_section = false;
    let mut field_updated = false;
    let mut section_exists = false;
    for line in &lines {
        let trimmed = line.trim();
        if trimmed == section_header {
            in_worker_section = true;
            section_exists = true;
            new_lines.push(line.to_string());
            continue;
        }
        if in_worker_section {
            if trimmed.starts_with('[') && trimmed != section_header {
                if !field_updated {
                    new_lines.push(format_field_line(field, value)?);
                    field_updated = true;
                }
                in_worker_section = false;
            } else if trimmed.starts_with(&format!("{} =", field)) {
                new_lines.push(format_field_line(field, value)?);
                field_updated = true;
                continue;
            }
        }
        new_lines.push(line.to_string());
    }
    if !section_exists {
        bail!(
            "Worker '{}' not found in configuration.\n\
             Use 'llmc add {}' to create this worker first.",
            worker_name,
            worker_name
        );
    }
    if in_worker_section && !field_updated {
        new_lines.push(format_field_line(field, value)?);
    }
    Ok(new_lines.join("\n") + "\n")
}
fn format_field_line(field: &str, value: &str) -> Result<String> {
    match field {
        "model" | "role_prompt" => Ok(format!("{} = \"{}\"", field, value)),
        "skip_permissions" | "sound_on_review" | "excluded_from_pool" => {
            let bool_value = value.parse::<bool>().with_context(|| {
                format!("Invalid boolean value: '{}'. Use 'true' or 'false'.", value)
            })?;
            Ok(format!("{} = {}", field, bool_value))
        }
        "patrol_interval_secs" => {
            let int_value = value.parse::<u32>().with_context(|| {
                format!("Invalid integer value: '{}'. Must be a positive number.", value)
            })?;
            Ok(format!("{} = {}", field, int_value))
        }
        "allowed_tools" => {
            let tools: Vec<&str> = value.split(',').map(str::trim).collect();
            let formatted_tools: Vec<String> = tools.iter().map(|t| format!("\"{}\"", t)).collect();
            Ok(format!("{} = [{}]", field, formatted_tools.join(", ")))
        }
        _ => bail!("Unknown field: '{}'", field),
    }
}
fn validate_config_key(key: &str) -> Result<()> {
    let parts: Vec<&str> = key.split('.').collect();
    match parts.as_slice() {
        ["defaults", field] => {
            let valid_fields = [
                "model",
                "skip_permissions",
                "allowed_tools",
                "patrol_interval_secs",
                "sound_on_review",
            ];
            if !valid_fields.contains(field) {
                bail!(
                    "Invalid defaults field: '{}'\n\
                     Valid fields: {}",
                    field,
                    valid_fields.join(", ")
                );
            }
            Ok(())
        }
        ["workers", _worker_name, field] => {
            let valid_fields = ["model", "role_prompt", "excluded_from_pool"];
            if !valid_fields.contains(field) {
                bail!(
                    "Invalid worker field: '{}'\n\
                     Valid fields: {}",
                    field,
                    valid_fields.join(", ")
                );
            }
            Ok(())
        }
        _ => {
            bail!(
                "Invalid configuration key format: '{}'\n\
             Expected formats:\n\
             - defaults.<field>\n\
             - workers.<worker>.<field>",
                key
            )
        }
    }
}
fn validate_config_value(key: &str, value: &str) -> Result<()> {
    let parts: Vec<&str> = key.split('.').collect();
    match parts.last() {
        Some(&"model") => {
            config::validate_model(value)?;
        }
        Some(&"skip_permissions" | &"sound_on_review" | &"excluded_from_pool") => {
            value.parse::<bool>().with_context(|| {
                format!("Invalid boolean value: '{}'. Use 'true' or 'false'.", value)
            })?;
        }
        Some(&"patrol_interval_secs") => {
            let interval = value.parse::<u32>().with_context(|| {
                format!("Invalid integer value: '{}'. Must be a positive number.", value)
            })?;
            if interval == 0 {
                bail!("patrol_interval_secs must be greater than 0");
            }
        }
        _ => {}
    }
    Ok(())
}
fn get_worker_names(config: &Config) -> String {
    let mut names: Vec<&String> = config.workers.keys().collect();
    names.sort();
    if names.is_empty() {
        "(none)".to_string()
    } else {
        names.into_iter().map(String::as_str).collect::<Vec<_>>().join(", ")
    }
}
