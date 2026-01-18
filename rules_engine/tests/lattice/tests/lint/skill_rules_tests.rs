use std::fs;
use std::io::Write;

use lattice::index::document_types::InsertDocument;
use lattice::index::{document_queries, schema_definition};
use lattice::lint::rule_engine::{LintConfig, LintContext, LintRule, Severity, execute_rules};
use lattice::lint::skill_rules::{
    DescriptionEmptyRule, NameContainsReservedWordRule, NameContainsXmlRule, all_skill_rules,
};
use rusqlite::Connection;
use tempfile::TempDir;

fn create_test_db() -> Connection {
    let conn = Connection::open_in_memory().expect("Failed to create in-memory database");
    schema_definition::create_schema(&conn).expect("Failed to create schema");
    conn
}

fn create_kb_document(id: &str, path: &str, name: &str, description: &str) -> InsertDocument {
    InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        name.to_string(),
        description.to_string(),
        None,
        None,
        None,
        None,
        None,
        "abc123".to_string(),
        100,
    )
}

fn create_temp_document(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    let mut file = fs::File::create(&path).expect("Failed to create temp file");
    file.write_all(content.as_bytes()).expect("Failed to write temp file");
    path
}

// =============================================================================
// S001: Name Contains Reserved Word
// =============================================================================

#[test]
fn s001_name_contains_reserved_word_rule_interface() {
    let rule = NameContainsReservedWordRule;
    assert_eq!(rule.codes(), &["S001"]);
    assert_eq!(rule.name(), "name-contains-reserved-word");
    assert!(rule.requires_document_body());
}

#[test]
fn s001_detects_claude_in_skill_name() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = r#"---
lattice-id: LDOCAA
name: claude-helper
description: A helpful skill
skill: true
---
Body content
"#;
    create_temp_document(&temp_dir, "claude_helper.md", content);

    let doc = create_kb_document("LDOCAA", "claude_helper.md", "claude-helper", "A helpful skill");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = NameContainsReservedWordRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.error_count, 1, "Should detect reserved word 'claude'");
    assert!(
        summary.results[0].message.contains("'claude'"),
        "Message should mention the reserved word"
    );
    assert_eq!(summary.results[0].severity, Severity::Error);
}

#[test]
fn s001_detects_anthropic_in_skill_name() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = r#"---
lattice-id: LDOCBB
name: anthropic-tools
description: Some tools
skill: true
---
Body content
"#;
    create_temp_document(&temp_dir, "anthropic_tools.md", content);

    let doc = create_kb_document("LDOCBB", "anthropic_tools.md", "anthropic-tools", "Some tools");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = NameContainsReservedWordRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.error_count, 1, "Should detect reserved word 'anthropic'");
    assert!(
        summary.results[0].message.contains("'anthropic'"),
        "Message should mention the reserved word"
    );
}

#[test]
fn s001_case_insensitive_detection() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = r#"---
lattice-id: LDOCCC
name: my-CLAUDE-skill
description: A skill
skill: true
---
Body content
"#;
    create_temp_document(&temp_dir, "my_claude_skill.md", content);

    let doc = create_kb_document("LDOCCC", "my_claude_skill.md", "my-CLAUDE-skill", "A skill");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = NameContainsReservedWordRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.error_count, 1, "Should detect 'claude' case-insensitively");
}

#[test]
fn s001_no_error_for_non_skill_document() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = r#"---
lattice-id: LDOCDD
name: claude-helper
description: Regular doc with claude in name
---
Body content
"#;
    create_temp_document(&temp_dir, "claude_helper.md", content);

    let doc = create_kb_document(
        "LDOCDD",
        "claude_helper.md",
        "claude-helper",
        "Regular doc with claude in name",
    );
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = NameContainsReservedWordRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "S001"),
        "Non-skill documents should not trigger S001"
    );
}

#[test]
fn s001_no_error_for_valid_skill_name() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = r#"---
lattice-id: LDOCEE
name: my-helper
description: A helpful skill
skill: true
---
Body content
"#;
    create_temp_document(&temp_dir, "my_helper.md", content);

    let doc = create_kb_document("LDOCEE", "my_helper.md", "my-helper", "A helpful skill");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = NameContainsReservedWordRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "S001"),
        "Valid skill name should not trigger S001"
    );
}

// =============================================================================
// S002: Description Empty
// =============================================================================

#[test]
fn s002_description_empty_rule_interface() {
    let rule = DescriptionEmptyRule;
    assert_eq!(rule.codes(), &["S002"]);
    assert_eq!(rule.name(), "description-empty");
    assert!(rule.requires_document_body());
}

#[test]
fn s002_detects_empty_description() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = r#"---
lattice-id: LDOCFF
name: my-skill
description: ""
skill: true
---
Body content
"#;
    create_temp_document(&temp_dir, "my_skill.md", content);

    let doc = create_kb_document("LDOCFF", "my_skill.md", "my-skill", "");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = DescriptionEmptyRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.error_count, 1, "Should detect empty description");
    assert!(
        summary.results[0].message.contains("non-empty description"),
        "Message should mention non-empty description"
    );
    assert_eq!(summary.results[0].severity, Severity::Error);
}

#[test]
fn s002_detects_whitespace_only_description() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = r#"---
lattice-id: LDOCGG
name: my-skill
description: "   "
skill: true
---
Body content
"#;
    create_temp_document(&temp_dir, "my_skill.md", content);

    let doc = create_kb_document("LDOCGG", "my_skill.md", "my-skill", "   ");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = DescriptionEmptyRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.error_count, 1, "Should detect whitespace-only description");
}

#[test]
fn s002_no_error_for_non_skill_document() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = r#"---
lattice-id: LDOCHH
name: my-doc
description: ""
---
Body content
"#;
    create_temp_document(&temp_dir, "my_doc.md", content);

    let doc = create_kb_document("LDOCHH", "my_doc.md", "my-doc", "");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = DescriptionEmptyRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "S002"),
        "Non-skill documents should not trigger S002"
    );
}

#[test]
fn s002_no_error_for_valid_description() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = r#"---
lattice-id: LDOCII
name: my-skill
description: A helpful skill that does things
skill: true
---
Body content
"#;
    create_temp_document(&temp_dir, "my_skill.md", content);

    let doc =
        create_kb_document("LDOCII", "my_skill.md", "my-skill", "A helpful skill that does things");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = DescriptionEmptyRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "S002"),
        "Valid description should not trigger S002"
    );
}

// =============================================================================
// S003: Name Contains XML
// =============================================================================

#[test]
fn s003_name_contains_xml_rule_interface() {
    let rule = NameContainsXmlRule;
    assert_eq!(rule.codes(), &["S003"]);
    assert_eq!(rule.name(), "name-contains-xml");
    assert!(rule.requires_document_body());
}

#[test]
fn s003_detects_less_than_in_name() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = r#"---
lattice-id: LDOCJJ
name: "my<skill"
description: A skill
skill: true
---
Body content
"#;
    create_temp_document(&temp_dir, "my_skill.md", content);

    let doc = create_kb_document("LDOCJJ", "my_skill.md", "my<skill", "A skill");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = NameContainsXmlRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.error_count, 1, "Should detect < in name");
    assert!(summary.results[0].message.contains("XML tags"), "Message should mention XML tags");
    assert_eq!(summary.results[0].severity, Severity::Error);
}

#[test]
fn s003_detects_greater_than_in_name() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = r#"---
lattice-id: LDOCKK
name: "my>skill"
description: A skill
skill: true
---
Body content
"#;
    create_temp_document(&temp_dir, "my_skill.md", content);

    let doc = create_kb_document("LDOCKK", "my_skill.md", "my>skill", "A skill");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = NameContainsXmlRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.error_count, 1, "Should detect > in name");
}

#[test]
fn s003_detects_xml_tag_in_name() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = r#"---
lattice-id: LDOCLL
name: "my-<tag>-skill"
description: A skill
skill: true
---
Body content
"#;
    create_temp_document(&temp_dir, "my_skill.md", content);

    let doc = create_kb_document("LDOCLL", "my_skill.md", "my-<tag>-skill", "A skill");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = NameContainsXmlRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert_eq!(summary.error_count, 1, "Should detect XML tag in name");
}

#[test]
fn s003_no_error_for_non_skill_document() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = r#"---
lattice-id: LDOCMM
name: "my<doc"
description: A doc
---
Body content
"#;
    create_temp_document(&temp_dir, "my_doc.md", content);

    let doc = create_kb_document("LDOCMM", "my_doc.md", "my<doc", "A doc");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = NameContainsXmlRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "S003"),
        "Non-skill documents should not trigger S003"
    );
}

#[test]
fn s003_no_error_for_valid_skill_name() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let content = r#"---
lattice-id: LDOCNN
name: my-valid-skill
description: A skill
skill: true
---
Body content
"#;
    create_temp_document(&temp_dir, "my_valid_skill.md", content);

    let doc = create_kb_document("LDOCNN", "my_valid_skill.md", "my-valid-skill", "A skill");
    document_queries::insert(&conn, &doc).expect("Insert should succeed");

    let ctx = LintContext::new(&conn, temp_dir.path());
    let config = LintConfig::default();

    let rule = NameContainsXmlRule;
    let rules: Vec<&dyn LintRule> = vec![&rule];
    let summary = execute_rules(&ctx, &rules, &config).expect("Execute should succeed");

    assert!(
        summary.results.iter().all(|r| r.code != "S003"),
        "Valid skill name should not trigger S003"
    );
}

// =============================================================================
// all_skill_rules Function
// =============================================================================

#[test]
fn all_skill_rules_returns_three_rules() {
    let rules = all_skill_rules();
    assert_eq!(rules.len(), 3, "Should return exactly 3 skill rules");
}

#[test]
fn all_skill_rules_covers_all_skill_codes() {
    let rules = all_skill_rules();
    let expected_codes = ["S001", "S002", "S003"];

    for expected in expected_codes {
        let found = rules.iter().any(|r| r.codes().contains(&expected));
        assert!(found, "Should include rule for code {expected}");
    }
}

#[test]
fn all_skill_rules_all_require_document_body() {
    let rules = all_skill_rules();
    for rule in &rules {
        assert!(
            rule.requires_document_body(),
            "Skill rule {} should require document body",
            rule.name()
        );
    }
}
