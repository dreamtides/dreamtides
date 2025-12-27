use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Expr, ExprCall, ExprPath, Type, TypePath};
use walkdir::WalkDir;

#[derive(Default)]
struct StyleViolation {
    file: PathBuf,
    line: usize,
    column: usize,
    kind: ViolationKind,
    path_str: String,
}

#[derive(Default, Clone, Copy)]
enum ViolationKind {
    #[default]
    TooManyQualifiers,
    TypeShouldNotBeQualified,
    EnumVariantTooManyQualifiers,
}

impl ViolationKind {
    fn description(&self) -> &str {
        match self {
            ViolationKind::TooManyQualifiers => {
                "function call has too many qualifiers (should be 0 for same-file or 1 for cross-file)"
            }
            ViolationKind::TypeShouldNotBeQualified => {
                "type name should not be qualified (should have zero qualifiers)"
            }
            ViolationKind::EnumVariantTooManyQualifiers => {
                "enum variant has too many qualifiers (should have exactly one)"
            }
        }
    }
}

struct QualifierChecker {
    violations: Vec<StyleViolation>,
    file_path: PathBuf,
}

impl QualifierChecker {
    fn new(file_path: PathBuf, _source: String) -> Self {
        Self { violations: Vec::new(), file_path }
    }

    fn get_line_column<T: Spanned>(&self, node: &T) -> (usize, usize) {
        let span = node.span();
        let byte_start = span.start().line;
        let byte_column = span.start().column;

        // syn gives us line/column already (1-indexed)
        (byte_start, byte_column + 1)
    }

    fn count_qualifiers(path: &syn::Path) -> usize {
        if path.segments.len() <= 1 {
            return 0;
        }
        path.segments.len() - 1
    }

    fn is_likely_function_call(&self, path: &syn::Path) -> bool {
        if let Some(last) = path.segments.last() {
            let ident = &last.ident;
            let name = ident.to_string();

            // Function names typically start with lowercase or are snake_case
            // This heuristic isn't perfect but should catch most cases
            name.chars().next().map(|c| c.is_lowercase()).unwrap_or(false)
        } else {
            false
        }
    }

    fn is_likely_enum_variant(&self, path: &syn::Path) -> bool {
        if let Some(last) = path.segments.last() {
            let ident = &last.ident;
            let name = ident.to_string();

            // Enum variants typically start with uppercase (PascalCase)
            // And we need at least one qualifier (TypeName::Variant)
            name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
                && path.segments.len() >= 2
        } else {
            false
        }
    }

    fn is_type_name(&self, path: &syn::Path) -> bool {
        // Type names are PascalCase and typically used without qualifiers
        if let Some(last) = path.segments.last() {
            let name = last.ident.to_string();
            name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
        } else {
            false
        }
    }

    fn is_likely_trait_method(&self, path: &syn::Path) -> bool {
        // Trait methods often appear as Type::method() or module::Type::method() in
        // explicit calls Common patterns: Display::fmt, Error::custom,
        // fs::File::open, etc.
        if path.segments.len() >= 2 {
            if let Some(last) = path.segments.last() {
                let method_name = last.ident.to_string();
                // Common trait method names that often need explicit qualification
                if matches!(
                    method_name.as_str(),
                    "fmt"
                        | "custom"
                        | "next"
                        | "from"
                        | "into"
                        | "try_from"
                        | "try_into"
                        | "open"
                        | "write"
                        | "read"
                        | "from_date_and_time"
                        | "bind"
                        | "spawn_blocking"
                        | "with_default"
                        | "to_string_pretty"
                        | "default"
                        | "layer"
                        | "pretty"
                ) {
                    return true;
                }
            }
        }
        false
    }

    fn should_skip_path(&self, path: &syn::Path) -> bool {
        if let Some(first) = path.segments.first() {
            let first_name = first.ident.to_string();

            // Skip self, crate, super
            if first_name == "self" || first_name == "crate" || first_name == "super" {
                return true;
            }

            // Skip std library paths (they follow different conventions)
            if first_name == "std" || first_name == "core" || first_name == "alloc" {
                return true;
            }
        }

        // Skip prelude items (Option, Result enum variants)
        if let Some(last) = path.segments.last() {
            let last_name = last.ident.to_string();
            if matches!(last_name.as_str(), "Some" | "None" | "Ok" | "Err") {
                return true;
            }
        }

        false
    }

    fn check_function_call(&mut self, call: &ExprCall) {
        // Only check if the function being called is a path expression
        if let Expr::Path(expr_path) = &*call.func {
            let path = &expr_path.path;

            if self.should_skip_path(path) {
                return;
            }

            // Skip tuple struct constructors (PascalCase) - they should have zero
            // qualifiers
            if !self.is_likely_function_call(path) {
                return;
            }

            // Skip trait method calls (e.g., Display::fmt)
            if self.is_likely_trait_method(path) {
                return;
            }

            let qualifier_count = Self::count_qualifiers(path);

            // Function calls should have 0 qualifiers (same file) or 1 qualifier
            // (cross-file) Only flag if 2+ qualifiers
            if qualifier_count > 1 {
                self.add_violation(
                    path,
                    ViolationKind::TooManyQualifiers,
                    format!("{}", quote::quote!(#path)),
                );
            }
        }
    }

    fn check_expr_path(&mut self, expr_path: &ExprPath) {
        let path = &expr_path.path;
        let qualifier_count = Self::count_qualifiers(path);

        // Skip if this is a `self` or `crate` path
        if let Some(first) = path.segments.first() {
            let first_name = first.ident.to_string();
            if first_name == "self" || first_name == "crate" || first_name == "super" {
                return;
            }
        }

        // Only check enum variants here (PascalCase paths that aren't function calls)
        if self.is_likely_enum_variant(path) && qualifier_count > 1 {
            self.add_violation(
                path,
                ViolationKind::EnumVariantTooManyQualifiers,
                format!("{}", quote::quote!(#path)),
            );
        }
    }

    fn is_associated_type(&self, path: &syn::Path) -> bool {
        // Associated types have generic arguments on non-terminal segments
        // e.g., Add<B>::Output, where Add<B> has arguments
        if path.segments.len() < 2 {
            return false;
        }

        // Check if any segment except the last has generic arguments
        path.segments
            .iter()
            .take(path.segments.len() - 1)
            .any(|seg| !matches!(seg.arguments, syn::PathArguments::None))
    }

    fn is_generic_associated_type(&self, path: &syn::Path) -> bool {
        // Check if this looks like an associated type from a generic parameter
        // e.g., S::Ok, D::Error, Self::Value, T::Item, etc.
        if let Some(first) = path.segments.first() {
            let first_name = first.ident.to_string();
            // Skip Self:: paths
            if first_name == "Self" {
                return true;
            }
            // Skip single capital letter generic parameters (S, T, U, E, D, etc.)
            if first_name.len() == 1 && first_name.chars().next().unwrap().is_uppercase() {
                return true;
            }
            // Skip common external crate module names used in types
            if matches!(first_name.as_str(), "fmt" | "io" | "serde_json" | "anyhow" | "std") {
                return true;
            }
        }
        false
    }

    fn check_type_path(&mut self, type_path: &TypePath) {
        let path = &type_path.path;
        let qualifier_count = Self::count_qualifiers(path);

        // Skip if this is a `self` or `crate` path
        if let Some(first) = path.segments.first() {
            let first_name = first.ident.to_string();
            if first_name == "self" || first_name == "crate" || first_name == "super" {
                return;
            }
        }

        // Skip associated types (e.g., Add<B>::Output)
        if self.is_associated_type(path) {
            return;
        }

        // Skip generic associated types (e.g., S::Ok, Self::Value)
        if self.is_generic_associated_type(path) {
            return;
        }

        // Type names (structs, enums) should have zero qualifiers
        if self.is_type_name(path) && qualifier_count > 0 {
            self.add_violation(
                path,
                ViolationKind::TypeShouldNotBeQualified,
                format!("{}", quote::quote!(#path)),
            );
        }
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
}

impl<'ast> Visit<'ast> for QualifierChecker {
    fn visit_expr(&mut self, i: &'ast Expr) {
        match i {
            Expr::Call(call) => {
                self.check_function_call(call);
            }
            Expr::Path(expr_path) => {
                self.check_expr_path(expr_path);
            }
            _ => {}
        }
        syn::visit::visit_expr(self, i);
    }

    fn visit_type(&mut self, i: &'ast Type) {
        if let Type::Path(type_path) = i {
            self.check_type_path(type_path);
        }
        syn::visit::visit_type(self, i);
    }
}

fn check_file(path: &Path) -> Result<Vec<StyleViolation>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let syntax = syn::parse_file(&content)
        .with_context(|| format!("Failed to parse file: {}", path.display()))?;

    let mut checker = QualifierChecker::new(path.to_path_buf(), content);
    checker.visit_file(&syntax);

    Ok(checker.violations)
}

fn find_rust_files(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            let path = e.path();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Skip target directory and hidden directories
            if file_name.starts_with('.') || file_name == "target" {
                return false;
            }

            // Skip docs, benchmarks, tests, and old code directories
            if matches!(file_name, "docs" | "benchmarks" | "tests" | "old_tabula_cli") {
                return false;
            }

            true
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
        .map(|e| e.path().to_path_buf())
        .collect()
}

fn main() -> Result<()> {
    let rules_engine_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();

    println!("Checking Rust files in: {}", rules_engine_path.display());

    let rust_files = find_rust_files(rules_engine_path);
    println!("Found {} Rust files", rust_files.len());

    let mut all_violations = Vec::new();
    let style_validator_path = Path::new(env!("CARGO_MANIFEST_DIR"));

    for file in &rust_files {
        // Skip the style_validator's own source files
        if file.starts_with(style_validator_path) {
            continue;
        }

        match check_file(file) {
            Ok(violations) => {
                all_violations.extend(violations);
            }
            Err(e) => {
                eprintln!("Error checking {}: {}", file.display(), e);
            }
        }
    }

    if all_violations.is_empty() {
        println!("\n✓ No style violations found!");
        Ok(())
    } else {
        println!("\n✗ Found {} style violations:\n", all_violations.len());

        for violation in &all_violations {
            println!("{}:{}:{}", violation.file.display(), violation.line, violation.column,);
            println!("  → {}", violation.path_str);
            println!("  ✗ {}\n", violation.kind.description());
        }

        std::process::exit(1);
    }
}
