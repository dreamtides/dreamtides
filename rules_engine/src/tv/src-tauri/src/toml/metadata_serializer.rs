use std::path::Path;

use toml_edit::{value, Array, ArrayOfTables, DocumentMut, InlineTable, Item, Table, Value};

use crate::error::error_types::{map_io_error_for_read, TvError};
use crate::toml::metadata_types::{
    Alignment, AppSettings, ColumnConfig, ColumnFilter, ConditionalFormatRule, DerivedColumnConfig,
    FilterCondition, FilterConfig, FormatCondition, FormatStyle, Metadata, RowConfig, RowHeight,
    ScrollPosition, SortConfig, TableStyle,
};
use crate::traits::{AtomicWriteError, TvConfig};
use crate::validation::validation_rules::{ValidationRule, ValueType};

/// Updates only the sort configuration in the metadata section of a TOML file.
pub fn update_sort_config(
    config: &TvConfig,
    file_path: &str,
    sort_config: Option<&SortConfig>,
) -> Result<(), TvError> {
    let content = config.fs().read_to_string(Path::new(file_path)).map_err(|e| {
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

    config.fs().write_atomic(Path::new(file_path), &doc.to_string()).map_err(|e| {
        map_atomic_write_error(e, file_path)
    })?;

    tracing::debug!(
        component = "tv.toml.metadata",
        file_path = %file_path,
        has_sort = sort_config.is_some(),
        "Sort config updated in metadata"
    );

    Ok(())
}

/// Updates only the filter configuration in the metadata section of a TOML file.
pub fn update_filter_config(
    config: &TvConfig,
    file_path: &str,
    filter_config: Option<&FilterConfig>,
) -> Result<(), TvError> {
    let content = config.fs().read_to_string(Path::new(file_path)).map_err(|e| {
        tracing::error!(
            component = "tv.toml.metadata",
            file_path = %file_path,
            error = %e,
            "Read failed during filter config update"
        );
        map_io_error_for_read(&e, file_path)
    })?;

    let mut doc: DocumentMut = content.parse().map_err(|e: toml_edit::TomlError| {
        tracing::error!(
            component = "tv.toml.metadata",
            file_path = %file_path,
            error = %e,
            "TOML parse failed during filter config update"
        );
        TvError::TomlParseError { path: file_path.to_string(), line: None, message: e.to_string() }
    })?;

    let metadata_table = doc.entry("metadata").or_insert_with(|| {
        let mut table = Table::new();
        table.insert("schema_version", value(1i64));
        Item::Table(table)
    });

    if let Some(table) = metadata_table.as_table_mut() {
        match filter_config {
            Some(config) => {
                table.insert("filter", Item::Table(serialize_filter_config(config)));
            }
            None => {
                table.remove("filter");
            }
        }
    }

    config.fs().write_atomic(Path::new(file_path), &doc.to_string()).map_err(|e| {
        map_atomic_write_error(e, file_path)
    })?;

    tracing::debug!(
        component = "tv.toml.metadata",
        file_path = %file_path,
        has_filter = filter_config.is_some(),
        "Filter config updated in metadata"
    );

    Ok(())
}

/// Updates the width of a single column in the metadata.columns array.
pub fn update_column_width(
    config: &TvConfig,
    file_path: &str,
    column_key: &str,
    width: u32,
) -> Result<(), TvError> {
    let content = config.fs().read_to_string(Path::new(file_path)).map_err(|e| {
        tracing::error!(
            component = "tv.toml.metadata",
            file_path = %file_path,
            error = %e,
            "Read failed during column width update"
        );
        map_io_error_for_read(&e, file_path)
    })?;

    let mut doc: DocumentMut = content.parse().map_err(|e: toml_edit::TomlError| {
        tracing::error!(
            component = "tv.toml.metadata",
            file_path = %file_path,
            error = %e,
            "TOML parse failed during column width update"
        );
        TvError::TomlParseError { path: file_path.to_string(), line: None, message: e.to_string() }
    })?;

    let metadata_table = doc.entry("metadata").or_insert_with(|| {
        let mut table = Table::new();
        table.insert("schema_version", value(1i64));
        Item::Table(table)
    });

    if let Some(table) = metadata_table.as_table_mut() {
        let columns = table
            .entry("columns")
            .or_insert(Item::ArrayOfTables(ArrayOfTables::new()));

        if let Some(array) = columns.as_array_of_tables_mut() {
            // Find existing entry for this column key
            let mut found = false;
            for entry in array.iter_mut() {
                let is_match = entry.get("key").and_then(|v| v.as_str()) == Some(column_key);
                if is_match {
                    found = true;
                    if width == 100 {
                        entry.remove("width");
                        // Remove the entry entirely if only the key field remains
                        if entry.len() == 1 && entry.contains_key("key") {
                            // Mark for removal — handled below
                        }
                    } else {
                        entry.insert("width", value(width as i64));
                    }
                    break;
                }
            }

            if !found && width != 100 {
                let mut new_entry = Table::new();
                new_entry.insert("key", value(column_key));
                new_entry.insert("width", value(width as i64));
                array.push(new_entry);
            }

            // Remove entries that only have the key field (all defaults)
            remove_default_only_columns(array);
        }
    }

    config.fs().write_atomic(Path::new(file_path), &doc.to_string()).map_err(|e| {
        map_atomic_write_error(e, file_path)
    })?;

    tracing::debug!(
        component = "tv.toml.metadata",
        file_path = %file_path,
        column_key = %column_key,
        width = width,
        "Column width updated in metadata"
    );

    Ok(())
}

/// Updates the width of a single derived column in the metadata.derived_columns array.
pub fn update_derived_column_width(
    config: &TvConfig,
    file_path: &str,
    column_name: &str,
    width: u32,
) -> Result<(), TvError> {
    let content = config.fs().read_to_string(Path::new(file_path)).map_err(|e| {
        tracing::error!(
            component = "tv.toml.metadata",
            file_path = %file_path,
            error = %e,
            "Read failed during derived column width update"
        );
        map_io_error_for_read(&e, file_path)
    })?;

    let mut doc: DocumentMut = content.parse().map_err(|e: toml_edit::TomlError| {
        tracing::error!(
            component = "tv.toml.metadata",
            file_path = %file_path,
            error = %e,
            "TOML parse failed during derived column width update"
        );
        TvError::TomlParseError { path: file_path.to_string(), line: None, message: e.to_string() }
    })?;

    let metadata_table = doc.entry("metadata").or_insert_with(|| {
        let mut table = Table::new();
        table.insert("schema_version", value(1i64));
        Item::Table(table)
    });

    if let Some(table) = metadata_table.as_table_mut() {
        let Some(derived_columns) = table.get_mut("derived_columns") else {
            tracing::debug!(
                component = "tv.toml.metadata",
                file_path = %file_path,
                column_name = %column_name,
                "No derived_columns section found, skipping width update"
            );
            return Ok(());
        };

        if let Some(array) = derived_columns.as_array_of_tables_mut() {
            for entry in array.iter_mut() {
                let is_match =
                    entry.get("name").and_then(|v| v.as_str()) == Some(column_name);
                if is_match {
                    if width == 100 {
                        entry.remove("width");
                    } else {
                        entry.insert("width", value(width as i64));
                    }
                    break;
                }
            }
        }
    }

    config.fs().write_atomic(Path::new(file_path), &doc.to_string()).map_err(|e| {
        map_atomic_write_error(e, file_path)
    })?;

    tracing::debug!(
        component = "tv.toml.metadata",
        file_path = %file_path,
        column_name = %column_name,
        width = width,
        "Derived column width updated in metadata"
    );

    Ok(())
}

/// Removes column entries from the array that only contain the key field (all other fields are defaults).
fn remove_default_only_columns(array: &mut ArrayOfTables) {
    // Collect indices to remove (entries with only the "key" field)
    let mut indices_to_remove = Vec::new();
    for (i, entry) in array.iter().enumerate() {
        if entry.len() == 1 && entry.contains_key("key") {
            indices_to_remove.push(i);
        }
    }

    // Remove in reverse order to preserve indices
    for i in indices_to_remove.into_iter().rev() {
        array.remove(i);
    }
}

/// Serializes metadata and writes it to the TOML file, preserving document structure.
pub fn save_metadata(config: &TvConfig, file_path: &str, metadata: &Metadata) -> Result<(), TvError> {
    let content = config.fs().read_to_string(Path::new(file_path)).map_err(|e| {
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

    config.fs().write_atomic(Path::new(file_path), &doc.to_string()).map_err(|e| {
        map_atomic_write_error(e, file_path)
    })?;

    tracing::debug!(
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
        FilterCondition::Values(vals) => {
            let mut arr = Array::new();
            for v in vals {
                arr.push(json_value_to_toml_value(v));
            }
            inline.insert("values", Value::Array(arr));
        }
    }
    Value::InlineTable(inline)
}

fn serialize_row_config(rows: &RowConfig) -> Table {
    let mut table = Table::new();
    if let Some(h) = rows.header_height {
        table.insert("header_height", value(h as i64));
    }
    if let Some(h) = rows.default_height {
        table.insert("default_height", value(h as i64));
    }
    if let Some(f) = rows.frozen_rows {
        table.insert("frozen_rows", value(f as i64));
    }
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
