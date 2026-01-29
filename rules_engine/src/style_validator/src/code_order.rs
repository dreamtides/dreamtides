use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use syn::spanned::Spanned;
use syn::{File, Item, Visibility};

use crate::violation::{StyleViolation, ViolationKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ItemCategory {
    PrivateConst,
    PrivateStatic,
    ThreadLocal,
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
        Self { violations: Vec::new(), file_path, current_phase: ItemCategory::PrivateConst }
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

    fn is_thread_local(item: &Item) -> bool {
        if let Item::Macro(m) = item {
            m.mac.path.is_ident("thread_local")
        } else {
            false
        }
    }

    fn categorize_item(item: &Item) -> ItemCategory {
        match item {
            Item::Const(c) if !matches!(c.vis, Visibility::Public(_)) => ItemCategory::PrivateConst,
            Item::Static(s) if !matches!(s.vis, Visibility::Public(_)) => {
                ItemCategory::PrivateStatic
            }
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
            ItemCategory::PrivateStatic => "private statics",
            ItemCategory::ThreadLocal => "thread_local items",
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
            } else if Self::is_thread_local(item) {
                ItemCategory::ThreadLocal
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

/// Analyzes spacing requirements between items.
/// Returns a list of line numbers (0-indexed) where blank lines need to be
/// inserted AFTER.
fn analyze_spacing(syntax: &File, lines: &[&str]) -> Vec<usize> {
    let mut insert_after: Vec<usize> = Vec::new();

    // Track item boundaries and categories
    let mut prev_item_end_line: Option<usize> = None;
    let mut prev_category: Option<ItemCategory> = None;

    for item in &syntax.items {
        let span = item.span();
        let start_line = span.start().line.saturating_sub(1);
        let end_line = span.end().line.saturating_sub(1);

        if matches!(item, Item::Use(_))
            || (matches!(item, Item::Mod(_)) && !CodeOrderChecker::is_test_module(item))
        {
            prev_item_end_line = Some(end_line);
            prev_category = None;
            continue;
        }

        let category = if CodeOrderChecker::is_test_module(item) {
            ItemCategory::TestModule
        } else if CodeOrderChecker::is_thread_local(item) {
            ItemCategory::ThreadLocal
        } else {
            CodeOrderChecker::categorize_item(item)
        };

        // Check if we need a blank line before this item
        let needs_blank_line = if prev_item_end_line.is_some() {
            if let Some(prev_cat) = prev_category {
                // Between two code items: need blank line unless both are constants
                let is_const =
                    matches!(category, ItemCategory::PrivateConst | ItemCategory::PublicConst);
                let prev_is_const =
                    matches!(prev_cat, ItemCategory::PrivateConst | ItemCategory::PublicConst);
                !is_const || !prev_is_const
            } else {
                // After use statements: always need blank line
                true
            }
        } else {
            false
        };

        if needs_blank_line {
            if let Some(prev_end) = prev_item_end_line {
                // Check if there's already a blank line
                // A blank line exists if there's at least one empty line between prev_end and
                // start_line
                let has_blank = (prev_end + 1 < start_line)
                    && (prev_end + 1..start_line)
                        .any(|i| i < lines.len() && lines[i].trim().is_empty());

                if !has_blank {
                    insert_after.push(prev_end);
                }
            }
        }

        prev_item_end_line = Some(end_line);
        prev_category = Some(category);
    }

    insert_after
}

pub fn check_file(path: &Path) -> Result<Vec<StyleViolation>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let syntax = syn::parse_file(&content)
        .with_context(|| format!("Failed to parse file: {}", path.display()))?;

    let mut checker = CodeOrderChecker::new(path.to_path_buf());
    checker.check_file(&syntax);

    // Check spacing by analyzing where blank lines are needed
    let lines: Vec<&str> = content.lines().collect();
    let insertions = analyze_spacing(&syntax, &lines);

    if !insertions.is_empty() {
        checker.violations.push(StyleViolation {
            file: path.to_path_buf(),
            line: 1,
            column: 1,
            kind: ViolationKind::CodeSpacing,
            path_str: "file has incorrect spacing between code elements".to_string(),
        });
    }

    Ok(checker.violations().to_vec())
}

pub fn fix_file(path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let syntax = syn::parse_file(&content)
        .with_context(|| format!("Failed to parse file: {}", path.display()))?;

    let lines: Vec<&str> = content.lines().collect();
    let insertions = analyze_spacing(&syntax, &lines);

    if insertions.is_empty() {
        return Ok(());
    }

    // Build the output by inserting blank lines at the specified locations
    // This ONLY adds blank lines - it never removes or modifies any existing
    // content
    let mut output = String::new();
    for (i, line) in lines.iter().enumerate() {
        output.push_str(line);
        output.push('\n');

        // If this line needs a blank line after it, insert one
        if insertions.contains(&i) {
            output.push('\n');
        }
    }

    std::fs::write(path, output)
        .with_context(|| format!("Failed to write file: {}", path.display()))?;

    Ok(())
}
