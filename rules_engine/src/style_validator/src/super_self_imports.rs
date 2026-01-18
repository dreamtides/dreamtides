use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::visit_mut::VisitMut;
use syn::UseTree;

use crate::violation::{StyleViolation, ViolationKind};

pub struct SuperSelfImportChecker {
    violations: Vec<StyleViolation>,
    file_path: PathBuf,
}

impl SuperSelfImportChecker {
    pub fn new(file_path: PathBuf) -> Self {
        Self { violations: Vec::new(), file_path }
    }

    fn get_line_column<T: Spanned>(&self, node: &T) -> (usize, usize) {
        let span = node.span();
        (span.start().line, span.start().column + 1)
    }

    fn add_violation<T: Spanned>(&mut self, node: &T, path_str: String) {
        let (line, column) = self.get_line_column(node);
        self.violations.push(StyleViolation {
            file: self.file_path.clone(),
            line,
            column,
            kind: ViolationKind::SuperOrSelfImport,
            path_str,
        });
    }

    fn check_use_tree(&mut self, tree: &UseTree, prefix: &str) {
        match tree {
            UseTree::Path(path) => {
                let ident = path.ident.to_string();
                let new_prefix =
                    if prefix.is_empty() { ident.clone() } else { format!("{prefix}::{ident}") };
                if ident == "super" || ident == "self" {
                    self.add_violation(tree, format!("use {new_prefix}::..."));
                }
                self.check_use_tree(&path.tree, &new_prefix);
            }
            UseTree::Group(group) => {
                for item in &group.items {
                    self.check_use_tree(item, prefix);
                }
            }
            UseTree::Name(_) | UseTree::Rename(_) | UseTree::Glob(_) => {}
        }
    }

    pub fn violations(&self) -> &[StyleViolation] {
        &self.violations
    }
}

impl<'ast> Visit<'ast> for SuperSelfImportChecker {
    fn visit_item_use(&mut self, i: &'ast syn::ItemUse) {
        self.check_use_tree(&i.tree, "");
    }
}

pub fn check_file(path: &Path) -> Result<Vec<StyleViolation>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let syntax = syn::parse_file(&content)
        .with_context(|| format!("Failed to parse file: {}", path.display()))?;

    let mut checker = SuperSelfImportChecker::new(path.to_path_buf());
    for item in &syntax.items {
        checker.visit_item(item);
    }

    Ok(checker.violations().to_vec())
}

fn convert_use_tree_to_crate(tree: &UseTree, module_path: &[String]) -> Option<UseTree> {
    match tree {
        UseTree::Path(path) => {
            let ident = path.ident.to_string();
            if ident == "super" {
                if module_path.is_empty() {
                    return None;
                }
                let parent_path = &module_path[..module_path.len() - 1];
                convert_use_tree_with_prefix(&path.tree, parent_path)
            } else if ident == "self" {
                convert_use_tree_with_prefix(&path.tree, module_path)
            } else {
                let new_tree = convert_use_tree_to_crate(&path.tree, module_path)?;
                Some(UseTree::Path(syn::UsePath {
                    ident: path.ident.clone(),
                    colon2_token: path.colon2_token,
                    tree: Box::new(new_tree),
                }))
            }
        }
        UseTree::Group(group) => {
            let new_items: Vec<_> = group
                .items
                .iter()
                .filter_map(|item| convert_use_tree_to_crate(item, module_path))
                .collect();
            if new_items.is_empty() {
                None
            } else {
                Some(UseTree::Group(syn::UseGroup {
                    brace_token: group.brace_token,
                    items: new_items.into_iter().collect(),
                }))
            }
        }
        UseTree::Name(_) | UseTree::Rename(_) | UseTree::Glob(_) => Some(tree.clone()),
    }
}

fn convert_use_tree_with_prefix(tree: &UseTree, prefix_segments: &[String]) -> Option<UseTree> {
    let inner_tree = match tree {
        UseTree::Path(path) => {
            let ident = path.ident.to_string();
            if ident == "super" {
                if prefix_segments.is_empty() {
                    return None;
                }
                return convert_use_tree_with_prefix(
                    &path.tree,
                    &prefix_segments[..prefix_segments.len() - 1],
                );
            } else if ident == "self" {
                return convert_use_tree_with_prefix(&path.tree, prefix_segments);
            }
            tree.clone()
        }
        _ => tree.clone(),
    };

    if prefix_segments.is_empty() {
        Some(UseTree::Path(syn::UsePath {
            ident: syn::Ident::new("crate", proc_macro2::Span::call_site()),
            colon2_token: syn::token::PathSep::default(),
            tree: Box::new(inner_tree),
        }))
    } else {
        let mut result = inner_tree;
        for segment in prefix_segments.iter().rev() {
            result = UseTree::Path(syn::UsePath {
                ident: syn::Ident::new(segment, proc_macro2::Span::call_site()),
                colon2_token: syn::token::PathSep::default(),
                tree: Box::new(result),
            });
        }
        Some(UseTree::Path(syn::UsePath {
            ident: syn::Ident::new("crate", proc_macro2::Span::call_site()),
            colon2_token: syn::token::PathSep::default(),
            tree: Box::new(result),
        }))
    }
}

fn find_crate_root(path: &Path) -> Option<PathBuf> {
    let mut current = path.parent()?;
    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            return Some(current.to_path_buf());
        }
        current = current.parent()?;
    }
}

fn get_module_path_from_file(path: &Path, _rules_engine_path: &Path) -> Vec<String> {
    let Some(crate_root) = find_crate_root(path) else {
        return Vec::new();
    };

    let src_dir = crate_root.join("src");
    let relative = path.strip_prefix(&src_dir).ok();
    let Some(relative) = relative else {
        return Vec::new();
    };

    let mut segments = Vec::new();
    for component in relative.components() {
        if let std::path::Component::Normal(os_str) = component {
            if let Some(s) = os_str.to_str() {
                let name = s.trim_end_matches(".rs");
                if name != "mod" && name != "lib" {
                    segments.push(name.to_string());
                }
            }
        }
    }

    segments
}

fn contains_super_or_self(tree: &UseTree) -> bool {
    match tree {
        UseTree::Path(path) => {
            let ident = path.ident.to_string();
            ident == "super" || ident == "self" || contains_super_or_self(&path.tree)
        }
        UseTree::Group(group) => group.items.iter().any(contains_super_or_self),
        UseTree::Name(_) | UseTree::Rename(_) | UseTree::Glob(_) => false,
    }
}

struct SuperSelfImportFixer {
    module_path: Vec<String>,
}

impl SuperSelfImportFixer {
    fn new(module_path: Vec<String>) -> Self {
        Self { module_path }
    }

    fn fix_item_use(&self, item_use: &mut syn::ItemUse) {
        if contains_super_or_self(&item_use.tree) {
            if let Some(new_tree) = convert_use_tree_to_crate(&item_use.tree, &self.module_path) {
                item_use.tree = new_tree;
            }
        }
    }
}

impl VisitMut for SuperSelfImportFixer {
    fn visit_item_use_mut(&mut self, i: &mut syn::ItemUse) {
        self.fix_item_use(i);
    }

    fn visit_item_mod_mut(&mut self, i: &mut syn::ItemMod) {
        if let Some((_, ref mut content)) = i.content {
            let mut nested_fixer = SuperSelfImportFixer::new({
                let mut path = self.module_path.clone();
                path.push(i.ident.to_string());
                path
            });
            for item in content.iter_mut() {
                nested_fixer.visit_item_mut(item);
            }
        }
    }
}

pub fn fix_file(path: &Path, rules_engine_path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let mut syntax = syn::parse_file(&content)
        .with_context(|| format!("Failed to parse file: {}", path.display()))?;

    let module_path = get_module_path_from_file(path, rules_engine_path);
    let mut fixer = SuperSelfImportFixer::new(module_path);

    for item in &mut syntax.items {
        fixer.visit_item_mut(item);
    }

    let output = prettyplease::unparse(&syntax);

    std::fs::write(path, output)
        .with_context(|| format!("Failed to write file: {}", path.display()))?;

    Ok(())
}
