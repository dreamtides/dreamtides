use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{File, Item, Visibility};

use crate::violation::{StyleViolation, ViolationKind};

pub struct PubUseChecker {
    violations: Vec<StyleViolation>,
    file_path: PathBuf,
}

impl PubUseChecker {
    pub fn new(file_path: PathBuf) -> Self {
        Self { violations: Vec::new(), file_path }
    }

    fn get_line_column<T: Spanned>(&self, node: &T) -> (usize, usize) {
        let span = node.span();
        let byte_start = span.start().line;
        let byte_column = span.start().column;

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

impl<'ast> Visit<'ast> for PubUseChecker {
    fn visit_file(&mut self, i: &'ast File) {
        for item in &i.items {
            if let Item::Use(item_use) = item {
                if matches!(item_use.vis, Visibility::Public(_)) {
                    self.add_violation(
                        item_use,
                        ViolationKind::PubUseStatement,
                        format!("{}", quote::quote!(#item_use)),
                    );
                }
            }
        }
    }
}

fn is_test_session_prelude(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n == "test_session_prelude.rs")
        .unwrap_or(false)
}

pub fn check_file(path: &Path) -> Result<Vec<StyleViolation>> {
    if is_test_session_prelude(path) {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let syntax = syn::parse_file(&content)
        .with_context(|| format!("Failed to parse file: {}", path.display()))?;

    let mut checker = PubUseChecker::new(path.to_path_buf());
    checker.visit_file(&syntax);

    Ok(checker.violations().to_vec())
}
