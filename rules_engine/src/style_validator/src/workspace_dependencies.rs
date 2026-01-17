use std::path::Path;

use anyhow::{Context, Result};

use crate::violation::{StyleViolation, ViolationKind};

pub fn check_file(path: &Path) -> Result<Vec<StyleViolation>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let toml_value: toml::Value = toml::from_str(&content)
        .with_context(|| format!("Failed to parse TOML: {}", path.display()))?;

    let Some(dependencies) = toml_value.get("dependencies") else {
        return Ok(Vec::new());
    };

    let Some(deps_table) = dependencies.as_table() else {
        return Ok(Vec::new());
    };

    let mut violations = Vec::new();

    for (name, value) in deps_table {
        // Skip path dependencies (internal crates) - they don't need workspace = true
        if let Some(table) = value.as_table() {
            if table.get("path").is_some() {
                continue;
            }

            // Check if workspace = true is specified
            if let Some(workspace_value) = table.get("workspace") {
                if workspace_value.as_bool() == Some(true) {
                    continue;
                }
            }

            // If we get here, this is an external dependency without workspace = true
            if let Some(line_number) = find_dependency_line(&content, name) {
                violations.push(StyleViolation {
                    file: path.to_path_buf(),
                    line: line_number,
                    column: 1,
                    kind: ViolationKind::WorkspaceDependencyNotUsed,
                    path_str: name.clone(),
                });
            }
        } else if value.is_str() {
            // Simple string version specification (e.g., dep = "1.0")
            // This should use workspace = true
            if let Some(line_number) = find_dependency_line(&content, name) {
                violations.push(StyleViolation {
                    file: path.to_path_buf(),
                    line: line_number,
                    column: 1,
                    kind: ViolationKind::WorkspaceDependencyNotUsed,
                    path_str: name.clone(),
                });
            }
        }
    }

    Ok(violations)
}

fn find_dependency_line(content: &str, dep_name: &str) -> Option<usize> {
    content
        .lines()
        .enumerate()
        .find(|(_, line)| {
            let trimmed = line.trim();
            trimmed.starts_with(&format!("{dep_name} ="))
                || trimmed.starts_with(&format!("{dep_name}="))
        })
        .map(|(idx, _)| idx + 1)
}
