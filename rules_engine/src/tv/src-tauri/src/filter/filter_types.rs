use serde::{Deserialize, Serialize};

use crate::sort::sort_types::CellValue;
use crate::toml::metadata_types::{ColumnFilter, FilterCondition};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FilterState {
    pub filters: Vec<ColumnFilter>,
    pub active: bool,
}

impl FilterState {
    pub fn new(filters: Vec<ColumnFilter>, active: bool) -> Self {
        Self { filters, active }
    }

    pub fn active(filters: Vec<ColumnFilter>) -> Self {
        Self::new(filters, true)
    }
}

/// Returns true if the given cell value matches the filter condition.
pub fn matches_condition(cell_value: &CellValue, condition: &FilterCondition) -> bool {
    match condition {
        FilterCondition::Contains(substring) => match cell_value {
            CellValue::String(s) => s.to_lowercase().contains(&substring.to_lowercase()),
            CellValue::Integer(i) => i.to_string().contains(substring),
            CellValue::Float(f) => f.to_string().contains(substring),
            CellValue::Boolean(b) => b.to_string().contains(&substring.to_lowercase()),
            CellValue::Null => false,
        },
        FilterCondition::Equals(json_val) => {
            let filter_cell = CellValue::from_json(json_val);
            match (cell_value, &filter_cell) {
                (CellValue::String(a), CellValue::String(b)) => a.to_lowercase() == b.to_lowercase(),
                (CellValue::Integer(a), CellValue::Integer(b)) => a == b,
                (CellValue::Float(a), CellValue::Float(b)) => (a - b).abs() < f64::EPSILON,
                (CellValue::Integer(a), CellValue::Float(b)) => ((*a as f64) - b).abs() < f64::EPSILON,
                (CellValue::Float(a), CellValue::Integer(b)) => (a - (*b as f64)).abs() < f64::EPSILON,
                (CellValue::Boolean(a), CellValue::Boolean(b)) => a == b,
                (CellValue::Null, CellValue::Null) => true,
                _ => false,
            }
        }
        FilterCondition::Range { min, max } => {
            let numeric = match cell_value {
                CellValue::Integer(i) => Some(*i as f64),
                CellValue::Float(f) => Some(*f),
                _ => None,
            };
            let Some(n) = numeric else {
                return false;
            };
            let above_min = min.is_none_or(|m| n >= m);
            let below_max = max.is_none_or(|m| n <= m);
            above_min && below_max
        }
        FilterCondition::Boolean(expected) => match cell_value {
            CellValue::Boolean(b) => b == expected,
            _ => false,
        },
    }
}

/// Represents the active filter state for a single column.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnFilterState {
    pub column: String,
    pub condition: FilterConditionState,
}

impl ColumnFilterState {
    pub fn contains(column: impl Into<String>, substring: impl Into<String>) -> Self {
        Self { column: column.into(), condition: FilterConditionState::Contains(substring.into()) }
    }

    pub fn equals(column: impl Into<String>, value: serde_json::Value) -> Self {
        Self { column: column.into(), condition: FilterConditionState::Equals(value) }
    }

    pub fn values(column: impl Into<String>, values: Vec<serde_json::Value>) -> Self {
        Self { column: column.into(), condition: FilterConditionState::Values(values) }
    }
}

/// Filter condition for runtime filter state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterConditionState {
    /// Text contains substring (case-insensitive).
    Contains(String),
    /// Exact value match.
    Equals(serde_json::Value),
    /// Numeric range.
    Range { min: Option<f64>, max: Option<f64> },
    /// Boolean value match.
    Boolean(bool),
    /// Set of allowed values (Univer filter sends these).
    Values(Vec<serde_json::Value>),
}
