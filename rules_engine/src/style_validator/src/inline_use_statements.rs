use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::visit_mut::VisitMut;
use syn::{Attribute, Item, ItemUse};

use crate::violation::{StyleViolation, ViolationKind};

pub struct InlineUseChecker {
    violations: Vec<StyleViolation>,
    file_path: PathBuf,
    nesting_depth: usize,
    in_test_module: bool,
}

impl InlineUseChecker {
    pub fn new(file_path: PathBuf) -> Self {
        Self { violations: Vec::new(), file_path, nesting_depth: 0, in_test_module: false }
    }

    fn get_line_column<T: Spanned>(&self, node: &T) -> (usize, usize) {
        let span = node.span();
        (span.start().line, span.start().column + 1)
    }

    fn add_violation<T: Spanned>(&mut self, node: &T, kind: ViolationKind, path_str: String) {
        let (line, column) = self.get_line_column(node);
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

    fn is_test_attribute(attr: &Attribute) -> bool {
        attr.path().is_ident("cfg")
            && attr.parse_args::<syn::Ident>().map(|ident| ident == "test").unwrap_or(false)
    }

    fn has_test_attribute(attrs: &[Attribute]) -> bool {
        attrs.iter().any(Self::is_test_attribute)
    }

    fn check_use_item(&mut self, item_use: &ItemUse) {
        if self.nesting_depth > 0 && !self.in_test_module {
            self.add_violation(
                item_use,
                ViolationKind::InlineUseStatement,
                format!("{}", quote::quote!(#item_use)),
            );
        }
    }
}

impl<'ast> Visit<'ast> for InlineUseChecker {
    fn visit_item(&mut self, i: &'ast Item) {
        match i {
            Item::Use(item_use) => {
                self.check_use_item(item_use);
            }
            Item::Mod(item_mod) => {
                if let Some((_, items)) = &item_mod.content {
                    let was_in_test = self.in_test_module;
                    if Self::has_test_attribute(&item_mod.attrs) {
                        self.in_test_module = true;
                    }

                    self.nesting_depth += 1;
                    for item in items {
                        self.visit_item(item);
                    }
                    self.nesting_depth -= 1;

                    self.in_test_module = was_in_test;
                }
            }
            Item::Fn(item_fn) => {
                self.nesting_depth += 1;
                for stmt in &item_fn.block.stmts {
                    if let syn::Stmt::Item(item) = stmt {
                        self.visit_item(item);
                    }
                }
                self.nesting_depth -= 1;
            }
            Item::Impl(item_impl) => {
                self.nesting_depth += 1;
                for item in &item_impl.items {
                    if let syn::ImplItem::Fn(method) = item {
                        for stmt in &method.block.stmts {
                            if let syn::Stmt::Item(item) = stmt {
                                self.visit_item(item);
                            }
                        }
                    }
                }
                self.nesting_depth -= 1;
            }
            Item::Trait(item_trait) => {
                self.nesting_depth += 1;
                for item in &item_trait.items {
                    if let syn::TraitItem::Fn(method) = item {
                        if let Some(block) = &method.default {
                            for stmt in &block.stmts {
                                if let syn::Stmt::Item(item) = stmt {
                                    self.visit_item(item);
                                }
                            }
                        }
                    }
                }
                self.nesting_depth -= 1;
            }
            _ => {}
        }
    }
}

pub fn check_file(path: &Path) -> Result<Vec<StyleViolation>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let syntax = syn::parse_file(&content)
        .with_context(|| format!("Failed to parse file: {}", path.display()))?;

    let mut checker = InlineUseChecker::new(path.to_path_buf());
    for item in &syntax.items {
        checker.visit_item(item);
    }

    Ok(checker.violations().to_vec())
}

pub fn fix_file(path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let mut syntax = syn::parse_file(&content)
        .with_context(|| format!("Failed to parse file: {}", path.display()))?;

    let mut extracted_uses = Vec::new();
    let mut extractor = UseExtractor::new();

    for item in &mut syntax.items {
        extractor.visit_item_mut(item);
    }

    extracted_uses.extend(extractor.extracted_uses);

    let existing_uses: std::collections::HashSet<String> = syntax
        .items
        .iter()
        .filter_map(|item| {
            if let Item::Use(u) = item {
                Some(quote::quote!(#u).to_string())
            } else {
                None
            }
        })
        .collect();

    let mut new_top_level_items = Vec::new();

    for item_use in extracted_uses {
        let use_str = quote::quote!(#item_use).to_string();
        if !existing_uses.contains(&use_str) {
            new_top_level_items.push(Item::Use(item_use));
        }
    }

    let mut use_items = Vec::new();
    let mut other_items = Vec::new();

    for item in syntax.items {
        if matches!(item, Item::Use(_)) {
            use_items.push(item);
        } else {
            other_items.push(item);
        }
    }

    use_items.extend(new_top_level_items);
    use_items.extend(other_items);

    let modified_file =
        syn::File { shebang: syntax.shebang, attrs: syntax.attrs, items: use_items };

    let output = prettyplease::unparse(&modified_file);

    std::fs::write(path, output)
        .with_context(|| format!("Failed to write file: {}", path.display()))?;

    Ok(())
}

struct UseExtractor {
    extracted_uses: Vec<ItemUse>,
    nesting_depth: usize,
    in_test_module: bool,
}

impl UseExtractor {
    fn new() -> Self {
        Self { extracted_uses: Vec::new(), nesting_depth: 0, in_test_module: false }
    }

    fn is_test_attribute(attr: &Attribute) -> bool {
        attr.path().is_ident("cfg")
            && attr.parse_args::<syn::Ident>().map(|ident| ident == "test").unwrap_or(false)
    }

    fn has_test_attribute(attrs: &[Attribute]) -> bool {
        attrs.iter().any(Self::is_test_attribute)
    }
}

impl syn::visit_mut::VisitMut for UseExtractor {
    fn visit_item_mut(&mut self, i: &mut Item) {
        match i {
            Item::Mod(item_mod) => {
                if let Some((_, items)) = &mut item_mod.content {
                    let was_in_test = self.in_test_module;
                    if Self::has_test_attribute(&item_mod.attrs) {
                        self.in_test_module = true;
                    }

                    self.nesting_depth += 1;
                    let mut filtered_items = Vec::new();

                    for mut item in items.drain(..) {
                        if let Item::Use(item_use) = &item {
                            if self.nesting_depth > 0 && !self.in_test_module {
                                self.extracted_uses.push(item_use.clone());
                                continue;
                            }
                        }
                        self.visit_item_mut(&mut item);
                        filtered_items.push(item);
                    }

                    *items = filtered_items;
                    self.nesting_depth -= 1;
                    self.in_test_module = was_in_test;
                }
            }
            Item::Fn(item_fn) => {
                self.nesting_depth += 1;
                let mut filtered_stmts = Vec::new();

                for stmt in item_fn.block.stmts.drain(..) {
                    if let syn::Stmt::Item(Item::Use(item_use)) = &stmt {
                        self.extracted_uses.push(item_use.clone());
                    } else {
                        filtered_stmts.push(stmt);
                    }
                }

                item_fn.block.stmts = filtered_stmts;
                self.nesting_depth -= 1;
            }
            Item::Impl(item_impl) => {
                self.nesting_depth += 1;
                for item in &mut item_impl.items {
                    if let syn::ImplItem::Fn(method) = item {
                        let mut filtered_stmts = Vec::new();

                        for stmt in method.block.stmts.drain(..) {
                            if let syn::Stmt::Item(Item::Use(item_use)) = &stmt {
                                self.extracted_uses.push(item_use.clone());
                            } else {
                                filtered_stmts.push(stmt);
                            }
                        }

                        method.block.stmts = filtered_stmts;
                    }
                }
                self.nesting_depth -= 1;
            }
            Item::Trait(item_trait) => {
                self.nesting_depth += 1;
                for item in &mut item_trait.items {
                    if let syn::TraitItem::Fn(method) = item {
                        if let Some(block) = &mut method.default {
                            let mut filtered_stmts = Vec::new();

                            for stmt in block.stmts.drain(..) {
                                if let syn::Stmt::Item(Item::Use(item_use)) = &stmt {
                                    self.extracted_uses.push(item_use.clone());
                                } else {
                                    filtered_stmts.push(stmt);
                                }
                            }

                            block.stmts = filtered_stmts;
                        }
                    }
                }
                self.nesting_depth -= 1;
            }
            _ => {}
        }
    }
}
