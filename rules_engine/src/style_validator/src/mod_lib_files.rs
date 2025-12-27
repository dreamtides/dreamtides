use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{File, Item};

use crate::violation::{StyleViolation, ViolationKind};

pub struct ModLibChecker {
    violations: Vec<StyleViolation>,
    file_path: PathBuf,
    is_mod_or_lib: bool,
}

impl ModLibChecker {
    pub fn new(file_path: PathBuf) -> Self {
        let is_mod_or_lib = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n == "mod.rs" || n == "lib.rs")
            .unwrap_or(false);

        Self { violations: Vec::new(), file_path, is_mod_or_lib }
    }

    fn get_line_column<T: Spanned>(&self, node: &T) -> (usize, usize) {
        let span = node.span();
        let byte_start = span.start().line;
        let byte_column = span.start().column;

        // syn gives us line/column already (1-indexed)
        (byte_start, byte_column + 1)
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
}

impl<'ast> Visit<'ast> for ModLibChecker {
    fn visit_file(&mut self, i: &'ast File) {
        if !self.is_mod_or_lib {
            return;
        }

        for item in &i.items {
            match item {
                Item::Mod(item_mod) => {
                    // Mod declarations are allowed, but only if they don't have inline content
                    if item_mod.content.is_some() {
                        self.add_violation(
                            item_mod,
                            ViolationKind::CodeInModLibFile,
                            format!("mod {} {{ ... }}", item_mod.ident),
                        );
                    }
                }
                Item::Use(_) => {
                    // Use statements are also allowed
                }
                _ => {
                    // Any other item is not allowed
                    let item_str = match item {
                        Item::Fn(f) => format!("fn {}", f.sig.ident),
                        Item::Struct(s) => format!("struct {}", s.ident),
                        Item::Enum(e) => format!("enum {}", e.ident),
                        Item::Impl(i) => {
                            if let Some((_, trait_path, _)) = &i.trait_ {
                                format!("impl {} for ...", quote::quote!(#trait_path))
                            } else if let syn::Type::Path(type_path) = &*i.self_ty {
                                format!("impl {}", quote::quote!(#type_path))
                            } else {
                                "impl".to_string()
                            }
                        }
                        Item::Const(c) => format!("const {}", c.ident),
                        Item::Static(s) => format!("static {}", s.ident),
                        Item::Type(t) => format!("type {}", t.ident),
                        Item::Trait(t) => format!("trait {}", t.ident),
                        _ => "item".to_string(),
                    };

                    self.add_violation(item, ViolationKind::CodeInModLibFile, item_str);
                }
            }
        }
    }
}

pub fn check_file(path: &Path) -> Result<Vec<StyleViolation>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let syntax = syn::parse_file(&content)
        .with_context(|| format!("Failed to parse file: {}", path.display()))?;

    let mut checker = ModLibChecker::new(path.to_path_buf());
    checker.visit_file(&syntax);

    Ok(checker.violations().to_vec())
}
