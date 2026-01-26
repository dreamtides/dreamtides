use std::path::Path;

use toml_edit::{value, Array, ArrayOfTables, DocumentMut, InlineTable, Item, Table, Value};

use crate::error::error_types::{map_io_error_for_read, TvError};
use crate::toml::metadata_types::{
    Alignment, AppSettings, ColumnConfig, ColumnFilter, ConditionalFormatRule, DerivedColumnConfig,
    FilterCondition, FilterConfig, FormatCondition, FormatStyle, Metadata, RowConfig, RowHeight,
    ScrollPosition, SortConfig, TableStyle,
};
use crate::traits::{AtomicWriteError, FileSystem, RealFileSystem};
use crate::validation::validation_rules::{ValidationRule, ValueType};

/// Updates only the sort configuration in the metadata section of a TOML file.
pub fn update_sort_config(file_path: &str, sort_config: Option<&SortConfig>) -> Result<(), TvError> {
    update_sort_config_with_fs(&RealFileSystem, file_path, sort_config)
}

/// Updates only the sort configuration using the provided filesystem.
pub fn update_sort_config_with_fs(
    fs: &dyn FileSystem,
    file_path: &str,
    sort_config: Option<&SortConfig>,
) -> Result<(), TvError> {
    let content = fs.read_to_string(Path::new(file_path)).map_err(|e| {
        tracing::error!(
            component = "tv.toml.metadata",
            file_path = %file_path,
            error = %e,
            "Read failed during sort config update"
        );
        map_io_error_for_read(&e, file_path)
    })?;

    let mut doc: DocumentMut = content.parse().map_err(|e: toml_edit::TomlError| {
        tracing::error!(
            component = "tv.toml.metadata",
            file_path = %file_path,
            error = %e,
            "TOML parse failed during sort config update"
        );
        TvError::TomlParseError { path: file_path.to_string(), line: None, message: e.to_string() }
    })?;

    let metadata_table = doc.entry("metadata").or_insert_with(|| {
        let mut table = Table::new();
        table.insert("schema_version", value(1i64));
        Item::Table(table)
    });

    if let Some(table) = metadata_table.as_table_mut() {
        match sort_config {
            Some(config) => {
                table.insert("sort", Item::Table(serialize_sort_config(config)));
            }
            None => {
                table.remove("sort");
            }
        }
    }

    fs.write_atomic(Path::new(file_path), &doc.to_string()).map_err(|e| {
        map_atomic_write_error(e, file_path)
    })?;

    tracing::info!(
        component = "tv.toml.metadata",
        file_path = %file_path,
        has_sort = sort_config.is_some(),
        "Sort config updated in metadata"
    );

    Ok(())
}

/// Serializes metadata and writes it to the TOML file, preserving document structure.
pub fn save_metadata(file_path: &str, metadata: &Metadata) -> Result<(), TvError> {
    save_metadata_with_fs(&RealFileSystem, file_path, metadata)
}

/// Serializes metadata using the provided filesystem.
pub fn save_metadata_with_fs(
    fs: &dyn FileSystem,
    file_path: &str,
    metadata: &Metadata,
) -> Result<(), TvError> {
    let content = fs.read_to_string(Path::new(file_path)).map_err(|e| {
        tracing::error!(
            component = "tv.toml.metadata",
            file_path = %file_path,
            error = %e,
            "Read failed during metadata save"
        );
        map_io_error_for_read(&e, file_path)
    })?;

    let mut doc: DocumentMut = content.parse().map_err(|e: toml_edit::TomlError| {
        tracing::error!(
            component = "tv.toml.metadata",
            file_path = %file_path,
            error = %e,
            "TOML parse failed during metadata save"
        );
        TvError::TomlParseError { path: file_path.to_string(), line: None, message: e.to_string() }
    })?;

    let existing_unknown_fields = extract_unknown_fields(&doc);

    let metadata_table = serialize_metadata_to_table(metadata, &existing_unknown_fields);
    doc["metadata"] = Item::Table(metadata_table);

    fs.write_atomic(Path::new(file_path), &doc.to_string()).map_err(|e| {
        map_atomic_write_error(e, file_path)
    })?;

    tracing::info!(
        component = "tv.toml.metadata",
        file_path = %file_path,
        "Metadata saved"
    );

    Ok(())
}

/// Extracts unknown fields from existing metadata for forward compatibility.
fn extract_unknown_fields(doc: &DocumentMut) -> Table {
    let Some(metadata) = doc.get("metadata") else {
        return Table::new();
    };
    let Some(metadata_table) = metadata.as_table() else {
        return Table::new();
    };

    let known_fields = [
        "schema_version",
        "columns",
        "derived_columns",
        "validation_rules",
        "conditional_formatting",
        "table_style",
        "sort",
        "filter",
        "rows",
        "app_settings",
    ];

    let mut unknown = Table::new();
    for (key, val) in metadata_table.iter() {
        if !known_fields.contains(&key) {
            unknown.insert(key, val.clone());
        }
    }
    unknown
}

/// Serializes Metadata struct to a toml_edit Table.
fn serialize_metadata_to_table(metadata: &Metadata, unknown_fields: &Table) -> Table {
    let mut table = Table::new();

    table.insert("schema_version", value(metadata.schema_version as i64));

    if !metadata.columns.is_empty() {
        table.insert("columns", Item::ArrayOfTables(serialize_columns(&metadata.columns)));
    }

    if !metadata.derived_columns.is_empty() {
        table.insert(
            "derived_columns",
            Item::ArrayOfTables(serialize_derived_columns(&metadata.derived_columns)),
        );
    }

    if !metadata.validation_rules.is_empty() {
        table.insert(
            "validation_rules",
            Item::ArrayOfTables(serialize_validation_rules(&metadata.validation_rules)),
        );
    }

    if !metadata.conditional_formatting.is_empty() {
        table.insert(
            "conditional_formatting",
            Item::ArrayOfTables(serialize_conditional_formatting(&metadata.conditional_formatting)),
        );
    }

    if let Some(ref style) = metadata.table_style {
        table.insert("table_style", Item::Table(serialize_table_style(style)));
    }

    if let Some(ref sort) = metadata.sort {
        table.insert("sort", Item::Table(serialize_sort_config(sort)));
    }

    if let Some(ref filter) = metadata.filter {
        table.insert("filter", Item::Table(serialize_filter_config(filter)));
    }

    if let Some(ref rows) = metadata.rows {
        table.insert("rows", Item::Table(serialize_row_config(rows)));
    }

    if let Some(ref app) = metadata.app_settings {
        table.insert("app_settings", Item::Table(serialize_app_settings(app)));
    }

    for (key, val) in unknown_fields.iter() {
        table.insert(key, val.clone());
    }

    table
}

fn serialize_columns(columns: &[ColumnConfig]) -> ArrayOfTables {
    let mut array = ArrayOfTables::new();
    for col in columns {
        let mut table = Table::new();
        table.insert("key", value(&col.key));

        if col.width != 100 {
            table.insert("width", value(col.width as i64));
        }

        if col.alignment != Alignment::Left {
            table.insert("alignment", value(alignment_to_str(&col.alignment)));
        }

        if col.wrap {
            table.insert("wrap", value(true));
        }

        if col.frozen {
            table.insert("frozen", value(true));
        }

        if col.hidden {
            table.insert("hidden", value(true));
        }

        if let Some(ref format) = col.format {
            table.insert("format", value(format));
        }

        array.push(table);
    }
    array
}

fn alignment_to_str(alignment: &Alignment) -> &'static str {
    match alignment {
        Alignment::Left => "left",
        Alignment::Center => "center",
        Alignment::Right => "right",
    }
}

fn serialize_derived_columns(columns: &[DerivedColumnConfig]) -> ArrayOfTables {
    let mut array = ArrayOfTables::new();
    for col in columns {
        let mut table = Table::new();
        table.insert("name", value(&col.name));
        table.insert("function", value(&col.function));

        if let Some(pos) = col.position {
            table.insert("position", value(pos as i64));
        }

        if col.width != 100 {
            table.insert("width", value(col.width as i64));
        }

        if !col.inputs.is_empty() {
            table.insert("inputs", Item::Value(serialize_string_array(&col.inputs)));
        }

        array.push(table);
    }
    array
}

fn serialize_string_array(strings: &[String]) -> Value {
    let mut arr = Array::new();
    for s in strings {
        arr.push(s.as_str());
    }
    Value::Array(arr)
}

fn serialize_validation_rules(rules: &[ValidationRule]) -> ArrayOfTables {
    let mut array = ArrayOfTables::new();
    for rule in rules {
        array.push(serialize_single_validation_rule(rule));
    }
    array
}

fn serialize_single_validation_rule(rule: &ValidationRule) -> Table {
    let mut table = Table::new();
    match rule {
        ValidationRule::Enum { column, allowed_values, message } => {
            table.insert("column", value(column));
            table.insert("type", value("enum"));
            table.insert("enum", Item::Value(serialize_string_array(allowed_values)));
            if let Some(msg) = message {
                table.insert("message", value(msg));
            }
        }
        ValidationRule::Range { column, min, max, message } => {
            table.insert("column", value(column));
            table.insert("type", value("range"));
            if let Some(m) = min {
                table.insert("min", value(*m));
            }
            if let Some(m) = max {
                table.insert("max", value(*m));
            }
            if let Some(msg) = message {
                table.insert("message", value(msg));
            }
        }
        ValidationRule::Pattern { column, pattern, message } => {
            table.insert("column", value(column));
            table.insert("type", value("pattern"));
            table.insert("pattern", value(pattern));
            if let Some(msg) = message {
                table.insert("message", value(msg));
            }
        }
        ValidationRule::Required { column, message } => {
            table.insert("column", value(column));
            table.insert("type", value("required"));
            if let Some(msg) = message {
                table.insert("message", value(msg));
            }
        }
        ValidationRule::Type { column, value_type, message } => {
            table.insert("column", value(column));
            table.insert("type", value(value_type_to_str(value_type)));
            if let Some(msg) = message {
                table.insert("message", value(msg));
            }
        }
    }
    table
}

fn value_type_to_str(vt: &ValueType) -> &'static str {
    match vt {
        ValueType::String => "string",
        ValueType::Integer => "integer",
        ValueType::Float => "float",
        ValueType::Boolean => "boolean",
    }
}

fn serialize_conditional_formatting(rules: &[ConditionalFormatRule]) -> ArrayOfTables {
    let mut array = ArrayOfTables::new();
    for rule in rules {
        let mut table = Table::new();
        table.insert("column", value(&rule.column));
        table.insert("condition", Item::Value(serialize_format_condition(&rule.condition)));
        table.insert("style", Item::Value(serialize_format_style(&rule.style)));
        array.push(table);
    }
    array
}

fn serialize_format_condition(condition: &FormatCondition) -> Value {
    let mut inline = InlineTable::new();
    match condition {
        FormatCondition::Equals(val) => {
            inline.insert("equals", json_value_to_toml_value(val));
        }
        FormatCondition::Contains(s) => {
            inline.insert("contains", Value::String(toml_edit::Formatted::new(s.clone())));
        }
        FormatCondition::GreaterThan(n) => {
            inline.insert("greater_than", Value::Float(toml_edit::Formatted::new(*n)));
        }
        FormatCondition::LessThan(n) => {
            inline.insert("less_than", Value::Float(toml_edit::Formatted::new(*n)));
        }
        FormatCondition::IsEmpty => {
            inline.insert("is_empty", Value::Boolean(toml_edit::Formatted::new(true)));
        }
        FormatCondition::NotEmpty => {
            inline.insert("not_empty", Value::Boolean(toml_edit::Formatted::new(true)));
        }
        FormatCondition::Matches(s) => {
            inline.insert("matches", Value::String(toml_edit::Formatted::new(s.clone())));
        }
    }
    Value::InlineTable(inline)
}

fn json_value_to_toml_value(val: &serde_json::Value) -> Value {
    match val {
        serde_json::Value::Null => Value::String(toml_edit::Formatted::new(String::new())),
        serde_json::Value::Bool(b) => Value::Boolean(toml_edit::Formatted::new(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Integer(toml_edit::Formatted::new(i))
            } else if let Some(f) = n.as_f64() {
                Value::Float(toml_edit::Formatted::new(f))
            } else {
                Value::String(toml_edit::Formatted::new(n.to_string()))
            }
        }
        serde_json::Value::String(s) => Value::String(toml_edit::Formatted::new(s.clone())),
        serde_json::Value::Array(arr) => {
            let mut toml_arr = Array::new();
            for item in arr {
                toml_arr.push(json_value_to_toml_value(item));
            }
            Value::Array(toml_arr)
        }
        serde_json::Value::Object(obj) => {
            let mut inline = InlineTable::new();
            for (k, v) in obj {
                inline.insert(k, json_value_to_toml_value(v));
            }
            Value::InlineTable(inline)
        }
    }
}

fn serialize_format_style(style: &FormatStyle) -> Value {
    let mut inline = InlineTable::new();
    if let Some(ref bg) = style.background_color {
        inline.insert("background_color", Value::String(toml_edit::Formatted::new(bg.clone())));
    }
    if let Some(ref fc) = style.font_color {
        inline.insert("font_color", Value::String(toml_edit::Formatted::new(fc.clone())));
    }
    if let Some(b) = style.bold {
        inline.insert("bold", Value::Boolean(toml_edit::Formatted::new(b)));
    }
    if let Some(i) = style.italic {
        inline.insert("italic", Value::Boolean(toml_edit::Formatted::new(i)));
    }
    if let Some(u) = style.underline {
        inline.insert("underline", Value::Boolean(toml_edit::Formatted::new(u)));
    }
    Value::InlineTable(inline)
}

fn serialize_table_style(style: &TableStyle) -> Table {
    let mut table = Table::new();
    if let Some(ref scheme) = style.color_scheme {
        table.insert("color_scheme", value(scheme));
    }
    if !style.show_row_stripes {
        table.insert("show_row_stripes", value(false));
    }
    if style.show_column_stripes {
        table.insert("show_column_stripes", value(true));
    }
    if !style.header_bold {
        table.insert("header_bold", value(false));
    }
    if let Some(ref bg) = style.header_background {
        table.insert("header_background", value(bg));
    }
    table
}

fn serialize_sort_config(sort: &SortConfig) -> Table {
    let mut table = Table::new();
    table.insert("column", value(&sort.column));
    if !sort.ascending {
        table.insert("ascending", value(false));
    }
    table
}

fn serialize_filter_config(filter: &FilterConfig) -> Table {
    let mut table = Table::new();
    if !filter.filters.is_empty() {
        table.insert(
            "filters",
            Item::ArrayOfTables(serialize_column_filters(&filter.filters)),
        );
    }
    if filter.active {
        table.insert("active", value(true));
    }
    table
}

fn serialize_column_filters(filters: &[ColumnFilter]) -> ArrayOfTables {
    let mut array = ArrayOfTables::new();
    for f in filters {
        let mut table = Table::new();
        table.insert("column", value(&f.column));
        table.insert("condition", Item::Value(serialize_filter_condition(&f.condition)));
        array.push(table);
    }
    array
}

fn serialize_filter_condition(condition: &FilterCondition) -> Value {
    let mut inline = InlineTable::new();
    match condition {
        FilterCondition::Contains(s) => {
            inline.insert("contains", Value::String(toml_edit::Formatted::new(s.clone())));
        }
        FilterCondition::Equals(val) => {
            inline.insert("equals", json_value_to_toml_value(val));
        }
        FilterCondition::Range { min, max } => {
            if let Some(m) = min {
                inline.insert("min", Value::Float(toml_edit::Formatted::new(*m)));
            }
            if let Some(m) = max {
                inline.insert("max", Value::Float(toml_edit::Formatted::new(*m)));
            }
        }
        FilterCondition::Boolean(b) => {
            inline.insert("boolean", Value::Boolean(toml_edit::Formatted::new(*b)));
        }
    }
    Value::InlineTable(inline)
}

fn serialize_row_config(rows: &RowConfig) -> Table {
    let mut table = Table::new();
    if !rows.heights.is_empty() {
        table.insert("heights", Item::ArrayOfTables(serialize_row_heights(&rows.heights)));
    }
    if !rows.hidden.is_empty() {
        table.insert("hidden", Item::Value(serialize_usize_array(&rows.hidden)));
    }
    table
}

fn serialize_row_heights(heights: &[RowHeight]) -> ArrayOfTables {
    let mut array = ArrayOfTables::new();
    for h in heights {
        let mut table = Table::new();
        table.insert("row", value(h.row as i64));
        table.insert("height", value(h.height as i64));
        array.push(table);
    }
    array
}

fn serialize_usize_array(values: &[usize]) -> Value {
    let mut arr = Array::new();
    for v in values {
        arr.push(*v as i64);
    }
    Value::Array(arr)
}

fn serialize_app_settings(app: &AppSettings) -> Table {
    let mut table = Table::new();
    if let Some(ref cell) = app.last_selected_cell {
        table.insert("last_selected_cell", value(cell));
    }
    if let Some(ref scroll) = app.scroll_position {
        table.insert("scroll_position", Item::Value(serialize_scroll_position(scroll)));
    }
    if (app.zoom_level - 1.0).abs() > f64::EPSILON {
        table.insert("zoom_level", value(app.zoom_level));
    }
    table
}

fn serialize_scroll_position(scroll: &ScrollPosition) -> Value {
    let mut inline = InlineTable::new();
    inline.insert("row", Value::Integer(toml_edit::Formatted::new(scroll.row as i64)));
    inline.insert("column", Value::Integer(toml_edit::Formatted::new(scroll.column as i64)));
    Value::InlineTable(inline)
}

fn map_atomic_write_error(error: AtomicWriteError, file_path: &str) -> TvError {
    match error {
        AtomicWriteError::TempFileCreate(e) => {
            tracing::error!(
                component = "tv.toml.metadata",
                file_path = %file_path,
                error = %e,
                "Failed to create temp file for atomic write"
            );
            crate::error::error_types::map_io_error_for_write(&e, file_path)
        }
        AtomicWriteError::Write(e) => {
            tracing::error!(
                component = "tv.toml.metadata",
                file_path = %file_path,
                error = %e,
                "Failed to write content to temp file"
            );
            crate::error::error_types::map_io_error_for_write(&e, file_path)
        }
        AtomicWriteError::Sync(e) => {
            tracing::error!(
                component = "tv.toml.metadata",
                file_path = %file_path,
                error = %e,
                "Failed to sync temp file"
            );
            crate::error::error_types::map_io_error_for_write(&e, file_path)
        }
        AtomicWriteError::Rename { source, temp_path } => {
            tracing::error!(
                component = "tv.toml.metadata",
                file_path = %file_path,
                temp_path = %temp_path,
                error = %source,
                "Atomic rename failed"
            );
            TvError::AtomicRenameFailed {
                temp_path,
                target_path: file_path.to_string(),
                message: source.to_string(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::io;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;

    use super::*;

    struct MockFileSystem {
        contents: Mutex<HashMap<PathBuf, String>>,
    }

    impl MockFileSystem {
        fn new() -> Self {
            Self { contents: Mutex::new(HashMap::new()) }
        }

        fn set_content(&self, path: &str, content: &str) {
            self.contents.lock().unwrap().insert(PathBuf::from(path), content.to_string());
        }

        fn get_content(&self, path: &str) -> Option<String> {
            self.contents.lock().unwrap().get(&PathBuf::from(path)).cloned()
        }
    }

    impl FileSystem for MockFileSystem {
        fn read_to_string(&self, path: &Path) -> io::Result<String> {
            self.contents
                .lock()
                .unwrap()
                .get(path)
                .cloned()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "File not found"))
        }

        fn write(&self, path: &Path, content: &str) -> io::Result<()> {
            self.contents.lock().unwrap().insert(path.to_path_buf(), content.to_string());
            Ok(())
        }

        fn write_atomic(&self, path: &Path, content: &str) -> Result<(), AtomicWriteError> {
            self.contents.lock().unwrap().insert(path.to_path_buf(), content.to_string());
            Ok(())
        }

        fn exists(&self, path: &Path) -> bool {
            self.contents.lock().unwrap().contains_key(path)
        }

        fn read_dir_temp_files(
            &self,
            _dir: &Path,
            _prefix: &str,
        ) -> io::Result<Vec<std::path::PathBuf>> {
            Ok(Vec::new())
        }

        fn remove_file(&self, path: &Path) -> io::Result<()> {
            self.contents.lock().unwrap().remove(path);
            Ok(())
        }
    }

    #[test]
    fn test_save_metadata_creates_section_if_missing() {
        let fs = MockFileSystem::new();
        let toml_content = r#"[[cards]]
name = "Card 1"
"#;
        fs.set_content("/test.toml", toml_content);

        let mut metadata = Metadata::new();
        metadata.columns.push(ColumnConfig::new("name").with_width(200));

        let result = save_metadata_with_fs(&fs, "/test.toml", &metadata);
        assert!(result.is_ok());

        let saved = fs.get_content("/test.toml").unwrap();
        assert!(saved.contains("[metadata]"), "Expected [metadata] section in:\n{}", saved);
        assert!(saved.contains("schema_version = 1"), "Expected schema_version in:\n{}", saved);
        assert!(saved.contains("[[metadata.columns]]"), "Expected columns in:\n{}", saved);
    }

    #[test]
    fn test_save_metadata_updates_existing_section() {
        let fs = MockFileSystem::new();
        let toml_content = r#"[[cards]]
name = "Card 1"

[metadata]
schema_version = 1

[[metadata.columns]]
key = "id"
width = 100
"#;
        fs.set_content("/test.toml", toml_content);

        let mut metadata = Metadata::new();
        metadata.columns.push(ColumnConfig::new("name").with_width(300));

        let result = save_metadata_with_fs(&fs, "/test.toml", &metadata);
        assert!(result.is_ok());

        let saved = fs.get_content("/test.toml").unwrap();
        assert!(saved.contains("width = 300"), "Expected width = 300 in:\n{}", saved);
        assert!(saved.contains("key = \"name\""), "Expected key = \"name\" in:\n{}", saved);
    }

    #[test]
    fn test_save_metadata_preserves_unknown_fields() {
        let fs = MockFileSystem::new();
        let toml_content = r#"[[cards]]
name = "Card 1"

[metadata]
schema_version = 1
unknown_future_field = "preserve me"
another_unknown = 42
"#;
        fs.set_content("/test.toml", toml_content);

        let metadata = Metadata::new();

        let result = save_metadata_with_fs(&fs, "/test.toml", &metadata);
        assert!(result.is_ok());

        let saved = fs.get_content("/test.toml").unwrap();
        assert!(
            saved.contains("unknown_future_field = \"preserve me\""),
            "Expected unknown_future_field preserved in:\n{}",
            saved
        );
        assert!(saved.contains("another_unknown = 42"), "Expected another_unknown preserved in:\n{}", saved);
    }

    #[test]
    fn test_save_metadata_round_trip_columns() {
        let fs = MockFileSystem::new();
        fs.set_content("/test.toml", "[[cards]]\nname = \"Card 1\"\n");

        let mut metadata = Metadata::new();
        metadata.columns.push(
            ColumnConfig::new("id")
                .with_width(300)
                .with_alignment(Alignment::Center)
                .with_frozen(true),
        );
        metadata.columns.push(ColumnConfig::new("name").with_width(200));

        let result = save_metadata_with_fs(&fs, "/test.toml", &metadata);
        assert!(result.is_ok());

        let saved = fs.get_content("/test.toml").unwrap();
        assert!(saved.contains("key = \"id\""), "Expected id column in:\n{}", saved);
        assert!(saved.contains("width = 300"), "Expected width 300 in:\n{}", saved);
        assert!(saved.contains("alignment = \"center\""), "Expected alignment in:\n{}", saved);
        assert!(saved.contains("frozen = true"), "Expected frozen in:\n{}", saved);
        assert!(saved.contains("key = \"name\""), "Expected name column in:\n{}", saved);
        assert!(saved.contains("width = 200"), "Expected width 200 in:\n{}", saved);
    }

    #[test]
    fn test_save_metadata_validation_rules() {
        let fs = MockFileSystem::new();
        fs.set_content("/test.toml", "[[cards]]\nname = \"Card 1\"\n");

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

        let result = save_metadata_with_fs(&fs, "/test.toml", &metadata);
        assert!(result.is_ok());

        let saved = fs.get_content("/test.toml").unwrap();
        assert!(
            saved.contains("[[metadata.validation_rules]]"),
            "Expected validation_rules in:\n{}",
            saved
        );
        assert!(saved.contains("type = \"enum\""), "Expected enum type in:\n{}", saved);
        assert!(saved.contains("type = \"range\""), "Expected range type in:\n{}", saved);
    }

    #[test]
    fn test_save_metadata_sort_config() {
        let fs = MockFileSystem::new();
        fs.set_content("/test.toml", "[[cards]]\nname = \"Card 1\"\n");

        let mut metadata = Metadata::new();
        metadata.sort = Some(SortConfig::descending("name"));

        let result = save_metadata_with_fs(&fs, "/test.toml", &metadata);
        assert!(result.is_ok());

        let saved = fs.get_content("/test.toml").unwrap();
        assert!(saved.contains("[metadata.sort]"), "Expected sort section in:\n{}", saved);
        assert!(saved.contains("column = \"name\""), "Expected column = \"name\" in:\n{}", saved);
        assert!(saved.contains("ascending = false"), "Expected ascending = false in:\n{}", saved);
    }

    #[test]
    fn test_save_metadata_table_style() {
        let fs = MockFileSystem::new();
        fs.set_content("/test.toml", "[[cards]]\nname = \"Card 1\"\n");

        let mut metadata = Metadata::new();
        metadata.table_style = Some(TableStyle::new().with_color_scheme("blue_light"));

        let result = save_metadata_with_fs(&fs, "/test.toml", &metadata);
        assert!(result.is_ok());

        let saved = fs.get_content("/test.toml").unwrap();
        assert!(saved.contains("[metadata.table_style]"), "Expected table_style section in:\n{}", saved);
        assert!(saved.contains("color_scheme = \"blue_light\""), "Expected color_scheme in:\n{}", saved);
    }

    #[test]
    fn test_save_metadata_preserves_card_data() {
        let fs = MockFileSystem::new();
        let toml_content = r#"[[cards]]
name = "Card 1"
id = "abc-123"

[[cards]]
name = "Card 2"
id = "def-456"
"#;
        fs.set_content("/test.toml", toml_content);

        let metadata = Metadata::new();

        let result = save_metadata_with_fs(&fs, "/test.toml", &metadata);
        assert!(result.is_ok());

        let saved = fs.get_content("/test.toml").unwrap();
        assert!(saved.contains("name = \"Card 1\""), "Expected Card 1 preserved in:\n{}", saved);
        assert!(saved.contains("name = \"Card 2\""), "Expected Card 2 preserved in:\n{}", saved);
        assert!(saved.contains("id = \"abc-123\""), "Expected abc-123 preserved in:\n{}", saved);
        assert!(saved.contains("id = \"def-456\""), "Expected def-456 preserved in:\n{}", saved);
    }

    #[test]
    fn test_save_metadata_derived_columns() {
        let fs = MockFileSystem::new();
        fs.set_content("/test.toml", "[[cards]]\nname = \"Card 1\"\n");

        let mut metadata = Metadata::new();
        metadata.derived_columns.push(
            DerivedColumnConfig::new("Preview", "rules_preview")
                .with_inputs(vec!["rules_text".to_string(), "variables".to_string()])
                .with_width(400)
                .with_position(5),
        );

        let result = save_metadata_with_fs(&fs, "/test.toml", &metadata);
        assert!(result.is_ok());

        let saved = fs.get_content("/test.toml").unwrap();
        assert!(
            saved.contains("[[metadata.derived_columns]]"),
            "Expected derived_columns in:\n{}",
            saved
        );
        assert!(saved.contains("name = \"Preview\""), "Expected name in:\n{}", saved);
        assert!(saved.contains("function = \"rules_preview\""), "Expected function in:\n{}", saved);
        assert!(saved.contains("position = 5"), "Expected position in:\n{}", saved);
        assert!(saved.contains("width = 400"), "Expected width in:\n{}", saved);
    }

    #[test]
    fn test_save_metadata_conditional_formatting() {
        let fs = MockFileSystem::new();
        fs.set_content("/test.toml", "[[cards]]\nname = \"Card 1\"\n");

        let mut metadata = Metadata::new();
        metadata.conditional_formatting.push(ConditionalFormatRule::new(
            "rarity",
            FormatCondition::Equals(serde_json::json!("Rare")),
            FormatStyle::new().with_background_color("#FFD700").with_bold(true),
        ));

        let result = save_metadata_with_fs(&fs, "/test.toml", &metadata);
        assert!(result.is_ok());

        let saved = fs.get_content("/test.toml").unwrap();
        assert!(
            saved.contains("[[metadata.conditional_formatting]]"),
            "Expected conditional_formatting in:\n{}",
            saved
        );
        assert!(saved.contains("column = \"rarity\""), "Expected column in:\n{}", saved);
    }

    #[test]
    fn test_save_metadata_app_settings() {
        let fs = MockFileSystem::new();
        fs.set_content("/test.toml", "[[cards]]\nname = \"Card 1\"\n");

        let mut metadata = Metadata::new();
        metadata.app_settings = Some(AppSettings {
            last_selected_cell: Some("B5".to_string()),
            scroll_position: Some(ScrollPosition::new(10, 2)),
            zoom_level: 1.5,
        });

        let result = save_metadata_with_fs(&fs, "/test.toml", &metadata);
        assert!(result.is_ok());

        let saved = fs.get_content("/test.toml").unwrap();
        assert!(
            saved.contains("[metadata.app_settings]"),
            "Expected app_settings section in:\n{}",
            saved
        );
        assert!(
            saved.contains("last_selected_cell = \"B5\""),
            "Expected last_selected_cell in:\n{}",
            saved
        );
        assert!(saved.contains("zoom_level = 1.5"), "Expected zoom_level in:\n{}", saved);
    }

    #[test]
    fn test_default_values_not_serialized() {
        let fs = MockFileSystem::new();
        fs.set_content("/test.toml", "[[cards]]\nname = \"Card 1\"\n");

        let mut metadata = Metadata::new();
        metadata.columns.push(ColumnConfig::new("name"));

        let result = save_metadata_with_fs(&fs, "/test.toml", &metadata);
        assert!(result.is_ok());

        let saved = fs.get_content("/test.toml").unwrap();
        assert!(!saved.contains("width = 100"), "Default width should not be serialized in:\n{}", saved);
        assert!(
            !saved.contains("alignment = \"left\""),
            "Default alignment should not be serialized in:\n{}",
            saved
        );
        assert!(!saved.contains("wrap = false"), "Default wrap should not be serialized in:\n{}", saved);
        assert!(!saved.contains("frozen = false"), "Default frozen should not be serialized in:\n{}", saved);
    }
}
