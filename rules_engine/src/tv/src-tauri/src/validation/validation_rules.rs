use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ValidationRule {
    Enum {
        column: String,
        #[serde(rename = "enum")]
        allowed_values: Vec<String>,
        #[serde(default)]
        message: Option<String>,
    },
    Range {
        column: String,
        #[serde(default)]
        min: Option<f64>,
        #[serde(default)]
        max: Option<f64>,
        #[serde(default)]
        message: Option<String>,
    },
    Pattern {
        column: String,
        pattern: String,
        #[serde(default)]
        message: Option<String>,
    },
    Required {
        column: String,
        #[serde(default)]
        message: Option<String>,
    },
    Type {
        column: String,
        value_type: ValueType,
        #[serde(default)]
        message: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ValueType {
    String,
    Integer,
    Float,
    Boolean,
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueType::String => write!(f, "string"),
            ValueType::Integer => write!(f, "integer"),
            ValueType::Float => write!(f, "float"),
            ValueType::Boolean => write!(f, "boolean"),
        }
    }
}

impl fmt::Display for ValidationRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationRule::Enum { column, allowed_values, .. } => {
                write!(f, "enum({column}: [{}])", allowed_values.join(", "))
            }
            ValidationRule::Range { column, min, max, .. } => match (min, max) {
                (Some(lo), Some(hi)) => write!(f, "range({column}: {lo}..{hi})"),
                (Some(lo), None) => write!(f, "range({column}: {lo}..)"),
                (None, Some(hi)) => write!(f, "range({column}: ..{hi})"),
                (None, None) => write!(f, "range({column})"),
            },
            ValidationRule::Pattern { column, pattern, .. } => {
                write!(f, "pattern({column}: /{pattern}/)")
            }
            ValidationRule::Required { column, .. } => write!(f, "required({column})"),
            ValidationRule::Type { column, value_type, .. } => {
                write!(f, "type({column}: {value_type})")
            }
        }
    }
}

impl ValidationRule {
    /// Returns the column this rule applies to.
    pub fn column(&self) -> &str {
        match self {
            ValidationRule::Enum { column, .. } => column,
            ValidationRule::Range { column, .. } => column,
            ValidationRule::Pattern { column, .. } => column,
            ValidationRule::Required { column, .. } => column,
            ValidationRule::Type { column, .. } => column,
        }
    }

    /// Returns the user-specified custom error message, if any.
    pub fn custom_message(&self) -> Option<&str> {
        match self {
            ValidationRule::Enum { message, .. } => message.as_deref(),
            ValidationRule::Range { message, .. } => message.as_deref(),
            ValidationRule::Pattern { message, .. } => message.as_deref(),
            ValidationRule::Required { message, .. } => message.as_deref(),
            ValidationRule::Type { message, .. } => message.as_deref(),
        }
    }

    /// Returns the rule type name as a static string (e.g. "enum", "range").
    pub fn rule_type_name(&self) -> &'static str {
        match self {
            ValidationRule::Enum { .. } => "enum",
            ValidationRule::Range { .. } => "range",
            ValidationRule::Pattern { .. } => "pattern",
            ValidationRule::Required { .. } => "required",
            ValidationRule::Type { .. } => "type",
        }
    }

    /// Returns a human-readable description of this rule's constraint.
    pub fn describe(&self) -> String {
        match self {
            ValidationRule::Enum { allowed_values, .. } => {
                format!("Must be one of: {}", allowed_values.join(", "))
            }
            ValidationRule::Range { min, max, .. } => match (min, max) {
                (Some(lo), Some(hi)) => format!("Must be between {lo} and {hi}"),
                (Some(lo), None) => format!("Must be at least {lo}"),
                (None, Some(hi)) => format!("Must be at most {hi}"),
                (None, None) => "No range constraint".to_string(),
            },
            ValidationRule::Pattern { pattern, .. } => {
                format!("Must match pattern: {pattern}")
            }
            ValidationRule::Required { .. } => "This field is required".to_string(),
            ValidationRule::Type { value_type, .. } => {
                format!("Must be a {value_type} value")
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub error_message: Option<String>,
    pub rule_type: String,
    pub column: String,
}

impl ValidationResult {
    pub fn success(column: &str, rule_type: &str) -> Self {
        Self { valid: true, error_message: None, rule_type: rule_type.to_string(), column: column.to_string() }
    }

    pub fn failure(column: &str, rule_type: &str, message: String) -> Self {
        Self {
            valid: false,
            error_message: Some(message),
            rule_type: rule_type.to_string(),
            column: column.to_string(),
        }
    }
}
