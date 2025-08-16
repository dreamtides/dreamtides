#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ErrorCode {
    FluentFormattingError,
    FluentParserError,
    FluentAddResourceError,
    FluentMissingMessage,
    FluentMissingValue,
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
        }
    }
}

/// Message describing why an error happened during initialization.
pub struct InitializationError {
    pub code: ErrorCode,
    pub name: String,
    pub details: Option<String>,
    pub tabula_sheet: Option<String>,
    pub tabula_row: Option<usize>,
    pub tabula_column: Option<String>,
}

impl InitializationError {
    pub fn with_name(code: ErrorCode, name: String) -> Self {
        Self {
            code,
            name,
            details: None,
            tabula_sheet: None,
            tabula_row: None,
            tabula_column: None,
        }
    }

    pub fn with_details(code: ErrorCode, name: String, details: String) -> Self {
        Self {
            code,
            name,
            details: Some(details),
            tabula_sheet: None,
            tabula_row: None,
            tabula_column: None,
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
                parts.push(format!("Row: {row}"));
            }
            if let Some(col) = &self.tabula_column {
                parts.push(format!("Column: {col}"));
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
