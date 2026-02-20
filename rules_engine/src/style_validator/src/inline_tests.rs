use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{File, Item, ItemMod};

use crate::violation::{StyleViolation, ViolationKind};

const WHITELISTED_FILES: &[&str] = &[
    "src/parser/src/error/parser_error_suggestions.rs",
    "src/battle_state/src/battle_cards/card_set.rs",
];

pub struct InlineTestsChecker {
    violations: Vec<StyleViolation>,
    file_path: PathBuf,
    is_whitelisted: bool,
}

impl InlineTestsChecker {
    pub fn new(file_path: PathBuf, rules_engine_path: &Path) -> Self {
        let relative_path = file_path
            .strip_prefix(rules_engine_path)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();

        let is_whitelisted = WHITELISTED_FILES.iter().any(|w| relative_path == *w);

        Self { violations: Vec::new(), file_path, is_whitelisted }
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
            kind: ViolationKind::InlineTestModule,
            path_str,
        });
    }

    fn check_mod(&mut self, item_mod: &ItemMod) {
        if item_mod.ident == "tests" && item_mod.content.is_some() {
            self.add_violation(item_mod, format!("mod {} {{ ... }}", item_mod.ident));
        }
    }

    pub fn violations(&self) -> &[StyleViolation] {
        &self.violations
    }
}

impl<'ast> Visit<'ast> for InlineTestsChecker {
    fn visit_file(&mut self, i: &'ast File) {
        if self.is_whitelisted {
            return;
        }

        for item in &i.items {
            if let Item::Mod(item_mod) = item {
                self.check_mod(item_mod);
            }
        }
    }
}

pub fn check_file(path: &Path, rules_engine_path: &Path) -> Result<Vec<StyleViolation>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let syntax = syn::parse_file(&content)
        .with_context(|| format!("Failed to parse file: {}", path.display()))?;

    let mut checker = InlineTestsChecker::new(path.to_path_buf(), rules_engine_path);
    checker.visit_file(&syntax);

    Ok(checker.violations().to_vec())
}
