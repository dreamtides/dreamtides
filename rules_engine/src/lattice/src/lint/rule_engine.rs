use std::collections::HashSet;
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use serde::Serialize;
use tracing::{debug, info, warn};

use crate::document::document_reader::{self, Document, ReadResult};
use crate::error::error_types::LatticeError;
use crate::index::document_filter::DocumentFilter;
use crate::index::document_queries;
use crate::index::document_types::DocumentRow;

/// Trait for lint rules that check documents for issues.
///
/// Rules implement this trait to define their checking logic. Each rule
/// receives a document and context, returning zero or more lint results.
pub trait LintRule: Send + Sync {
    /// Returns the rule codes this rule can emit (e.g., ["E001", "E002"]).
    fn codes(&self) -> &[&str];

    /// Returns a human-readable name for this rule.
    fn name(&self) -> &str;

    /// Returns whether this rule requires the full document body.
    ///
    /// If false, only the index metadata (DocumentRow) will be provided,
    /// which is faster. Return true if the rule needs to examine the
    /// markdown body content.
    fn requires_document_body(&self) -> bool {
        false
    }

    /// Checks a document and returns any lint issues found.
    fn check(&self, doc: &LintDocument, ctx: &LintContext<'_>) -> Vec<LintResult>;
}

/// Severity level for lint results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Warnings are advisory and don't block operations.
    Warning,
    /// Errors prevent operations and must be fixed.
    Error,
}

/// A single lint result from checking a document.
#[derive(Debug, Clone, Serialize)]
pub struct LintResult {
    /// Rule code (e.g., "E001", "W001").
    pub code: String,
    /// Severity level (error or warning).
    pub severity: Severity,
    /// Path to the document relative to repository root.
    pub path: PathBuf,
    /// Optional line number where the issue was found (1-indexed).
    pub line: Option<usize>,
    /// Human-readable description of the issue.
    pub message: String,
}

/// Summary of lint execution results.
#[derive(Debug, Clone, Default, Serialize)]
pub struct LintSummary {
    /// Total number of documents checked.
    pub documents_checked: usize,
    /// Number of error-level issues found.
    pub error_count: usize,
    /// Number of warning-level issues found.
    pub warning_count: usize,
    /// Number of documents with at least one issue.
    pub affected_documents: usize,
    /// All lint results collected.
    pub results: Vec<LintResult>,
}

/// Configuration for lint execution.
#[derive(Debug, Clone, Default)]
pub struct LintConfig {
    /// If true, only return error-level results (suppress warnings).
    pub errors_only: bool,
    /// If set, only check documents under this path prefix.
    pub path_prefix: Option<PathBuf>,
}

/// Context provided to lint rules during execution.
///
/// Provides access to the SQLite index for cross-document checks and the
/// repository root for path resolution.
pub struct LintContext<'a> {
    conn: &'a Connection,
    repo_root: &'a Path,
}

/// A document with both its index metadata and parsed content.
///
/// Some rules only need index metadata (fast path), while others need the full
/// parsed document with body content.
pub struct LintDocument {
    /// Cached metadata from the index.
    pub row: DocumentRow,
    /// Full parsed document (lazily loaded on demand).
    pub document: Option<Document>,
    /// Validation diagnostics from parsing.
    pub read_result: Option<ReadResult>,
}

/// Executes lint rules against all indexed documents.
///
/// This is the main entry point for the lint system. It queries all documents
/// from the index, applies path filtering, and runs each rule against
/// matching documents.
pub fn execute_rules(
    ctx: &LintContext<'_>,
    rules: &[&dyn LintRule],
    config: &LintConfig,
) -> Result<LintSummary, LatticeError> {
    let filter = DocumentFilter::default();
    let all_docs = document_queries::query(ctx.conn, &filter)?;

    let docs_to_check: Vec<_> = all_docs
        .into_iter()
        .filter(|doc| matches_path_filter(&doc.path, &config.path_prefix))
        .collect();

    debug!(count = docs_to_check.len(), "Documents to lint");

    execute_rules_on_documents(ctx, rules, config, docs_to_check)
}

/// Executes lint rules against a specific list of documents.
///
/// Use this when you have a pre-filtered list of documents (e.g., staged files
/// only). Documents are loaded from the filesystem as needed based on rule
/// requirements.
pub fn execute_rules_on_documents(
    ctx: &LintContext<'_>,
    rules: &[&dyn LintRule],
    config: &LintConfig,
    documents: Vec<DocumentRow>,
) -> Result<LintSummary, LatticeError> {
    if rules.is_empty() {
        info!("No lint rules registered, skipping");
        return Ok(LintSummary { documents_checked: documents.len(), ..Default::default() });
    }

    let any_needs_body = rules.iter().any(|r| r.requires_document_body());
    let mut summary = LintSummary { documents_checked: documents.len(), ..Default::default() };
    let mut affected_paths: HashSet<PathBuf> = HashSet::new();

    for row in documents {
        let lint_doc = load_lint_document(ctx, row, any_needs_body)?;
        let doc_results = check_document_with_rules(&lint_doc, ctx, rules);

        for result in doc_results {
            if config.errors_only && !result.severity.is_error() {
                continue;
            }

            affected_paths.insert(result.path.clone());
            match result.severity {
                Severity::Error => summary.error_count += 1,
                Severity::Warning => summary.warning_count += 1,
            }
            summary.results.push(result);
        }
    }

    summary.affected_documents = affected_paths.len();

    info!(
        documents = summary.documents_checked,
        errors = summary.error_count,
        warnings = summary.warning_count,
        affected = summary.affected_documents,
        "Lint execution complete"
    );

    Ok(summary)
}

/// Executes lint rules against a single document by path.
///
/// Useful for linting a single file (e.g., during incremental validation).
pub fn execute_rules_on_path(
    ctx: &LintContext<'_>,
    rules: &[&dyn LintRule],
    config: &LintConfig,
    path: &Path,
) -> Result<LintSummary, LatticeError> {
    let path_str = path.to_string_lossy();
    let row = document_queries::lookup_by_path(ctx.conn, &path_str)?;

    match row {
        Some(doc) => execute_rules_on_documents(ctx, rules, config, vec![doc]),
        None => {
            warn!(path = %path.display(), "Document not found in index");
            Ok(LintSummary::default())
        }
    }
}

impl LintResult {
    /// Creates a new error-level lint result.
    pub fn error(
        code: impl Into<String>,
        path: impl Into<PathBuf>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: Severity::Error,
            path: path.into(),
            line: None,
            message: message.into(),
        }
    }

    /// Creates a new warning-level lint result.
    pub fn warning(
        code: impl Into<String>,
        path: impl Into<PathBuf>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: Severity::Warning,
            path: path.into(),
            line: None,
            message: message.into(),
        }
    }

    /// Adds a line number to this result.
    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }
}

impl Severity {
    /// Returns the string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Warning => "warning",
            Severity::Error => "error",
        }
    }

    /// Returns true if this is an error severity.
    pub fn is_error(&self) -> bool {
        matches!(self, Severity::Error)
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl<'a> LintContext<'a> {
    /// Creates a new lint context.
    pub fn new(conn: &'a Connection, repo_root: &'a Path) -> Self {
        Self { conn, repo_root }
    }

    /// Returns the database connection for cross-document queries.
    pub fn connection(&self) -> &Connection {
        self.conn
    }

    /// Returns the repository root path.
    pub fn repo_root(&self) -> &Path {
        self.repo_root
    }

    /// Looks up a document by ID from the index.
    pub fn lookup_document(&self, id: &str) -> Result<Option<DocumentRow>, LatticeError> {
        document_queries::lookup_by_id(self.conn, id)
    }

    /// Checks if a document ID exists in the index.
    pub fn document_exists(&self, id: &str) -> Result<bool, LatticeError> {
        document_queries::exists(self.conn, id)
    }
}

impl LintConfig {
    /// Creates a new lint config with errors_only setting.
    pub fn with_errors_only(mut self, errors_only: bool) -> Self {
        self.errors_only = errors_only;
        self
    }

    /// Creates a new lint config with path prefix filtering.
    pub fn with_path_prefix(mut self, prefix: impl Into<PathBuf>) -> Self {
        self.path_prefix = Some(prefix.into());
        self
    }
}

impl LintSummary {
    /// Returns true if there are any errors.
    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    /// Returns true if there are any warnings.
    pub fn has_warnings(&self) -> bool {
        self.warning_count > 0
    }

    /// Returns true if there are no issues.
    pub fn is_clean(&self) -> bool {
        self.error_count == 0 && self.warning_count == 0
    }
}

fn load_lint_document(
    ctx: &LintContext<'_>,
    row: DocumentRow,
    load_body: bool,
) -> Result<LintDocument, LatticeError> {
    if !load_body {
        return Ok(LintDocument { row, document: None, read_result: None });
    }

    let full_path = ctx.repo_root.join(&row.path);

    match document_reader::read_and_validate(&full_path) {
        Ok(read_result) => Ok(LintDocument {
            row,
            document: Some(read_result.document.clone()),
            read_result: Some(read_result),
        }),
        Err(e) => {
            warn!(path = %full_path.display(), error = %e, "Failed to read document for linting");
            Ok(LintDocument { row, document: None, read_result: None })
        }
    }
}

fn check_document_with_rules(
    doc: &LintDocument,
    ctx: &LintContext<'_>,
    rules: &[&dyn LintRule],
) -> Vec<LintResult> {
    let mut results = Vec::new();

    for rule in rules {
        if rule.requires_document_body() && doc.document.is_none() {
            debug!(
                rule = rule.name(),
                path = doc.row.path,
                "Skipping rule that requires body on unreadable document"
            );
            continue;
        }

        let rule_results = rule.check(doc, ctx);
        debug!(
            rule = rule.name(),
            path = doc.row.path,
            issues = rule_results.len(),
            "Rule execution complete"
        );
        results.extend(rule_results);
    }

    results
}

fn matches_path_filter(path: &str, filter: &Option<PathBuf>) -> bool {
    let Some(prefix) = filter else {
        return true;
    };
    let prefix_str = prefix.to_string_lossy();
    path.starts_with(prefix_str.as_ref())
}
