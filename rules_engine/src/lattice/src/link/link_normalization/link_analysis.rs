use std::path::Path;

use rusqlite::Connection;
use tracing::debug;

use crate::error::error_types::LatticeError;
use crate::id::lattice_id::LatticeId;
use crate::index::document_queries;
use crate::link::link_extractor::{ExtractedLink, LinkCategory};
use crate::link::link_resolver;

/// Classification of what normalization action is needed for a link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NormalizationAction {
    /// Link is already in canonical form, no changes needed.
    None,

    /// Link needs fragment added (path exists but no Lattice ID fragment).
    AddFragment { target_id: LatticeId },

    /// Shorthand ID needs to be expanded to full path+fragment.
    ExpandShorthand { relative_path: String },

    /// Link path is stale (document was moved), needs path update.
    UpdatePath { new_relative_path: String },
}

/// A link that could not be normalized with details about why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnresolvableLink {
    /// The line number where this link appears.
    pub line: usize,

    /// The reason normalization failed.
    pub reason: UnresolvableReason,
}

/// Reason why a link could not be normalized.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnresolvableReason {
    /// The target document was not found in the index.
    TargetNotFound { target: String },

    /// Path-only link could not be resolved to a known document.
    PathNotFound { path: String },

    /// The target path contains invalid UTF-8.
    InvalidTargetPath { path: String },
}

/// Result of analyzing a single link for normalization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnalysisResult {
    /// Link can be normalized with the specified action.
    Normalizable { action: NormalizationAction, link: ExtractedLink },

    /// Link cannot be normalized.
    Unresolvable(UnresolvableLink),

    /// Link should be skipped (external URL, other).
    Skip,
}

/// Analyzes an extracted link to determine what normalization is needed.
///
/// Examines the link structure and compares against the index to determine:
/// - If fragment needs to be added (path-only links)
/// - If shorthand ID needs expansion (ID-only links)
/// - If path needs updating (stale/moved documents)
/// - If link is already canonical (no action needed)
pub fn analyze(
    conn: &Connection,
    source_path: &Path,
    link: &ExtractedLink,
) -> Result<AnalysisResult, LatticeError> {
    debug!(
        source = %source_path.display(),
        line = link.line,
        link_type = ?link.link_type,
        "Analyzing link for normalization"
    );

    match link.link_type {
        LinkCategory::External | LinkCategory::Other => {
            debug!(line = link.line, "Skipping external or other link");
            Ok(AnalysisResult::Skip)
        }
        LinkCategory::ShorthandId => analyze_shorthand(conn, source_path, link),
        LinkCategory::PathOnly => analyze_path_only(conn, source_path, link),
        LinkCategory::Canonical => analyze_canonical(conn, source_path, link),
    }
}

/// Normalizes a path by removing `.` and `..` components.
pub fn normalize_path(path: &str) -> String {
    let path = Path::new(path);
    let mut components = Vec::new();

    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::CurDir => {}
            _ => {
                components.push(component);
            }
        }
    }

    let result: std::path::PathBuf = components.into_iter().collect();
    result.to_string_lossy().to_string()
}

/// Analyzes a shorthand ID-only link (e.g., `[text](LJCQ2X)`).
///
/// These links need to be expanded to include the relative path to the target.
fn analyze_shorthand(
    conn: &Connection,
    source_path: &Path,
    link: &ExtractedLink,
) -> Result<AnalysisResult, LatticeError> {
    let target_id =
        link.fragment.as_ref().unwrap_or_else(|| panic!("ShorthandId link missing fragment"));

    debug!(target_id = %target_id, "Analyzing shorthand link");

    match link_resolver::resolve(conn, source_path, target_id)? {
        link_resolver::LinkResolution::Resolved(resolved) => {
            debug!(
                target_id = %target_id,
                relative_path = %resolved.relative_path,
                "Shorthand link resolved"
            );
            Ok(AnalysisResult::Normalizable {
                action: NormalizationAction::ExpandShorthand {
                    relative_path: resolved.relative_path,
                },
                link: link.clone(),
            })
        }
        link_resolver::LinkResolution::Unresolved(unresolved) => {
            debug!(target_id = %target_id, reason = ?unresolved.reason, "Shorthand link unresolved");
            Ok(AnalysisResult::Unresolvable(UnresolvableLink {
                line: link.line,
                reason: UnresolvableReason::TargetNotFound { target: target_id.to_string() },
            }))
        }
    }
}

/// Analyzes a path-only link (e.g., `[text](path.md)`).
///
/// These links need their Lattice ID fragment added.
fn analyze_path_only(
    conn: &Connection,
    source_path: &Path,
    link: &ExtractedLink,
) -> Result<AnalysisResult, LatticeError> {
    let path = link.path.as_ref().unwrap_or_else(|| panic!("PathOnly link missing path"));

    debug!(path = %path, "Analyzing path-only link");

    let resolved_path = resolve_link_path(source_path, path);
    let normalized = normalize_path(&resolved_path);

    let Some(target_doc) = document_queries::lookup_by_path(conn, &normalized)? else {
        debug!(path = %path, resolved = %normalized, "Path-only link target not found");
        return Ok(AnalysisResult::Unresolvable(UnresolvableLink {
            line: link.line,
            reason: UnresolvableReason::PathNotFound { path: path.clone() },
        }));
    };

    let target_id = LatticeId::parse(&target_doc.id)?;
    debug!(path = %path, target_id = %target_id, "Path-only link target found");

    Ok(AnalysisResult::Normalizable {
        action: NormalizationAction::AddFragment { target_id },
        link: link.clone(),
    })
}

/// Analyzes a canonical link (e.g., `[text](path.md#LJCQ2X)`).
///
/// Checks if the path is stale (document moved) and needs updating.
fn analyze_canonical(
    conn: &Connection,
    source_path: &Path,
    link: &ExtractedLink,
) -> Result<AnalysisResult, LatticeError> {
    let target_id =
        link.fragment.as_ref().unwrap_or_else(|| panic!("Canonical link missing fragment"));
    let current_path = link.path.as_ref().unwrap_or_else(|| panic!("Canonical link missing path"));

    debug!(target_id = %target_id, current_path = %current_path, "Analyzing canonical link");

    match link_resolver::resolve(conn, source_path, target_id)? {
        link_resolver::LinkResolution::Resolved(resolved) => {
            if resolved.relative_path == *current_path {
                debug!(target_id = %target_id, "Canonical link path is current");
                Ok(AnalysisResult::Normalizable {
                    action: NormalizationAction::None,
                    link: link.clone(),
                })
            } else {
                debug!(
                    target_id = %target_id,
                    current_path = %current_path,
                    new_path = %resolved.relative_path,
                    "Canonical link path is stale"
                );
                Ok(AnalysisResult::Normalizable {
                    action: NormalizationAction::UpdatePath {
                        new_relative_path: resolved.relative_path,
                    },
                    link: link.clone(),
                })
            }
        }
        link_resolver::LinkResolution::Unresolved(unresolved) => {
            debug!(target_id = %target_id, reason = ?unresolved.reason, "Canonical link unresolved");
            let reason = match unresolved.reason {
                link_resolver::UnresolvedReason::TargetNotFound => {
                    UnresolvableReason::TargetNotFound { target: target_id.to_string() }
                }
                link_resolver::UnresolvedReason::InvalidTargetPath { path } => {
                    UnresolvableReason::InvalidTargetPath { path }
                }
            };
            Ok(AnalysisResult::Unresolvable(UnresolvableLink { line: link.line, reason }))
        }
    }
}

/// Resolves a link path relative to the source document.
///
/// Takes a relative path from a link (e.g., `../other/doc.md`) and resolves it
/// against the source document's directory to produce an absolute-style path
/// from the repository root.
fn resolve_link_path(source_path: &Path, link_path: &str) -> String {
    let source_dir = source_path.parent().unwrap_or(Path::new(""));
    let joined = source_dir.join(link_path);
    joined.to_string_lossy().to_string()
}
