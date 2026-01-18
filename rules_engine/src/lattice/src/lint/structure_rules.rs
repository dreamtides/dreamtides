use std::path::Path;

use regex::Regex;
use tracing::debug;

use crate::lint::rule_engine::{LintContext, LintDocument, LintResult, LintRule};
use crate::task::directory_structure;

/// W017: Document not in standard location.
///
/// Non-root documents must be in a tasks/ or docs/ directory.
pub struct NotInStandardLocationRule;

/// W018: Task in docs/ directory.
///
/// Documents with task-type field should be in tasks/ directory, not docs/.
pub struct TaskInDocsDirRule;

/// W019: Knowledge base document in tasks/ directory.
///
/// Documents without task-type in tasks/ (excluding .closed/ subdirectories)
/// should be in docs/ directory instead.
pub struct KnowledgeBaseInTasksDirRule;

/// W020: Invalid document name format.
///
/// Non-root document filenames must have at least two underscore-separated
/// words and consist only of lowercase letters, numbers, and underscores.
pub struct InvalidDocumentNameFormatRule;

/// Returns all structure-related warning rules (W017-W020).
pub fn all_structure_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(NotInStandardLocationRule),
        Box::new(TaskInDocsDirRule),
        Box::new(KnowledgeBaseInTasksDirRule),
        Box::new(InvalidDocumentNameFormatRule),
    ]
}

impl LintRule for NotInStandardLocationRule {
    fn codes(&self) -> &[&str] {
        &["W017"]
    }

    fn name(&self) -> &str {
        "not-in-standard-location"
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        if doc.row.is_root {
            return vec![];
        }

        if doc.row.in_tasks_dir || doc.row.in_docs_dir {
            return vec![];
        }

        let message = format!("{} is not in tasks/ or docs/ directory", doc.row.path);
        debug!(path = %doc.row.path, "Document not in standard location");
        vec![LintResult::warning("W017", &doc.row.path, message)]
    }
}

impl LintRule for TaskInDocsDirRule {
    fn codes(&self) -> &[&str] {
        &["W018"]
    }

    fn name(&self) -> &str {
        "task-in-docs-dir"
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        if doc.row.task_type.is_none() {
            return vec![];
        }

        if !doc.row.in_docs_dir || doc.row.in_tasks_dir {
            return vec![];
        }

        let message = format!("{} is a task but located in docs/", doc.row.path);
        debug!(path = %doc.row.path, "Task document found in docs/ directory");
        vec![LintResult::warning("W018", &doc.row.path, message)]
    }
}

impl LintRule for KnowledgeBaseInTasksDirRule {
    fn codes(&self) -> &[&str] {
        &["W019"]
    }

    fn name(&self) -> &str {
        "kb-in-tasks-dir"
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        if doc.row.task_type.is_some() {
            return vec![];
        }

        if !doc.row.in_tasks_dir || doc.row.is_closed {
            return vec![];
        }

        let message =
            format!("{} is a knowledge base document but located in tasks/", doc.row.path);
        debug!(path = %doc.row.path, "Knowledge base document found in tasks/ directory");
        vec![LintResult::warning("W019", &doc.row.path, message)]
    }
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

        let suggested = directory_structure::target_dir_name(doc.row.task_type.is_some());
        let message = format!(
            "{} should be at least two underscore-separated words (e.g., example_name.md in {}/ directory)",
            doc.row.path, suggested
        );
        debug!(
            path = %doc.row.path,
            file_stem = file_stem,
            "Invalid document name format"
        );
        vec![LintResult::warning("W020", &doc.row.path, message)]
    }
}
