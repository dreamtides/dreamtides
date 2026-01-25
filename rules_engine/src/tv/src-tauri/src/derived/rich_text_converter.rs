use serde::{Deserialize, Serialize};

use crate::derived::derived_types::StyledSpan;

/// Converts StyledSpan vector to Univer-compatible ICellData rich text format.
pub fn styled_spans_to_univer_rich_text(spans: &[StyledSpan]) -> UniverRichText {
    let runs: Vec<TextRun> = spans.iter().map(styled_span_to_text_run).collect();
    UniverRichText { p: vec![Paragraph { ts: runs }] }
}

fn styled_span_to_text_run(span: &StyledSpan) -> TextRun {
    let style = build_text_style(span);
    TextRun { t: span.text.clone(), s: style }
}

fn build_text_style(span: &StyledSpan) -> TextStyle {
    TextStyle {
        bl: if span.bold { Some(1) } else { None },
        it: if span.italic { Some(1) } else { None },
        ul: if span.underline { Some(UnderlineStyle { s: 1 }) } else { None },
        cl: span.color.as_ref().map(|c| FontColor { rgb: normalize_color(c) }),
    }
}

fn normalize_color(color: &str) -> String {
    let c = color.trim_start_matches('#');
    c.to_uppercase()
}

/// Univer ICellData rich text structure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UniverRichText {
    pub p: Vec<Paragraph>,
}

/// A paragraph containing text runs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Paragraph {
    pub ts: Vec<TextRun>,
}

/// A text run with content and styling.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextRun {
    pub t: String,
    #[serde(skip_serializing_if = "TextStyle::is_empty")]
    pub s: TextStyle,
}

/// Style properties for a text run.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TextStyle {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bl: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub it: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ul: Option<UnderlineStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cl: Option<FontColor>,
}

impl TextStyle {
    pub fn is_empty(&self) -> bool {
        self.bl.is_none() && self.it.is_none() && self.ul.is_none() && self.cl.is_none()
    }
}

/// Underline style specification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnderlineStyle {
    pub s: u8,
}

/// Font color as RGB hex string.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FontColor {
    pub rgb: String,
}
