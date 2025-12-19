use anyhow::Result;

use super::super::listener_runner::{Listener, ListenerContext, ListenerResult};
use super::super::model::{Change, Span};
use super::super::server_workbook_snapshot::{CellValue, WorkbookSnapshot};

pub struct BoxiconsListener;

impl Listener for BoxiconsListener {
    fn name(&self) -> &str {
        "boxicons"
    }

    fn run(
        &self,
        snapshot: &WorkbookSnapshot,
        _context: &ListenerContext,
    ) -> Result<ListenerResult> {
        const BOXICONS_FONT: &str = "boxicons";
        const FONT_SIZE: f32 = 20.0;

        let mut changes = Vec::new();

        for sheet in &snapshot.sheets {
            for (cell_ref, value) in &sheet.cell_values {
                if let CellValue::String(text) = value {
                    let (new_text, spans) = find_and_replace_boxicons(text);
                    if !spans.is_empty() {
                        changes.push(Change::SetValue {
                            sheet: sheet.name.clone(),
                            cell: cell_ref.clone(),
                            value: new_text,
                        });

                        changes.push(Change::SetFontNameSpans {
                            sheet: sheet.name.clone(),
                            cell: cell_ref.clone(),
                            font_name: BOXICONS_FONT.to_string(),
                            spans: spans.clone(),
                        });

                        changes.push(Change::SetFontSizeSpans {
                            sheet: sheet.name.clone(),
                            cell: cell_ref.clone(),
                            points: FONT_SIZE,
                            spans: spans.clone(),
                        });

                        changes.push(Change::SetSubscriptSpans {
                            sheet: sheet.name.clone(),
                            cell: cell_ref.clone(),
                            subscript: true,
                            spans,
                        });
                    }
                }
            }
        }

        Ok(ListenerResult { changes, warnings: vec![] })
    }
}

struct IconReplacement {
    pattern: &'static str,
    icon: char,
}

const ICON_REPLACEMENTS: &[IconReplacement] = &[
    IconReplacement { pattern: "{fast}", icon: '\u{F93A}' },
    IconReplacement { pattern: "{x}", icon: '\u{FEFC}' },
    IconReplacement { pattern: "{e}", icon: '\u{F407}' },
    IconReplacement { pattern: "{p}", icon: '\u{FC6A}' },
];

fn find_and_replace_boxicons(text: &str) -> (String, Vec<Span>) {
    const LRM: char = '\u{200E}';

    let mut replacements = Vec::new();

    for icon_replacement in ICON_REPLACEMENTS {
        let mut search_start = 0;
        while let Some(pos) = text[search_start..].find(icon_replacement.pattern) {
            let absolute_pos = search_start + pos;
            replacements.push((
                absolute_pos,
                icon_replacement.pattern.len(),
                icon_replacement.icon,
            ));
            search_start = absolute_pos + icon_replacement.pattern.len();
        }
    }

    if replacements.is_empty() {
        return (text.to_string(), vec![]);
    }

    replacements.sort_by_key(|(pos, _, _)| *pos);

    let mut result = String::new();
    let mut spans = Vec::new();
    let mut text_pos = 0;

    for (pattern_pos, pattern_len, icon_char) in replacements {
        result.push_str(&text[text_pos..pattern_pos]);

        let char_position = result.chars().count() + 1;
        result.push(LRM);
        result.push(icon_char);

        spans.push(Span { start: char_position as u32, length: 2 });

        text_pos = pattern_pos + pattern_len;
    }

    result.push_str(&text[text_pos..]);

    (result, spans)
}
