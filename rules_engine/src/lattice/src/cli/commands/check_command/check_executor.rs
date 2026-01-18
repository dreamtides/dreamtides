use std::path::Path;

use rusqlite::Connection;
use tracing::{debug, info};

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::commands::check_command::{check_fixes, check_output};
use crate::cli::maintenance_args::CheckArgs;
use crate::cli::output_format::OutputFormat;
use crate::git::git_ops::GitOps;
use crate::index::reconciliation::reconciliation_coordinator;
use crate::index::schema_definition;
use crate::lint::rule_engine::{self, LintConfig, LintContext, LintRule, LintSummary};
use crate::lint::{error_rules, skill_rules, structure_rules, warning_rules};

/// Executes the `lat check` command to validate documents.
///
/// This command runs all lint rules against documents and reports any errors
/// or warnings found. It supports filtering by path, checking only staged
/// files, and applying automatic fixes.
pub fn execute(context: CommandContext, args: CheckArgs) -> LatticeResult<()> {
    info!(
        path = ?args.path,
        errors_only = args.errors_only,
        fix = args.fix,
        staged_only = args.staged_only,
        rebuild_index = args.rebuild_index,
        "Executing check command"
    );

    if args.rebuild_index {
        force_index_rebuild(&context.repo_root, context.git.as_ref(), &context.conn)?;
    }

    let output_format = OutputFormat::from_flags(context.global.json, false);
    let summary = run_lint_checks(&context.conn, &context.repo_root, &args, context.git.as_ref())?;

    if args.fix && !summary.is_clean() {
        let fix_summary = check_fixes::apply_fixes(&context.repo_root, &summary)?;
        check_output::print_output(output_format, &summary, Some(&fix_summary));
    } else {
        check_output::print_output(output_format, &summary, None);
    }

    check_output::exit_with_code(&summary)
}

/// Forces a full index rebuild before checking.
///
/// Resets the schema first to ensure reconciliation performs a complete rebuild
/// rather than skipping or doing an incremental update.
fn force_index_rebuild(repo_root: &Path, git: &dyn GitOps, conn: &Connection) -> LatticeResult<()> {
    info!("Forcing index rebuild before check");

    // Reset the schema to force a full rebuild during reconciliation
    schema_definition::reset_schema(conn)?;

    let result = reconciliation_coordinator::reconcile(repo_root, git, conn)?;

    debug!(?result, "Index rebuild complete");
    Ok(())
}

/// Runs all lint rules and returns the summary.
fn run_lint_checks(
    conn: &Connection,
    repo_root: &Path,
    args: &CheckArgs,
    git: &dyn GitOps,
) -> LatticeResult<LintSummary> {
    let rules = collect_all_rules();
    let rule_refs: Vec<&dyn LintRule> = rules.iter().map(AsRef::as_ref).collect();

    let config = build_lint_config(args);
    let ctx = LintContext::new(conn, repo_root);

    let summary = if args.staged_only {
        run_staged_only_check(&ctx, &rule_refs, &config, git)?
    } else {
        rule_engine::execute_rules(&ctx, &rule_refs, &config)?
    };

    info!(
        documents = summary.documents_checked,
        errors = summary.error_count,
        warnings = summary.warning_count,
        "Lint check complete"
    );

    Ok(summary)
}

/// Collects all lint rules from all categories.
fn collect_all_rules() -> Vec<Box<dyn LintRule>> {
    let mut rules = Vec::new();
    rules.extend(error_rules::all_error_rules());
    rules.extend(warning_rules::all_warning_rules());
    rules.extend(structure_rules::all_structure_rules());
    rules.extend(skill_rules::all_skill_rules());
    rules
}

/// Builds the lint configuration from command arguments.
fn build_lint_config(args: &CheckArgs) -> LintConfig {
    let mut config = LintConfig::default();

    if args.errors_only {
        config = config.with_errors_only(true);
    }

    if let Some(ref path) = args.path {
        config = config.with_path_prefix(path);
    }

    config
}

/// Runs lint checks only on staged files.
fn run_staged_only_check(
    ctx: &LintContext<'_>,
    rules: &[&dyn LintRule],
    config: &LintConfig,
    git: &dyn GitOps,
) -> LatticeResult<LintSummary> {
    let staged_docs = check_fixes::get_staged_documents(ctx, git)?;

    debug!(count = staged_docs.len(), "Checking staged documents only");

    if staged_docs.is_empty() {
        info!("No staged markdown files to check");
        return Ok(LintSummary::default());
    }

    rule_engine::execute_rules_on_documents(ctx, rules, config, staged_docs)
}
