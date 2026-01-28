use tv_lib::toml::conditional_formatting::{evaluate_condition, evaluate_rules};
use tv_lib::toml::metadata::parse_conditional_formatting_from_content;
use tv_lib::toml::metadata_serializer::save_metadata;
use tv_lib::toml::metadata_types::{ConditionalFormatRule, FormatCondition, FormatStyle, Metadata};

use crate::test_utils::mock_filesystem::{MockFileSystem, MockTestConfig};

#[test]
fn test_parse_conditional_formatting_equals_string() {
    let content = r##"
[[cards]]
name = "Card 1"

[metadata]
schema_version = 1

[[metadata.conditional_formatting]]
column = "rarity"
condition = { equals = "Rare" }
style = { background_color = "#FFD700", bold = true }
"##;

    let rules = parse_conditional_formatting_from_content(content, "test.toml").unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].column, "rarity");
    assert_eq!(rules[0].condition, FormatCondition::Equals(serde_json::json!("Rare")));
    assert_eq!(rules[0].style.background_color, Some("#FFD700".to_string()));
    assert_eq!(rules[0].style.bold, Some(true));
}

#[test]
fn test_parse_conditional_formatting_contains() {
    let content = r##"
[[cards]]
name = "Card 1"

[metadata]
schema_version = 1

[[metadata.conditional_formatting]]
column = "name"
condition = { contains = "Dragon" }
style = { font_color = "#FF0000" }
"##;

    let rules = parse_conditional_formatting_from_content(content, "test.toml").unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].condition, FormatCondition::Contains("Dragon".to_string()));
    assert_eq!(rules[0].style.font_color, Some("#FF0000".to_string()));
}

#[test]
fn test_parse_conditional_formatting_numeric_comparisons() {
    let content = r##"
[[cards]]
name = "Card 1"

[metadata]
schema_version = 1

[[metadata.conditional_formatting]]
column = "cost"
condition = { greater_than = 5.0 }
style = { background_color = "#FF0000" }

[[metadata.conditional_formatting]]
column = "cost"
condition = { less_than = 2.0 }
style = { background_color = "#00FF00" }
"##;

    let rules = parse_conditional_formatting_from_content(content, "test.toml").unwrap();
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].condition, FormatCondition::GreaterThan(5.0));
    assert_eq!(rules[1].condition, FormatCondition::LessThan(2.0));
}

#[test]
fn test_parse_conditional_formatting_empty_checks() {
    let content = r##"
[[cards]]
name = "Card 1"

[metadata]
schema_version = 1

[[metadata.conditional_formatting]]
column = "notes"
condition = { is_empty = true }
style = { background_color = "#FFCCCC" }

[[metadata.conditional_formatting]]
column = "description"
condition = { not_empty = true }
style = { italic = true }
"##;

    let rules = parse_conditional_formatting_from_content(content, "test.toml").unwrap();
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].condition, FormatCondition::IsEmpty);
    assert_eq!(rules[1].condition, FormatCondition::NotEmpty);
    assert_eq!(rules[1].style.italic, Some(true));
}

#[test]
fn test_parse_conditional_formatting_regex_match() {
    let content = r##"
[[cards]]
name = "Card 1"

[metadata]
schema_version = 1

[[metadata.conditional_formatting]]
column = "id"
condition = { matches = "^[A-Z]{3}-\\d{3}$" }
style = { font_color = "#0000FF" }
"##;

    let rules = parse_conditional_formatting_from_content(content, "test.toml").unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].condition, FormatCondition::Matches("^[A-Z]{3}-\\d{3}$".to_string()));
}

#[test]
fn test_parse_conditional_formatting_no_metadata() {
    let content = r##"
[[cards]]
name = "Card 1"
"##;

    let rules = parse_conditional_formatting_from_content(content, "test.toml").unwrap();
    assert!(rules.is_empty());
}

#[test]
fn test_parse_conditional_formatting_no_rules() {
    let content = r##"
[[cards]]
name = "Card 1"

[metadata]
schema_version = 1
"##;

    let rules = parse_conditional_formatting_from_content(content, "test.toml").unwrap();
    assert!(rules.is_empty());
}

#[test]
fn test_parse_conditional_formatting_all_style_properties() {
    let content = r##"
[[cards]]
name = "Card 1"

[metadata]
schema_version = 1

[[metadata.conditional_formatting]]
column = "status"
condition = { equals = "Active" }
style = { background_color = "#00FF00", font_color = "#FFFFFF", bold = true, italic = true, underline = true }
"##;

    let rules = parse_conditional_formatting_from_content(content, "test.toml").unwrap();
    assert_eq!(rules.len(), 1);
    let style = &rules[0].style;
    assert_eq!(style.background_color, Some("#00FF00".to_string()));
    assert_eq!(style.font_color, Some("#FFFFFF".to_string()));
    assert_eq!(style.bold, Some(true));
    assert_eq!(style.italic, Some(true));
    assert_eq!(style.underline, Some(true));
}

#[test]
fn test_parse_conditional_formatting_integer_greater_than() {
    let content = r##"
[[cards]]
name = "Card 1"

[metadata]
schema_version = 1

[[metadata.conditional_formatting]]
column = "cost"
condition = { greater_than = 5 }
style = { bold = true }
"##;

    let rules = parse_conditional_formatting_from_content(content, "test.toml").unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].condition, FormatCondition::GreaterThan(5.0));
}

#[test]
fn test_evaluate_condition_equals_string() {
    let condition = FormatCondition::Equals(serde_json::json!("Rare"));
    assert!(evaluate_condition(&condition, &serde_json::json!("Rare")));
    assert!(!evaluate_condition(&condition, &serde_json::json!("Common")));
}

#[test]
fn test_evaluate_condition_equals_number() {
    let condition = FormatCondition::Equals(serde_json::json!(42));
    assert!(evaluate_condition(&condition, &serde_json::json!(42)));
    assert!(!evaluate_condition(&condition, &serde_json::json!(43)));
}

#[test]
fn test_evaluate_condition_equals_boolean() {
    let condition = FormatCondition::Equals(serde_json::json!(true));
    assert!(evaluate_condition(&condition, &serde_json::json!(true)));
    assert!(!evaluate_condition(&condition, &serde_json::json!(false)));
}

#[test]
fn test_evaluate_condition_contains() {
    let condition = FormatCondition::Contains("Dragon".to_string());
    assert!(evaluate_condition(&condition, &serde_json::json!("Red Dragon")));
    assert!(evaluate_condition(&condition, &serde_json::json!("Dragon")));
    assert!(!evaluate_condition(&condition, &serde_json::json!("Goblin")));
}

#[test]
fn test_evaluate_condition_contains_number_coercion() {
    let condition = FormatCondition::Contains("42".to_string());
    assert!(evaluate_condition(&condition, &serde_json::json!(42)));
    assert!(evaluate_condition(&condition, &serde_json::json!(142)));
}

#[test]
fn test_evaluate_condition_greater_than() {
    let condition = FormatCondition::GreaterThan(5.0);
    assert!(evaluate_condition(&condition, &serde_json::json!(6)));
    assert!(evaluate_condition(&condition, &serde_json::json!(5.5)));
    assert!(!evaluate_condition(&condition, &serde_json::json!(5)));
    assert!(!evaluate_condition(&condition, &serde_json::json!(4)));
    assert!(!evaluate_condition(&condition, &serde_json::json!("text")));
}

#[test]
fn test_evaluate_condition_less_than() {
    let condition = FormatCondition::LessThan(3.0);
    assert!(evaluate_condition(&condition, &serde_json::json!(2)));
    assert!(evaluate_condition(&condition, &serde_json::json!(2.5)));
    assert!(!evaluate_condition(&condition, &serde_json::json!(3)));
    assert!(!evaluate_condition(&condition, &serde_json::json!(4)));
}

#[test]
fn test_evaluate_condition_is_empty() {
    let condition = FormatCondition::IsEmpty;
    assert!(evaluate_condition(&condition, &serde_json::Value::Null));
    assert!(evaluate_condition(&condition, &serde_json::json!("")));
    assert!(!evaluate_condition(&condition, &serde_json::json!("text")));
    assert!(!evaluate_condition(&condition, &serde_json::json!(0)));
}

#[test]
fn test_evaluate_condition_not_empty() {
    let condition = FormatCondition::NotEmpty;
    assert!(!evaluate_condition(&condition, &serde_json::Value::Null));
    assert!(!evaluate_condition(&condition, &serde_json::json!("")));
    assert!(evaluate_condition(&condition, &serde_json::json!("text")));
    assert!(evaluate_condition(&condition, &serde_json::json!(0)));
}

#[test]
fn test_evaluate_condition_matches() {
    let condition = FormatCondition::Matches(r"^[A-Z]{3}-\d{3}$".to_string());
    assert!(evaluate_condition(&condition, &serde_json::json!("ABC-123")));
    assert!(!evaluate_condition(&condition, &serde_json::json!("abc-123")));
    assert!(!evaluate_condition(&condition, &serde_json::json!("ABCD-1234")));
}

#[test]
fn test_evaluate_condition_matches_invalid_regex() {
    let condition = FormatCondition::Matches("[invalid".to_string());
    assert!(!evaluate_condition(&condition, &serde_json::json!("test")));
}

#[test]
fn test_evaluate_rules_basic() {
    let rules = vec![ConditionalFormatRule::new(
        "rarity",
        FormatCondition::Equals(serde_json::json!("Rare")),
        FormatStyle::new().with_background_color("#FFD700"),
    )];

    let headers = vec!["name".to_string(), "rarity".to_string()];
    let rows = vec![
        vec![serde_json::json!("Card A"), serde_json::json!("Common")],
        vec![serde_json::json!("Card B"), serde_json::json!("Rare")],
        vec![serde_json::json!("Card C"), serde_json::json!("Rare")],
    ];

    let results = evaluate_rules(&rules, &headers, &rows);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].row, 1);
    assert_eq!(results[0].col_index, 1);
    assert_eq!(results[0].style.background_color, Some("#FFD700".to_string()));
    assert_eq!(results[1].row, 2);
}

#[test]
fn test_evaluate_rules_no_matches() {
    let rules = vec![ConditionalFormatRule::new(
        "rarity",
        FormatCondition::Equals(serde_json::json!("Mythic")),
        FormatStyle::new().with_background_color("#FF00FF"),
    )];

    let headers = vec!["rarity".to_string()];
    let rows = vec![vec![serde_json::json!("Common")], vec![serde_json::json!("Rare")]];

    let results = evaluate_rules(&rules, &headers, &rows);
    assert!(results.is_empty());
}

#[test]
fn test_evaluate_rules_unknown_column() {
    let rules = vec![ConditionalFormatRule::new(
        "nonexistent",
        FormatCondition::Equals(serde_json::json!("test")),
        FormatStyle::new().with_bold(true),
    )];

    let headers = vec!["name".to_string()];
    let rows = vec![vec![serde_json::json!("Card A")]];

    let results = evaluate_rules(&rules, &headers, &rows);
    assert!(results.is_empty());
}

#[test]
fn test_evaluate_rules_multiple_rules_merge_styles() {
    let rules = vec![
        ConditionalFormatRule::new(
            "cost",
            FormatCondition::GreaterThan(3.0),
            FormatStyle::new().with_background_color("#FF0000"),
        ),
        ConditionalFormatRule::new(
            "cost",
            FormatCondition::GreaterThan(5.0),
            FormatStyle::new().with_bold(true),
        ),
    ];

    let headers = vec!["cost".to_string()];
    let rows = vec![vec![serde_json::json!(7)]];

    let results = evaluate_rules(&rules, &headers, &rows);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].style.background_color, Some("#FF0000".to_string()));
    assert_eq!(results[0].style.bold, Some(true));
}

#[test]
fn test_evaluate_rules_later_rule_overrides_same_property() {
    let rules = vec![
        ConditionalFormatRule::new(
            "status",
            FormatCondition::NotEmpty,
            FormatStyle::new().with_background_color("#CCCCCC"),
        ),
        ConditionalFormatRule::new(
            "status",
            FormatCondition::Equals(serde_json::json!("Active")),
            FormatStyle::new().with_background_color("#00FF00"),
        ),
    ];

    let headers = vec!["status".to_string()];
    let rows = vec![vec![serde_json::json!("Active")]];

    let results = evaluate_rules(&rules, &headers, &rows);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].style.background_color, Some("#00FF00".to_string()));
}

#[test]
fn test_evaluate_rules_multiple_columns() {
    let rules = vec![
        ConditionalFormatRule::new(
            "name",
            FormatCondition::Contains("Dragon".to_string()),
            FormatStyle::new().with_font_color("#FF0000"),
        ),
        ConditionalFormatRule::new(
            "cost",
            FormatCondition::GreaterThan(5.0),
            FormatStyle::new().with_bold(true),
        ),
    ];

    let headers = vec!["name".to_string(), "cost".to_string()];
    let rows = vec![vec![serde_json::json!("Red Dragon"), serde_json::json!(7)]];

    let results = evaluate_rules(&rules, &headers, &rows);
    assert_eq!(results.len(), 2);

    let name_result = results.iter().find(|r| r.column == "name").unwrap();
    assert_eq!(name_result.style.font_color, Some("#FF0000".to_string()));

    let cost_result = results.iter().find(|r| r.column == "cost").unwrap();
    assert_eq!(cost_result.style.bold, Some(true));
}

#[test]
fn test_save_and_parse_conditional_formatting_roundtrip() {
    let mock_config =
        MockTestConfig::new(MockFileSystem::with_read_and_write("[[cards]]\nname = \"Card 1\"\n"));

    let mut metadata = Metadata::new();
    metadata.conditional_formatting = vec![
        ConditionalFormatRule::new(
            "rarity",
            FormatCondition::Equals(serde_json::json!("Rare")),
            FormatStyle::new().with_background_color("#FFD700").with_bold(true),
        ),
        ConditionalFormatRule::new(
            "cost",
            FormatCondition::GreaterThan(5.0),
            FormatStyle::new().with_font_color("#FF0000"),
        ),
    ];

    save_metadata(&mock_config.config(), "/test.toml", &metadata).unwrap();

    let saved = mock_config.last_written_content().unwrap();
    let parsed = parse_conditional_formatting_from_content(&saved, "/test.toml").unwrap();

    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].column, "rarity");
    assert_eq!(parsed[0].condition, FormatCondition::Equals(serde_json::json!("Rare")));
    assert_eq!(parsed[0].style.background_color, Some("#FFD700".to_string()));
    assert_eq!(parsed[0].style.bold, Some(true));

    assert_eq!(parsed[1].column, "cost");
    assert_eq!(parsed[1].style.font_color, Some("#FF0000".to_string()));
}

#[test]
fn test_evaluate_condition_string_number_cross_comparison() {
    let condition = FormatCondition::Equals(serde_json::json!(42));
    assert!(evaluate_condition(&condition, &serde_json::json!("42")));
}

#[test]
fn test_evaluate_condition_greater_than_string_number() {
    let condition = FormatCondition::GreaterThan(5.0);
    assert!(evaluate_condition(&condition, &serde_json::json!("10")));
    assert!(!evaluate_condition(&condition, &serde_json::json!("3")));
}

#[test]
fn test_evaluate_rules_with_null_cells() {
    let rules = vec![ConditionalFormatRule::new(
        "notes",
        FormatCondition::IsEmpty,
        FormatStyle::new().with_background_color("#FFCCCC"),
    )];

    let headers = vec!["name".to_string(), "notes".to_string()];
    let rows = vec![vec![serde_json::json!("Card A"), serde_json::Value::Null], vec![
        serde_json::json!("Card B"),
        serde_json::json!("Has notes"),
    ]];

    let results = evaluate_rules(&rules, &headers, &rows);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].row, 0);
    assert_eq!(results[0].column, "notes");
}

#[test]
fn test_parse_skips_malformed_rules() {
    let content = r##"
[[cards]]
name = "Card 1"

[metadata]
schema_version = 1

[[metadata.conditional_formatting]]
column = "rarity"
condition = { equals = "Rare" }
style = { bold = true }

[[metadata.conditional_formatting]]
condition = { equals = "Bad" }
style = { bold = true }

[[metadata.conditional_formatting]]
column = "cost"
condition = { greater_than = 5.0 }
style = { font_color = "#FF0000" }
"##;

    let rules = parse_conditional_formatting_from_content(content, "test.toml").unwrap();
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].column, "rarity");
    assert_eq!(rules[1].column, "cost");
}
