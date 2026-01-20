use std::process::ExitCode;

use tracing::{info, instrument};

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::commands::doctor_command::doctor_types::{DoctorConfig, DoctorReport};
use crate::cli::commands::doctor_command::{doctor_checks, doctor_output, doctor_types};
use crate::cli::maintenance_args::DoctorArgs;
use crate::cli::output_format::OutputFormat;
use crate::error::error_types::LatticeError;

/// Executes the `lat doctor` command.
#[instrument(skip_all, name = "doctor_command")]
pub fn execute(context: CommandContext, args: DoctorArgs) -> LatticeResult<()> {
    info!(
        fix = args.fix,
        dry_run = args.dry_run,
        deep = args.deep,
        quiet = args.quiet,
        "Executing doctor command"
    );

    validate_args(&args)?;

    let config = DoctorConfig::from(&args);
    let checks = doctor_checks::run_all_checks(&context, &config)?;
    let report = DoctorReport::new(checks);

    let output_format = OutputFormat::from_flags(context.global.json, false);
    doctor_output::output_report(&report, output_format, &config);

    exit_with_code(&report)
}

/// Validates doctor command arguments.
fn validate_args(args: &DoctorArgs) -> LatticeResult<()> {
    if args.dry_run && !args.fix {
        return Err(LatticeError::ConflictingOptions {
            option1: "--dry-run".to_string(),
            option2: "--fix (required when using --dry-run)".to_string(),
        });
    }
    Ok(())
}

/// Determines the exit code based on the doctor report.
///
/// Exit codes:
/// - 0: All checks passed (or only info)
/// - 1: System error during checks (handled elsewhere via panic)
/// - 2: One or more checks failed (errors)
/// - 3: One or more warnings (no errors)
fn exit_with_code(report: &DoctorReport) -> LatticeResult<()> {
    let code = doctor_types::compute_exit_code(report);
    // Exit code 0 means success, allow normal return
    // Non-zero codes require explicit exit since the command framework
    // only supports success (0) or error codes from LatticeError
    if code != ExitCode::SUCCESS {
        // Convert ExitCode to i32 for std::process::exit
        // ExitCode doesn't expose its value, so we recompute it
        let code_value = if report.summary.has_errors() { 2 } else { 3 };
        std::process::exit(code_value);
    }
    Ok(())
}
