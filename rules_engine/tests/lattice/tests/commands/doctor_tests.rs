//! Tests for the `lat doctor` command.

use std::process::ExitCode;

use lattice::cli::commands::doctor_command::doctor_types::{
    CheckCategory, CheckResult, CheckStatus, DoctorConfig, DoctorReport, DoctorSummary,
    EXIT_CODE_WARNINGS_ONLY, compute_exit_code,
};
use lattice::cli::maintenance_args::DoctorArgs;
use lattice::error::exit_codes;

fn default_args() -> DoctorArgs {
    DoctorArgs { fix: false, dry_run: false, deep: false, quiet: false }
}

// ============================================================================
// Exit Code Computation Tests
// ============================================================================

#[test]
fn compute_exit_code_returns_success_when_all_passed() {
    let checks = vec![
        CheckResult::passed(CheckCategory::Core, "test", "passed"),
        CheckResult::passed(CheckCategory::Index, "test2", "also passed"),
    ];
    let report = DoctorReport::new(checks);

    assert_eq!(
        compute_exit_code(&report),
        exit_codes::success(),
        "All passed checks should return exit code 0"
    );
}

#[test]
fn compute_exit_code_returns_success_when_only_info() {
    let checks = vec![
        CheckResult::passed(CheckCategory::Core, "test", "passed"),
        CheckResult::info(CheckCategory::Git, "info-check", "just info"),
    ];
    let report = DoctorReport::new(checks);

    assert_eq!(
        compute_exit_code(&report),
        exit_codes::success(),
        "Info-only (no warnings or errors) should return exit code 0"
    );
}

#[test]
fn compute_exit_code_returns_validation_error_when_errors_present() {
    let checks = vec![
        CheckResult::passed(CheckCategory::Core, "test", "passed"),
        CheckResult::error(CheckCategory::Core, "broken", "something broke"),
    ];
    let report = DoctorReport::new(checks);

    assert_eq!(
        compute_exit_code(&report),
        exit_codes::validation_error(),
        "Errors should return exit code 2"
    );
}

#[test]
fn compute_exit_code_returns_warnings_only_code_when_no_errors() {
    let checks = vec![
        CheckResult::passed(CheckCategory::Core, "test", "passed"),
        CheckResult::warning(CheckCategory::Claims, "stale", "stale claim found"),
    ];
    let report = DoctorReport::new(checks);

    assert_eq!(
        compute_exit_code(&report),
        ExitCode::from(EXIT_CODE_WARNINGS_ONLY),
        "Warnings without errors should return exit code 3"
    );
}

#[test]
fn compute_exit_code_prefers_errors_over_warnings() {
    let checks = vec![
        CheckResult::warning(CheckCategory::Claims, "warn1", "warning"),
        CheckResult::error(CheckCategory::Core, "err1", "error"),
        CheckResult::warning(CheckCategory::Config, "warn2", "another warning"),
    ];
    let report = DoctorReport::new(checks);

    assert_eq!(
        compute_exit_code(&report),
        exit_codes::validation_error(),
        "Errors should take precedence over warnings"
    );
}

// ============================================================================
// DoctorConfig From DoctorArgs Tests
// ============================================================================

#[test]
fn doctor_config_from_default_args() {
    let args = default_args();
    let config = DoctorConfig::from(&args);

    assert!(!config.fix);
    assert!(!config.dry_run);
    assert!(!config.deep);
    assert!(!config.quiet);
}

#[test]
fn doctor_config_preserves_all_flags() {
    let args = DoctorArgs { fix: true, dry_run: true, deep: true, quiet: true };
    let config = DoctorConfig::from(&args);

    assert!(config.fix);
    assert!(config.dry_run);
    assert!(config.deep);
    assert!(config.quiet);
}

// ============================================================================
// DoctorSummary Tests
// ============================================================================

#[test]
fn summary_starts_empty() {
    let summary = DoctorSummary::default();
    assert_eq!(summary.passed, 0);
    assert_eq!(summary.info, 0);
    assert_eq!(summary.warnings, 0);
    assert_eq!(summary.failed, 0);
}

#[test]
fn summary_add_increments_correct_counter() {
    let mut summary = DoctorSummary::default();

    summary.add(CheckStatus::Passed);
    assert_eq!(summary.passed, 1);

    summary.add(CheckStatus::Info);
    assert_eq!(summary.info, 1);

    summary.add(CheckStatus::Warning);
    assert_eq!(summary.warnings, 1);

    summary.add(CheckStatus::Error);
    assert_eq!(summary.failed, 1);
}

#[test]
fn summary_has_errors_is_true_when_failed_positive() {
    let mut summary = DoctorSummary::default();
    assert!(!summary.has_errors());

    summary.add(CheckStatus::Error);
    assert!(summary.has_errors());
}

#[test]
fn summary_has_warnings_is_true_when_warnings_positive() {
    let mut summary = DoctorSummary::default();
    assert!(!summary.has_warnings());

    summary.add(CheckStatus::Warning);
    assert!(summary.has_warnings());
}

// ============================================================================
// CheckResult Builder Tests
// ============================================================================

#[test]
fn check_result_passed_has_correct_status() {
    let result = CheckResult::passed(CheckCategory::Core, "test", "message");
    assert_eq!(result.status, CheckStatus::Passed);
    assert_eq!(result.category, CheckCategory::Core);
    assert_eq!(result.name, "test");
    assert_eq!(result.message, "message");
    assert!(result.details.is_empty());
    assert!(!result.fixable);
    assert!(result.fix_command.is_none());
}

#[test]
fn check_result_info_has_correct_status() {
    let result = CheckResult::info(CheckCategory::Git, "test", "info message");
    assert_eq!(result.status, CheckStatus::Info);
}

#[test]
fn check_result_warning_has_correct_status() {
    let result = CheckResult::warning(CheckCategory::Claims, "test", "warning message");
    assert_eq!(result.status, CheckStatus::Warning);
}

#[test]
fn check_result_error_has_correct_status() {
    let result = CheckResult::error(CheckCategory::Index, "test", "error message");
    assert_eq!(result.status, CheckStatus::Error);
}

#[test]
fn check_result_with_details_adds_details() {
    let result = CheckResult::warning(CheckCategory::Claims, "stale", "stale claims")
        .with_details(vec!["LABC01".to_string(), "LDEF02".to_string()]);

    assert_eq!(result.details.len(), 2);
    assert_eq!(result.details[0], "LABC01");
    assert_eq!(result.details[1], "LDEF02");
}

#[test]
fn check_result_with_fix_sets_fixable_and_command() {
    let result = CheckResult::error(CheckCategory::Index, "missing", "index missing")
        .with_fix("lat doctor --fix");

    assert!(result.fixable);
    assert_eq!(result.fix_command, Some("lat doctor --fix".to_string()));
}

// ============================================================================
// CheckStatus Tests
// ============================================================================

#[test]
fn check_status_is_failure_only_for_error() {
    assert!(!CheckStatus::Passed.is_failure());
    assert!(!CheckStatus::Info.is_failure());
    assert!(!CheckStatus::Warning.is_failure());
    assert!(CheckStatus::Error.is_failure());
}

#[test]
fn check_status_is_warning_only_for_warning() {
    assert!(!CheckStatus::Passed.is_warning());
    assert!(!CheckStatus::Info.is_warning());
    assert!(CheckStatus::Warning.is_warning());
    assert!(!CheckStatus::Error.is_warning());
}

#[test]
fn check_status_icons_are_correct() {
    assert_eq!(CheckStatus::Passed.icon(), "✓");
    assert_eq!(CheckStatus::Info.icon(), "ℹ");
    assert_eq!(CheckStatus::Warning.icon(), "⚠");
    assert_eq!(CheckStatus::Error.icon(), "✖");
}

// ============================================================================
// CheckCategory Tests
// ============================================================================

#[test]
fn check_category_display_names() {
    assert_eq!(CheckCategory::Core.display_name(), "CORE SYSTEM");
    assert_eq!(CheckCategory::Index.display_name(), "INDEX INTEGRITY");
    assert_eq!(CheckCategory::Git.display_name(), "GIT INTEGRATION");
    assert_eq!(CheckCategory::Config.display_name(), "CONFIGURATION");
    assert_eq!(CheckCategory::Claims.display_name(), "CLAIMS");
    assert_eq!(CheckCategory::Skills.display_name(), "SKILLS");
}

#[test]
fn check_category_all_returns_all_categories_in_order() {
    let all = CheckCategory::all();
    assert_eq!(all.len(), 6);
    assert_eq!(all[0], CheckCategory::Core);
    assert_eq!(all[1], CheckCategory::Index);
    assert_eq!(all[2], CheckCategory::Git);
    assert_eq!(all[3], CheckCategory::Config);
    assert_eq!(all[4], CheckCategory::Claims);
    assert_eq!(all[5], CheckCategory::Skills);
}

// ============================================================================
// DoctorReport Tests
// ============================================================================

#[test]
fn doctor_report_new_computes_summary() {
    let checks = vec![
        CheckResult::passed(CheckCategory::Core, "p1", "passed"),
        CheckResult::passed(CheckCategory::Core, "p2", "passed"),
        CheckResult::info(CheckCategory::Git, "i1", "info"),
        CheckResult::warning(CheckCategory::Claims, "w1", "warning"),
        CheckResult::error(CheckCategory::Index, "e1", "error"),
    ];
    let report = DoctorReport::new(checks);

    assert_eq!(report.summary.passed, 2);
    assert_eq!(report.summary.info, 1);
    assert_eq!(report.summary.warnings, 1);
    assert_eq!(report.summary.failed, 1);
}

#[test]
fn doctor_report_checks_for_category_filters_correctly() {
    let checks = vec![
        CheckResult::passed(CheckCategory::Core, "c1", "core check 1"),
        CheckResult::passed(CheckCategory::Core, "c2", "core check 2"),
        CheckResult::passed(CheckCategory::Git, "g1", "git check"),
    ];
    let report = DoctorReport::new(checks);

    let core_checks = report.checks_for_category(CheckCategory::Core);
    assert_eq!(core_checks.len(), 2);

    let git_checks = report.checks_for_category(CheckCategory::Git);
    assert_eq!(git_checks.len(), 1);

    let config_checks = report.checks_for_category(CheckCategory::Config);
    assert!(config_checks.is_empty());
}

#[test]
fn doctor_report_issues_returns_only_warnings_and_errors() {
    let checks = vec![
        CheckResult::passed(CheckCategory::Core, "p1", "passed"),
        CheckResult::info(CheckCategory::Git, "i1", "info"),
        CheckResult::warning(CheckCategory::Claims, "w1", "warning"),
        CheckResult::error(CheckCategory::Index, "e1", "error"),
    ];
    let report = DoctorReport::new(checks);

    let issues = report.issues();
    assert_eq!(issues.len(), 2);
    assert!(issues.iter().any(|i| i.status == CheckStatus::Warning));
    assert!(issues.iter().any(|i| i.status == CheckStatus::Error));
}

#[test]
fn doctor_report_includes_version() {
    let report = DoctorReport::new(vec![]);
    assert!(!report.version.is_empty(), "Report should include version string");
}
