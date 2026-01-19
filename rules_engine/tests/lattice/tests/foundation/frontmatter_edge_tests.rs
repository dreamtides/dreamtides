use std::path::Path;

use lattice::document::frontmatter_parser;

fn test_path() -> &'static Path {
    Path::new("test/edge.md")
}

#[test]
fn suggestion_for_single_char_typo_in_priority() {
    let content = r#"---
lattice-id: LABCDT
name: test
description: Test
priorty: 1
---
"#;

    let (_, unknown_keys) =
        frontmatter_parser::parse_with_unknown_key_detection(content, test_path())
            .expect("Should parse despite typo");

    assert_eq!(unknown_keys.len(), 1);
    assert_eq!(unknown_keys[0].key, "priorty");
    assert_eq!(
        unknown_keys[0].suggestion,
        Some("priority".to_string()),
        "Single char deletion should suggest 'priority'"
    );
}

#[test]
fn suggestion_for_single_char_typo_in_blocked_by() {
    let content = r#"---
lattice-id: LABCDT
name: test
description: Test
blockd-by:
  - LXYZAB
---
"#;

    let (_, unknown_keys) =
        frontmatter_parser::parse_with_unknown_key_detection(content, test_path())
            .expect("Should parse");

    assert_eq!(unknown_keys.len(), 1);
    assert_eq!(unknown_keys[0].key, "blockd-by");
    assert_eq!(
        unknown_keys[0].suggestion,
        Some("blocked-by".to_string()),
        "Missing 'e' should suggest 'blocked-by'"
    );
}

#[test]
fn suggestion_for_transposition_typo() {
    let content = r#"---
lattice-id: LABCDT
name: test
description: Test
creaetd-at: 2025-01-01
---
"#;

    let (_, unknown_keys) =
        frontmatter_parser::parse_with_unknown_key_detection(content, test_path())
            .expect("Should parse");

    assert_eq!(unknown_keys.len(), 1);
    assert_eq!(unknown_keys[0].key, "creaetd-at");
    assert_eq!(
        unknown_keys[0].suggestion,
        Some("created-at".to_string()),
        "Transposition should suggest 'created-at'"
    );
}

#[test]
fn no_suggestion_for_distant_typo() {
    let content = r#"---
lattice-id: LABCDT
name: test
description: Test
xyz-field: value
---
"#;

    let (_, unknown_keys) =
        frontmatter_parser::parse_with_unknown_key_detection(content, test_path())
            .expect("Should parse");

    assert_eq!(unknown_keys.len(), 1);
    assert_eq!(unknown_keys[0].key, "xyz-field");
    assert!(
        unknown_keys[0].suggestion.is_none(),
        "Completely different key should have no suggestion"
    );
}

#[test]
fn no_suggestion_for_three_char_edit_distance() {
    let content = r#"---
lattice-id: LABCDT
name: test
description: Test
prixyzity: 1
---
"#;

    let (_, unknown_keys) =
        frontmatter_parser::parse_with_unknown_key_detection(content, test_path())
            .expect("Should parse");

    assert_eq!(unknown_keys.len(), 1);
    assert_eq!(unknown_keys[0].key, "prixyzity");
    assert!(
        unknown_keys[0].suggestion.is_none(),
        "3+ edit distance (pri-xyz-ity vs pri-or-ity) should have no suggestion"
    );
}

#[test]
fn suggestion_threshold_allows_two_edits() {
    let content = r#"---
lattice-id: LABCDT
name: test
description: Test
bloking: []
---
"#;

    let (_, unknown_keys) =
        frontmatter_parser::parse_with_unknown_key_detection(content, test_path())
            .expect("Should parse");

    assert_eq!(unknown_keys.len(), 1);
    assert_eq!(unknown_keys[0].key, "bloking");
    assert_eq!(
        unknown_keys[0].suggestion,
        Some("blocking".to_string()),
        "Two-edit distance should still suggest 'blocking'"
    );
}

#[test]
fn unicode_in_description_value() {
    let content = r#"---
lattice-id: LABCDT
name: test
description: 日本語の説明文
---
"#;

    let result = frontmatter_parser::parse(content, test_path());
    let parsed = result.expect("Should parse with Japanese description");

    assert_eq!(parsed.frontmatter.description, "日本語の説明文");
}

#[test]
fn unicode_in_name_value() {
    let content = r#"---
lattice-id: LABCDT
name: test-émoji-café
description: Test with unicode name
---
"#;

    let result = frontmatter_parser::parse(content, test_path());
    let parsed = result.expect("Should parse with unicode in name");

    assert_eq!(parsed.frontmatter.name, "test-émoji-café");
}

#[test]
fn multiline_description_with_unicode() {
    let content = r#"---
lattice-id: LABCDT
name: test
description: |
  First line with 한글
  Second line with العربية
  Third line with Ελληνικά
---
"#;

    let result = frontmatter_parser::parse(content, test_path());
    let parsed = result.expect("Should parse multiline unicode description");

    assert!(parsed.frontmatter.description.contains("한글"));
    assert!(parsed.frontmatter.description.contains("العربية"));
    assert!(parsed.frontmatter.description.contains("Ελληνικά"));
}

#[test]
fn emoji_in_description() {
    let content = r#"---
lattice-id: LABCDT
name: test
description: "Task: Fix 🐛 bug and add 🎉 feature"
---
"#;

    let result = frontmatter_parser::parse(content, test_path());
    let parsed = result.expect("Should parse emoji in description");

    assert!(parsed.frontmatter.description.contains("🐛"));
    assert!(parsed.frontmatter.description.contains("🎉"));
}

#[test]
fn special_yaml_characters_in_description() {
    let content = r#"---
lattice-id: LABCDT
name: test
description: "Contains: colon, [brackets], {braces}, and #hash"
---
"#;

    let result = frontmatter_parser::parse(content, test_path());
    let parsed = result.expect("Should parse special YAML chars when quoted");

    assert!(parsed.frontmatter.description.contains(":"));
    assert!(parsed.frontmatter.description.contains("["));
    assert!(parsed.frontmatter.description.contains("{"));
    assert!(parsed.frontmatter.description.contains("#"));
}

#[test]
fn case_insensitive_suggestion_matching() {
    let content = r#"---
lattice-id: LABCDT
name: test
description: Test
PRIORITY: 1
---
"#;

    let (_, unknown_keys) =
        frontmatter_parser::parse_with_unknown_key_detection(content, test_path())
            .expect("Should parse");

    assert_eq!(unknown_keys.len(), 1);
    assert_eq!(unknown_keys[0].key, "PRIORITY");
    assert_eq!(
        unknown_keys[0].suggestion,
        Some("priority".to_string()),
        "Uppercase key should suggest lowercase equivalent"
    );
}

#[test]
fn multiple_unknown_keys_all_get_suggestions() {
    let content = r#"---
lattice-id: LABCDT
name: test
description: Test
priorty: 1
taks-type: bug
---
"#;

    let (_, unknown_keys) =
        frontmatter_parser::parse_with_unknown_key_detection(content, test_path())
            .expect("Should parse");

    assert_eq!(unknown_keys.len(), 2);

    let priorty_key =
        unknown_keys.iter().find(|k| k.key == "priorty").expect("Should find priorty");
    let taks_key =
        unknown_keys.iter().find(|k| k.key == "taks-type").expect("Should find taks-type");

    assert_eq!(priorty_key.suggestion, Some("priority".to_string()));
    assert_eq!(taks_key.suggestion, Some("task-type".to_string()));
}

#[test]
fn empty_unknown_keys_for_valid_document() {
    let content = r#"---
lattice-id: LABCDT
name: test
description: Test
priority: 1
task-type: bug
labels:
  - urgent
---
"#;

    let (_, unknown_keys) =
        frontmatter_parser::parse_with_unknown_key_detection(content, test_path())
            .expect("Should parse");

    assert!(unknown_keys.is_empty(), "Valid document should have no unknown keys");
}

#[test]
fn hyphen_vs_underscore_typo() {
    let content = r#"---
lattice-id: LABCDT
name: test
description: Test
task_type: bug
---
"#;

    let (_, unknown_keys) =
        frontmatter_parser::parse_with_unknown_key_detection(content, test_path())
            .expect("Should parse");

    assert_eq!(unknown_keys.len(), 1);
    assert_eq!(unknown_keys[0].key, "task_type");
    assert_eq!(
        unknown_keys[0].suggestion,
        Some("task-type".to_string()),
        "Underscore variant should suggest hyphenated version"
    );
}

#[test]
fn very_long_unknown_key_no_suggestion() {
    let content = r#"---
lattice-id: LABCDT
name: test
description: Test
this-is-a-very-long-custom-field-name: value
---
"#;

    let (_, unknown_keys) =
        frontmatter_parser::parse_with_unknown_key_detection(content, test_path())
            .expect("Should parse");

    assert_eq!(unknown_keys.len(), 1);
    assert!(unknown_keys[0].suggestion.is_none(), "Very long key should not match any allowed key");
}
