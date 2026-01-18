use std::process::ExitCode;

use serde::Serialize;

use crate::cli::output_format::OutputFormat;
use crate::error::error_types::LatticeError;
use crate::error::exit_codes;
use crate::lint::autofix_engine::AutofixSummary;
use crate::lint::result_reporter;
use crate::lint::result_reporter::{LintReportJson, ResultReporter};
use crate::lint::rule_engine::LintSummary;

/// Exit code when only warnings were found (no errors).
const EXIT_CODE_WARNINGS_ONLY: u8 = 3;

/// JSON report with optional fix information.
#[derive(Debug, Clone, Serialize)]
pub struct CheckReportJson {
    #[serde(flatten)]
    pub lint: LintReportJson,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixes: Option<FixSummaryJson>,
}

/// JSON representation of fix results.
#[derive(Debug, Clone, Serialize)]
pub struct FixSummaryJson {
    pub documents_fixed: usize,
    pub total_fixes: usize,
    pub skipped_fixes: usize,
}

/// Prints the check command output in the appropriate format.
///
/// Handles both text and JSON output modes, including optional fix summary.
pub fn print_output(
    output_format: OutputFormat,
    summary: &LintSummary,
    fix_summary: Option<&AutofixSummary>,
) {
    match output_format {
        OutputFormat::Json => print_json_output(summary, fix_summary),
        OutputFormat::Text | OutputFormat::Pretty => print_text_output(summary, fix_summary),
    }
}

/// Determines the appropriate exit code based on lint results.
///
/// Exit codes:
/// - 0: No errors or warnings
/// - 2: Validation errors found (VALIDATION_ERROR)
/// - 3: Only warnings found
///
/// This function calls std::process::exit directly for non-success cases
/// because the lint results have already been printed and we don't want
/// the command dispatch to print additional error messages.
pub fn exit_with_code(summary: &LintSummary) -> Result<(), LatticeError> {
    if summary.has_errors() {
        std::process::exit(exit_codes::VALIDATION_ERROR as i32);
    } else if summary.has_warnings() {
        std::process::exit(EXIT_CODE_WARNINGS_ONLY as i32);
    } else {
        Ok(())
    }
}

/// Returns the exit code for the given summary without exiting.
///
/// Useful for testing.
pub fn compute_exit_code(summary: &LintSummary) -> ExitCode {
    if summary.has_errors() {
        exit_codes::validation_error()
    } else if summary.has_warnings() {
        ExitCode::from(EXIT_CODE_WARNINGS_ONLY)
    } else {
        exit_codes::success()
    }
}

/// Prints output in JSON format.
fn print_json_output(summary: &LintSummary, fix_summary: Option<&AutofixSummary>) {
    let lint_report = result_reporter::build_json_report(summary);
    let fixes = fix_summary.map(|f| FixSummaryJson {
        documents_fixed: f.documents_fixed,
        total_fixes: f.total_fixes,
        skipped_fixes: f.skipped_fixes,
    });

    let report = CheckReportJson { lint: lint_report, fixes };

    let json = serde_json::to_string_pretty(&report)
        .unwrap_or_else(|e| panic!("Failed to serialize check report to JSON: {e}"));

    println!("{json}");
}

/// Prints output in human-readable text format.
fn print_text_output(summary: &LintSummary, fix_summary: Option<&AutofixSummary>) {
    let reporter = ResultReporter::new(OutputFormat::Text);

    // Print header
    let header = reporter.format_header(summary.documents_checked);
    if !header.is_empty() {
        print!("{header}");
    }

    // Print lint results
    let results = reporter.format(summary);
    if !results.is_empty() {
        print!("{results}");
    }

    // Print fix summary if present
    if let Some(fixes) = fix_summary {
        print_fix_summary(fixes);
    }

    // Print summary line
    let summary_line = reporter.format_summary_line(summary);
    print!("{summary_line}");
}

/// Prints the fix summary in text format.
fn print_fix_summary(fixes: &AutofixSummary) {
    if fixes.documents_fixed == 0 && fixes.skipped_fixes == 0 {
        return;
    }

    println!();
    if fixes.documents_fixed > 0 {
        println!("Fixed {} issue(s) in {} document(s).", fixes.total_fixes, fixes.documents_fixed);
    }

    if fixes.skipped_fixes > 0 {
        println!("Skipped {} unfixable issue(s).", fixes.skipped_fixes);
    }
    println!();
}
