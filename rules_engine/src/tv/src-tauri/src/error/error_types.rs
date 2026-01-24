use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum TvError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Permission denied reading file: {path}")]
    PermissionDenied { path: String },

    #[error("TOML parse error at line {line:?}: {message}")]
    TomlParseError { line: Option<usize>, message: String },

    #[error("Invalid UTF-8 content in file: {path}")]
    InvalidUtf8 { path: String },

    #[error("Table '{table_name}' not found in TOML file")]
    TableNotFound { table_name: String },

    #[error("'{table_name}' is not an array of tables")]
    NotAnArrayOfTables { table_name: String },

    #[error("Failed to write file {path}: {message}")]
    WriteError { path: String, message: String },
}

impl Serialize for TvError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
