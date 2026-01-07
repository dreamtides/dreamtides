use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use syn::spanned::Spanned;
use syn::{Item, UseTree, Visibility};

use crate::violation::{StyleViolation, ViolationKind};

const ALLOWLIST: &[&str] = &["parser::parser_utils", "parser_v2::parser::parser_helpers"];

pub fn check_all_files(files: &[PathBuf], root: &Path) -> Result<Vec<StyleViolation>> {
    // First pass: collect all public functions
    let public_functions = collect_public_functions(files, root)?;

    // Second pass: check use statements
    let mut all_violations = Vec::new();

    for file in files {
        let violations = check_file_use_statements(file, root, &public_functions)?;
        all_violations.extend(violations);
    }

    Ok(all_violations)
}

fn collect_public_functions(files: &[PathBuf], root: &Path) -> Result<HashMap<String, PathBuf>> {
    let mut functions = HashMap::new();

    for file in files {
        let content = std::fs::read_to_string(file)
            .with_context(|| format!("Failed to read file: {}", file.display()))?;

        let syntax = syn::parse_file(&content)
            .with_context(|| format!("Failed to parse file: {}", file.display()))?;

        let module_path = file_to_module_path(file, root);

        for item in &syntax.items {
            if let Item::Fn(func) = item {
                if is_public(&func.vis) {
                    let func_name = func.sig.ident.to_string();
                    let full_path = if module_path.is_empty() {
                        func_name.clone()
                    } else {
                        format!("{module_path}::{func_name}")
                    };
                    functions.insert(full_path, file.clone());
                }
            }
        }
    }

    Ok(functions)
}

fn is_public(vis: &Visibility) -> bool {
    !matches!(vis, Visibility::Inherited)
}

fn file_to_module_path(file: &Path, root: &Path) -> String {
    let Ok(relative) = file.strip_prefix(root) else {
        return String::new();
    };

    let components: Vec<_> =
        relative.components().map(|c| c.as_os_str().to_str().unwrap()).collect();

    // Expected: src/<crate_name>/src/<module_path>.rs
    if components.len() < 4 {
        return String::new();
    }

    let crate_name = components[1];
    let module_components = &components[3..];

    let module_path = module_components.join("::");
    let module_path = module_path.strip_suffix(".rs").unwrap_or(&module_path);
    let module_path = if module_path.ends_with("::mod") {
        module_path.strip_suffix("::mod").unwrap()
    } else if module_path.ends_with("::lib") {
        module_path.strip_suffix("::lib").unwrap()
    } else if module_path.ends_with("::main") {
        module_path.strip_suffix("::main").unwrap()
    } else {
        module_path
    };

    if module_path.is_empty() {
        crate_name.to_string()
    } else {
        format!("{crate_name}::{module_path}")
    }
}

fn get_crate_name(file: &Path, root: &Path) -> String {
    let Ok(relative) = file.strip_prefix(root) else {
        return String::new();
    };

    let components: Vec<_> = relative.components().collect();

    // Expected: src/<crate_name>/src/<module_path>.rs
    if components.len() >= 2 {
        components[1].as_os_str().to_str().unwrap().to_string()
    } else {
        String::new()
    }
}

fn check_file_use_statements(
    file: &Path,
    root: &Path,
    public_functions: &HashMap<String, PathBuf>,
) -> Result<Vec<StyleViolation>> {
    let content = std::fs::read_to_string(file)
        .with_context(|| format!("Failed to read file: {}", file.display()))?;

    let syntax = syn::parse_file(&content)
        .with_context(|| format!("Failed to parse file: {}", file.display()))?;

    let current_crate = get_crate_name(file, root);
    let current_module = file_to_module_path(file, root);
    let mut violations = Vec::new();

    for item in &syntax.items {
        if let Item::Use(use_item) = item {
            check_use_tree(
                &use_item.tree,
                public_functions,
                &mut violations,
                file,
                &current_crate,
                &current_module,
                String::new(),
            );
        }
    }

    Ok(violations)
}

fn check_use_tree(
    tree: &UseTree,
    public_functions: &HashMap<String, PathBuf>,
    violations: &mut Vec<StyleViolation>,
    current_file: &Path,
    current_crate: &str,
    current_module: &str,
    path_so_far: String,
) {
    match tree {
        UseTree::Path(use_path) => {
            let segment = use_path.ident.to_string();

            let new_path = if segment == "crate" {
                current_crate.to_string()
            } else if segment == "super" {
                // Resolve super to the parent module
                let parent = if path_so_far.is_empty() {
                    // This is the first segment, so resolve based on current_module
                    current_module.rsplit_once("::").map(|(parent, _)| parent.to_string())
                        .unwrap_or_else(|| current_crate.to_string())
                } else {
                    // Already building a path, go up one level
                    path_so_far.rsplit_once("::").map(|(parent, _)| parent.to_string())
                        .unwrap_or_default()
                };
                parent
            } else if path_so_far.is_empty() {
                segment
            } else {
                format!("{path_so_far}::{segment}")
            };

            check_use_tree(
                &use_path.tree,
                public_functions,
                violations,
                current_file,
                current_crate,
                current_module,
                new_path,
            );
        }
        UseTree::Name(use_name) => {
            let name = use_name.ident.to_string();
            let full_path =
                if path_so_far.is_empty() { name } else { format!("{path_so_far}::{name}") };

            if public_functions.contains_key(&full_path) && !is_allowlisted(&full_path) {
                violations.push(StyleViolation {
                    file: current_file.to_path_buf(),
                    line: use_name.span().start().line,
                    column: use_name.span().start().column + 1,
                    kind: ViolationKind::DirectFunctionImport,
                    path_str: full_path,
                });
            }
        }
        UseTree::Rename(use_rename) => {
            let name = use_rename.ident.to_string();
            let full_path =
                if path_so_far.is_empty() { name } else { format!("{path_so_far}::{name}") };

            if public_functions.contains_key(&full_path) && !is_allowlisted(&full_path) {
                violations.push(StyleViolation {
                    file: current_file.to_path_buf(),
                    line: use_rename.span().start().line,
                    column: use_rename.span().start().column + 1,
                    kind: ViolationKind::DirectFunctionImport,
                    path_str: full_path,
                });
            }
        }
        UseTree::Glob(_) => {
            // Skip glob imports (conservative)
        }
        UseTree::Group(use_group) => {
            for item in &use_group.items {
                check_use_tree(
                    item,
                    public_functions,
                    violations,
                    current_file,
                    current_crate,
                    current_module,
                    path_so_far.clone(),
                );
            }
        }
    }
}

fn is_allowlisted(full_path: &str) -> bool {
    ALLOWLIST
        .iter()
        .any(|allowed| full_path == *allowed || full_path.starts_with(&format!("{allowed}::")))
}
