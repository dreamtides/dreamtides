use std::path::Path;

use regex::Regex;
use tracing::debug;

use crate::lint::rule_engine::{LintContext, LintDocument, LintResult, LintRule};

/// W020: Invalid document name format.
///
/// Non-root document filenames must have at least two underscore-separated
/// words and consist only of lowercase letters, numbers, and underscores.
pub struct InvalidDocumentNameFormatRule;

/// Returns all structure-related warning rules.
pub fn all_structure_rules() -> Vec<Box<dyn LintRule>> {
    vec![Box::new(InvalidDocumentNameFormatRule)]
}

impl LintRule for InvalidDocumentNameFormatRule {
    fn codes(&self) -> &[&str] {
        &["W020"]
    }

    fn name(&self) -> &str {
        "invalid-document-name-format"
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        if doc.row.is_root {
            return vec![];
        }

        let path = Path::new(&doc.row.path);
        let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) else {
            return vec![];
        };

        let valid_filename_regex = Regex::new(r"^[a-z0-9]+(_[a-z0-9]+)+$")
            .unwrap_or_else(|e| panic!("Invalid regex for filename validation: {e}"));

        if valid_filename_regex.is_match(file_stem) {
            return vec![];
        }

        let message = format!(
            "{} should be at least two underscore-separated words (e.g., example_name.md)",
            doc.row.path
        );
        debug!(
            path = %doc.row.path,
            file_stem = file_stem,
            "Invalid document name format"
        );
        vec![LintResult::warning("W020", &doc.row.path, message)]
    }
}
