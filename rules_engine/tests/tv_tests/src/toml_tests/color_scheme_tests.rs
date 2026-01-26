use tv_lib::toml::color_schemes::{available_schemes, resolve_color_scheme};
use tv_lib::toml::metadata_parser::parse_table_style_from_content;
use tv_lib::toml::metadata_serializer::save_metadata_with_fs;
use tv_lib::toml::metadata_types::{Metadata, TableStyle};

use crate::test_utils::mock_filesystem::MockFileSystem;

#[test]
fn test_resolve_blue_light_scheme() {
    let palette = resolve_color_scheme("blue_light").expect("Expected blue_light scheme");
    assert_eq!(palette.header_background, "#4472C4");
    assert_eq!(palette.header_font_color, "#FFFFFF");
    assert_eq!(palette.row_even_background, "#D6E4F0");
    assert_eq!(palette.row_odd_background, "#FFFFFF");
    assert_eq!(palette.accent_color, "#4472C4");
}

#[test]
fn test_resolve_green_medium_scheme() {
    let palette = resolve_color_scheme("green_medium").expect("Expected green_medium scheme");
    assert_eq!(palette.header_background, "#70AD47");
    assert_eq!(palette.row_even_background, "#C6E0B4");
    assert_eq!(palette.row_odd_background, "#E2EFDA");
}

#[test]
fn test_resolve_gray_classic_scheme() {
    let palette = resolve_color_scheme("gray_classic").expect("Expected gray_classic scheme");
    assert_eq!(palette.header_background, "#A5A5A5");
    assert_eq!(palette.row_even_background, "#EDEDED");
    assert_eq!(palette.row_odd_background, "#FFFFFF");
}

#[test]
fn test_resolve_unknown_scheme_returns_none() {
    assert!(resolve_color_scheme("nonexistent").is_none());
    assert!(resolve_color_scheme("").is_none());
}

#[test]
fn test_all_available_schemes_resolve() {
    for name in available_schemes() {
        let palette = resolve_color_scheme(name);
        assert!(palette.is_some(), "Scheme '{name}' should resolve to a palette");
        let p = palette.unwrap();
        assert!(
            p.header_background.starts_with('#'),
            "Scheme '{name}' header_background should be hex"
        );
        assert!(
            p.header_font_color.starts_with('#'),
            "Scheme '{name}' header_font_color should be hex"
        );
        assert!(
            p.row_even_background.starts_with('#'),
            "Scheme '{name}' row_even_background should be hex"
        );
        assert!(
            p.row_odd_background.starts_with('#'),
            "Scheme '{name}' row_odd_background should be hex"
        );
        assert!(p.accent_color.starts_with('#'), "Scheme '{name}' accent_color should be hex");
    }
}

#[test]
fn test_available_schemes_not_empty() {
    let schemes = available_schemes();
    assert!(!schemes.is_empty());
    assert!(schemes.contains(&"blue_light"));
    assert!(schemes.contains(&"green_medium"));
    assert!(schemes.contains(&"gray_classic"));
}

#[test]
fn test_parse_table_style_from_content() {
    let content = r#"
[[cards]]
name = "Card 1"

[metadata]
schema_version = 1

[metadata.table_style]
color_scheme = "blue_light"
show_row_stripes = true
header_bold = true
"#;

    let result = parse_table_style_from_content(content, "test.toml").unwrap();
    let style = result.expect("Expected table style");
    assert_eq!(style.color_scheme, Some("blue_light".to_string()));
    assert!(style.show_row_stripes);
    assert!(style.header_bold);
    assert!(!style.show_column_stripes);
    assert!(style.header_background.is_none());
}

#[test]
fn test_parse_table_style_with_header_background_override() {
    let content = r##"
[[cards]]
name = "Card 1"

[metadata]
schema_version = 1

[metadata.table_style]
color_scheme = "blue_light"
header_background = "#FF0000"
header_bold = false
"##;

    let result = parse_table_style_from_content(content, "test.toml").unwrap();
    let style = result.unwrap();
    assert_eq!(style.color_scheme, Some("blue_light".to_string()));
    assert_eq!(style.header_background, Some("#FF0000".to_string()));
    assert!(!style.header_bold);
}

#[test]
fn test_parse_table_style_with_column_stripes() {
    let content = r#"
[[cards]]
name = "Card 1"

[metadata]
schema_version = 1

[metadata.table_style]
show_row_stripes = false
show_column_stripes = true
"#;

    let result = parse_table_style_from_content(content, "test.toml").unwrap();
    let style = result.unwrap();
    assert!(!style.show_row_stripes);
    assert!(style.show_column_stripes);
    assert!(style.color_scheme.is_none());
}

#[test]
fn test_parse_table_style_missing_metadata() {
    let content = r#"
[[cards]]
name = "Card 1"
"#;

    let result = parse_table_style_from_content(content, "test.toml").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_parse_table_style_missing_table_style() {
    let content = r#"
[[cards]]
name = "Card 1"

[metadata]
schema_version = 1
"#;

    let result = parse_table_style_from_content(content, "test.toml").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_parse_table_style_defaults_for_missing_fields() {
    let content = r#"
[[cards]]
name = "Card 1"

[metadata]
schema_version = 1

[metadata.table_style]
color_scheme = "gray_classic"
"#;

    let result = parse_table_style_from_content(content, "test.toml").unwrap();
    let style = result.unwrap();
    assert_eq!(style.color_scheme, Some("gray_classic".to_string()));
    assert!(style.show_row_stripes);
    assert!(!style.show_column_stripes);
    assert!(style.header_bold);
    assert!(style.header_background.is_none());
}

#[test]
fn test_save_and_parse_table_style_roundtrip() {
    let fs = MockFileSystem::with_read_and_write("[[cards]]\nname = \"Card 1\"\n");

    let mut metadata = Metadata::new();
    metadata.table_style = Some(TableStyle {
        color_scheme: Some("green_light".to_string()),
        show_row_stripes: true,
        show_column_stripes: false,
        header_bold: true,
        header_background: Some("#123456".to_string()),
    });

    save_metadata_with_fs(&fs, "/test.toml", &metadata).unwrap();

    let saved = fs.last_written_content().unwrap();
    let parsed = parse_table_style_from_content(&saved, "/test.toml").unwrap();
    let style = parsed.expect("Expected table style after roundtrip");

    assert_eq!(style.color_scheme, Some("green_light".to_string()));
    assert!(style.show_row_stripes);
    assert!(!style.show_column_stripes);
    assert!(style.header_bold);
    assert_eq!(style.header_background, Some("#123456".to_string()));
}

#[test]
fn test_save_table_style_only_non_defaults() {
    let fs = MockFileSystem::with_read_and_write("[[cards]]\nname = \"Card 1\"\n");

    let mut metadata = Metadata::new();
    metadata.table_style = Some(TableStyle::new().with_color_scheme("blue_light"));

    save_metadata_with_fs(&fs, "/test.toml", &metadata).unwrap();

    let saved = fs.last_written_content().unwrap();
    assert!(saved.contains("color_scheme = \"blue_light\""), "Expected color_scheme in:\n{saved}");
    assert!(
        !saved.contains("show_row_stripes"),
        "Default show_row_stripes should not be serialized in:\n{saved}",
    );
    assert!(
        !saved.contains("show_column_stripes"),
        "Default show_column_stripes should not be serialized in:\n{saved}"
    );
    assert!(
        !saved.contains("header_bold"),
        "Default header_bold should not be serialized in:\n{saved}"
    );
}

#[test]
fn test_color_palette_serialization_roundtrip() {
    let palette = resolve_color_scheme("blue_light").unwrap();
    let serialized = serde_json::to_string(&palette).unwrap();
    let deserialized: tv_lib::toml::color_schemes::ColorPalette =
        serde_json::from_str(&serialized).unwrap();

    assert_eq!(palette, deserialized);
}
