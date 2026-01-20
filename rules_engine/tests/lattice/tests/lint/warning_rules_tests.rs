use std::fs;
use std::io::Write;

use lattice::index::document_types::InsertDocument;
use lattice::index::{document_queries, schema_definition};
use lattice::lint::rule_engine::{LintConfig, LintContext, LintRule, Severity, execute_rules};
use lattice::lint::warning_rules::{
    BackslashInPathRule, BareUrlRule, DescriptionTooLongRule, DocumentTooLargeRule,
    HeadingWithoutBlankLinesRule, InconsistentHeaderStyleRule, InconsistentListMarkersRule,
    InvalidNameCharactersRule, LinkPathMismatchRule, ListWithoutBlankLinesRule,
    MissingFinalNewlineRule, MissingLinkFragmentRule, MultipleBlankLinesRule, NameTooLongRule,
    SelfReferenceRule, TemplateSectionInNonRootRule, TrailingWhitespaceRule, all_warning_rules,
};
use rusqlite::Connection;
use tempfile::TempDir;

fn create_test_db() -> Connection {
    let conn = Connection::open_in_memory().expect("Failed to create in-memory database");
    schema_definition::create_schema(&conn).expect("Failed to create schema");
    conn
}

fn create_kb_document(id: &str, path: &str, name: &str) -> InsertDocument {
    InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        name.to_string(),
        format!("Description for {name}"),
        None,
        None,
        None,
        None,
        None,
        "abc123".to_string(),
        100,
        false,
    )
}

fn create_temp_document(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("Failed to create parent dirs");
    }
    let mut file = fs::File::create(&path).expect("Failed to create temp file");
    file.write_all(content.as_bytes()).expect("Failed to write temp file");
    path
}

fn create_document_with_name_length(id: &str, path: &str, name_len: usize) -> InsertDocument {
    let name: String = std::iter::repeat('a').take(name_len).collect();
    InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        name,
        "Description".to_string(),
        None,
        None,
        None,
        None,
        None,
        "abc123".to_string(),
        100,
        false,
    )
}

fn create_document_with_description_length(
    id: &str,
    path: &str,
    desc_len: usize,
) -> InsertDocument {
    let description: String = std::iter::repeat('a').take(desc_len).collect();
    InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        "test-doc".to_string(),
        description,
        None,
        None,
        None,
        None,
        None,
        "abc123".to_string(),
        100,
        false,
    )
}

// =============================================================================
// W001: Document Too Large
// =============================================================================

#[test]
fn w001_document_too_large_rule_interface() {
    let rule = DocumentTooLargeRule;
    assert_eq!(rule.codes(), &["W001"]);
    assert_eq!(rule.name(), "document-too-large");
    assert!(rule.requires_document_body());
}

#[test]
fn w001_detects_large_document() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let body_lines: String = (0..600).map(|i| format!("Line {i}\n")).collect();
    let content = format!(
        "---\nlattice-id: LDOCAA\nname: large-doc\ndescription: A large document\n---\n\n{body_lines}"
    );
    create_temp_document(&temp_dir, "large_doc.md", &content);

    let doc = create_kb_document("LDOCAA", "large_doc.md", "large-doc");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = DocumentTooLargeRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.warning_count, 1, "Should detect large document");
    assert!(summary.results[0].message.contains("600 lines"));
}

#[test]
fn w001_no_warning_for_small_document() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = "---\nlattice-id: LDOCAA\nname: small-doc\ndescription: A small document\n---\n\nSmall body.\n";
    create_temp_document(&temp_dir, "small_doc.md", content);

    let doc = create_kb_document("LDOCAA", "small_doc.md", "small-doc");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = DocumentTooLargeRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "W001"),
        "Small document should not trigger W001"
    );
}

// =============================================================================
// W002: Name Too Long
// =============================================================================

#[test]
fn w002_name_too_long_rule_interface() {
    let rule = NameTooLongRule;
    assert_eq!(rule.codes(), &["W002"]);
    assert_eq!(rule.name(), "name-too-long");
    assert!(!rule.requires_document_body());
}

#[test]
fn w002_detects_long_name() {
    let conn = create_test_db();
    let doc = create_document_with_name_length("LDOCAA", "api/docs/long_name.md", 70);
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = NameTooLongRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.warning_count, 1, "Should detect long name");
    assert!(summary.results[0].message.contains("70 characters"));
}

#[test]
fn w002_no_warning_for_short_name() {
    let conn = create_test_db();
    let doc = create_document_with_name_length("LDOCAA", "api/docs/short_name.md", 64);
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = NameTooLongRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(summary.results.iter().all(|r| r.code != "W002"), "Short name should not trigger W002");
}

// =============================================================================
// W003: Description Too Long
// =============================================================================

#[test]
fn w003_description_too_long_rule_interface() {
    let rule = DescriptionTooLongRule;
    assert_eq!(rule.codes(), &["W003"]);
    assert_eq!(rule.name(), "description-too-long");
    assert!(!rule.requires_document_body());
}

#[test]
fn w003_detects_long_description() {
    let conn = create_test_db();
    let doc = create_document_with_description_length("LDOCAA", "api/docs/doc.md", 1100);
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = DescriptionTooLongRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.warning_count, 1, "Should detect long description");
    assert!(summary.results[0].message.contains("1100 characters"));
}

#[test]
fn w003_no_warning_for_short_description() {
    let conn = create_test_db();
    let doc = create_document_with_description_length("LDOCAA", "api/docs/doc.md", 1024);
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = DescriptionTooLongRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "W003"),
        "Short description should not trigger W003"
    );
}

// =============================================================================
// W004: Invalid Name Characters
// =============================================================================

#[test]
fn w004_invalid_name_characters_rule_interface() {
    let rule = InvalidNameCharactersRule;
    assert_eq!(rule.codes(), &["W004"]);
    assert_eq!(rule.name(), "invalid-name-characters");
    assert!(!rule.requires_document_body());
}

#[test]
fn w004_detects_uppercase_in_name() {
    let conn = create_test_db();
    let doc = InsertDocument::new(
        "LDOCAA".to_string(),
        None,
        "api/docs/MyDoc.md".to_string(),
        "MyDoc".to_string(),
        "Description".to_string(),
        None,
        None,
        None,
        None,
        None,
        "abc123".to_string(),
        100,
        false,
    );
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = InvalidNameCharactersRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.warning_count, 1, "Should detect invalid name");
    assert!(summary.results[0].message.contains("invalid characters"));
}

#[test]
fn w004_detects_underscore_in_name() {
    let conn = create_test_db();
    let doc = InsertDocument::new(
        "LDOCAA".to_string(),
        None,
        "api/docs/my_doc.md".to_string(),
        "my_doc".to_string(),
        "Description".to_string(),
        None,
        None,
        None,
        None,
        None,
        "abc123".to_string(),
        100,
        false,
    );
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = InvalidNameCharactersRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.warning_count, 1, "Should detect underscore in name");
}

#[test]
fn w004_no_warning_for_valid_name() {
    let conn = create_test_db();
    let doc = create_kb_document("LDOCAA", "api/docs/my-doc.md", "my-doc");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = InvalidNameCharactersRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(summary.results.iter().all(|r| r.code != "W004"), "Valid name should not trigger W004");
}

// =============================================================================
// W005: Inconsistent Header Style
// =============================================================================

#[test]
fn w005_inconsistent_header_style_rule_interface() {
    let rule = InconsistentHeaderStyleRule;
    assert_eq!(rule.codes(), &["W005"]);
    assert_eq!(rule.name(), "inconsistent-header-style");
    assert!(rule.requires_document_body());
}

#[test]
fn w005_detects_mixed_header_styles() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = r#"---
lattice-id: LDOCAA
name: mixed-headers
description: Document with mixed headers
---

# ATX Header

Setext Header
=============

Some content.
"#;
    create_temp_document(&temp_dir, "mixed_headers.md", content);

    let doc = create_kb_document("LDOCAA", "mixed_headers.md", "mixed-headers");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = InconsistentHeaderStyleRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.warning_count, 1, "Should detect mixed header styles");
    assert!(summary.results[0].message.contains("mixes header styles"));
}

#[test]
fn w005_no_warning_for_consistent_atx_headers() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = r#"---
lattice-id: LDOCAA
name: atx-headers
description: Document with ATX headers
---

# Header 1

## Header 2

### Header 3
"#;
    create_temp_document(&temp_dir, "atx_headers.md", content);

    let doc = create_kb_document("LDOCAA", "atx_headers.md", "atx-headers");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = InconsistentHeaderStyleRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "W005"),
        "Consistent ATX headers should not trigger W005"
    );
}

// =============================================================================
// W006: Inconsistent List Markers
// =============================================================================

#[test]
fn w006_inconsistent_list_markers_rule_interface() {
    let rule = InconsistentListMarkersRule;
    assert_eq!(rule.codes(), &["W006"]);
    assert_eq!(rule.name(), "inconsistent-list-markers");
    assert!(rule.requires_document_body());
}

#[test]
fn w006_detects_mixed_list_markers() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = r#"---
lattice-id: LDOCAA
name: mixed-lists
description: Document with mixed list markers
---

- Item with dash
* Item with asterisk
+ Item with plus
"#;
    create_temp_document(&temp_dir, "mixed_lists.md", content);

    let doc = create_kb_document("LDOCAA", "mixed_lists.md", "mixed-lists");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = InconsistentListMarkersRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.warning_count, 1, "Should detect mixed list markers");
    assert!(summary.results[0].message.contains("mixes list markers"));
}

#[test]
fn w006_no_warning_for_consistent_markers() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = r#"---
lattice-id: LDOCAA
name: dash-lists
description: Document with dash lists
---

- Item one
- Item two
- Item three
"#;
    create_temp_document(&temp_dir, "dash_lists.md", content);

    let doc = create_kb_document("LDOCAA", "dash_lists.md", "dash-lists");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = InconsistentListMarkersRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "W006"),
        "Consistent markers should not trigger W006"
    );
}

// =============================================================================
// W007: Bare URL
// =============================================================================

#[test]
fn w007_bare_url_rule_interface() {
    let rule = BareUrlRule;
    assert_eq!(rule.codes(), &["W007"]);
    assert_eq!(rule.name(), "bare-url");
    assert!(rule.requires_document_body());
}

#[test]
fn w007_detects_bare_url() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = r#"---
lattice-id: LDOCAA
name: bare-url-doc
description: Document with bare URL
---

Visit https://example.com for more info.
"#;
    create_temp_document(&temp_dir, "bare_url_doc.md", content);

    let doc = create_kb_document("LDOCAA", "bare_url_doc.md", "bare-url-doc");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = BareUrlRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.warning_count, 1, "Should detect bare URL");
    assert!(summary.results[0].message.contains("bare URL"));
}

#[test]
fn w007_no_warning_for_markdown_link() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = r#"---
lattice-id: LDOCAA
name: proper-link
description: Document with proper link
---

Visit [Example](https://example.com) for more info.
"#;
    create_temp_document(&temp_dir, "proper_link.md", content);

    let doc = create_kb_document("LDOCAA", "proper_link.md", "proper-link");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = BareUrlRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "W007"),
        "Proper markdown link should not trigger W007"
    );
}

// =============================================================================
// W008: Self-Reference
// =============================================================================

#[test]
fn w008_self_reference_rule_interface() {
    let rule = SelfReferenceRule;
    assert_eq!(rule.codes(), &["W008"]);
    assert_eq!(rule.name(), "self-reference");
    assert!(rule.requires_document_body());
}

#[test]
fn w008_detects_self_reference() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = r#"---
lattice-id: LSELFA
name: self-ref
description: Document that references itself
---

See [myself](LSELFA) for more info.
"#;
    create_temp_document(&temp_dir, "self_ref.md", content);

    let doc = create_kb_document("LSELFA", "self_ref.md", "self-ref");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = SelfReferenceRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.warning_count, 1, "Should detect self-reference");
    assert!(summary.results[0].message.contains("self-reference"));
}

// =============================================================================
// W009: Backslash in Path
// =============================================================================

#[test]
fn w009_backslash_in_path_rule_interface() {
    let rule = BackslashInPathRule;
    assert_eq!(rule.codes(), &["W009"]);
    assert_eq!(rule.name(), "backslash-in-path");
    assert!(rule.requires_document_body());
}

#[test]
fn w009_detects_backslash_in_path() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = r#"---
lattice-id: LDOCAA
name: backslash-doc
description: Document with backslash path
---

See [other](docs\other.md#LOTHER) for more info.
"#;
    create_temp_document(&temp_dir, "backslash_doc.md", content);

    let doc = create_kb_document("LDOCAA", "backslash_doc.md", "backslash-doc");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = BackslashInPathRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.warning_count, 1, "Should detect backslash in path");
    assert!(summary.results[0].message.contains("backslash"));
}

// =============================================================================
// W010: Link Path Mismatch
// =============================================================================

#[test]
fn w010_link_path_mismatch_rule_interface() {
    let rule = LinkPathMismatchRule;
    assert_eq!(rule.codes(), &["W010"]);
    assert_eq!(rule.name(), "link-path-mismatch");
    assert!(rule.requires_document_body());
}

#[test]
fn w010_detects_stale_link_path() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Source document links to target using an old/wrong path
    let source_content = r#"---
lattice-id: LSRCAA
name: source-doc
description: Document with stale link
---

See [target](old/path/target.md#LTGTAA) for more info.
"#;
    create_temp_document(&temp_dir, "source_doc.md", source_content);

    // Target document is at a different path than what the link says
    let target_content = r#"---
lattice-id: LTGTAA
name: target-doc
description: Target document
---

Target content.
"#;
    create_temp_document(&temp_dir, "new/path/target_doc.md", target_content);

    let source_doc = create_kb_document("LSRCAA", "source_doc.md", "source-doc");
    let target_doc = create_kb_document("LTGTAA", "new/path/target_doc.md", "target-doc");
    document_queries::insert(&conn, &source_doc).expect("Insert source should succeed");
    document_queries::insert(&conn, &target_doc).expect("Insert target should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = LinkPathMismatchRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.warning_count, 1, "Should detect stale link path");
    assert!(summary.results[0].message.contains("stale link path"));
}

#[test]
fn w010_no_warning_for_correct_path() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Source document with correct link path
    let source_content = r#"---
lattice-id: LSRCAA
name: source-doc
description: Document with correct link
---

See [target](target_doc.md#LTGTAA) for more info.
"#;
    create_temp_document(&temp_dir, "source_doc.md", source_content);

    let target_content = r#"---
lattice-id: LTGTAA
name: target-doc
description: Target document
---

Target content.
"#;
    create_temp_document(&temp_dir, "target_doc.md", target_content);

    let source_doc = create_kb_document("LSRCAA", "source_doc.md", "source-doc");
    let target_doc = create_kb_document("LTGTAA", "target_doc.md", "target-doc");
    document_queries::insert(&conn, &source_doc).expect("Insert source should succeed");
    document_queries::insert(&conn, &target_doc).expect("Insert target should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = LinkPathMismatchRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "W010"),
        "Correct link path should not trigger W010"
    );
}

// =============================================================================
// W010b: Missing Link Fragment
// =============================================================================

#[test]
fn w010b_missing_link_fragment_rule_interface() {
    let rule = MissingLinkFragmentRule;
    assert_eq!(rule.codes(), &["W010b"]);
    assert_eq!(rule.name(), "missing-link-fragment");
    assert!(rule.requires_document_body());
}

#[test]
fn w010b_detects_missing_fragment() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = r#"---
lattice-id: LDOCAA
name: no-fragment
description: Document with missing fragment
---

See [other](other.md) for more info.
"#;
    create_temp_document(&temp_dir, "no_fragment.md", content);

    let doc = create_kb_document("LDOCAA", "no_fragment.md", "no-fragment");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = MissingLinkFragmentRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.warning_count, 1, "Should detect missing fragment");
    assert!(summary.results[0].message.contains("missing Lattice ID fragment"));
}

#[test]
fn w010b_no_warning_for_link_with_fragment() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = r#"---
lattice-id: LDOCAA
name: with-fragment
description: Document with fragment
---

See [other](other.md#LOTHER) for more info.
"#;
    create_temp_document(&temp_dir, "with_fragment.md", content);

    let doc = create_kb_document("LDOCAA", "with_fragment.md", "with-fragment");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = MissingLinkFragmentRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "W010b"),
        "Link with fragment should not trigger W010b"
    );
}

// =============================================================================
// W011: Trailing Whitespace
// =============================================================================

#[test]
fn w011_trailing_whitespace_rule_interface() {
    let rule = TrailingWhitespaceRule;
    assert_eq!(rule.codes(), &["W011"]);
    assert_eq!(rule.name(), "trailing-whitespace");
    assert!(rule.requires_document_body());
}

#[test]
fn w011_detects_trailing_whitespace() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = "---\nlattice-id: LDOCAA\nname: trailing\ndescription: Doc with trailing whitespace\n---\n\nLine with trailing   \n";
    create_temp_document(&temp_dir, "trailing.md", content);

    let doc = create_kb_document("LDOCAA", "trailing.md", "trailing");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = TrailingWhitespaceRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.warning_count, 1, "Should detect trailing whitespace");
    assert!(summary.results[0].message.contains("trailing whitespace"));
}

// =============================================================================
// W012: Multiple Blank Lines
// =============================================================================

#[test]
fn w012_multiple_blank_lines_rule_interface() {
    let rule = MultipleBlankLinesRule;
    assert_eq!(rule.codes(), &["W012"]);
    assert_eq!(rule.name(), "multiple-blank-lines");
    assert!(rule.requires_document_body());
}

#[test]
fn w012_detects_multiple_blank_lines() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = "---\nlattice-id: LDOCAA\nname: blanks\ndescription: Doc with blanks\n---\n\nParagraph one.\n\n\n\nParagraph two.\n";
    create_temp_document(&temp_dir, "blanks.md", content);

    let doc = create_kb_document("LDOCAA", "blanks.md", "blanks");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = MultipleBlankLinesRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.warning_count, 1, "Should detect multiple blank lines");
    assert!(summary.results[0].message.contains("consecutive blank lines"));
}

// =============================================================================
// W013: Missing Final Newline
// =============================================================================

#[test]
fn w013_missing_final_newline_rule_interface() {
    let rule = MissingFinalNewlineRule;
    assert_eq!(rule.codes(), &["W013"]);
    assert_eq!(rule.name(), "missing-final-newline");
    assert!(rule.requires_document_body());
}

#[test]
fn w013_detects_missing_final_newline() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = "---\nlattice-id: LDOCAA\nname: no-newline\ndescription: Doc without final newline\n---\n\nNo newline at end";
    create_temp_document(&temp_dir, "no_newline.md", content);

    let doc = create_kb_document("LDOCAA", "no_newline.md", "no-newline");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = MissingFinalNewlineRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.warning_count, 1, "Should detect missing final newline");
    assert!(summary.results[0].message.contains("end with newline"));
}

#[test]
fn w013_no_warning_for_file_with_newline() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = "---\nlattice-id: LDOCAA\nname: with-newline\ndescription: Doc with final newline\n---\n\nContent.\n";
    create_temp_document(&temp_dir, "with_newline.md", content);

    let doc = create_kb_document("LDOCAA", "with_newline.md", "with-newline");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = MissingFinalNewlineRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "W013"),
        "File with newline should not trigger W013"
    );
}

// =============================================================================
// W014: Heading Without Blank Lines
// =============================================================================

#[test]
fn w014_heading_without_blank_lines_rule_interface() {
    let rule = HeadingWithoutBlankLinesRule;
    assert_eq!(rule.codes(), &["W014"]);
    assert_eq!(rule.name(), "heading-without-blank-lines");
    assert!(rule.requires_document_body());
}

#[test]
fn w014_detects_heading_without_blank_lines() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = r#"---
lattice-id: LDOCAA
name: tight-heading
description: Doc with tight heading
---

Some text.
# Heading
More text.
"#;
    create_temp_document(&temp_dir, "tight_heading.md", content);

    let doc = create_kb_document("LDOCAA", "tight_heading.md", "tight-heading");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = HeadingWithoutBlankLinesRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.warning_count, 1, "Should detect heading without blank lines");
    assert!(summary.results[0].message.contains("blank line before/after"));
}

#[test]
fn w014_line_numbers_account_for_frontmatter() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // This document has:
    // Line 1:  ---
    // Line 2:  lattice-id: LDOCAA
    // Line 3:  name: line-test
    // Line 4:  description: Test line numbers
    // Line 5:  ---
    // Line 6:  (empty)
    // Line 7:  Some text.
    // Line 8:  # Heading <- This is the issue (no blank before)
    // Line 9:  More text.
    //
    // The heading without blank line is on FILE LINE 8.
    // The body starts at line 7, so it's BODY LINE 2.
    // Bug: code reports body line (2) not file line (8).
    let content = "---\nlattice-id: LDOCAA\nname: line-test\ndescription: Test line numbers\n---\n\nSome text.\n# Heading\nMore text.\n";
    create_temp_document(&temp_dir, "line_test.md", content);

    let doc = create_kb_document("LDOCAA", "line_test.md", "line-test");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = HeadingWithoutBlankLinesRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.warning_count, 1, "Should detect heading warning");

    let result = &summary.results[0];
    assert_eq!(
        result.line,
        Some(8),
        "Line number should be 8 (file line), not 2 (body line). Got {:?}",
        result.line
    );
}

// =============================================================================
// W015: List Without Blank Lines
// =============================================================================

#[test]
fn w015_list_without_blank_lines_rule_interface() {
    let rule = ListWithoutBlankLinesRule;
    assert_eq!(rule.codes(), &["W015"]);
    assert_eq!(rule.name(), "list-without-blank-lines");
    assert!(rule.requires_document_body());
}

#[test]
fn w015_detects_list_without_blank_lines() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = r#"---
lattice-id: LDOCAA
name: tight-list
description: Doc with tight list
---

Some text.
- Item one
- Item two
More text.
"#;
    create_temp_document(&temp_dir, "tight_list.md", content);

    let doc = create_kb_document("LDOCAA", "tight_list.md", "tight-list");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = ListWithoutBlankLinesRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(summary.warning_count >= 1, "Should detect list without blank lines");
}

// =============================================================================
// W016: Template Section in Non-Root
// =============================================================================

#[test]
fn w016_template_section_in_non_root_rule_interface() {
    let rule = TemplateSectionInNonRootRule;
    assert_eq!(rule.codes(), &["W016"]);
    assert_eq!(rule.name(), "template-section-in-non-root");
    assert!(rule.requires_document_body());
}

#[test]
fn w016_detects_template_section_in_non_root() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = r#"---
lattice-id: LDOCAA
name: task-with-template
description: Task with template section
---

## [Lattice] Context

This should not be here.
"#;
    create_temp_document(&temp_dir, "api/tasks/task_with_template.md", content);

    let doc = create_kb_document("LDOCAA", "api/tasks/task_with_template.md", "task-with-template");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = TemplateSectionInNonRootRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.warning_count, 1, "Should detect template section in non-root");
    assert!(summary.results[0].message.contains("[Lattice] sections"));
}

#[test]
fn w016_no_warning_for_template_in_root() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = r#"---
lattice-id: LDOCAA
name: api
description: API root document
---

## [Lattice] Context

This is allowed in root documents.
"#;
    fs::create_dir_all(temp_dir.path().join("api")).expect("Create dir");
    create_temp_document(&temp_dir, "api/api.md", content);

    // Using path "api/api.md" where filename matches directory name makes
    // is_root=true
    let doc = create_kb_document("LDOCAA", "api/api.md", "api");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = TemplateSectionInNonRootRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "W016"),
        "Template in root should not trigger W016"
    );
}

// =============================================================================
// Integration: All Warning Rules
// =============================================================================

#[test]
fn all_warning_rules_returns_seventeen_rules() {
    let rules = all_warning_rules();
    assert_eq!(rules.len(), 17, "Should have 17 warning rules");
}

#[test]
fn all_warning_rules_covers_all_warning_codes() {
    let rules = all_warning_rules();
    let mut codes: Vec<&str> = rules.iter().flat_map(|r| r.codes()).copied().collect();
    codes.sort();

    let expected = vec![
        "W001", "W002", "W003", "W004", "W005", "W006", "W007", "W008", "W009", "W010", "W010b",
        "W011", "W012", "W013", "W014", "W015", "W016",
    ];

    assert_eq!(codes, expected, "All warning codes should be covered");
}

#[test]
fn all_warning_rules_are_warning_severity() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content =
        "---\nlattice-id: LDOCAA\nname: test-doc\ndescription: Test document\n---\n\nBody.\n";
    create_temp_document(&temp_dir, "test_doc.md", content);

    let doc = create_kb_document("LDOCAA", "test_doc.md", "test-doc");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rules = all_warning_rules();
    let rule_refs: Vec<&dyn LintRule> = rules.iter().map(|r| r.as_ref()).collect();
    let summary = execute_rules(&ctx, &rule_refs, &config).expect("Execute should succeed");

    for result in &summary.results {
        assert_eq!(
            result.severity,
            Severity::Warning,
            "All warning rules should produce Warning severity, got {:?} for {}",
            result.severity,
            result.code
        );
    }
}

#[test]
fn w014_should_ignore_content_inside_code_blocks() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = r#"---
lattice-id: LDOCAA
name: code-block-doc
description: Doc with code block containing heading-like content
---

Here is some configuration:

```toml
[defaults]
timeout = 30

# This is a TOML comment that looks like a markdown heading
port = 8080

[workers.adam]
threads = 4
```

More text after the code block.
"#;
    create_temp_document(&temp_dir, "code_block_doc.md", content);

    let doc = create_kb_document("LDOCAA", "code_block_doc.md", "code-block-doc");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = HeadingWithoutBlankLinesRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(
        summary.warning_count, 0,
        "Should not detect heading warnings inside code blocks, but got: {:?}",
        summary.results
    );
}

#[test]
fn w014_should_ignore_content_inside_tilde_code_blocks() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = r#"---
lattice-id: LDOCAA
name: tilde-fence-doc
description: Doc with tilde code block
---

Here is a bash script:

~~~bash
# This is a bash comment that looks like a markdown heading
echo "hello"

# Another comment
~~~

More text.
"#;
    create_temp_document(&temp_dir, "tilde_fence_doc.md", content);

    let doc = create_kb_document("LDOCAA", "tilde_fence_doc.md", "tilde-fence-doc");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = HeadingWithoutBlankLinesRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(
        summary.warning_count, 0,
        "Should not detect heading warnings inside tilde code blocks, but got: {:?}",
        summary.results
    );
}

#[test]
fn w014_should_still_detect_issues_outside_code_blocks() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = r#"---
lattice-id: LDOCAA
name: mixed-doc
description: Doc with code blocks and real headings
---

Some intro text.
# Heading Without Blank Line Before
More text.

```toml
# This comment inside code block should be ignored
[section]
```

"#;
    create_temp_document(&temp_dir, "mixed_doc.md", content);

    let doc = create_kb_document("LDOCAA", "mixed_doc.md", "mixed-doc");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = HeadingWithoutBlankLinesRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(
        summary.warning_count, 1,
        "Should detect heading without blank line outside code block"
    );
    assert!(summary.results[0].message.contains("blank line before/after"));
}
