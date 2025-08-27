#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ErrorCode {
    FluentFormattingError,
    FluentParserError,
    FluentAddResourceError,
    FluentMissingMessage,
    FluentMissingValue,
    DatabaseError,
    IOError,
    JsonError,
    TabulaBuildError,
    MutexLockError,
    NotInitializedError,
    AlreadyInitializedWithDifferentPath,
    InvalidUnsignedInteger,
    InvalidCardSubtype,
    AbilitiesNotPresent,
    AbilityParsingError,
}

impl ErrorCode {
    /// Returns a 2-4 character code for this error code. Each code must be
    /// unique.
    pub fn shortname(&self) -> &'static str {
        match self {
            ErrorCode::FluentFormattingError => "FFE",
            ErrorCode::FluentParserError => "FPE",
            ErrorCode::FluentAddResourceError => "FAR",
            ErrorCode::FluentMissingMessage => "FMM",
            ErrorCode::FluentMissingValue => "FMV",
            ErrorCode::DatabaseError => "DBE",
            ErrorCode::IOError => "IOE",
            ErrorCode::JsonError => "JSE",
            ErrorCode::TabulaBuildError => "TBE",
            ErrorCode::MutexLockError => "MLE",
            ErrorCode::NotInitializedError => "NIE",
            ErrorCode::AlreadyInitializedWithDifferentPath => "AID",
            ErrorCode::InvalidUnsignedInteger => "IUI",
            ErrorCode::InvalidCardSubtype => "ICS",
            ErrorCode::AbilitiesNotPresent => "ANP",
            ErrorCode::AbilityParsingError => "APE",
        }
    }
}

/// Message describing why an error happened during initialization.
#[derive(Debug, Clone)]
pub struct InitializationError {
    pub code: ErrorCode,
    pub name: String,
    pub details: Option<String>,
    pub tabula_sheet: Option<String>,
    pub tabula_row: Option<usize>,
    pub tabula_column: Option<String>,
    pub tabula_id: Option<String>,
}

impl InitializationError {
    pub fn with_name(code: ErrorCode, name: impl Into<String>) -> Self {
        Self {
            code,
            name: name.into(),
            details: None,
            tabula_sheet: None,
            tabula_row: None,
            tabula_column: None,
            tabula_id: None,
        }
    }

    pub fn with_details(
        code: ErrorCode,
        name: impl Into<String>,
        details: impl Into<String>,
    ) -> Self {
        Self {
            code,
            name: name.into(),
            details: Some(details.into()),
            tabula_sheet: None,
            tabula_row: None,
            tabula_column: None,
            tabula_id: None,
        }
    }

    pub fn format(&self) -> String {
        let header = format!("ERR{}: {}", self.code.shortname(), self.name);
        let mut lines: Vec<String> = vec![header];
        if self.tabula_sheet.is_some() || self.tabula_row.is_some() || self.tabula_column.is_some()
        {
            let mut parts: Vec<String> = Vec::new();
            if let Some(sheet) = &self.tabula_sheet {
                parts.push(format!("Sheet: {sheet}"));
            }
            if let Some(row) = self.tabula_row {
                // Add 1 for 0-indexed, 1 for header row
                parts.push(format!("Row: {}", row + 2));
            }
            if let Some(col) = &self.tabula_column {
                parts.push(format!("Column: {col}"));
            }
            if let Some(id) = &self.tabula_id {
                parts.push(format!("ID: {id}"));
            }
            if !parts.is_empty() {
                lines.push(parts.join(" "));
            }
        }
        if let Some(details) = &self.details {
            lines.push(details.clone());
        }
        lines.join("\n ")
    }
}
