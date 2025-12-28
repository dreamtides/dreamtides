use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use syn::spanned::Spanned;
use syn::{File, Item, Visibility};

use crate::violation::{StyleViolation, ViolationKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ItemCategory {
    PrivateConst,
    PublicTypeAlias,
    PublicConst,
    PublicTrait,
    PublicStructOrEnum,
    PublicFunction,
    PrivateItems,
    TestModule,
}

pub struct CodeOrderChecker {
    violations: Vec<StyleViolation>,
    file_path: PathBuf,
    current_phase: ItemCategory,
}

impl CodeOrderChecker {
    pub fn new(file_path: PathBuf) -> Self {
        Self { violations: Vec::new(), file_path, current_phase: ItemCategory::PublicTypeAlias }
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

    fn is_test_module(item: &Item) -> bool {
        if let Item::Mod(m) = item {
            m.attrs.iter().any(|attr| {
                attr.path().is_ident("cfg")
                    && attr.parse_args::<syn::Ident>().map(|ident| ident == "test").unwrap_or(false)
            })
        } else {
            false
        }
    }

    fn categorize_item(item: &Item) -> ItemCategory {
        match item {
            Item::Const(c) if !matches!(c.vis, Visibility::Public(_)) => ItemCategory::PrivateConst,
            Item::Type(t) if matches!(t.vis, Visibility::Public(_)) => {
                ItemCategory::PublicTypeAlias
            }
            Item::Const(c) if matches!(c.vis, Visibility::Public(_)) => ItemCategory::PublicConst,
            Item::Trait(tr) if matches!(tr.vis, Visibility::Public(_)) => ItemCategory::PublicTrait,
            Item::Struct(s) if matches!(s.vis, Visibility::Public(_)) => {
                ItemCategory::PublicStructOrEnum
            }
            Item::Enum(e) if matches!(e.vis, Visibility::Public(_)) => {
                ItemCategory::PublicStructOrEnum
            }
            Item::Fn(f) if matches!(f.vis, Visibility::Public(_)) => ItemCategory::PublicFunction,
            _ => ItemCategory::PrivateItems,
        }
    }

    fn category_name(category: ItemCategory) -> &'static str {
        match category {
            ItemCategory::PrivateConst => "private constants",
            ItemCategory::PublicTypeAlias => "public type aliases",
            ItemCategory::PublicConst => "public constants",
            ItemCategory::PublicTrait => "public traits",
            ItemCategory::PublicStructOrEnum => "public structs and enums",
            ItemCategory::PublicFunction => "public functions",
            ItemCategory::PrivateItems => "private items",
            ItemCategory::TestModule => "test modules",
        }
    }

    pub fn check_file(&mut self, file: &File) {
        for item in &file.items {
            if matches!(item, Item::Use(_)) {
                continue;
            }

            if matches!(item, Item::Mod(_)) && !Self::is_test_module(item) {
                continue;
            }

            let category = if Self::is_test_module(item) {
                ItemCategory::TestModule
            } else {
                Self::categorize_item(item)
            };

            if category < self.current_phase {
                self.add_violation(
                    item,
                    ViolationKind::CodeOrder,
                    format!(
                        "{} should come before {}",
                        Self::category_name(category),
                        Self::category_name(self.current_phase)
                    ),
                );
            } else {
                self.current_phase = category;
            }
        }
    }
}

pub fn check_file(path: &Path) -> Result<Vec<StyleViolation>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let syntax = syn::parse_file(&content)
        .with_context(|| format!("Failed to parse file: {}", path.display()))?;

    let mut checker = CodeOrderChecker::new(path.to_path_buf());
    checker.check_file(&syntax);

    Ok(checker.violations().to_vec())
}

pub fn fix_file(path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let syntax = syn::parse_file(&content)
        .with_context(|| format!("Failed to parse file: {}", path.display()))?;

    let lines: Vec<&str> = content.lines().collect();

    let mut categorized_items: Vec<(ItemCategory, String)> = Vec::new();
    let mut use_and_mod_items: Vec<String> = Vec::new();
    let mut test_modules: Vec<String> = Vec::new();

    for item in &syntax.items {
        let span = item.span();
        let start_line = span.start().line.saturating_sub(1);
        let end_line = span.end().line.saturating_sub(1);

        let item_text = if start_line == end_line {
            lines[start_line].to_string()
        } else {
            lines[start_line..=end_line.min(lines.len().saturating_sub(1))].join("\n")
        };

        if CodeOrderChecker::is_test_module(item) {
            test_modules.push(item_text);
        } else if matches!(item, Item::Use(_) | Item::Mod(_)) {
            use_and_mod_items.push(item_text);
        } else {
            categorized_items.push((CodeOrderChecker::categorize_item(item), item_text));
        }
    }

    categorized_items.sort_by_key(|(category, _)| *category);

    let mut output = String::new();

    for item_str in &use_and_mod_items {
        output.push_str(item_str.trim());
        output.push('\n');
    }

    if !use_and_mod_items.is_empty() && !categorized_items.is_empty() {
        output.push('\n');
    }

    for (i, (_, item_str)) in categorized_items.iter().enumerate() {
        if i > 0 {
            output.push('\n');
        }
        output.push_str(item_str.trim());
        output.push('\n');
    }

    if !test_modules.is_empty() {
        if !categorized_items.is_empty() {
            output.push('\n');
        }
        for (i, item_str) in test_modules.iter().enumerate() {
            if i > 0 {
                output.push('\n');
            }
            output.push_str(item_str.trim());
            output.push('\n');
        }
    }

    std::fs::write(path, output)
        .with_context(|| format!("Failed to write file: {}", path.display()))?;

    Ok(())
}
