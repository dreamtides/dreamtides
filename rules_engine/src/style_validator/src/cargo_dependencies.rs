use std::path::Path;

use anyhow::{Context, Result};

use crate::violation::{StyleViolation, ViolationKind};

#[derive(Debug)]
struct Dependency {
    name: String,
    line: usize,
    is_internal: bool,
}

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

    let mut dependencies_list = Vec::new();

    for (name, value) in deps_table {
        if let Some(line_number) = find_dependency_line(&content, name) {
            let is_internal = value.as_table().and_then(|t| t.get("path")).is_some();

            dependencies_list.push(Dependency {
                name: name.clone(),
                line: line_number,
                is_internal,
            });
        }
    }

    dependencies_list.sort_by_key(|d| d.line);

    validate_dependency_order(path, &dependencies_list)
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

fn validate_dependency_order(
    path: &Path,
    dependencies: &[Dependency],
) -> Result<Vec<StyleViolation>> {
    let mut violations = Vec::new();

    let internal_deps: Vec<_> = dependencies.iter().filter(|d| d.is_internal).collect();
    let external_deps: Vec<_> = dependencies.iter().filter(|d| !d.is_internal).collect();

    if !is_alphabetized(&internal_deps) {
        if let Some(first_internal) = internal_deps.first() {
            violations.push(StyleViolation {
                file: path.to_path_buf(),
                line: first_internal.line,
                column: 1,
                kind: ViolationKind::CargoTomlDependencyOrder,
                path_str: "Internal dependencies not alphabetized".to_string(),
            });
        }
    }

    if !external_deps.is_empty() && !internal_deps.is_empty() {
        if let Some(first_external) = external_deps.first() {
            if let Some(last_internal) = internal_deps.last() {
                if first_external.line < last_internal.line {
                    violations.push(StyleViolation {
                        file: path.to_path_buf(),
                        line: first_external.line,
                        column: 1,
                        kind: ViolationKind::CargoTomlDependencyOrder,
                        path_str: format!(
                            "External dependency '{}' appears before internal dependency '{}'",
                            first_external.name, last_internal.name
                        ),
                    });
                }
            }
        }
    }

    if !is_alphabetized(&external_deps) {
        if let Some(first_external) = external_deps.first() {
            violations.push(StyleViolation {
                file: path.to_path_buf(),
                line: first_external.line,
                column: 1,
                kind: ViolationKind::CargoTomlDependencyOrder,
                path_str: "External dependencies not alphabetized".to_string(),
            });
        }
    }

    Ok(violations)
}

fn is_alphabetized(deps: &[&Dependency]) -> bool {
    if deps.len() <= 1 {
        return true;
    }

    for window in deps.windows(2) {
        if window[0].name > window[1].name {
            return false;
        }
    }

    true
}
