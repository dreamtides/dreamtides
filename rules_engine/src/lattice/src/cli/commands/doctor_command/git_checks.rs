use tracing::{debug, info, warn};

use crate::cli::command_dispatch::CommandContext;
use crate::cli::commands::doctor_command::doctor_types::{CheckCategory, CheckResult, CheckStatus};
use crate::error::error_types::LatticeError;
use crate::git::repo_detection::{InProgressOp, RepoConfig};

/// Runs all git integration checks.
pub fn run_git_checks(context: &CommandContext) -> Result<Vec<CheckResult>, LatticeError> {
    let mut results = Vec::new();

    // 1. Repository validity check (Error severity)
    let repo_valid = check_repository_validity(context);
    let is_valid_repo = repo_valid.status == CheckStatus::Passed;
    results.push(repo_valid);

    // Only run remaining checks if we have a valid git repository
    if !is_valid_repo {
        return Ok(results);
    }

    // Load repo configuration for edge case detection
    let repo_config = RepoConfig::load_or_detect(&context.repo_root, context.git.as_ref())?;

    // 2. Edge case detection (Info severity)
    results.extend(check_edge_cases(&repo_config, context)?);

    // 3. Working tree state check (Warning severity)
    results.push(check_working_tree_state(&repo_config));

    // 4. Detached HEAD check (Info severity)
    results.push(check_detached_head(context));

    Ok(results)
}

/// Checks that we are in a valid git repository using the GitOps trait.
fn check_repository_validity(context: &CommandContext) -> CheckResult {
    debug!("Checking repository validity using git rev-parse HEAD");

    match context.git.rev_parse("HEAD") {
        Ok(commit_hash) => {
            info!(commit_hash = %commit_hash, "Valid git repository detected");
            CheckResult::passed(CheckCategory::Git, "Repository", "Valid git repository")
        }
        Err(e) => {
            warn!(?e, "Failed to verify git repository");
            CheckResult::error(
                CheckCategory::Git,
                "Repository",
                format!("Not a valid git repository: {e}"),
            )
        }
    }
}

/// Detects and reports git edge cases (shallow clone, sparse checkout,
/// worktree, submodules).
fn check_edge_cases(
    repo_config: &RepoConfig,
    context: &CommandContext,
) -> Result<Vec<CheckResult>, LatticeError> {
    let mut results = Vec::new();
    let mut edge_cases = Vec::new();

    if repo_config.is_shallow {
        edge_cases.push("shallow clone".to_string());
    }
    if repo_config.is_partial {
        let filter = repo_config.partial_filter.as_deref().unwrap_or("unknown");
        edge_cases.push(format!("partial clone (filter: {filter})"));
    }
    if repo_config.is_sparse {
        edge_cases.push("sparse checkout".to_string());
    }
    if repo_config.is_worktree {
        edge_cases.push("worktree".to_string());
    }
    if repo_config.has_submodules {
        edge_cases.push("submodules".to_string());
    }

    if edge_cases.is_empty() {
        info!("No git edge cases detected");
        results.push(CheckResult::passed(
            CheckCategory::Git,
            "Configuration",
            "Standard git repository (no edge cases)",
        ));
    } else {
        info!(edge_cases = ?edge_cases, "Git edge cases detected");
        let mut result = CheckResult::info(
            CheckCategory::Git,
            "Configuration",
            format!("Detected: {}", edge_cases.join(", ")),
        );

        // Add detail about documents outside sparse patterns if applicable
        if repo_config.is_sparse {
            let count = count_documents_outside_sparse(context)?;
            if count > 0 {
                result = result
                    .with_details(vec![format!("{count} document(s) outside sparse patterns")]);
            }
        }

        results.push(result);
    }

    Ok(results)
}

/// Counts documents that are tracked by git but not materialized due to sparse
/// checkout.
fn count_documents_outside_sparse(context: &CommandContext) -> Result<usize, LatticeError> {
    // Get all tracked markdown files from git
    let tracked_files = context.git.ls_files("*.md")?;

    // Count files that don't exist on disk (outside sparse checkout)
    let count = tracked_files
        .iter()
        .filter(|path| {
            let full_path = context.repo_root.join(path);
            !full_path.exists()
        })
        .count();

    debug!(
        tracked = tracked_files.len(),
        outside_sparse = count,
        "Counted documents outside sparse patterns"
    );

    Ok(count)
}

/// Checks for in-progress git operations (merge, rebase, cherry-pick, revert).
fn check_working_tree_state(repo_config: &RepoConfig) -> CheckResult {
    debug!("Checking working tree state for in-progress operations");

    match &repo_config.in_progress_op {
        Some(op) => {
            let op_name = match op {
                InProgressOp::Rebase => "rebase",
                InProgressOp::Merge => "merge",
                InProgressOp::CherryPick => "cherry-pick",
                InProgressOp::Revert => "revert",
            };
            warn!(operation = op_name, "Git operation in progress");
            CheckResult::warning(
                CheckCategory::Git,
                "Working Tree",
                format!("Git {op_name} in progress"),
            )
            .with_details(vec![format!(
                "Files with unresolved conflicts will be skipped during indexing"
            )])
        }
        None => {
            info!("No in-progress git operations");
            CheckResult::passed(
                CheckCategory::Git,
                "Working Tree",
                "Clean (no in-progress operations)",
            )
        }
    }
}

/// Checks if HEAD is detached (not pointing to a branch).
fn check_detached_head(context: &CommandContext) -> CheckResult {
    debug!("Checking for detached HEAD state");

    match context.git.rev_parse("--abbrev-ref HEAD") {
        Ok(ref branch) if branch == "HEAD" => {
            info!("Detached HEAD state detected");
            CheckResult::info(CheckCategory::Git, "HEAD State", "Detached HEAD (not on a branch)")
        }
        Ok(branch) => {
            info!(branch = %branch, "HEAD is on a branch");
            CheckResult::passed(CheckCategory::Git, "HEAD State", format!("On branch: {branch}"))
        }
        Err(e) => {
            warn!(?e, "Failed to determine HEAD state");
            CheckResult::warning(
                CheckCategory::Git,
                "HEAD State",
                format!("Could not determine HEAD state: {e}"),
            )
        }
    }
}
