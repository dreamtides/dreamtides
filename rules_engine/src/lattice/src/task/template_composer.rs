use std::path::Path;

use rusqlite::Connection;
use serde::Serialize;
use tracing::{debug, info};

use crate::document::document_reader;
use crate::error::error_types::LatticeError;
use crate::index::directory_roots::{self, DirectoryRoot};

/// Prefix for Lattice template sections in markdown headings.
const LATTICE_SECTION_PREFIX: &str = "[Lattice]";
/// The context section name.
const CONTEXT_SECTION: &str = "Context";
/// The acceptance criteria section name.
const ACCEPTANCE_CRITERIA_SECTION: &str = "Acceptance Criteria";

/// Composed template content from ancestor root documents.
///
/// Contains context and acceptance criteria extracted and composed from
/// all ancestor root documents in the correct order for display.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ComposedTemplate {
    /// Composed context content (general → specific order).
    ///
    /// Context from the root of the hierarchy appears first, followed by
    /// progressively more specific context from child directories.
    pub context: Option<String>,

    /// Composed acceptance criteria content (specific → general order).
    ///
    /// Acceptance criteria from the immediate parent appears first, followed
    /// by progressively more general criteria from ancestor directories.
    pub acceptance_criteria: Option<String>,

    /// IDs of ancestors that contributed template content.
    pub contributor_ids: Vec<String>,
}

/// Template sections extracted from a single document.
#[derive(Debug, Clone, Default)]
pub struct ExtractedSections {
    /// The `[Lattice] Context` section content, if present.
    pub context: Option<String>,

    /// The `[Lattice] Acceptance Criteria` section content, if present.
    pub acceptance_criteria: Option<String>,
}

/// Composes template content from ancestor root documents.
///
/// Walks up the directory hierarchy via the `directory_roots` table and
/// extracts `\[Lattice\] Context` and `\[Lattice\] Acceptance Criteria`
/// sections from each ancestor root document, composing them in the correct
/// order.
///
/// Context is composed in general→specific order (root first).
/// Acceptance criteria is composed in specific→general order (nearest first).
///
/// # Arguments
///
/// * `conn` - Database connection for querying directory roots
/// * `document_path` - Relative path to the document
/// * `repo_root` - Absolute path to the repository root
///
/// # Returns
///
/// A `ComposedTemplate` containing the composed content. Returns empty content
/// if the document has no ancestors or if no ancestors contain template
/// sections.
pub fn compose_templates(
    conn: &Connection,
    document_path: &Path,
    repo_root: &Path,
) -> Result<ComposedTemplate, LatticeError> {
    let ancestors = find_ancestor_roots(conn, document_path)?;

    if ancestors.is_empty() {
        debug!(
            document_path = %document_path.display(),
            "No ancestors found for template composition"
        );
        return Ok(ComposedTemplate::default());
    }

    let mut context_parts: Vec<String> = Vec::new();
    let mut acceptance_parts: Vec<String> = Vec::new();
    let mut contributor_ids: Vec<String> = Vec::new();

    for ancestor in &ancestors {
        let root_doc_path = compute_root_doc_path(&ancestor.directory_path);
        let absolute_path = repo_root.join(&root_doc_path);

        let document = match document_reader::read(&absolute_path) {
            Ok(doc) => doc,
            Err(e) => {
                debug!(
                    root_id = ancestor.root_id,
                    path = root_doc_path,
                    error = %e,
                    "Failed to read ancestor root document, skipping"
                );
                continue;
            }
        };

        let sections = extract_template_sections(&document.body);

        let mut contributed = false;
        if let Some(ctx) = sections.context {
            context_parts.push(ctx);
            contributed = true;
        }
        if let Some(acc) = sections.acceptance_criteria {
            acceptance_parts.push(acc);
            contributed = true;
        }

        if contributed {
            contributor_ids.push(ancestor.root_id.clone());
        }
    }

    acceptance_parts.reverse();

    let context = if context_parts.is_empty() { None } else { Some(context_parts.join("\n\n")) };

    let acceptance_criteria =
        if acceptance_parts.is_empty() { None } else { Some(acceptance_parts.join("\n\n")) };

    info!(
        document_path = %document_path.display(),
        ancestor_count = ancestors.len(),
        contributor_count = contributor_ids.len(),
        has_context = context.is_some(),
        has_acceptance = acceptance_criteria.is_some(),
        "Composed template content from ancestors"
    );

    Ok(ComposedTemplate { context, acceptance_criteria, contributor_ids })
}

/// Extracts `\[Lattice\]` template sections from markdown content.
///
/// Parses the markdown to find headings with the `\[Lattice\]` prefix and
/// extracts their content up to the next heading of the same or higher level.
///
/// # Arguments
///
/// * `body` - The markdown body content to parse
///
/// # Returns
///
/// An `ExtractedSections` struct containing any found sections.
pub fn extract_template_sections(body: &str) -> ExtractedSections {
    let mut sections = ExtractedSections::default();
    let lines: Vec<&str> = body.lines().collect();

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];

        if let Some((level, section_type)) = parse_lattice_heading(line) {
            let content = extract_section_content(&lines, i + 1, level);

            match section_type {
                SectionType::Context => {
                    sections.context = Some(content);
                    debug!("Extracted [Lattice] Context section");
                }
                SectionType::AcceptanceCriteria => {
                    sections.acceptance_criteria = Some(content);
                    debug!("Extracted [Lattice] Acceptance Criteria section");
                }
            }
        }

        i += 1;
    }

    sections
}

/// Finds ancestor root documents for a document by walking up the directory
/// tree.
///
/// This function walks up from the document's parent directory until it finds
/// a directory that has an entry in the `directory_roots` table. From there,
/// it retrieves the full ancestor chain.
///
/// This handles the case where a document is in a directory (like `tasks/`)
/// that doesn't have its own root document.
pub fn find_ancestor_roots(
    conn: &Connection,
    document_path: &Path,
) -> Result<Vec<DirectoryRoot>, LatticeError> {
    let mut current_dir = document_path.parent();

    while let Some(dir) = current_dir {
        let dir_str = dir.to_string_lossy().to_string();

        if dir_str.is_empty() {
            break;
        }

        let ancestors = directory_roots::get_ancestors(conn, &dir_str)?;
        if !ancestors.is_empty() {
            debug!(
                document_path = %document_path.display(),
                starting_dir = dir_str,
                ancestor_count = ancestors.len(),
                "Found ancestor roots starting from directory"
            );
            return Ok(ancestors);
        }

        current_dir = dir.parent();
    }

    debug!(
        document_path = %document_path.display(),
        "No directory roots found in any ancestor directory"
    );
    Ok(Vec::new())
}

/// Computes the root document path from a directory path.
///
/// For directory `api/tasks`, returns `api/tasks/tasks.md`.
pub fn compute_root_doc_path(directory_path: &str) -> String {
    let dir_name =
        Path::new(directory_path).file_name().and_then(|n| n.to_str()).unwrap_or(directory_path);

    if directory_path.is_empty() {
        format!("{dir_name}.md")
    } else {
        format!("{directory_path}/{dir_name}.md")
    }
}

/// Type of Lattice template section.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SectionType {
    Context,
    AcceptanceCriteria,
}

/// Parses a markdown heading line to check if it's a Lattice template section.
///
/// Returns the heading level (1-6) and section type if this is a valid
/// `\[Lattice\] Context` or `\[Lattice\] Acceptance Criteria` heading.
fn parse_lattice_heading(line: &str) -> Option<(usize, SectionType)> {
    let trimmed = line.trim_start();

    if !trimmed.starts_with('#') {
        return None;
    }

    let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
    if hash_count == 0 || hash_count > 6 {
        return None;
    }

    let after_hashes = trimmed[hash_count..].trim();

    if !after_hashes.starts_with(LATTICE_SECTION_PREFIX) {
        return None;
    }

    let after_prefix = after_hashes[LATTICE_SECTION_PREFIX.len()..].trim();

    if after_prefix.eq_ignore_ascii_case(CONTEXT_SECTION) {
        Some((hash_count, SectionType::Context))
    } else if after_prefix.eq_ignore_ascii_case(ACCEPTANCE_CRITERIA_SECTION) {
        Some((hash_count, SectionType::AcceptanceCriteria))
    } else {
        None
    }
}

/// Extracts section content from a starting line until the next heading of
/// same or higher level.
///
/// # Arguments
///
/// * `lines` - All lines in the document
/// * `start_index` - The line index to start extracting from (after heading)
/// * `heading_level` - The level of the section heading (1-6)
///
/// # Returns
///
/// The section content with leading/trailing whitespace trimmed.
fn extract_section_content(lines: &[&str], start_index: usize, heading_level: usize) -> String {
    let mut content_lines: Vec<&str> = Vec::new();
    let mut i = start_index;

    while i < lines.len() {
        let line = lines[i];

        if let Some(level) = get_heading_level(line)
            && level <= heading_level
        {
            break;
        }

        content_lines.push(line);
        i += 1;
    }

    while content_lines.first().is_some_and(|l| l.is_empty()) {
        content_lines.remove(0);
    }
    while content_lines.last().is_some_and(|l| l.is_empty()) {
        content_lines.pop();
    }

    content_lines.join("\n")
}

/// Gets the heading level (1-6) of a markdown heading line.
///
/// Returns `None` if the line is not a valid ATX heading.
fn get_heading_level(line: &str) -> Option<usize> {
    let trimmed = line.trim_start();

    if !trimmed.starts_with('#') {
        return None;
    }

    let hash_count = trimmed.chars().take_while(|&c| c == '#').count();

    if hash_count == 0 || hash_count > 6 {
        return None;
    }

    let after_hashes = &trimmed[hash_count..];
    if after_hashes.is_empty() || after_hashes.starts_with(' ') || after_hashes.starts_with('\t') {
        Some(hash_count)
    } else {
        None
    }
}
