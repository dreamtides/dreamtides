use crate::cli::color_theme;
use crate::cli::commands::doctor_command::doctor_fixer::FixReport;
use crate::cli::commands::doctor_command::doctor_types::{
    CheckCategory, CheckStatus, DoctorConfig, DoctorReport,
};
use crate::cli::output_format::OutputFormat;

/// Outputs the doctor report in the appropriate format.
pub fn output_report(report: &DoctorReport, format: OutputFormat, config: &DoctorConfig) {
    match format {
        OutputFormat::Json => output_json(report),
        OutputFormat::Text | OutputFormat::Pretty => output_text(report, config),
    }
}

/// Outputs the fix report in the appropriate format.
pub fn output_fix_report(report: &FixReport, format: OutputFormat, config: &DoctorConfig) {
    match format {
        OutputFormat::Json => output_fix_json(report),
        OutputFormat::Text | OutputFormat::Pretty => output_fix_text(report, config),
    }
}

/// Outputs the report in JSON format.
fn output_json(report: &DoctorReport) {
    let json = serde_json::to_string_pretty(report)
        .unwrap_or_else(|e| panic!("Failed to serialize doctor report to JSON: {e}"));
    println!("{json}");
}

/// Outputs the report in text format.
fn output_text(report: &DoctorReport, config: &DoctorConfig) {
    let use_color = color_theme::colors_enabled();

    // Header
    println!("lat doctor v{}", report.version);
    println!();

    // Output by category
    for category in CheckCategory::all() {
        let category_checks = report.checks_for_category(*category);
        if category_checks.is_empty() {
            continue;
        }

        // In quiet mode, skip categories with no warnings or errors
        if config.quiet {
            let has_issues = category_checks
                .iter()
                .any(|c| matches!(c.status, CheckStatus::Warning | CheckStatus::Error));
            if !has_issues {
                continue;
            }
        }

        println!("{}", category.display_name());

        for check in &category_checks {
            // In quiet mode, skip passed and info checks
            if config.quiet && matches!(check.status, CheckStatus::Passed | CheckStatus::Info) {
                continue;
            }
            println!("{}", check.format_text(use_color));
        }
        println!();
    }

    // Summary line
    print_summary_line(report, use_color);

    // Issues section
    let issues = report.issues();
    if !issues.is_empty() {
        println!();
        let warnings: Vec<_> = issues.iter().filter(|c| c.status == CheckStatus::Warning).collect();
        let errors: Vec<_> = issues.iter().filter(|c| c.status == CheckStatus::Error).collect();

        if !errors.is_empty() {
            if use_color {
                println!("{}  ERRORS", color_theme::error("✖"));
            } else {
                println!("✖  ERRORS");
            }
            for (i, check) in errors.iter().enumerate() {
                println!("  {}. {}: {}", i + 1, check.name, check.message);
                if let Some(fix) = &check.fix_command {
                    println!("     └─ Fix: {fix}");
                }
            }
        }

        if !warnings.is_empty() {
            if use_color {
                println!("{}  WARNINGS", color_theme::warning("⚠"));
            } else {
                println!("⚠  WARNINGS");
            }
            for (i, check) in warnings.iter().enumerate() {
                println!("  {}. {}: {}", i + 1, check.name, check.message);
                if let Some(fix) = &check.fix_command {
                    println!("     └─ Fix: {fix}");
                }
            }
        }
    }
}

/// Outputs the fix report in JSON format.
fn output_fix_json(report: &FixReport) {
    #[derive(serde::Serialize)]
    struct FixReportJson<'a> {
        applied: usize,
        failed: usize,
        applied_descriptions: &'a [String],
        failed_descriptions: &'a [String],
    }

    let json_report = FixReportJson {
        applied: report.applied,
        failed: report.failed,
        applied_descriptions: &report.applied_descriptions,
        failed_descriptions: &report.failed_descriptions,
    };

    let json = serde_json::to_string_pretty(&json_report)
        .unwrap_or_else(|e| panic!("Failed to serialize fix report to JSON: {e}"));
    println!("{json}");
}

/// Outputs the fix report in text format.
fn output_fix_text(report: &FixReport, config: &DoctorConfig) {
    let use_color = color_theme::colors_enabled();
    let prefix = if config.dry_run { "DRY RUN: " } else { "" };

    println!();
    println!("{}FIX RESULTS", prefix);
    println!("──────────────────────────────────────────");

    if report.applied > 0 {
        if use_color {
            println!(
                "{} {} fix{} applied:",
                color_theme::success("✓"),
                report.applied,
                if report.applied == 1 { "" } else { "es" }
            );
        } else {
            println!(
                "✓ {} fix{} applied:",
                report.applied,
                if report.applied == 1 { "" } else { "es" }
            );
        }
        for desc in &report.applied_descriptions {
            println!("  • {}", desc);
        }
    }

    if report.failed > 0 {
        if use_color {
            println!(
                "{} {} fix{} failed:",
                color_theme::error("✖"),
                report.failed,
                if report.failed == 1 { "" } else { "es" }
            );
        } else {
            println!(
                "✖ {} fix{} failed:",
                report.failed,
                if report.failed == 1 { "" } else { "es" }
            );
        }
        for desc in &report.failed_descriptions {
            println!("  • {}", desc);
        }
    }

    if report.applied == 0 && report.failed == 0 {
        println!("No fixes applied.");
    }
}

/// Prints the summary line.
fn print_summary_line(report: &DoctorReport, use_color: bool) {
    let separator = "──────────────────────────────────────────";
    println!("{separator}");

    let summary = &report.summary;

    if use_color {
        print!("{} {} passed", color_theme::success("✓"), summary.passed);
        if summary.warnings > 0 {
            print!(
                "  {} {} warning{}",
                color_theme::warning("⚠"),
                summary.warnings,
                if summary.warnings == 1 { "" } else { "s" }
            );
        }
        if summary.info > 0 {
            print!("  {} {} info", color_theme::accent("ℹ"), summary.info);
        }
        if summary.failed > 0 {
            print!("  {} {} failed", color_theme::error("✖"), summary.failed);
        }
    } else {
        print!("✓ {} passed", summary.passed);
        if summary.warnings > 0 {
            print!(
                "  ⚠ {} warning{}",
                summary.warnings,
                if summary.warnings == 1 { "" } else { "s" }
            );
        }
        if summary.info > 0 {
            print!("  ℹ {} info", summary.info);
        }
        if summary.failed > 0 {
            print!("  ✖ {} failed", summary.failed);
        }
    }
    println!();
}
