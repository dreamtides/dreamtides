use std::path::PathBuf;

use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur when loading or building Tabula data.
///
/// All variants include file path information for error reporting.
#[derive(Debug, Clone, Error)]
pub enum TabulaError {
    /// Failed to parse a TOML file.
    #[error("TOML parse error in {file}: {message}")]
    TomlParse { file: PathBuf, line: Option<usize>, message: String },

    /// A required field was missing from a card definition.
    #[error("Missing required field '{field}' in {file}")]
    MissingField { file: PathBuf, card_id: Option<Uuid>, field: &'static str },

    /// An unexpected field was present in a card definition.
    #[error("Unexpected field '{field}' in {file}")]
    UnexpectedField { file: PathBuf, card_id: Option<Uuid>, field: String },

    /// Failed to parse a card's ability text.
    #[error("Ability parse error for card '{card_name}' in {file}: {message}")]
    AbilityParse { file: PathBuf, card_name: String, message: String },

    /// A field value was invalid or could not be parsed.
    #[error("Invalid value for field '{field}' in {file}: {message}")]
    InvalidField { file: PathBuf, card_id: Option<Uuid>, field: &'static str, message: String },
}
