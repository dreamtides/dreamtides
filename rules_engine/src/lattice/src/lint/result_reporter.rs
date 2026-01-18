use std::collections::BTreeMap;

use serde::Serialize;
use tracing::debug;

use crate::cli::output_format::OutputFormat;
use crate::cli::{color_theme, output_format};
use crate::lint::rule_engine::{LintResult, LintSummary, Severity};

/// JSON-serializable lint result for a single issue.
#[derive(Debug, Clone, Serialize)]
pub struct LintResultJson {
    pub code: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    pub message: String,
}

/// JSON-serializable summary of lint results.
#[derive(Debug, Clone, Serialize)]
pub struct LintSummaryJson {
    pub error_count: usize,
    pub warning_count: usize,
    pub affected_documents: usize,
}

/// JSON-serializable full lint report.
#[derive(Debug, Clone, Serialize)]
pub struct LintReportJson {
    pub documents_checked: usize,
    pub errors: Vec<LintResultJson>,
    pub warnings: Vec<LintResultJson>,
    pub summary: LintSummaryJson,
}

/// Reporter for lint results.
///
/// Formats and outputs lint results in either human-readable text format
/// or structured JSON. Text output groups results by file path and uses
/// colors for severity levels.
pub struct ResultReporter {
    output_format: OutputFormat,
}

/// Builds the JSON report structure from a lint summary.
pub fn build_json_report(summary: &LintSummary) -> LintReportJson {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for result in &summary.results {
        let json_result = LintResultJson {
            code: result.code.clone(),
            path: result.path.to_string_lossy().to_string(),
            line: result.line,
            message: result.message.clone(),
        };

        match result.severity {
            Severity::Error => errors.push(json_result),
            Severity::Warning => warnings.push(json_result),
        }
    }

    LintReportJson {
        documents_checked: summary.documents_checked,
        errors,
        warnings,
        summary: LintSummaryJson {
            error_count: summary.error_count,
            warning_count: summary.warning_count,
            affected_documents: summary.affected_documents,
        },
    }
}

impl ResultReporter {
    /// Creates a new result reporter with the specified output format.
    pub fn new(output_format: OutputFormat) -> Self {
        Self { output_format }
    }

    /// Formats the lint summary for output.
    ///
    /// For text output, groups results by file path with colored severity.
    /// For JSON output, returns the full structured report.
    pub fn format(&self, summary: &LintSummary) -> String {
        debug!(
            format = ?self.output_format,
            errors = summary.error_count,
            warnings = summary.warning_count,
            "Formatting lint report"
        );

        match self.output_format {
            OutputFormat::Json => format_json(summary),
            OutputFormat::Text | OutputFormat::Pretty => format_text(summary),
        }
    }

    /// Formats a header line shown before checking begins.
    pub fn format_header(&self, document_count: usize) -> String {
        if self.output_format.is_json() {
            String::new()
        } else {
            format!(
                "Checking {}...\n",
                output_format::format_count(document_count, "document", "documents")
            )
        }
    }

    /// Formats the final summary line shown after all results.
    pub fn format_summary_line(&self, summary: &LintSummary) -> String {
        if self.output_format.is_json() {
            return String::new();
        }

        if summary.is_clean() {
            format!("{}\n", color_theme::success("No issues found."))
        } else {
            let errors = output_format::format_count(summary.error_count, "error", "errors");
            let warnings =
                output_format::format_count(summary.warning_count, "warning", "warnings");
            let docs =
                output_format::format_count(summary.affected_documents, "document", "documents");

            format!("Found {errors}, {warnings} in {docs}.\n")
        }
    }
}

/// Formats the lint summary as JSON.
fn format_json(summary: &LintSummary) -> String {
    let report = build_json_report(summary);
    serde_json::to_string_pretty(&report)
        .unwrap_or_else(|e| panic!("Failed to serialize lint report to JSON: {e}"))
}

/// Formats the lint summary as human-readable text.
fn format_text(summary: &LintSummary) -> String {
    let mut output = String::new();

    let grouped = group_results_by_path(&summary.results);

    for (path, results) in grouped {
        output.push_str(&format!("{}:\n", color_theme::bold(&path)));

        for result in results {
            output.push_str(&format_result_line(result));
        }

        output.push('\n');
    }

    output
}

/// Groups lint results by file path for display.
///
/// Returns a map ordered by path for consistent output ordering.
fn group_results_by_path(results: &[LintResult]) -> BTreeMap<String, Vec<&LintResult>> {
    let mut grouped: BTreeMap<String, Vec<&LintResult>> = BTreeMap::new();

    for result in results {
        let path = result.path.to_string_lossy().to_string();
        grouped.entry(path).or_default().push(result);
    }

    grouped
}

/// Formats a single lint result line for text output.
fn format_result_line(result: &LintResult) -> String {
    let severity_str = format_severity(result.severity);
    let code_str = format!("[{}]", result.code);
    let line_str = format_line_number(result.line);
    let message = &result.message;

    format!("  {severity_str} {code_str}: {line_str}{message}\n")
}

/// Formats the severity with appropriate color.
fn format_severity(severity: Severity) -> String {
    match severity {
        Severity::Error => format!("{}", color_theme::error("Error")),
        Severity::Warning => format!("{}", color_theme::warning("Warning")),
    }
}

/// Formats the line number prefix if present.
fn format_line_number(line: Option<usize>) -> String {
    match line {
        Some(n) => format!("Line {n} "),
        None => String::new(),
    }
}
