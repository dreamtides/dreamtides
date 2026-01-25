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

impl ValidationRule {
    pub fn column(&self) -> &str {
        match self {
            ValidationRule::Enum { column, .. } => column,
            ValidationRule::Range { column, .. } => column,
            ValidationRule::Pattern { column, .. } => column,
            ValidationRule::Required { column, .. } => column,
            ValidationRule::Type { column, .. } => column,
        }
    }

    pub fn custom_message(&self) -> Option<&str> {
        match self {
            ValidationRule::Enum { message, .. } => message.as_deref(),
            ValidationRule::Range { message, .. } => message.as_deref(),
            ValidationRule::Pattern { message, .. } => message.as_deref(),
            ValidationRule::Required { message, .. } => message.as_deref(),
            ValidationRule::Type { message, .. } => message.as_deref(),
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
