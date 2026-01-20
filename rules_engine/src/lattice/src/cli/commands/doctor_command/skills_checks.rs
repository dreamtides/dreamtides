use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use tracing::{debug, info, warn};

use crate::cli::command_dispatch::CommandContext;
use crate::cli::commands::doctor_command::doctor_types::{CheckCategory, CheckResult};
use crate::error::error_types::LatticeError;
use crate::index::document_filter::DocumentFilter;
use crate::index::document_queries;

const SKILLS_DIR: &str = ".claude/skills";

/// Runs all skill symlink checks.
pub fn run_skills_checks(context: &CommandContext) -> Result<Vec<CheckResult>, LatticeError> {
    let mut results = Vec::new();

    let skills_dir = context.repo_root.join(SKILLS_DIR);

    if !skills_dir.exists() {
        info!("No .claude/skills/ directory found");
        results.push(CheckResult::info(
            CheckCategory::Skills,
            "Skills Directory",
            "No .claude/skills/ directory",
        ));
        return Ok(results);
    }

    let skill_docs = query_skill_documents(context)?;
    let filesystem_symlinks = read_symlinks_from_filesystem(&skills_dir)?;

    results.push(check_symlink_validity(&skills_dir, &context.repo_root, &filesystem_symlinks));

    results.push(check_symlink_coverage(&skill_docs, &filesystem_symlinks));

    results.push(check_symlink_staleness(
        &skills_dir,
        &context.repo_root,
        &skill_docs,
        &filesystem_symlinks,
    ));

    Ok(results)
}

/// Information about a skill document from the index.
struct SkillDocInfo {
    name: String,
    path: String,
}

/// Information about a symlink from the filesystem.
struct SymlinkInfo {
    name: String,
    target: PathBuf,
    resolves: bool,
}

/// Queries the index for all skill-enabled documents.
fn query_skill_documents(context: &CommandContext) -> Result<Vec<SkillDocInfo>, LatticeError> {
    let filter = DocumentFilter::including_closed().with_skill(true);
    let docs = document_queries::query(&context.conn, &filter)?;

    Ok(docs.into_iter().map(|d| SkillDocInfo { name: d.name, path: d.path }).collect())
}

/// Reads all symlinks from the skills directory.
fn read_symlinks_from_filesystem(
    skills_dir: &std::path::Path,
) -> Result<Vec<SymlinkInfo>, LatticeError> {
    let mut symlinks = Vec::new();

    let entries = match fs::read_dir(skills_dir) {
        Ok(entries) => entries,
        Err(e) => {
            return Err(LatticeError::ReadError {
                path: skills_dir.to_path_buf(),
                reason: format!("Failed to read skills directory: {e}"),
            });
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if !path.is_symlink() {
            continue;
        }

        let Some(filename) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        let Some(name) = filename.strip_suffix(".md") else {
            continue;
        };

        match fs::read_link(&path) {
            Ok(target) => {
                let resolved =
                    if target.is_absolute() { target.clone() } else { skills_dir.join(&target) };
                let resolves = resolved.exists();

                symlinks.push(SymlinkInfo { name: name.to_string(), target, resolves });
            }
            Err(e) => {
                debug!(name, error = %e, "Failed to read symlink target");
                symlinks.push(SymlinkInfo {
                    name: name.to_string(),
                    target: PathBuf::new(),
                    resolves: false,
                });
            }
        }
    }

    Ok(symlinks)
}

/// Checks that all symlinks in .claude/skills/ resolve to existing files.
fn check_symlink_validity(
    skills_dir: &std::path::Path,
    _repo_root: &std::path::Path,
    symlinks: &[SymlinkInfo],
) -> CheckResult {
    debug!("Checking symlink validity");

    let broken: Vec<&str> =
        symlinks.iter().filter(|s| !s.resolves).map(|s| s.name.as_str()).collect();

    if broken.is_empty() {
        let count = symlinks.len();
        if count == 0 {
            info!("No symlinks to validate");
            CheckResult::info(CheckCategory::Skills, "Symlink Validity", "No symlinks to validate")
        } else {
            info!(count, "All symlinks resolve");
            CheckResult::passed(
                CheckCategory::Skills,
                "Symlink Validity",
                format!("All {count} symlinks resolve to existing files"),
            )
        }
    } else {
        let count = broken.len();
        warn!(count, ?broken, "Found broken symlinks");
        CheckResult::warning(
            CheckCategory::Skills,
            "Symlink Validity",
            format!("{count} broken symlink(s) in {}", skills_dir.display()),
        )
        .with_details(broken.into_iter().map(String::from).collect())
        .with_fix("lat doctor --fix")
    }
}

/// Checks that all skill: true documents have corresponding symlinks.
fn check_symlink_coverage(skill_docs: &[SkillDocInfo], symlinks: &[SymlinkInfo]) -> CheckResult {
    debug!("Checking symlink coverage");

    let symlink_names: HashSet<&str> = symlinks.iter().map(|s| s.name.as_str()).collect();
    let missing: Vec<&str> = skill_docs
        .iter()
        .filter(|d| !symlink_names.contains(d.name.as_str()))
        .map(|d| d.name.as_str())
        .collect();

    if missing.is_empty() {
        let count = skill_docs.len();
        if count == 0 {
            info!("No skill documents to check");
            CheckResult::info(
                CheckCategory::Skills,
                "Symlink Coverage",
                "No skill documents in index",
            )
        } else {
            info!(count, "All skill documents have symlinks");
            CheckResult::passed(
                CheckCategory::Skills,
                "Symlink Coverage",
                format!("All {count} skill documents have symlinks"),
            )
        }
    } else {
        let count = missing.len();
        warn!(count, ?missing, "Skill documents missing symlinks");
        CheckResult::warning(
            CheckCategory::Skills,
            "Symlink Coverage",
            format!("{count} skill document(s) missing symlinks"),
        )
        .with_details(missing.into_iter().map(String::from).collect())
        .with_fix("lat doctor --fix")
    }
}

/// Checks that symlinks point to current document paths.
fn check_symlink_staleness(
    skills_dir: &std::path::Path,
    repo_root: &std::path::Path,
    skill_docs: &[SkillDocInfo],
    symlinks: &[SymlinkInfo],
) -> CheckResult {
    debug!("Checking symlink staleness");

    let doc_by_name: HashMap<&str, &SkillDocInfo> =
        skill_docs.iter().map(|d| (d.name.as_str(), d)).collect();

    let mut stale = Vec::new();

    for symlink in symlinks {
        if !symlink.resolves {
            continue;
        }

        let Some(doc) = doc_by_name.get(symlink.name.as_str()) else {
            continue;
        };

        let expected_target = compute_relative_target(skills_dir, repo_root, &doc.path);
        if symlink.target != expected_target {
            debug!(
                name = symlink.name,
                actual = %symlink.target.display(),
                expected = %expected_target.display(),
                "Symlink target mismatch"
            );
            stale.push(symlink.name.as_str());
        }
    }

    if stale.is_empty() {
        let valid_count = symlinks.iter().filter(|s| s.resolves).count();
        if valid_count == 0 {
            CheckResult::info(
                CheckCategory::Skills,
                "Symlink Staleness",
                "No valid symlinks to check",
            )
        } else {
            info!(valid_count, "All symlinks point to current paths");
            CheckResult::passed(
                CheckCategory::Skills,
                "Symlink Staleness",
                format!("All {valid_count} symlinks point to current document paths"),
            )
        }
    } else {
        let count = stale.len();
        warn!(count, ?stale, "Found stale symlinks");
        CheckResult::warning(
            CheckCategory::Skills,
            "Symlink Staleness",
            format!("{count} symlink(s) point to old paths"),
        )
        .with_details(stale.into_iter().map(String::from).collect())
        .with_fix("lat doctor --fix")
    }
}

/// Computes the relative path from skills_dir to the document.
fn compute_relative_target(
    skills_dir: &std::path::Path,
    repo_root: &std::path::Path,
    doc_path: &str,
) -> PathBuf {
    let absolute_target = repo_root.join(doc_path);
    pathdiff::diff_paths(&absolute_target, skills_dir).unwrap_or(absolute_target)
}
