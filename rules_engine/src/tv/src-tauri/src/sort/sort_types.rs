use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

impl SortDirection {
    pub fn toggle(self) -> Self {
        match self {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SortState {
    pub column: String,
    pub direction: SortDirection,
}

impl SortState {
    pub fn new(column: String, direction: SortDirection) -> Self {
        Self { column, direction }
    }

    pub fn ascending(column: String) -> Self {
        Self::new(column, SortDirection::Ascending)
    }

    pub fn descending(column: String) -> Self {
        Self::new(column, SortDirection::Descending)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CellValue {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
}

impl CellValue {
    pub fn from_json(value: &serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => CellValue::Null,
            serde_json::Value::Bool(b) => CellValue::Boolean(*b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    CellValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    CellValue::Float(f)
                } else {
                    CellValue::String(n.to_string())
                }
            }
            serde_json::Value::String(s) => CellValue::String(s.clone()),
            serde_json::Value::Array(arr) => CellValue::String(format!("{arr:?}")),
            serde_json::Value::Object(obj) => CellValue::String(format!("{obj:?}")),
        }
    }

    fn type_order(&self) -> u8 {
        match self {
            CellValue::Null => 0,
            CellValue::Boolean(_) => 1,
            CellValue::Integer(_) => 2,
            CellValue::Float(_) => 2,
            CellValue::String(_) => 3,
        }
    }
}

impl PartialOrd for CellValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp_values(other))
    }
}

impl CellValue {
    pub fn cmp_values(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;

        match (self, other) {
            (CellValue::Null, CellValue::Null) => Ordering::Equal,
            (CellValue::Null, _) => Ordering::Greater,
            (_, CellValue::Null) => Ordering::Less,
            (CellValue::Boolean(a), CellValue::Boolean(b)) => a.cmp(b),
            (CellValue::Integer(a), CellValue::Integer(b)) => a.cmp(b),
            (CellValue::Float(a), CellValue::Float(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
            (CellValue::Integer(a), CellValue::Float(b)) => {
                (*a as f64).partial_cmp(b).unwrap_or(Ordering::Equal)
            }
            (CellValue::Float(a), CellValue::Integer(b)) => {
                a.partial_cmp(&(*b as f64)).unwrap_or(Ordering::Equal)
            }
            (CellValue::String(a), CellValue::String(b)) => a.to_lowercase().cmp(&b.to_lowercase()),
            _ => self.type_order().cmp(&other.type_order()),
        }
    }
}
