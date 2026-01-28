use crate::error::error_types::TvError;
use crate::toml::metadata;
use crate::traits::RealFileSystem;
use crate::validation::validation_rules::ValidationRule;

#[tauri::command]
pub fn get_validation_rules(file_path: String) -> Result<Vec<ValidationRule>, TvError> {
    tracing::debug!(
        component = "tv.commands.validation",
        file_path = %file_path,
        "Loading validation rules"
    );

    let rules = metadata::parse_validation_rules_with_fs(&RealFileSystem, &file_path)?;

    tracing::debug!(
        component = "tv.commands.validation",
        file_path = %file_path,
        rule_count = rules.len(),
        "Validation rules loaded"
    );

    Ok(rules)
}

#[tauri::command]
pub fn get_enum_validation_rules(file_path: String) -> Result<Vec<EnumValidationInfo>, TvError> {
    tracing::debug!(
        component = "tv.commands.validation",
        file_path = %file_path,
        "Loading enum validation rules for dropdown support"
    );

    let rules = metadata::parse_validation_rules_with_fs(&RealFileSystem, &file_path)?;

    let enum_rules: Vec<EnumValidationInfo> = rules
        .into_iter()
        .filter_map(|rule| {
            if let ValidationRule::Enum { column, allowed_values, .. } = rule {
                Some(EnumValidationInfo { column, allowed_values })
            } else {
                None
            }
        })
        .collect();

    tracing::debug!(
        component = "tv.commands.validation",
        file_path = %file_path,
        enum_rule_count = enum_rules.len(),
        "Enum validation rules loaded for dropdown support"
    );

    Ok(enum_rules)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnumValidationInfo {
    pub column: String,
    pub allowed_values: Vec<String>,
}
