use crate::error::error_types::TvError;

/// Resolves the actual TOML array-of-tables key name for a given table name
/// hint, returning it as an owned `String`.
///
/// Tries the following in order:
/// 1. Exact match on `table_name`
/// 2. Hyphen-to-underscore variant (e.g. `rendered-cards` → `rendered_cards`)
/// 3. First array-of-tables key in the document (fallback for files where the
///    TOML key doesn't match the filename, e.g. `rendered-cards.toml` containing
///    `[[cards]]`)
pub fn resolve_key_name(
    doc: &toml_edit::DocumentMut,
    table_name: &str,
    file_path: &str,
    operation: &str,
) -> Result<String, TvError> {
    find_key(doc, table_name).ok_or_else(|| {
        tracing::error!(
            component = "tv.toml",
            file_path = %file_path,
            table_name = %table_name,
            error = "Table not found or not an array of tables",
            operation = %operation,
        );
        TvError::TableNotFound { table_name: table_name.to_string() }
    })
}

fn find_key(doc: &toml_edit::DocumentMut, table_name: &str) -> Option<String> {
    if doc.get(table_name).is_some() {
        return Some(table_name.to_string());
    }
    let resolved_name = table_name.replace('-', "_");
    if doc.get(&resolved_name).is_some() {
        return Some(resolved_name);
    }
    // Fall back to the sole array-of-tables key in the document. Only
    // applies when there is exactly one such key, avoiding ambiguity.
    let mut array_keys = doc.as_table().iter().filter(|(_, item)| item.is_array_of_tables());
    let first = array_keys.next();
    if array_keys.next().is_none() {
        return first.map(|(key, _)| key.to_string());
    }
    None
}
