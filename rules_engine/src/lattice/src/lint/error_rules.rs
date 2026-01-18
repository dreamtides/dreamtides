use std::collections::HashMap;
use std::path::Path;

use tracing::debug;

use crate::document::field_validation;
use crate::document::frontmatter_schema::{MAX_PRIORITY, MIN_PRIORITY};
use crate::error::error_types::LatticeError;
use crate::id::lattice_id::LatticeId;
use crate::index::{document_queries, link_queries};
use crate::lint::rule_engine::{LintContext, LintDocument, LintResult, LintRule};
use crate::task::closed_directory;
use crate::task::dependency_graph::DependencyGraph;

/// E001: Duplicate Lattice ID.
///
/// Two or more documents share the same Lattice ID.
pub struct DuplicateIdRule;

/// E002: Missing reference target.
///
/// A link references an ID that doesn't exist.
pub struct MissingReferenceRule;

/// E003: Invalid frontmatter key.
///
/// YAML frontmatter contains an unrecognized key.
pub struct InvalidKeyRule;

/// E004: Missing required field (priority).
///
/// Task document lacks required `priority` field.
pub struct MissingPriorityRule;

/// E005: Invalid field value.
///
/// A field contains an invalid value.
pub struct InvalidFieldValueRule;

/// E006: Circular blocking.
///
/// Blocking dependencies form a cycle.
pub struct CircularBlockingRule;

/// E007: Invalid ID format.
///
/// A Lattice ID doesn't match the expected format.
pub struct InvalidIdFormatRule;

/// E008: Name-filename mismatch.
///
/// The `name` field doesn't match the filename.
pub struct NameMismatchRule;

/// E009: Missing required field (name).
///
/// Document lacks required `name` field.
pub struct MissingNameRule;

/// E010: Missing required field (description).
///
/// Document lacks required `description` field.
pub struct MissingDescriptionRule;

/// E011: Invalid closed directory structure.
///
/// A `.closed/` directory contains nested `.closed/` directories.
pub struct NestedClosedRule;

/// E012: Non-task in closed directory.
///
/// A knowledge base document (no `task-type`) is in a `.closed/` directory.
pub struct NonTaskInClosedRule;

/// Returns all error-level lint rules.
pub fn all_error_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(DuplicateIdRule),
        Box::new(MissingReferenceRule),
        Box::new(InvalidKeyRule),
        Box::new(MissingPriorityRule),
        Box::new(InvalidFieldValueRule),
        Box::new(CircularBlockingRule),
        Box::new(InvalidIdFormatRule),
        Box::new(NameMismatchRule),
        Box::new(MissingNameRule),
        Box::new(MissingDescriptionRule),
        Box::new(NestedClosedRule),
        Box::new(NonTaskInClosedRule),
    ]
}

impl LintRule for DuplicateIdRule {
    fn codes(&self) -> &[&str] {
        &["E001"]
    }

    fn name(&self) -> &str {
        "duplicate-id"
    }

    fn check(&self, doc: &LintDocument, ctx: &LintContext<'_>) -> Vec<LintResult> {
        // Find all documents with the same ID
        let duplicates = match find_documents_with_id(ctx, &doc.row.id) {
            Ok(paths) => paths,
            Err(e) => {
                debug!(error = %e, id = doc.row.id, "Failed to query duplicates");
                return vec![];
            }
        };

        // Only report if there are duplicates (more than just this document)
        if duplicates.len() > 1 {
            let other_paths: Vec<_> =
                duplicates.into_iter().filter(|p| *p != doc.row.path).collect();
            if !other_paths.is_empty() {
                let message = format!(
                    "Duplicate Lattice ID {} found in: {}",
                    doc.row.id,
                    other_paths.join(", ")
                );
                return vec![LintResult::error("E001", &doc.row.path, message)];
            }
        }

        vec![]
    }
}

impl LintRule for MissingReferenceRule {
    fn codes(&self) -> &[&str] {
        &["E002"]
    }

    fn name(&self) -> &str {
        "missing-reference"
    }

    fn check(&self, doc: &LintDocument, ctx: &LintContext<'_>) -> Vec<LintResult> {
        // Query outgoing links from this document
        let links = match link_queries::query_outgoing(ctx.connection(), &doc.row.id) {
            Ok(links) => links,
            Err(e) => {
                debug!(error = %e, id = doc.row.id, "Failed to query outgoing links");
                return vec![];
            }
        };

        let mut results = Vec::new();
        for link in links {
            // Check if target exists
            let exists = match ctx.document_exists(&link.target_id) {
                Ok(exists) => exists,
                Err(e) => {
                    debug!(error = %e, target = link.target_id, "Failed to check target existence");
                    continue;
                }
            };

            if !exists {
                let message = format!("links to unknown ID {}", link.target_id);
                results.push(LintResult::error("E002", &doc.row.path, message));
            }
        }

        results
    }
}

impl LintRule for InvalidKeyRule {
    fn codes(&self) -> &[&str] {
        &["E003"]
    }

    fn name(&self) -> &str {
        "invalid-key"
    }

    fn requires_document_body(&self) -> bool {
        true
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        let Some(read_result) = &doc.read_result else {
            return vec![];
        };

        let mut results = Vec::new();
        for unknown_key in &read_result.unknown_keys {
            let message = if let Some(suggestion) = &unknown_key.suggestion {
                format!(
                    "has invalid frontmatter key '{}' (did you mean '{}'?)",
                    unknown_key.key, suggestion
                )
            } else {
                format!("has invalid frontmatter key '{}'", unknown_key.key)
            };
            results.push(LintResult::error("E003", &doc.row.path, message));
        }

        results
    }
}

impl LintRule for MissingPriorityRule {
    fn codes(&self) -> &[&str] {
        &["E004"]
    }

    fn name(&self) -> &str {
        "missing-priority"
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        // Only applies to tasks
        if doc.row.task_type.is_none() {
            return vec![];
        }

        if doc.row.priority.is_none() {
            let message = "is a task but missing 'priority' field";
            return vec![LintResult::error("E004", &doc.row.path, message)];
        }

        vec![]
    }
}

impl LintRule for InvalidFieldValueRule {
    fn codes(&self) -> &[&str] {
        &["E005"]
    }

    fn name(&self) -> &str {
        "invalid-field-value"
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        let mut results = Vec::new();

        // Validate priority range
        if let Some(priority) = doc.row.priority
            && priority > MAX_PRIORITY
        {
            let message = format!(
                "has invalid priority '{}' (allowed: {}-{})",
                priority, MIN_PRIORITY, MAX_PRIORITY
            );
            results.push(LintResult::error("E005", &doc.row.path, message));
        }

        // Note: task_type is already validated by serde deserialization.
        // Additional field validations can be added here as needed.

        results
    }
}

impl LintRule for CircularBlockingRule {
    fn codes(&self) -> &[&str] {
        &["E006"]
    }

    fn name(&self) -> &str {
        "circular-blocking"
    }

    fn check(&self, doc: &LintDocument, ctx: &LintContext<'_>) -> Vec<LintResult> {
        // Build dependency graph and check for cycles
        let graph = match DependencyGraph::build_from_connection(ctx.connection()) {
            Ok(graph) => graph,
            Err(e) => {
                debug!(error = %e, "Failed to build dependency graph");
                return vec![];
            }
        };

        let cycle_result = graph.detect_cycle();
        if !cycle_result.has_cycle {
            return vec![];
        }

        // Only report the cycle on the first document involved
        if let Some(first_id) = cycle_result.involved_ids.first()
            && first_id == &doc.row.id
        {
            let cycle_path = cycle_result.cycle_path.unwrap_or_default();
            let message = format!("Circular blocking dependency: {}", cycle_path);
            return vec![LintResult::error("E006", &doc.row.path, message)];
        }

        vec![]
    }
}

impl LintRule for InvalidIdFormatRule {
    fn codes(&self) -> &[&str] {
        &["E007"]
    }

    fn name(&self) -> &str {
        "invalid-id-format"
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        // Validate the document's own ID
        if LatticeId::parse(&doc.row.id).is_err() {
            let message = format!("has malformed lattice-id '{}'", doc.row.id);
            return vec![LintResult::error("E007", &doc.row.path, message)];
        }

        // Parent ID is validated during parsing, so if it's stored in the row,
        // it passed initial validation.

        vec![]
    }
}

impl LintRule for NameMismatchRule {
    fn codes(&self) -> &[&str] {
        &["E008"]
    }

    fn name(&self) -> &str {
        "name-mismatch"
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        let path = Path::new(&doc.row.path);
        let Some(expected_name) = field_validation::derive_name_from_path(path) else {
            return vec![];
        };

        if doc.row.name != expected_name {
            let message = format!(
                "has name '{}' but should be '{}' (derived from filename)",
                doc.row.name, expected_name
            );
            return vec![LintResult::error("E008", &doc.row.path, message)];
        }

        vec![]
    }
}

impl LintRule for MissingNameRule {
    fn codes(&self) -> &[&str] {
        &["E009"]
    }

    fn name(&self) -> &str {
        "missing-name"
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        if doc.row.name.is_empty() {
            let message = "is missing required 'name' field";
            return vec![LintResult::error("E009", &doc.row.path, message)];
        }

        vec![]
    }
}

impl LintRule for MissingDescriptionRule {
    fn codes(&self) -> &[&str] {
        &["E010"]
    }

    fn name(&self) -> &str {
        "missing-description"
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        if doc.row.description.is_empty() {
            let message = "is missing required 'description' field";
            return vec![LintResult::error("E010", &doc.row.path, message)];
        }

        vec![]
    }
}

impl LintRule for NestedClosedRule {
    fn codes(&self) -> &[&str] {
        &["E011"]
    }

    fn name(&self) -> &str {
        "nested-closed"
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        // Check for nested .closed directories in path
        let count = doc.row.path.matches("/.closed/").count();
        if count > 1 {
            let message = "is in a nested closed directory";
            return vec![LintResult::error("E011", &doc.row.path, message)];
        }

        vec![]
    }
}

impl LintRule for NonTaskInClosedRule {
    fn codes(&self) -> &[&str] {
        &["E012"]
    }

    fn name(&self) -> &str {
        "non-task-in-closed"
    }

    fn check(&self, doc: &LintDocument, _ctx: &LintContext<'_>) -> Vec<LintResult> {
        // Check if document is in a closed directory
        if !closed_directory::is_in_closed(&doc.row.path) {
            return vec![];
        }

        // Check if it's a knowledge base document (no task_type)
        if doc.row.task_type.is_none() {
            let message = "is a knowledge base document in closed directory";
            return vec![LintResult::error("E012", &doc.row.path, message)];
        }

        vec![]
    }
}

/// Finds all documents with a given ID.
fn find_documents_with_id(ctx: &LintContext<'_>, id: &str) -> Result<Vec<String>, LatticeError> {
    let all_paths = document_queries::all_paths(ctx.connection())?;

    // Build map of ID to paths
    let mut id_to_paths: HashMap<String, Vec<String>> = HashMap::new();

    for path in all_paths {
        if let Some(doc) = document_queries::lookup_by_path(ctx.connection(), &path)? {
            id_to_paths.entry(doc.id.clone()).or_default().push(path);
        }
    }

    Ok(id_to_paths.remove(id).unwrap_or_default())
}
