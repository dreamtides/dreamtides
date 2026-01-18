use std::path::PathBuf;

use lattice::cli::output_format::OutputFormat;
use lattice::lint::result_reporter::{ResultReporter, build_json_report};
use lattice::lint::rule_engine::{LintResult, LintSummary};

fn sample_results() -> Vec<LintResult> {
    vec![
        LintResult::error("E002", PathBuf::from("path/to/doc1.md"), "links to unknown ID LYYYYY")
            .with_line(42),
        LintResult::warning(
            "W001",
            PathBuf::from("path/to/doc1.md"),
            "750 lines (recommended max: 500)",
        ),
        LintResult::warning(
            "W002",
            PathBuf::from("path/to/doc2.md"),
            "name is 78 characters (max: 64)",
        ),
    ]
}

fn sample_summary() -> LintSummary {
    LintSummary {
        documents_checked: 234,
        error_count: 1,
        warning_count: 2,
        affected_documents: 2,
        results: sample_results(),
    }
}

// =============================================================================
// JSON Output Tests
// =============================================================================

#[test]
fn json_output_has_correct_document_count() {
    let summary = sample_summary();
    let report = build_json_report(&summary);

    assert_eq!(report.documents_checked, 234, "documents_checked should be 234");
}

#[test]
fn json_output_separates_errors_and_warnings() {
    let summary = sample_summary();
    let report = build_json_report(&summary);

    assert_eq!(report.errors.len(), 1, "should have exactly 1 error in errors array");
    assert_eq!(report.warnings.len(), 2, "should have exactly 2 warnings in warnings array");
}

#[test]
fn json_summary_matches_counts() {
    let summary = sample_summary();
    let report = build_json_report(&summary);

    assert_eq!(report.summary.error_count, 1, "summary.error_count should be 1");
    assert_eq!(report.summary.warning_count, 2, "summary.warning_count should be 2");
    assert_eq!(report.summary.affected_documents, 2, "summary.affected_documents should be 2");
}

#[test]
fn json_error_fields_are_correct() {
    let summary = sample_summary();
    let report = build_json_report(&summary);

    let error = &report.errors[0];
    assert_eq!(error.code, "E002", "error code should be E002");
    assert_eq!(error.path, "path/to/doc1.md", "error path should be path/to/doc1.md");
    assert_eq!(error.line, Some(42), "error line should be Some(42)");
    assert_eq!(error.message, "links to unknown ID LYYYYY", "error message should match");
}

#[test]
fn json_warning_without_line_has_none() {
    let summary = sample_summary();
    let report = build_json_report(&summary);

    let warning = &report.warnings[0];
    assert!(warning.line.is_none(), "warning without line should have None");
}

// =============================================================================
// Text Output Tests
// =============================================================================

#[test]
fn text_output_groups_results_by_path() {
    let summary = sample_summary();
    let reporter = ResultReporter::new(OutputFormat::Text);
    let output = reporter.format(&summary);

    assert!(output.contains("path/to/doc1.md"), "output should contain doc1.md path");
    assert!(output.contains("path/to/doc2.md"), "output should contain doc2.md path");
}

#[test]
fn text_output_includes_error_codes() {
    let summary = sample_summary();
    let reporter = ResultReporter::new(OutputFormat::Text);
    let output = reporter.format(&summary);

    assert!(output.contains("[E002]"), "output should contain error code E002");
    assert!(output.contains("[W001]"), "output should contain warning code W001");
    assert!(output.contains("[W002]"), "output should contain warning code W002");
}

#[test]
fn text_output_includes_line_number_when_present() {
    let summary = sample_summary();
    let reporter = ResultReporter::new(OutputFormat::Text);
    let output = reporter.format(&summary);

    assert!(
        output.contains("Line 42"),
        "output should contain Line 42 for the error with line number"
    );
}

#[test]
fn text_output_includes_severity_labels() {
    let summary = sample_summary();
    let reporter = ResultReporter::new(OutputFormat::Text);
    let output = reporter.format(&summary);

    assert!(output.contains("Error"), "output should contain 'Error' label");
    assert!(output.contains("Warning"), "output should contain 'Warning' label");
}

// =============================================================================
// Summary Line Tests
// =============================================================================

#[test]
fn summary_line_shows_no_issues_for_clean_run() {
    let reporter = ResultReporter::new(OutputFormat::Text);
    let summary = LintSummary::default();
    let line = reporter.format_summary_line(&summary);

    assert!(line.contains("No issues found"), "clean run should show 'No issues found'");
}

#[test]
fn summary_line_shows_singular_error_count() {
    let reporter = ResultReporter::new(OutputFormat::Text);
    let summary = sample_summary();
    let line = reporter.format_summary_line(&summary);

    assert!(line.contains("1 error"), "summary should contain '1 error' (singular)");
}

#[test]
fn summary_line_shows_plural_warnings_count() {
    let reporter = ResultReporter::new(OutputFormat::Text);
    let summary = sample_summary();
    let line = reporter.format_summary_line(&summary);

    assert!(line.contains("2 warnings"), "summary should contain '2 warnings' (plural)");
}

#[test]
fn summary_line_shows_affected_documents() {
    let reporter = ResultReporter::new(OutputFormat::Text);
    let summary = sample_summary();
    let line = reporter.format_summary_line(&summary);

    assert!(line.contains("2 documents"), "summary should contain '2 documents'");
}

// =============================================================================
// Header Tests
// =============================================================================

#[test]
fn header_formats_singular_document_count() {
    let reporter = ResultReporter::new(OutputFormat::Text);
    let header = reporter.format_header(1);

    assert!(header.contains("1 document"), "singular should use 'document' not 'documents'");
}

#[test]
fn header_formats_plural_document_count() {
    let reporter = ResultReporter::new(OutputFormat::Text);
    let header = reporter.format_header(234);

    assert!(header.contains("234 documents"), "plural should use 'documents'");
}

#[test]
fn header_includes_checking_prefix() {
    let reporter = ResultReporter::new(OutputFormat::Text);
    let header = reporter.format_header(100);

    assert!(header.starts_with("Checking"), "header should start with 'Checking'");
}

// =============================================================================
// JSON Format Suppression Tests
// =============================================================================

#[test]
fn json_format_returns_empty_header() {
    let reporter = ResultReporter::new(OutputFormat::Json);
    let header = reporter.format_header(100);

    assert!(header.is_empty(), "JSON format should return empty header");
}

#[test]
fn json_format_returns_empty_summary_line() {
    let reporter = ResultReporter::new(OutputFormat::Json);
    let summary = sample_summary();
    let line = reporter.format_summary_line(&summary);

    assert!(line.is_empty(), "JSON format should return empty summary line");
}

#[test]
fn json_format_outputs_valid_json() {
    let summary = sample_summary();
    let reporter = ResultReporter::new(OutputFormat::Json);
    let output = reporter.format(&summary);

    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&output);
    assert!(parsed.is_ok(), "JSON output should be valid JSON: {output}");
}
