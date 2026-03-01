/// Parses an array column key like `"resonance[0]"` into `("resonance", 0)`.
///
/// Returns `None` if the key does not match the `key[N]` pattern.
pub fn parse_array_column_key(key: &str) -> Option<(&str, usize)> {
    let bracket_start = key.find('[')?;
    if !key.ends_with(']') {
        return None;
    }
    let base = &key[..bracket_start];
    let index_str = &key[bracket_start + 1..key.len() - 1];
    let index = index_str.parse::<usize>().ok()?;
    Some((base, index))
}

/// Produces an array column key like `"resonance[0]"` from a base key and index.
pub fn make_array_column_key(base: &str, index: usize) -> String {
    format!("{base}[{index}]")
}

/// Describes a group of headers that belong to the same expanded array.
pub struct ArrayGroup {
    /// The original TOML key name (e.g., `"resonance"`).
    pub base_key: String,
    /// Pairs of `(header_index, array_element_index)` sorted by array index.
    pub entries: Vec<(usize, usize)>,
}

/// Result of grouping headers into regular and array-expanded columns.
pub struct ArrayHeaderGroups {
    /// Indices of headers that are not part of any array expansion.
    pub regular_indices: Vec<usize>,
    /// Groups of headers belonging to expanded arrays.
    pub array_groups: Vec<ArrayGroup>,
    array_header_set: Vec<bool>,
}

impl ArrayHeaderGroups {
    /// Returns `true` if the header at `idx` is part of an array group.
    pub fn is_array_index(&self, idx: usize) -> bool {
        self.array_header_set.get(idx).copied().unwrap_or(false)
    }
}

/// Analyzes headers to separate regular columns from expanded array columns.
///
/// Array columns are identified by the `key[N]` naming pattern and grouped
/// by their base key.
pub fn group_array_headers(headers: &[String]) -> ArrayHeaderGroups {
    let mut regular_indices = Vec::new();
    let mut groups: Vec<ArrayGroup> = Vec::new();
    let mut array_header_set = vec![false; headers.len()];

    for (idx, header) in headers.iter().enumerate() {
        if let Some((base, array_idx)) = parse_array_column_key(header) {
            array_header_set[idx] = true;
            if let Some(group) = groups.iter_mut().find(|g| g.base_key == base) {
                group.entries.push((idx, array_idx));
            } else {
                groups.push(ArrayGroup {
                    base_key: base.to_string(),
                    entries: vec![(idx, array_idx)],
                });
            }
        } else {
            regular_indices.push(idx);
        }
    }

    for group in &mut groups {
        group.entries.sort_by_key(|&(_, array_idx)| array_idx);
    }

    ArrayHeaderGroups { regular_indices, array_groups: groups, array_header_set }
}
