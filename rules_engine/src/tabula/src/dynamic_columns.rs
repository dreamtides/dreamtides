use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Represents a set of columns whose names are not known at compile time.
///
/// Tabula will understand each column as its own entry in the master
/// spreadsheet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicColumns<K, V>
where
    K: Ord,
{
    pub columns: BTreeMap<K, V>,
}
