use regex::{Captures, Regex};
use tracing::debug;

use crate::link::link_extractor::ExtractedLink;
use crate::link::link_normalization::link_analysis::NormalizationAction;

/// A transformation to apply to a link in document content.
#[derive(Debug, Clone)]
pub struct LinkTransform {
    /// The extracted link to transform.
    pub link: ExtractedLink,

    /// The normalization action to apply.
    pub action: NormalizationAction,
}

/// Result of applying transforms to document content.
#[derive(Debug, Clone)]
pub struct TransformResult {
    /// The transformed content.
    pub content: String,

    /// Number of links that were modified.
    pub modified_count: usize,
}

/// Applies a list of link transforms to document content.
///
/// Transforms are applied by finding each link's original text representation
/// and replacing it with the normalized form. Links are processed in reverse
/// document order to maintain correct offsets.
pub fn apply_transforms(content: &str, transforms: &[LinkTransform]) -> TransformResult {
    let mut result = content.to_string();
    let mut modified_count = 0;

    let mut sorted_transforms: Vec<_> = transforms.iter().collect();
    sorted_transforms.sort_by(|a, b| b.link.line.cmp(&a.link.line));

    for transform in sorted_transforms {
        if matches!(transform.action, NormalizationAction::None) {
            continue;
        }

        if let Some(new_content) = apply_single_transform(&result, transform) {
            result = new_content;
            modified_count += 1;
        }
    }

    TransformResult { content: result, modified_count }
}

/// Applies a single transform to the content.
///
/// Returns Some with the new content if successful, None if the link couldn't
/// be found or transformed.
fn apply_single_transform(content: &str, transform: &LinkTransform) -> Option<String> {
    let link = &transform.link;
    let pattern = build_link_pattern(link)?;

    debug!(
        line = link.line,
        pattern = %pattern,
        action = ?transform.action,
        "Applying link transform"
    );

    let regex = Regex::new(&pattern).ok()?;
    let replacement = build_replacement(link, &transform.action)?;

    let new_content = regex.replace(content, |caps: &Captures<'_>| {
        let text = caps.name("text").map_or("", |m| m.as_str());
        replacement.replace("{text}", text)
    });

    if new_content == content {
        debug!(line = link.line, "Link pattern not found in content");
        None
    } else {
        Some(new_content.to_string())
    }
}

/// Builds a regex pattern to match a link in content.
///
/// The pattern captures the link text for use in the replacement.
fn build_link_pattern(link: &ExtractedLink) -> Option<String> {
    let escaped_text = regex::escape(&link.text);

    match (&link.path, &link.fragment) {
        (Some(path), Some(fragment)) => {
            let escaped_path = regex::escape(path);
            let escaped_fragment = regex::escape(fragment.as_str());
            Some(format!(r"\[(?P<text>{escaped_text})\]\({escaped_path}#{escaped_fragment}\)"))
        }
        (Some(path), None) => {
            let escaped_path = regex::escape(path);
            Some(format!(r"\[(?P<text>{escaped_text})\]\({escaped_path}\)"))
        }
        (None, Some(fragment)) => {
            let escaped_fragment = regex::escape(fragment.as_str());
            Some(format!(r"\[(?P<text>{escaped_text})\]\({escaped_fragment}\)"))
        }
        (None, None) => None,
    }
}

/// Builds the replacement string for a link transform.
fn build_replacement(link: &ExtractedLink, action: &NormalizationAction) -> Option<String> {
    match action {
        NormalizationAction::None => None,
        NormalizationAction::AddFragment { target_id } => {
            let path = link.path.as_ref()?;
            Some(format!("[{{text}}]({path}#{target_id})"))
        }
        NormalizationAction::ExpandShorthand { relative_path } => {
            let fragment = link.fragment.as_ref()?;
            Some(format!("[{{text}}]({relative_path}#{fragment})"))
        }
        NormalizationAction::UpdatePath { new_relative_path } => {
            let fragment = link.fragment.as_ref()?;
            Some(format!("[{{text}}]({}#{})", new_relative_path, fragment))
        }
    }
}
