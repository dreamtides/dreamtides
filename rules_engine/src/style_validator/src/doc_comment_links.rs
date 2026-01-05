use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use regex::Regex;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Attribute, File, Item, UsePath, UseTree};

use crate::violation::{StyleViolation, ViolationKind};

pub struct DocCommentLinkChecker {
    violations: Vec<StyleViolation>,
    file_path: PathBuf,
    imported_types: HashSet<String>,
    local_types: HashSet<String>,
    doc_comment_refs: Vec<(String, usize, usize)>,
}

impl DocCommentLinkChecker {
    pub fn new(file_path: PathBuf) -> Self {
        let mut builtin_types = HashSet::new();
        // Add common Rust built-in types
        for ty in &[
            "String", "Vec", "Option", "Result", "Box", "Arc", "Rc", "HashMap", "HashSet",
            "BTreeMap", "BTreeSet", "Cell", "RefCell",
        ] {
            builtin_types.insert(ty.to_string());
        }

        Self {
            violations: Vec::new(),
            file_path,
            imported_types: builtin_types,
            local_types: HashSet::new(),
            doc_comment_refs: Vec::new(),
        }
    }

    fn get_line_column<T: Spanned>(&self, node: &T) -> (usize, usize) {
        let span = node.span();
        (span.start().line, span.start().column + 1)
    }

    fn add_violation(&mut self, line: usize, column: usize, kind: ViolationKind, path_str: String) {
        self.violations.push(StyleViolation {
            file: self.file_path.clone(),
            line,
            column,
            kind,
            path_str,
        });
    }

    pub fn violations(&self) -> &[StyleViolation] {
        &self.violations
    }

    fn extract_doc_comment_refs(&mut self, attrs: &[Attribute], line: usize) {
        let link_pattern = Regex::new(r"\[([A-Z][a-zA-Z0-9_]*)\]").unwrap();

        for attr in attrs {
            if attr.path().is_ident("doc") {
                if let syn::Meta::NameValue(meta) = &attr.meta {
                    if let syn::Expr::Lit(expr_lit) = &meta.value {
                        if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                            let comment = lit_str.value();
                            for cap in link_pattern.captures_iter(&comment) {
                                if let Some(type_name) = cap.get(1) {
                                    self.doc_comment_refs.push((
                                        type_name.as_str().to_string(),
                                        line,
                                        1,
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn collect_imported_types(&mut self, use_tree: &UseTree, prefix: String) {
        match use_tree {
            UseTree::Path(UsePath { ident, tree, .. }) => {
                let new_prefix = if prefix.is_empty() {
                    ident.to_string()
                } else {
                    format!("{}::{}", prefix, ident)
                };
                self.collect_imported_types(tree, new_prefix);
            }
            UseTree::Name(name) => {
                // Extract just the final identifier as the type name
                let type_name = name.ident.to_string();
                self.imported_types.insert(type_name);
            }
            UseTree::Rename(rename) => {
                // Use the renamed identifier
                let type_name = rename.rename.to_string();
                self.imported_types.insert(type_name);
            }
            UseTree::Glob(_) => {
                // Can't validate glob imports
            }
            UseTree::Group(group) => {
                for tree in &group.items {
                    self.collect_imported_types(tree, prefix.clone());
                }
            }
        }
    }
}

impl<'ast> Visit<'ast> for DocCommentLinkChecker {
    fn visit_file(&mut self, i: &'ast File) {
        // First pass: collect all imported types and local types
        for item in &i.items {
            match item {
                Item::Use(item_use) => {
                    self.collect_imported_types(&item_use.tree, String::new());
                }
                Item::Struct(s) => {
                    self.local_types.insert(s.ident.to_string());
                }
                Item::Enum(e) => {
                    self.local_types.insert(e.ident.to_string());
                }
                Item::Trait(t) => {
                    self.local_types.insert(t.ident.to_string());
                }
                Item::Type(t) => {
                    self.local_types.insert(t.ident.to_string());
                }
                _ => {}
            }
        }

        // Second pass: collect doc comment references
        for item in &i.items {
            let (attrs, line) = match item {
                Item::Const(i) => (&i.attrs, self.get_line_column(i).0),
                Item::Enum(i) => (&i.attrs, self.get_line_column(i).0),
                Item::Fn(i) => (&i.attrs, self.get_line_column(i).0),
                Item::Mod(i) => (&i.attrs, self.get_line_column(i).0),
                Item::Static(i) => (&i.attrs, self.get_line_column(i).0),
                Item::Struct(i) => (&i.attrs, self.get_line_column(i).0),
                Item::Trait(i) => (&i.attrs, self.get_line_column(i).0),
                Item::Type(i) => (&i.attrs, self.get_line_column(i).0),
                Item::Impl(i) => (&i.attrs, self.get_line_column(i).0),
                _ => continue,
            };
            self.extract_doc_comment_refs(attrs, line);
        }

        // Third pass: validate references
        let refs_to_check = self.doc_comment_refs.clone();
        for (type_name, line, column) in refs_to_check {
            if !self.imported_types.contains(&type_name) && !self.local_types.contains(&type_name) {
                self.add_violation(
                    line,
                    column,
                    ViolationKind::MissingDocCommentImport,
                    format!("[{}] not in use statements", type_name),
                );
            }
        }
    }
}

pub fn check_file(path: &Path) -> Result<Vec<StyleViolation>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let syntax = syn::parse_file(&content)
        .with_context(|| format!("Failed to parse file: {}", path.display()))?;

    let mut checker = DocCommentLinkChecker::new(path.to_path_buf());
    checker.visit_file(&syntax);

    Ok(checker.violations().to_vec())
}
