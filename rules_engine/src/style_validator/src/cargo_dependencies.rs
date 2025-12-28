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

pub fn fix_file(path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let toml_value: toml::Value = toml::from_str(&content)
        .with_context(|| format!("Failed to parse TOML: {}", path.display()))?;

    let Some(dependencies) = toml_value.get("dependencies") else {
        return Ok(());
    };

    let Some(deps_table) = dependencies.as_table() else {
        return Ok(());
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

    let mut internal_deps: Vec<_> = dependencies_list.iter().filter(|d| d.is_internal).collect();
    let mut external_deps: Vec<_> = dependencies_list.iter().filter(|d| !d.is_internal).collect();

    internal_deps.sort_by(|a, b| a.name.cmp(&b.name));
    external_deps.sort_by(|a, b| a.name.cmp(&b.name));

    let mut sorted_deps = Vec::new();
    sorted_deps.extend(internal_deps);
    sorted_deps.extend(external_deps);

    let lines: Vec<_> = content.lines().collect();
    let mut new_content = String::new();
    let mut dep_lines_map = std::collections::HashMap::new();

    for dep in &dependencies_list {
        if let Some(line_text) = lines.get(dep.line - 1) {
            dep_lines_map.insert(&dep.name, *line_text);
        }
    }

    let mut in_dependencies_section = false;
    let mut first_dep_line = usize::MAX;
    let mut last_dep_line = 0;

    for dep in &dependencies_list {
        first_dep_line = first_dep_line.min(dep.line);
        last_dep_line = last_dep_line.max(dep.line);
    }

    let mut sorted_dep_lines = Vec::new();
    for dep in sorted_deps {
        if let Some(line) = dep_lines_map.get(&dep.name) {
            sorted_dep_lines.push(*line);
        }
    }

    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;

        if line_num == first_dep_line {
            in_dependencies_section = true;
            for dep_line in &sorted_dep_lines {
                new_content.push_str(dep_line);
                new_content.push('\n');
            }
        }

        if in_dependencies_section && line_num >= first_dep_line && line_num <= last_dep_line {
            continue;
        }

        if in_dependencies_section && line_num > last_dep_line {
            in_dependencies_section = false;
        }

        new_content.push_str(line);
        new_content.push('\n');
    }

    std::fs::write(path, new_content)
        .with_context(|| format!("Failed to write file: {}", path.display()))?;

    Ok(())
}
