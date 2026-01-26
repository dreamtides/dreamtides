use tv_lib::toml::metadata_types::{
    Alignment, ColumnConfig, DerivedColumnConfig, FormatStyle, Metadata, SortConfig, TableStyle,
    CURRENT_SCHEMA_VERSION,
};

#[test]
fn test_metadata_defaults() {
    let metadata = Metadata::new();
    assert_eq!(metadata.schema_version, CURRENT_SCHEMA_VERSION);
    assert!(metadata.columns.is_empty());
    assert!(metadata.is_version_compatible());
}

#[test]
fn test_metadata_version_compatibility() {
    let mut metadata = Metadata::new();
    assert!(metadata.is_version_compatible());

    metadata.schema_version = CURRENT_SCHEMA_VERSION + 1;
    assert!(!metadata.is_version_compatible());
}

#[test]
fn test_column_config_builder() {
    let config = ColumnConfig::new("name")
        .with_width(200)
        .with_alignment(Alignment::Center)
        .with_frozen(true);

    assert_eq!(config.key, "name");
    assert_eq!(config.width, 200);
    assert_eq!(config.alignment, Alignment::Center);
    assert!(config.frozen);
}

#[test]
fn test_derived_column_config() {
    let config = DerivedColumnConfig::new("Preview", "rules_preview")
        .with_inputs(vec!["rules_text".to_string(), "variables".to_string()])
        .with_width(400)
        .with_position(5);

    assert_eq!(config.name, "Preview");
    assert_eq!(config.function, "rules_preview");
    assert_eq!(config.inputs, vec!["rules_text", "variables"]);
    assert_eq!(config.width, 400);
    assert_eq!(config.position, Some(5));
}

#[test]
fn test_format_style_builder() {
    let style = FormatStyle::new().with_background_color("#FFD700").with_bold(true);

    assert_eq!(style.background_color, Some("#FFD700".to_string()));
    assert_eq!(style.bold, Some(true));
    assert_eq!(style.font_color, None);
}

#[test]
fn test_sort_config() {
    let asc = SortConfig::ascending("name");
    assert_eq!(asc.column, "name");
    assert!(asc.ascending);

    let desc = SortConfig::descending("id");
    assert_eq!(desc.column, "id");
    assert!(!desc.ascending);
}

#[test]
fn test_table_style_defaults() {
    let style = TableStyle::default();
    assert!(style.show_row_stripes);
    assert!(!style.show_column_stripes);
    assert!(style.header_bold);
    assert!(style.color_scheme.is_none());
}

#[test]
fn test_metadata_serialization_roundtrip() {
    let mut metadata = Metadata::new();
    metadata.columns.push(ColumnConfig::new("id").with_width(300).with_frozen(true));
    metadata.table_style = Some(TableStyle::new().with_color_scheme("blue_light"));
    metadata.sort = Some(SortConfig::ascending("name"));

    let serialized = serde_json::to_string(&metadata).unwrap();
    let deserialized: Metadata = serde_json::from_str(&serialized).unwrap();

    assert_eq!(metadata, deserialized);
}

#[test]
fn test_get_column_config() {
    let mut metadata = Metadata::new();
    metadata.columns.push(ColumnConfig::new("id"));
    metadata.columns.push(ColumnConfig::new("name").with_width(200));

    assert!(metadata.get_column_config("id").is_some());
    assert_eq!(metadata.get_column_config("name").unwrap().width, 200);
    assert!(metadata.get_column_config("missing").is_none());
}
