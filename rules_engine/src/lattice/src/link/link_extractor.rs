use pulldown_cmark::{Event, LinkType, Options, Parser, Tag, TagEnd};

use crate::id::lattice_id::LatticeId;

/// A link extracted from markdown content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedLink {
    /// The visible link text (e.g., "design document" in `[design
    /// document](path)`).
    pub text: String,

    /// The file path portion of the link URL, if present.
    /// Empty for shorthand ID-only links (e.g., `[text](LJCQ2X)`).
    pub path: Option<String>,

    /// The URL fragment (Lattice ID), if present.
    /// Extracted from `#LJCQ2X` portion of the URL.
    pub fragment: Option<LatticeId>,

    /// The 1-based line number where this link appears.
    pub line: usize,

    /// The link type classification.
    pub link_type: LinkCategory,
}

/// Classification of a link's structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkCategory {
    /// Full canonical link: `[text](path.md#LJCQ2X)`
    Canonical,
    /// Path-only link (missing fragment): `[text](path.md)`
    PathOnly,
    /// Shorthand ID-only link: `[text](LJCQ2X)`
    ShorthandId,
    /// External URL (http/https): `[text](https://example.com)`
    External,
    /// Other unrecognized link format
    Other,
}

/// Result of extracting links from a document.
#[derive(Debug, Clone)]
pub struct ExtractionResult {
    /// All extracted links in document order.
    pub links: Vec<ExtractedLink>,
}

/// Extracts all links from markdown content.
pub fn extract(content: &str) -> ExtractionResult {
    let mut links = Vec::new();
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = Parser::new_ext(content, options);
    let mut current_link: Option<PendingLink> = None;
    let mut in_code_block = false;

    for (event, range) in parser.into_offset_iter() {
        match event {
            Event::Start(Tag::CodeBlock(_)) => {
                in_code_block = true;
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
            }
            Event::Start(Tag::Link { link_type, dest_url, .. }) if !in_code_block => {
                if !is_inline_or_standard_link(link_type) {
                    continue;
                }
                let line = compute_line_number(content, range.start);
                current_link =
                    Some(PendingLink { dest: dest_url.to_string(), text: String::new(), line });
            }
            Event::Text(text) if current_link.is_some() => {
                if let Some(ref mut pending) = current_link {
                    pending.text.push_str(&text);
                }
            }
            Event::Code(code) if current_link.is_some() => {
                if let Some(ref mut pending) = current_link {
                    pending.text.push_str(&code);
                }
            }
            Event::End(TagEnd::Link) if current_link.is_some() => {
                if let Some(pending) = current_link.take()
                    && let Some(link) = parse_pending_link(pending)
                {
                    links.push(link);
                }
            }
            _ => {}
        }
    }

    ExtractionResult { links }
}

/// Returns true if the link type represents a standard markdown link.
fn is_inline_or_standard_link(link_type: LinkType) -> bool {
    matches!(
        link_type,
        LinkType::Inline
            | LinkType::Reference
            | LinkType::ReferenceUnknown
            | LinkType::Collapsed
            | LinkType::CollapsedUnknown
            | LinkType::Shortcut
            | LinkType::ShortcutUnknown
    )
}

/// Temporary structure for collecting link data during parsing.
struct PendingLink {
    dest: String,
    text: String,
    line: usize,
}

/// Converts a pending link into an ExtractedLink.
fn parse_pending_link(pending: PendingLink) -> Option<ExtractedLink> {
    let dest = pending.dest.trim();

    if dest.is_empty() {
        return None;
    }

    if is_external_url(dest) {
        return Some(ExtractedLink {
            text: pending.text,
            path: Some(dest.to_string()),
            fragment: None,
            line: pending.line,
            link_type: LinkCategory::External,
        });
    }

    if is_shorthand_id(dest) {
        let id = LatticeId::parse(dest).ok()?;
        return Some(ExtractedLink {
            text: pending.text,
            path: None,
            fragment: Some(id),
            line: pending.line,
            link_type: LinkCategory::ShorthandId,
        });
    }

    let (path, fragment) = split_path_and_fragment(dest);
    let fragment_id = fragment.and_then(|f| LatticeId::parse(f).ok());

    let link_type = match (&path, &fragment_id) {
        (Some(_), Some(_)) => LinkCategory::Canonical,
        (Some(_), None) => LinkCategory::PathOnly,
        (None, Some(_)) => LinkCategory::ShorthandId,
        (None, None) => LinkCategory::Other,
    };

    Some(ExtractedLink {
        text: pending.text,
        path,
        fragment: fragment_id,
        line: pending.line,
        link_type,
    })
}

/// Returns true if the destination looks like an external URL.
fn is_external_url(dest: &str) -> bool {
    dest.starts_with("http://")
        || dest.starts_with("https://")
        || dest.starts_with("mailto:")
        || dest.starts_with("ftp://")
}

/// Returns true if the destination is a shorthand Lattice ID (no path or
/// fragment marker).
fn is_shorthand_id(dest: &str) -> bool {
    !dest.contains('/')
        && !dest.contains('#')
        && !dest.contains('.')
        && LatticeId::parse(dest).is_ok()
}

/// Splits a destination URL into path and fragment components.
fn split_path_and_fragment(dest: &str) -> (Option<String>, Option<&str>) {
    match dest.split_once('#') {
        Some((path, fragment)) => {
            let path = if path.is_empty() { None } else { Some(path.to_string()) };
            let fragment = if fragment.is_empty() { None } else { Some(fragment) };
            (path, fragment)
        }
        None => (Some(dest.to_string()), None),
    }
}

/// Computes the 1-based line number for a byte offset.
fn compute_line_number(content: &str, offset: usize) -> usize {
    content[..offset.min(content.len())].chars().filter(|&c| c == '\n').count() + 1
}
