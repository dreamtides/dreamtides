use uuid::Uuid;

/// Ensures all rows with content have a UUID in the "id" column.
/// Returns true if any UUIDs were generated.
pub fn ensure_uuids(array: &mut toml_edit::ArrayOfTables, headers: &[String]) -> bool {
    let Some(id_key) = find_id_column(headers) else {
        return false;
    };

    let mut generated = false;
    for i in 0..array.len() {
        let Some(table) = array.get_mut(i) else {
            continue;
        };

        if !id_is_empty(table, id_key) {
            continue;
        }

        if row_has_other_content(table, id_key) {
            let uuid = Uuid::new_v4().to_string();
            tracing::info!(
                component = "tv.uuid",
                row_index = i,
                uuid = %uuid,
                "Generated UUID for row"
            );
            table.insert(id_key, toml_edit::value(&uuid));
            generated = true;
        }
    }

    generated
}

fn find_id_column(headers: &[String]) -> Option<&str> {
    headers.iter().find(|h| h.eq_ignore_ascii_case("id")).map(|s| s.as_str())
}

fn id_is_empty(table: &toml_edit::Table, id_key: &str) -> bool {
    match table.get(id_key) {
        None => true,
        Some(item) => match item.as_str() {
            Some(s) => s.trim().is_empty(),
            None => false,
        },
    }
}

fn row_has_other_content(table: &toml_edit::Table, id_key: &str) -> bool {
    for (key, item) in table.iter() {
        if key.eq_ignore_ascii_case(id_key) {
            continue;
        }
        if let Some(s) = item.as_str() {
            if s.trim().is_empty() {
                continue;
            }
        }
        return true;
    }
    false
}
