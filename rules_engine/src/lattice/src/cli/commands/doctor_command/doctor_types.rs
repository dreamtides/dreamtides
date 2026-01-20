use std::process::ExitCode;

use serde::Serialize;

use crate::cli::color_theme;
use crate::cli::maintenance_args::DoctorArgs;
use crate::error::exit_codes;

/// Version string for doctor output.
pub const DOCTOR_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Exit code when only warnings were found (no errors).
pub const EXIT_CODE_WARNINGS_ONLY: u8 = 3;

/// Status result for an individual check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    /// Check passed successfully.
    Passed,
    /// Informational notice (not a problem).
    Info,
    /// Warning that should be addressed but isn't critical.
    Warning,
    /// Error that requires attention.
    Error,
}

/// Categories of health checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckCategory {
    /// Core system checks (installation, index existence, schema, WAL).
    Core,
    /// Index integrity checks (filesystem sync, coverage, duplicates).
    Index,
    /// Git integration checks (repository, edge cases, working tree).
    Git,
    /// Configuration checks (user config, repo config, client ID).
    Config,
    /// Claims checks (stale, missing tasks, orphaned worktrees).
    Claims,
    /// Skills checks (symlink validity, coverage, staleness).
    Skills,
}

/// Result of a single doctor check.
#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    /// The category this check belongs to.
    pub category: CheckCategory,
    /// Short identifier for the check (e.g., "installation", "stale_claims").
    pub name: String,
    /// The status of the check.
    pub status: CheckStatus,
    /// Human-readable message describing the result.
    pub message: String,
    /// Optional additional details (e.g., affected IDs, paths).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<String>,
    /// Whether this issue can be automatically fixed.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub fixable: bool,
    /// Command to fix this issue, if fixable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_command: Option<String>,
}

/// Summary statistics from running all doctor checks.
#[derive(Debug, Clone, Default, Serialize)]
pub struct DoctorSummary {
    /// Number of checks that passed.
    pub passed: usize,
    /// Number of informational notices.
    pub info: usize,
    /// Number of warnings.
    pub warnings: usize,
    /// Number of errors.
    pub failed: usize,
}

/// Complete report from the doctor command.
#[derive(Debug, Clone, Serialize)]
pub struct DoctorReport {
    /// Version of the lat CLI.
    pub version: String,
    /// All check results.
    pub checks: Vec<CheckResult>,
    /// Summary statistics.
    pub summary: DoctorSummary,
}

/// Configuration for running doctor checks.
#[derive(Debug, Clone, Default)]
pub struct DoctorConfig {
    /// Whether to automatically repair fixable issues.
    pub fix: bool,
    /// Whether to preview fixes without applying them.
    pub dry_run: bool,
    /// Whether to run additional thorough checks.
    pub deep: bool,
    /// Whether to only show warnings and errors (quiet mode).
    pub quiet: bool,
}

/// Returns the exit code for the given report without exiting.
///
/// Useful for testing.
pub fn compute_exit_code(report: &DoctorReport) -> ExitCode {
    if report.summary.has_errors() {
        exit_codes::validation_error()
    } else if report.summary.has_warnings() {
        ExitCode::from(EXIT_CODE_WARNINGS_ONLY)
    } else {
        exit_codes::success()
    }
}

impl CheckStatus {
    /// Returns true if this status is considered a failure.
    pub fn is_failure(self) -> bool {
        matches!(self, CheckStatus::Error)
    }

    /// Returns true if this status is a warning.
    pub fn is_warning(self) -> bool {
        matches!(self, CheckStatus::Warning)
    }

    /// Returns the icon for this status.
    pub fn icon(self) -> &'static str {
        match self {
            CheckStatus::Passed => "✓",
            CheckStatus::Info => "ℹ",
            CheckStatus::Warning => "⚠",
            CheckStatus::Error => "✖",
        }
    }
}

impl CheckCategory {
    /// Returns the display name for this category.
    pub fn display_name(self) -> &'static str {
        match self {
            CheckCategory::Core => "CORE SYSTEM",
            CheckCategory::Index => "INDEX INTEGRITY",
            CheckCategory::Git => "GIT INTEGRATION",
            CheckCategory::Config => "CONFIGURATION",
            CheckCategory::Claims => "CLAIMS",
            CheckCategory::Skills => "SKILLS",
        }
    }

    /// Returns all categories in display order.
    pub fn all() -> &'static [CheckCategory] {
        &[
            CheckCategory::Core,
            CheckCategory::Index,
            CheckCategory::Git,
            CheckCategory::Config,
            CheckCategory::Claims,
            CheckCategory::Skills,
        ]
    }
}

impl CheckResult {
    /// Creates a new check result with passed status.
    pub fn passed(
        category: CheckCategory,
        name: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            category,
            name: name.into(),
            status: CheckStatus::Passed,
            message: message.into(),
            details: Vec::new(),
            fixable: false,
            fix_command: None,
        }
    }

    /// Creates a new check result with info status.
    pub fn info(
        category: CheckCategory,
        name: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            category,
            name: name.into(),
            status: CheckStatus::Info,
            message: message.into(),
            details: Vec::new(),
            fixable: false,
            fix_command: None,
        }
    }

    /// Creates a new check result with warning status.
    pub fn warning(
        category: CheckCategory,
        name: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            category,
            name: name.into(),
            status: CheckStatus::Warning,
            message: message.into(),
            details: Vec::new(),
            fixable: false,
            fix_command: None,
        }
    }

    /// Creates a new check result with error status.
    pub fn error(
        category: CheckCategory,
        name: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            category,
            name: name.into(),
            status: CheckStatus::Error,
            message: message.into(),
            details: Vec::new(),
            fixable: false,
            fix_command: None,
        }
    }

    /// Adds details to this check result.
    pub fn with_details(mut self, details: Vec<String>) -> Self {
        self.details = details;
        self
    }

    /// Marks this check result as fixable with the given command.
    pub fn with_fix(mut self, fix_command: impl Into<String>) -> Self {
        self.fixable = true;
        self.fix_command = Some(fix_command.into());
        self
    }

    /// Formats this check for text output.
    pub fn format_text(&self, use_color: bool) -> String {
        let icon = self.status.icon();
        let styled_icon = if use_color {
            match self.status {
                CheckStatus::Passed => format!("{}", color_theme::success(icon)),
                CheckStatus::Info => format!("{}", color_theme::accent(icon)),
                CheckStatus::Warning => format!("{}", color_theme::warning(icon)),
                CheckStatus::Error => format!("{}", color_theme::error(icon)),
            }
        } else {
            icon.to_string()
        };

        let mut output = format!("  {}  {} {}", styled_icon, self.name, self.message);

        for detail in &self.details {
            output.push_str(&format!("\n     └─ {detail}"));
        }

        output
    }
}

impl DoctorSummary {
    /// Adds a check result to the summary.
    pub fn add(&mut self, status: CheckStatus) {
        match status {
            CheckStatus::Passed => self.passed += 1,
            CheckStatus::Info => self.info += 1,
            CheckStatus::Warning => self.warnings += 1,
            CheckStatus::Error => self.failed += 1,
        }
    }

    /// Returns true if there are any errors.
    pub fn has_errors(&self) -> bool {
        self.failed > 0
    }

    /// Returns true if there are any warnings.
    pub fn has_warnings(&self) -> bool {
        self.warnings > 0
    }
}

impl DoctorReport {
    /// Creates a new doctor report from check results.
    pub fn new(checks: Vec<CheckResult>) -> Self {
        let mut summary = DoctorSummary::default();
        for check in &checks {
            summary.add(check.status);
        }

        Self { version: DOCTOR_VERSION.to_string(), checks, summary }
    }

    /// Returns checks filtered by category.
    pub fn checks_for_category(&self, category: CheckCategory) -> Vec<&CheckResult> {
        self.checks.iter().filter(|c| c.category == category).collect()
    }

    /// Returns all checks with warning or error status.
    pub fn issues(&self) -> Vec<&CheckResult> {
        self.checks
            .iter()
            .filter(|c| matches!(c.status, CheckStatus::Warning | CheckStatus::Error))
            .collect()
    }
}

impl From<&DoctorArgs> for DoctorConfig {
    fn from(args: &DoctorArgs) -> Self {
        Self { fix: args.fix, dry_run: args.dry_run, deep: args.deep, quiet: args.quiet }
    }
}
