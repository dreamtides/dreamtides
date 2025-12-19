pub const PROTOCOL_VERSION: &str = "TABULA/1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Request {
    pub request_id: String,
    pub workbook_path: String,
    pub workbook_mtime: i64,
    pub workbook_size: u64,
    pub changed_range: Option<ChangedRange>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangedRange {
    pub sheet: String,
    pub range: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Response {
    pub request_id: Option<String>,
    pub status: ResponseStatus,
    pub retry_after_ms: Option<u64>,
    pub warnings: Vec<String>,
    pub changes: Vec<Change>,
    pub changeset_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Span {
    pub start: u32,
    pub length: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResponseStatus {
    Ok,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HorizontalAlignment {
    Left,
    Center,
    Right,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Change {
    SetBold { sheet: String, cell: String, bold: bool },
    SetFontColorSpans { sheet: String, cell: String, rgb: String, spans: Vec<Span> },
    SetValue { sheet: String, cell: String, value: String },
    ClearValue { sheet: String, cell: String },
    SetFontColor { sheet: String, cell: String, rgb: String },
    SetFontSize { sheet: String, cell: String, points: f32 },
    SetFillColor { sheet: String, cell: String, rgb: String },
    SetNumberFormat { sheet: String, cell: String, format: String },
    SetHorizontalAlignment { sheet: String, cell: String, alignment: HorizontalAlignment },
    SetItalic { sheet: String, cell: String, italic: bool },
    SetUnderline { sheet: String, cell: String, underline: bool },
    SetFontNameSpans { sheet: String, cell: String, font_name: String, spans: Vec<Span> },
    SetFontSizeSpans { sheet: String, cell: String, points: f32, spans: Vec<Span> },
    SetSubscriptSpans { sheet: String, cell: String, subscript: bool, spans: Vec<Span> },
}
