//! Document content generators for benchmarking.
//!
//! Provides functions to generate document content with specific
//! characteristics for measuring parsing, link extraction, and formatting
//! performance.

use std::sync::atomic::{AtomicU64, Ordering};

/// Counter for generating unique IDs in document generators.
static GENERATOR_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Generates a unique Lattice ID for document generators.
fn next_id() -> String {
    let counter = GENERATOR_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    let encoded = encode_base32(counter);
    format!("L{encoded}GEN") // GEN = generator client ID
}

/// Encodes a number as Base32 with minimum 2 characters.
fn encode_base32(mut value: u64) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

    if value == 0 {
        return "AA".to_string();
    }

    let mut result = Vec::new();
    while value > 0 {
        result.push(ALPHABET[(value % 32) as usize]);
        value /= 32;
    }
    while result.len() < 2 {
        result.push(b'A');
    }
    result.reverse();
    String::from_utf8(result).expect("Base32 is always valid UTF-8")
}

/// Generates minimal frontmatter content (required fields only).
pub fn generate_minimal_frontmatter() -> String {
    let id = next_id();
    format!(
        "---\n\
         lattice-id: {id}\n\
         name: test-document\n\
         description: A test document for benchmarking\n\
         ---\n\n\
         # Test Document\n\n\
         Some body content.\n"
    )
}

/// Generates full frontmatter content (all optional fields populated).
pub fn generate_full_frontmatter() -> String {
    let id = next_id();
    let parent_id = next_id();
    let blocking_id = next_id();
    let blocked_by_id = next_id();
    let discovered_id = next_id();

    format!(
        "---\n\
         lattice-id: {id}\n\
         name: full-frontmatter-test\n\
         description: A test document with all frontmatter fields populated\n\
         parent-id: {parent_id}\n\
         task-type: feature\n\
         priority: 1\n\
         labels:\n\
         - backend\n\
         - security\n\
         - urgent\n\
         blocking:\n\
         - {blocking_id}\n\
         blocked-by:\n\
         - {blocked_by_id}\n\
         discovered-from:\n\
         - {discovered_id}\n\
         created-at: 2024-01-15T10:30:00Z\n\
         updated-at: 2024-01-16T14:45:00Z\n\
         skill: false\n\
         ---\n\n\
         # Full Frontmatter Test\n\n\
         Document with all fields populated.\n"
    )
}

/// Generates frontmatter with the specified number of labels.
pub fn generate_frontmatter_with_labels(label_count: usize) -> String {
    let id = next_id();
    let mut labels = String::new();
    for i in 0..label_count {
        labels.push_str(&format!("  - label-{i}\n"));
    }

    format!(
        "---\n\
         lattice-id: {id}\n\
         name: many-labels-test\n\
         description: Document with {label_count} labels\n\
         task-type: task\n\
         priority: 2\n\
         labels:\n\
         {labels}\
         ---\n\n\
         # Many Labels Test\n\n\
         Document with many labels.\n"
    )
}

/// Generates frontmatter with the specified number of blocked-by dependencies.
pub fn generate_frontmatter_with_dependencies(dep_count: usize) -> String {
    let id = next_id();
    let mut deps = String::new();
    for _ in 0..dep_count {
        let dep_id = next_id();
        deps.push_str(&format!("  - {dep_id}\n"));
    }

    format!(
        "---\n\
         lattice-id: {id}\n\
         name: many-deps-test\n\
         description: Document with {dep_count} dependencies\n\
         task-type: bug\n\
         priority: 1\n\
         blocked-by:\n\
         {deps}\
         ---\n\n\
         # Many Dependencies Test\n\n\
         Document with many blocked-by entries.\n"
    )
}

/// Generates a markdown body with approximately the specified line count.
pub fn generate_body_with_lines(line_count: usize) -> String {
    let id = next_id();
    let mut body = String::new();

    body.push_str(&format!(
        "---\n\
         lattice-id: {id}\n\
         name: sized-body-test\n\
         description: Document with approximately {line_count} lines\n\
         ---\n\n\
         # Sized Body Test\n\n"
    ));

    #[expect(clippy::integer_division, reason = "Truncating division for section count")]
    let sections = line_count / 20;
    let lines_per_section = if sections > 0 { 15 } else { line_count };

    for s in 0..sections.max(1) {
        body.push_str(&format!("## Section {}\n\n", s + 1));
        for l in 0..lines_per_section {
            body.push_str(&format!(
                "This is line {} in section {}. It contains some text for parsing.\n",
                l + 1,
                s + 1
            ));
        }
        body.push('\n');
    }

    body
}

/// Generates a document with the specified number of markdown links.
pub fn generate_body_with_links(link_count: usize) -> String {
    let id = next_id();
    let mut body = String::new();

    body.push_str(&format!(
        "---\n\
         lattice-id: {id}\n\
         name: many-links-test\n\
         description: Document with {link_count} links\n\
         ---\n\n\
         # Many Links Test\n\n\
         This document contains many links for extraction benchmarks.\n\n\
         ## Links Section\n\n"
    ));

    for i in 0..link_count {
        let target_id = next_id();
        if i % 3 == 0 {
            body.push_str(&format!("- See [shorthand link {}]({target_id})\n", i + 1));
        } else if i % 3 == 1 {
            body.push_str(&format!(
                "- See [path link {}](docs/related_{}.md#{})\n",
                i + 1,
                i,
                target_id
            ));
        } else {
            body.push_str(&format!("- See [path only {}](docs/document_{}.md)\n", i + 1, i));
        }
    }

    body
}

/// Generates a document with many frontmatter links (blocking and blocked-by).
pub fn generate_frontmatter_links_document(
    blocking_count: usize,
    blocked_by_count: usize,
) -> String {
    let id = next_id();
    let mut blocking = String::new();
    let mut blocked_by = String::new();

    for _ in 0..blocking_count {
        let target_id = next_id();
        blocking.push_str(&format!("  - {target_id}\n"));
    }

    for _ in 0..blocked_by_count {
        let target_id = next_id();
        blocked_by.push_str(&format!("  - {target_id}\n"));
    }

    format!(
        "---\n\
         lattice-id: {id}\n\
         name: frontmatter-links-test\n\
         description: Document with frontmatter links\n\
         task-type: task\n\
         priority: 2\n\
         blocking:\n\
         {blocking}\
         blocked-by:\n\
         {blocked_by}\
         ---\n\n\
         # Frontmatter Links Test\n\n\
         Document for testing frontmatter link extraction.\n"
    )
}

/// Generates content that needs formatting (long lines, inconsistent spacing).
pub fn generate_unformatted_content() -> String {
    let id = next_id();

    format!(
        "---\n\
         lattice-id: {id}\n\
         name: unformatted-test\n\
         description: Document with formatting issues\n\
         ---\n\n\
         # Unformatted Document\n\n\
         This is a very long line that exceeds the standard 80 character limit and should be wrapped by the formatter to maintain readability and consistency across the codebase.\n\n\n\n\
         ## Section with Issues\n\n\
         Another extremely long line that contains a lot of text and goes well beyond what should be on a single line in a properly formatted markdown document for better readability.   \n\
         This line has trailing spaces.   \n\
         * Wrong list marker\n\
         + Another wrong marker\n\n\n\n\n\
         ## Final Section\n\
         No blank line after header.\n"
    )
}
