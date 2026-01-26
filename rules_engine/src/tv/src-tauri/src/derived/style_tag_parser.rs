use crate::derived::derived_types::StyledSpan;

#[derive(Clone, Debug, PartialEq, Eq)]
struct StyleState {
    bold: bool,
    italic: bool,
    underline: bool,
    color: Option<String>,
}

impl StyleState {
    fn new() -> Self {
        Self { bold: false, italic: false, underline: false, color: None }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum HtmlTag {
    BoldStart,
    BoldEnd,
    ItalicStart,
    ItalicEnd,
    UnderlineStart,
    UnderlineEnd,
    ColorStart(String),
    ColorEnd,
}

/// Parses HTML-like style tags from formatted text and returns a list of styled spans.
///
/// Supports the following tags:
/// - `<b>...</b>` for bold text
/// - `<i>...</i>` for italic text
/// - `<u>...</u>` for underlined text
/// - `<color=#RRGGBB>...</color>` for colored text (6-digit hex RGB)
///
/// Tags can be nested and are handled independently. Invalid or unrecognized
/// tags are passed through as literal text.
pub fn parse_style_tags(text: &str) -> Vec<StyledSpan> {
    let mut spans = Vec::new();
    let mut current_text = String::new();
    let mut bold_depth = 0usize;
    let mut italic_depth = 0usize;
    let mut underline_depth = 0usize;
    let mut color_stack: Vec<String> = Vec::new();
    let mut current_style = StyleState::new();

    let mut idx = 0usize;
    while idx < text.len() {
        let Some(open_rel) = text[idx..].find('<') else {
            current_text.push_str(&text[idx..]);
            break;
        };

        let tag_start = idx + open_rel;
        if tag_start > idx {
            current_text.push_str(&text[idx..tag_start]);
        }

        let Some(close_rel) = text[tag_start..].find('>') else {
            current_text.push('<');
            idx = tag_start + 1;
            continue;
        };

        let tag_end = tag_start + close_rel;
        let tag_text = text[tag_start + 1..tag_end].trim();

        if let Some(tag) = parse_html_tag(tag_text) {
            let next_style = match tag {
                HtmlTag::BoldStart => {
                    bold_depth += 1;
                    StyleState {
                        bold: bold_depth > 0,
                        italic: italic_depth > 0,
                        underline: underline_depth > 0,
                        color: color_stack.last().cloned(),
                    }
                }
                HtmlTag::BoldEnd => {
                    bold_depth = bold_depth.saturating_sub(1);
                    StyleState {
                        bold: bold_depth > 0,
                        italic: italic_depth > 0,
                        underline: underline_depth > 0,
                        color: color_stack.last().cloned(),
                    }
                }
                HtmlTag::ItalicStart => {
                    italic_depth += 1;
                    StyleState {
                        bold: bold_depth > 0,
                        italic: italic_depth > 0,
                        underline: underline_depth > 0,
                        color: color_stack.last().cloned(),
                    }
                }
                HtmlTag::ItalicEnd => {
                    italic_depth = italic_depth.saturating_sub(1);
                    StyleState {
                        bold: bold_depth > 0,
                        italic: italic_depth > 0,
                        underline: underline_depth > 0,
                        color: color_stack.last().cloned(),
                    }
                }
                HtmlTag::UnderlineStart => {
                    underline_depth += 1;
                    StyleState {
                        bold: bold_depth > 0,
                        italic: italic_depth > 0,
                        underline: underline_depth > 0,
                        color: color_stack.last().cloned(),
                    }
                }
                HtmlTag::UnderlineEnd => {
                    underline_depth = underline_depth.saturating_sub(1);
                    StyleState {
                        bold: bold_depth > 0,
                        italic: italic_depth > 0,
                        underline: underline_depth > 0,
                        color: color_stack.last().cloned(),
                    }
                }
                HtmlTag::ColorStart(color) => {
                    color_stack.push(color);
                    StyleState {
                        bold: bold_depth > 0,
                        italic: italic_depth > 0,
                        underline: underline_depth > 0,
                        color: color_stack.last().cloned(),
                    }
                }
                HtmlTag::ColorEnd => {
                    color_stack.pop();
                    StyleState {
                        bold: bold_depth > 0,
                        italic: italic_depth > 0,
                        underline: underline_depth > 0,
                        color: color_stack.last().cloned(),
                    }
                }
            };

            if next_style != current_style && !current_text.is_empty() {
                spans.push(StyledSpan {
                    text: std::mem::take(&mut current_text),
                    bold: current_style.bold,
                    italic: current_style.italic,
                    underline: current_style.underline,
                    color: current_style.color.clone(),
                });
            }
            current_style = next_style;
            idx = tag_end + 1;
            continue;
        }

        let literal = &text[tag_start..=tag_end];
        current_text.push_str(literal);
        idx = tag_end + 1;
    }

    if !current_text.is_empty() {
        spans.push(StyledSpan {
            text: current_text,
            bold: current_style.bold,
            italic: current_style.italic,
            underline: current_style.underline,
            color: current_style.color,
        });
    }

    spans
}

fn parse_html_tag(tag: &str) -> Option<HtmlTag> {
    let trimmed = tag.trim();
    let lower = trimmed.to_ascii_lowercase();

    if lower == "b" {
        return Some(HtmlTag::BoldStart);
    }
    if lower == "/b" {
        return Some(HtmlTag::BoldEnd);
    }
    if lower == "i" {
        return Some(HtmlTag::ItalicStart);
    }
    if lower == "/i" {
        return Some(HtmlTag::ItalicEnd);
    }
    if lower == "u" {
        return Some(HtmlTag::UnderlineStart);
    }
    if lower == "/u" {
        return Some(HtmlTag::UnderlineEnd);
    }
    if lower == "/color" {
        return Some(HtmlTag::ColorEnd);
    }
    if lower.starts_with("color=") {
        let value = trimmed.split_once('=')?.1;
        let hex = value.trim().strip_prefix('#').unwrap_or(value.trim());
        if hex.len() == 6 && hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
            return Some(HtmlTag::ColorStart(hex.to_ascii_uppercase()));
        }
    }
    None
}
