use std::path::Path;

use rusqlite::Connection;
use tracing::{debug, info};

use crate::error::error_types::LatticeError;
use crate::link::link_extractor::{self, ExtractedLink, ExtractionResult};
use crate::link::link_normalization::link_analysis::{
    self, AnalysisResult, NormalizationAction, UnresolvableLink,
};
use crate::link::link_normalization::link_transforms::{self, LinkTransform};

/// Result of normalizing links in a document.
#[derive(Debug, Clone)]
pub struct NormalizationResult {
    /// The normalized document content (with all link transformations applied).
    pub content: String,

    /// Number of links that were modified.
    pub modified_count: usize,

    /// Links that could not be normalized (broken references, etc.).
    pub unresolvable: Vec<UnresolvableLink>,

    /// Whether any changes were made to the content.
    pub has_changes: bool,
}

/// Configuration for normalization behavior.
#[derive(Debug, Clone, Default)]
pub struct NormalizationConfig {
    /// If true, stop on first unresolvable link instead of collecting all.
    pub fail_fast: bool,
}

/// Normalizes all links in a document's content.
///
/// This is the main entry point for link normalization. It:
/// 1. Extracts all links from the content
/// 2. Analyzes each link to determine what normalization is needed
/// 3. Applies transformations to produce normalized content
///
/// # Arguments
///
/// * `conn` - Database connection to the index
/// * `source_path` - Path of the document being normalized
/// * `content` - The document's markdown content
/// * `config` - Configuration options for normalization
///
/// # Returns
///
/// A `NormalizationResult` containing the normalized content and any issues.
pub fn normalize(
    conn: &Connection,
    source_path: &Path,
    content: &str,
    config: &NormalizationConfig,
) -> Result<NormalizationResult, LatticeError> {
    info!(
        source = %source_path.display(),
        content_len = content.len(),
        "Starting link normalization"
    );

    let extraction = link_extractor::extract(content);
    debug!(link_count = extraction.links.len(), "Extracted links from document");

    let analysis = analyze_all(conn, source_path, &extraction, config)?;
    let transforms = collect_transforms(&analysis);

    debug!(
        normalizable = analysis.normalizable.len(),
        unresolvable = analysis.unresolvable.len(),
        transforms = transforms.len(),
        "Analysis complete"
    );

    let transform_result = link_transforms::apply_transforms(content, &transforms);

    info!(
        modified = transform_result.modified_count,
        unresolvable = analysis.unresolvable.len(),
        "Link normalization complete"
    );

    Ok(NormalizationResult {
        content: transform_result.content,
        modified_count: transform_result.modified_count,
        unresolvable: analysis.unresolvable,
        has_changes: transform_result.modified_count > 0,
    })
}

/// Aggregated analysis results.
struct AnalysisResults {
    normalizable: Vec<(ExtractedLink, NormalizationAction)>,
    unresolvable: Vec<UnresolvableLink>,
}

/// Analyzes all extracted links and categorizes them.
fn analyze_all(
    conn: &Connection,
    source_path: &Path,
    extraction: &ExtractionResult,
    config: &NormalizationConfig,
) -> Result<AnalysisResults, LatticeError> {
    let mut normalizable = Vec::new();
    let mut unresolvable = Vec::new();

    for link in &extraction.links {
        let result = link_analysis::analyze(conn, source_path, link)?;

        match result {
            AnalysisResult::Normalizable { action, link } => {
                normalizable.push((link, action));
            }
            AnalysisResult::Unresolvable(unresolved) => {
                if config.fail_fast {
                    return Ok(AnalysisResults { normalizable, unresolvable: vec![unresolved] });
                }
                unresolvable.push(unresolved);
            }
            AnalysisResult::Skip => {}
        }
    }

    Ok(AnalysisResults { normalizable, unresolvable })
}

/// Converts analysis results into transforms.
fn collect_transforms(analysis: &AnalysisResults) -> Vec<LinkTransform> {
    analysis
        .normalizable
        .iter()
        .filter(|(_, action)| !matches!(action, NormalizationAction::None))
        .map(|(link, action)| LinkTransform { link: link.clone(), action: action.clone() })
        .collect()
}
