use tv_lib::toml::metadata_serializer::save_metadata;
use tv_lib::toml::metadata_types::{
    Alignment, AppSettings, ColumnConfig, ConditionalFormatRule, DerivedColumnConfig,
    FormatCondition, FormatStyle, Metadata, ScrollPosition, SortConfig, TableStyle,
};
use tv_lib::validation::validation_rules::ValidationRule;

use crate::test_utils::mock_filesystem::{MockFileSystem, MockTestConfig};

#[test]
fn test_save_metadata_creates_section_if_missing() {
    let mock_config = MockTestConfig::new(MockFileSystem::with_read_and_write(
        r#"[[cards]]
name = "Card 1"
"#,
    ));

    let mut metadata = Metadata::new();
    metadata.columns.push(ColumnConfig::new("name").with_width(200));

    let result = save_metadata(&mock_config.config(), "/test.toml", &metadata);
    assert!(result.is_ok());

    let saved = mock_config.last_written_content().unwrap();
    assert!(saved.contains("[metadata]"), "Expected [metadata] section in:\n{saved}");
    assert!(saved.contains("schema_version = 1"), "Expected schema_version in:\n{saved}");
    assert!(saved.contains("[[metadata.columns]]"), "Expected columns in:\n{saved}");
}

#[test]
fn test_save_metadata_updates_existing_section() {
    let mock_config = MockTestConfig::new(MockFileSystem::with_read_and_write(
        r#"[[cards]]
name = "Card 1"

[metadata]
schema_version = 1

[[metadata.columns]]
key = "id"
width = 100
"#,
    ));

    let mut metadata = Metadata::new();
    metadata.columns.push(ColumnConfig::new("name").with_width(300));

    let result = save_metadata(&mock_config.config(), "/test.toml", &metadata);
    assert!(result.is_ok());

    let saved = mock_config.last_written_content().unwrap();
    assert!(saved.contains("width = 300"), "Expected width = 300 in:\n{saved}");
    assert!(saved.contains("key = \"name\""), "Expected key = \"name\" in:\n{saved}");
}

#[test]
fn test_save_metadata_preserves_unknown_fields() {
    let mock_config = MockTestConfig::new(MockFileSystem::with_read_and_write(
        r#"[[cards]]
name = "Card 1"

[metadata]
schema_version = 1
unknown_future_field = "preserve me"
another_unknown = 42
"#,
    ));

    let metadata = Metadata::new();

    let result = save_metadata(&mock_config.config(), "/test.toml", &metadata);
    assert!(result.is_ok());

    let saved = mock_config.last_written_content().unwrap();
    assert!(
        saved.contains("unknown_future_field = \"preserve me\""),
        "Expected unknown_future_field preserved in:\n{saved}",
    );
    assert!(
        saved.contains("another_unknown = 42"),
        "Expected another_unknown preserved in:\n{saved}"
    );
}

#[test]
fn test_save_metadata_round_trip_columns() {
    let mock_config =
        MockTestConfig::new(MockFileSystem::with_read_and_write("[[cards]]\nname = \"Card 1\"\n"));

    let mut metadata = Metadata::new();
    metadata.columns.push(
        ColumnConfig::new("id").with_width(300).with_alignment(Alignment::Center).with_frozen(true),
    );
    metadata.columns.push(ColumnConfig::new("name").with_width(200));

    let result = save_metadata(&mock_config.config(), "/test.toml", &metadata);
    assert!(result.is_ok());

    let saved = mock_config.last_written_content().unwrap();
    assert!(saved.contains("key = \"id\""), "Expected id column in:\n{saved}");
    assert!(saved.contains("width = 300"), "Expected width 300 in:\n{saved}");
    assert!(saved.contains("alignment = \"center\""), "Expected alignment in:\n{saved}");
    assert!(saved.contains("frozen = true"), "Expected frozen in:\n{saved}");
    assert!(saved.contains("key = \"name\""), "Expected name column in:\n{saved}");
    assert!(saved.contains("width = 200"), "Expected width 200 in:\n{saved}");
}

#[test]
fn test_save_metadata_validation_rules() {
    let mock_config =
        MockTestConfig::new(MockFileSystem::with_read_and_write("[[cards]]\nname = \"Card 1\"\n"));

    let mut metadata = Metadata::new();
    metadata.validation_rules.push(ValidationRule::Enum {
        column: "card_type".to_string(),
        allowed_values: vec!["Character".to_string(), "Event".to_string()],
        message: Some("Invalid card type".to_string()),
    });
    metadata.validation_rules.push(ValidationRule::Range {
        column: "cost".to_string(),
        min: Some(0.0),
        max: Some(10.0),
        message: None,
    });

    let result = save_metadata(&mock_config.config(), "/test.toml", &metadata);
    assert!(result.is_ok());

    let saved = mock_config.last_written_content().unwrap();
    assert!(
        saved.contains("[[metadata.validation_rules]]"),
        "Expected validation_rules in:\n{saved}",
    );
    assert!(saved.contains("type = \"enum\""), "Expected enum type in:\n{saved}");
    assert!(saved.contains("type = \"range\""), "Expected range type in:\n{saved}");
}

#[test]
fn test_save_metadata_sort_config() {
    let mock_config =
        MockTestConfig::new(MockFileSystem::with_read_and_write("[[cards]]\nname = \"Card 1\"\n"));

    let mut metadata = Metadata::new();
    metadata.sort = Some(SortConfig::descending("name"));

    let result = save_metadata(&mock_config.config(), "/test.toml", &metadata);
    assert!(result.is_ok());

    let saved = mock_config.last_written_content().unwrap();
    assert!(saved.contains("[metadata.sort]"), "Expected sort section in:\n{saved}");
    assert!(saved.contains("column = \"name\""), "Expected column = \"name\" in:\n{saved}");
    assert!(saved.contains("ascending = false"), "Expected ascending = false in:\n{saved}");
}

#[test]
fn test_save_metadata_table_style() {
    let mock_config =
        MockTestConfig::new(MockFileSystem::with_read_and_write("[[cards]]\nname = \"Card 1\"\n"));

    let mut metadata = Metadata::new();
    metadata.table_style = Some(TableStyle::new().with_color_scheme("blue_light"));

    let result = save_metadata(&mock_config.config(), "/test.toml", &metadata);
    assert!(result.is_ok());

    let saved = mock_config.last_written_content().unwrap();
    assert!(saved.contains("[metadata.table_style]"), "Expected table_style section in:\n{saved}");
    assert!(saved.contains("color_scheme = \"blue_light\""), "Expected color_scheme in:\n{saved}");
}

#[test]
fn test_save_metadata_preserves_card_data() {
    let mock_config = MockTestConfig::new(MockFileSystem::with_read_and_write(
        r#"[[cards]]
name = "Card 1"
id = "abc-123"

[[cards]]
name = "Card 2"
id = "def-456"
"#,
    ));

    let metadata = Metadata::new();

    let result = save_metadata(&mock_config.config(), "/test.toml", &metadata);
    assert!(result.is_ok());

    let saved = mock_config.last_written_content().unwrap();
    assert!(saved.contains("name = \"Card 1\""), "Expected Card 1 preserved in:\n{saved}");
    assert!(saved.contains("name = \"Card 2\""), "Expected Card 2 preserved in:\n{saved}");
    assert!(saved.contains("id = \"abc-123\""), "Expected abc-123 preserved in:\n{saved}");
    assert!(saved.contains("id = \"def-456\""), "Expected def-456 preserved in:\n{saved}");
}

#[test]
fn test_save_metadata_derived_columns() {
    let mock_config =
        MockTestConfig::new(MockFileSystem::with_read_and_write("[[cards]]\nname = \"Card 1\"\n"));

    let mut metadata = Metadata::new();
    metadata.derived_columns.push(
        DerivedColumnConfig::new("Preview", "rules_preview")
            .with_inputs(vec!["rules_text".to_string(), "variables".to_string()])
            .with_width(400)
            .with_position(5),
    );

    let result = save_metadata(&mock_config.config(), "/test.toml", &metadata);
    assert!(result.is_ok());

    let saved = mock_config.last_written_content().unwrap();
    assert!(
        saved.contains("[[metadata.derived_columns]]"),
        "Expected derived_columns in:\n{saved}",
    );
    assert!(saved.contains("name = \"Preview\""), "Expected name in:\n{saved}");
    assert!(saved.contains("function = \"rules_preview\""), "Expected function in:\n{saved}");
    assert!(saved.contains("position = 5"), "Expected position in:\n{saved}");
    assert!(saved.contains("width = 400"), "Expected width in:\n{saved}");
}

#[test]
fn test_save_metadata_conditional_formatting() {
    let mock_config =
        MockTestConfig::new(MockFileSystem::with_read_and_write("[[cards]]\nname = \"Card 1\"\n"));

    let mut metadata = Metadata::new();
    metadata.conditional_formatting.push(ConditionalFormatRule::new(
        "rarity",
        FormatCondition::Equals(serde_json::json!("Rare")),
        FormatStyle::new().with_background_color("#FFD700").with_bold(true),
    ));

    let result = save_metadata(&mock_config.config(), "/test.toml", &metadata);
    assert!(result.is_ok());

    let saved = mock_config.last_written_content().unwrap();
    assert!(
        saved.contains("[[metadata.conditional_formatting]]"),
        "Expected conditional_formatting in:\n{saved}",
    );
    assert!(saved.contains("column = \"rarity\""), "Expected column in:\n{saved}");
}

#[test]
fn test_save_metadata_app_settings() {
    let mock_config =
        MockTestConfig::new(MockFileSystem::with_read_and_write("[[cards]]\nname = \"Card 1\"\n"));

    let mut metadata = Metadata::new();
    metadata.app_settings = Some(AppSettings {
        last_selected_cell: Some("B5".to_string()),
        scroll_position: Some(ScrollPosition::new(10, 2)),
        zoom_level: 1.5,
    });

    let result = save_metadata(&mock_config.config(), "/test.toml", &metadata);
    assert!(result.is_ok());

    let saved = mock_config.last_written_content().unwrap();
    assert!(
        saved.contains("[metadata.app_settings]"),
        "Expected app_settings section in:\n{saved}",
    );
    assert!(
        saved.contains("last_selected_cell = \"B5\""),
        "Expected last_selected_cell in:\n{saved}",
    );
    assert!(saved.contains("zoom_level = 1.5"), "Expected zoom_level in:\n{saved}");
}

#[test]
fn test_default_values_not_serialized() {
    let mock_config =
        MockTestConfig::new(MockFileSystem::with_read_and_write("[[cards]]\nname = \"Card 1\"\n"));

    let mut metadata = Metadata::new();
    metadata.columns.push(ColumnConfig::new("name"));

    let result = save_metadata(&mock_config.config(), "/test.toml", &metadata);
    assert!(result.is_ok());

    let saved = mock_config.last_written_content().unwrap();
    assert!(!saved.contains("width = 100"), "Default width should not be serialized in:\n{saved}");
    assert!(
        !saved.contains("alignment = \"left\""),
        "Default alignment should not be serialized in:\n{saved}",
    );
    assert!(!saved.contains("wrap = false"), "Default wrap should not be serialized in:\n{saved}");
    assert!(
        !saved.contains("frozen = false"),
        "Default frozen should not be serialized in:\n{saved}"
    );
}
