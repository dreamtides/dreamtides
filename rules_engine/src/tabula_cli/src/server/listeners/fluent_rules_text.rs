use std::collections::BTreeMap;
use std::sync::Arc;

use anyhow::Result;
use fluent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentError, FluentResource, FluentValue};

use super::super::listener_runner::{Listener, ListenerContext, ListenerResult};
use super::super::model::{Change, Span};
use super::super::server_workbook_snapshot::{CellValue, WorkbookSnapshot};

pub struct FluentRulesTextListener {
    resource: Arc<FluentResource>,
}

impl FluentRulesTextListener {
    pub fn new() -> Result<Self> {
        let ftl_source = include_str!("card_rules.ftl");
        let resource = FluentResource::try_new(ftl_source.to_string()).map_err(|(_, errors)| {
            let error_vec: Vec<FluentError> =
                errors.into_iter().map(FluentError::ParserError).collect();
            anyhow::anyhow!("Failed to parse Fluent resource: {}", format_fluent_errors(&error_vec))
        })?;
        Ok(FluentRulesTextListener { resource: Arc::new(resource) })
    }
}

impl Listener for FluentRulesTextListener {
    fn name(&self) -> &str {
        "fluent_rules_text"
    }

    fn run(
        &self,
        snapshot: &WorkbookSnapshot,
        _context: &ListenerContext,
    ) -> Result<ListenerResult> {
        let mut changes = Vec::new();
        let mut warnings = Vec::new();

        let cards_table = match snapshot.tables.iter().find(|t| t.name == "Cards2") {
            Some(t) => t,
            None => return Ok(ListenerResult { changes, warnings }),
        };

        let rules_text_col_idx = match find_column_index(&cards_table.columns, "RulesText3") {
            Some(idx) => idx,
            None => return Ok(ListenerResult { changes, warnings }),
        };

        let variables_col_idx = find_column_index(&cards_table.columns, "Variables");
        let output_col_idx = rules_text_col_idx + 1;
        if output_col_idx >= cards_table.columns.len() {
            warnings.push(
                "Cannot create output column: 'RulesText3' is the last column in table 'Cards2'"
                    .to_string(),
            );
            return Ok(ListenerResult { changes, warnings });
        }

        let sheet = match snapshot.sheets.iter().find(|s| s.name == cards_table.sheet_name) {
            Some(s) => s,
            None => {
                warnings.push(format!(
                    "Sheet '{}' for table 'Cards2' not found",
                    cards_table.sheet_name
                ));
                return Ok(ListenerResult { changes, warnings });
            }
        };

        for row in cards_table.data_range.start_row..=cards_table.data_range.end_row {
            let input_col = cards_table.data_range.start_col + rules_text_col_idx as u32;
            let output_col = cards_table.data_range.start_col + output_col_idx as u32;

            let input_cell_ref = format!("{}{}", col_index_to_letter(input_col), row + 1);
            let output_cell_ref = format!("{}{}", col_index_to_letter(output_col), row + 1);

            if let Some(CellValue::String(input_text)) = sheet.cell_values.get(&input_cell_ref) {
                if input_text.trim().is_empty() {
                    continue;
                }

                let variables_cell_ref = variables_col_idx.map(|idx| {
                    format!(
                        "{}{}",
                        col_index_to_letter(cards_table.data_range.start_col + idx as u32),
                        row + 1
                    )
                });

                let args = match parse_fluent_args(
                    variables_cell_ref
                        .as_ref()
                        .and_then(|cell_ref| sheet.cell_values.get(cell_ref)),
                ) {
                    Ok(args) => args,
                    Err(e) => {
                        changes.push(Change::SetValue {
                            sheet: sheet.name.clone(),
                            cell: output_cell_ref.clone(),
                            value: format!("Error: {e}"),
                        });
                        continue;
                    }
                };

                match format_fluent_expression(&self.resource, input_text, &args) {
                    Ok(formatted) => {
                        let styled = apply_simple_html_styles(&formatted);
                        changes.push(Change::SetValue {
                            sheet: sheet.name.clone(),
                            cell: output_cell_ref.clone(),
                            value: styled.text,
                        });
                        if !styled.unbold_spans.is_empty() {
                            changes.push(Change::SetBoldSpans {
                                sheet: sheet.name.clone(),
                                cell: output_cell_ref.clone(),
                                bold: false,
                                spans: styled.unbold_spans,
                            });
                        }
                        if !styled.bold_spans.is_empty() {
                            changes.push(Change::SetBoldSpans {
                                sheet: sheet.name.clone(),
                                cell: output_cell_ref.clone(),
                                bold: true,
                                spans: styled.bold_spans,
                            });
                        }
                        for (rgb, spans) in styled.color_spans {
                            changes.push(Change::SetFontColorSpans {
                                sheet: sheet.name.clone(),
                                cell: output_cell_ref.clone(),
                                rgb,
                                spans,
                            });
                        }
                    }
                    Err(e) => {
                        changes.push(Change::SetValue {
                            sheet: sheet.name.clone(),
                            cell: output_cell_ref,
                            value: format!("Error: {e}"),
                        });
                    }
                }
            }
        }

        Ok(ListenerResult { changes, warnings })
    }
}

fn find_column_index(columns: &[String], name: &str) -> Option<usize> {
    let normalized_name = normalize_column_name(name);
    columns.iter().position(|col| normalize_column_name(col) == normalized_name)
}

fn normalize_column_name(name: &str) -> String {
    name.trim().replace(['\u{00A0}', '\u{202F}'], " ").to_lowercase()
}

fn parse_fluent_args(cell_value: Option<&CellValue>) -> Result<FluentArgs> {
    let mut args = FluentArgs::new();
    let variables_text = match cell_value {
        None | Some(CellValue::Empty) => return Ok(args),
        Some(CellValue::String(text)) => text,
        Some(_) => {
            return Err(anyhow::anyhow!("Variables cell must be text with entries like 'e: 3'"));
        }
    };

    for line in variables_text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let Some((key, value)) = line.split_once(':') else {
            return Err(anyhow::anyhow!("Invalid variable definition '{line}'"));
        };

        if key.trim().is_empty() || value.trim().is_empty() {
            return Err(anyhow::anyhow!("Invalid variable definition '{line}'"));
        }

        args.set(key.trim(), FluentValue::try_number(value.trim()));
    }

    Ok(args)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StyleState {
    bold: bool,
    color: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StyledRun {
    start: usize,
    length: usize,
    bold: bool,
    color: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StyledText {
    text: String,
    bold_spans: Vec<Span>,
    unbold_spans: Vec<Span>,
    color_spans: Vec<(String, Vec<Span>)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum HtmlTag {
    BoldStart,
    BoldEnd,
    ColorStart(String),
    ColorEnd,
}

fn apply_simple_html_styles(text: &str) -> StyledText {
    let mut output = String::new();
    let mut runs = Vec::new();
    let mut bold_depth = 0usize;
    let mut color_stack: Vec<String> = Vec::new();
    let mut current_style = StyleState { bold: false, color: None };
    let mut output_len = 0usize;
    let mut run_start = 0usize;
    let mut idx = 0usize;

    while idx < text.len() {
        let Some(open_rel) = text[idx..].find('<') else {
            let tail = &text[idx..];
            output.push_str(tail);
            output_len += tail.chars().count();
            break;
        };

        let tag_start = idx + open_rel;
        if tag_start > idx {
            let chunk = &text[idx..tag_start];
            output.push_str(chunk);
            output_len += chunk.chars().count();
        }

        let Some(close_rel) = text[tag_start..].find('>') else {
            output.push('<');
            output_len += 1;
            idx = tag_start + 1;
            continue;
        };

        let tag_end = tag_start + close_rel;
        let tag_text = text[tag_start + 1..tag_end].trim();
        if let Some(tag) = parse_simple_html_tag(tag_text) {
            let next_style = match tag {
                HtmlTag::BoldStart => {
                    bold_depth += 1;
                    StyleState { bold: true, color: color_stack.last().cloned() }
                }
                HtmlTag::BoldEnd => {
                    bold_depth = bold_depth.saturating_sub(1);
                    StyleState { bold: bold_depth > 0, color: color_stack.last().cloned() }
                }
                HtmlTag::ColorStart(color) => {
                    color_stack.push(color);
                    StyleState { bold: bold_depth > 0, color: color_stack.last().cloned() }
                }
                HtmlTag::ColorEnd => {
                    color_stack.pop();
                    StyleState { bold: bold_depth > 0, color: color_stack.last().cloned() }
                }
            };
            if next_style != current_style {
                if output_len > run_start {
                    runs.push(StyledRun {
                        start: run_start,
                        length: output_len - run_start,
                        bold: current_style.bold,
                        color: current_style.color.clone(),
                    });
                }
                current_style = next_style;
                run_start = output_len;
            }
            idx = tag_end + 1;
            continue;
        }

        let literal = &text[tag_start..=tag_end];
        output.push_str(literal);
        output_len += literal.chars().count();
        idx = tag_end + 1;
    }

    if output_len > run_start {
        runs.push(StyledRun {
            start: run_start,
            length: output_len - run_start,
            bold: current_style.bold,
            color: current_style.color.clone(),
        });
    }

    let mut bold_spans = Vec::new();
    let mut unbold_spans = Vec::new();
    let mut color_spans: BTreeMap<String, Vec<Span>> = BTreeMap::new();
    for run in runs {
        let start = (run.start + 1) as u32;
        let length = run.length as u32;
        if run.bold {
            bold_spans.push(Span { start, length });
        } else {
            unbold_spans.push(Span { start, length });
        }
        if let Some(color) = run.color {
            color_spans.entry(color).or_default().push(Span { start, length });
        }
    }

    StyledText {
        text: output,
        bold_spans,
        unbold_spans,
        color_spans: color_spans.into_iter().collect(),
    }
}

fn inline_named_argument_variables(expression: &str, args: &FluentArgs) -> String {
    let chars: Vec<char> = expression.chars().collect();
    let mut result = String::with_capacity(expression.len());
    let mut brace_depth = 0usize;
    let mut paren_depth = 0usize;
    let mut in_string = false;
    let mut escape_next = false;
    let mut idx = 0usize;

    while idx < chars.len() {
        let ch = chars[idx];

        if in_string {
            result.push(ch);
            if escape_next {
                escape_next = false;
            } else if ch == '\\' {
                escape_next = true;
            } else if ch == '"' {
                in_string = false;
            }
            idx += 1;
            continue;
        }

        match ch {
            '"' if brace_depth > 0 => {
                in_string = true;
                result.push(ch);
                idx += 1;
                continue;
            }
            '{' => {
                brace_depth += 1;
            }
            '}' => {
                brace_depth = brace_depth.saturating_sub(1);
            }
            '(' if brace_depth > 0 => {
                paren_depth += 1;
            }
            ')' if brace_depth > 0 => {
                paren_depth = paren_depth.saturating_sub(1);
            }
            ':' if brace_depth > 0 && paren_depth > 0 => {
                let mut cursor = idx + 1;
                while cursor < chars.len() && chars[cursor].is_whitespace() {
                    cursor += 1;
                }
                if cursor < chars.len() && chars[cursor] == '$' {
                    let mut name_end = cursor + 1;
                    while name_end < chars.len()
                        && (chars[name_end] == '_' || chars[name_end].is_ascii_alphanumeric())
                    {
                        name_end += 1;
                    }
                    if name_end > cursor + 1 {
                        let name = chars[cursor + 1..name_end].iter().collect::<String>();
                        if let Some(value) = args.get(name.as_str()) {
                            if let Some(literal) = format_fluent_literal(value) {
                                result.push(':');
                                result.extend(chars[idx + 1..cursor].iter());
                                result.push_str(&literal);
                                idx = name_end;
                                continue;
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        result.push(ch);
        idx += 1;
    }

    result
}

fn format_fluent_literal(value: &FluentValue) -> Option<String> {
    match value {
        FluentValue::String(text) => Some(format!("\"{}\"", escape_fluent_string(text.as_ref()))),
        FluentValue::Number(number) => Some(number.as_string().to_string()),
        _ => None,
    }
}

fn escape_fluent_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn parse_simple_html_tag(tag: &str) -> Option<HtmlTag> {
    let trimmed = tag.trim();
    let lower = trimmed.to_ascii_lowercase();
    if lower == "b" {
        return Some(HtmlTag::BoldStart);
    }
    if lower == "/b" {
        return Some(HtmlTag::BoldEnd);
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

fn expand_plain_variables(expression: &str) -> String {
    let mut result = String::with_capacity(expression.len());
    let mut chars = expression.chars().peekable();
    let mut depth = 0;

    while let Some(ch) = chars.next() {
        match ch {
            '{' => {
                depth += 1;
                result.push(ch);
            }
            '}' => {
                if depth > 0 {
                    depth -= 1;
                }
                result.push(ch);
            }
            '$' if depth == 0 => {
                let mut name = String::new();
                while let Some(&next) = chars.peek() {
                    if next == '_' || next.is_ascii_alphanumeric() {
                        name.push(next);
                        chars.next();
                    } else {
                        break;
                    }
                }

                if name.is_empty() {
                    result.push('$');
                } else {
                    result.push_str("{ $");
                    result.push_str(&name);
                    result.push_str(" }");
                }
            }
            _ => result.push(ch),
        }
    }

    result
}

fn format_fluent_expression(
    resource: &Arc<FluentResource>,
    expression: &str,
    args: &FluentArgs,
) -> Result<String> {
    let mut bundle: FluentBundle<Arc<FluentResource>> = FluentBundle::default();
    bundle.set_use_isolating(false);
    if let Err(errors) = bundle.add_resource(Arc::clone(resource)) {
        return Err(anyhow::anyhow!(
            "Failed to add Fluent resource to bundle: {}",
            format_fluent_errors(&errors)
        ));
    }

    let temp_ftl = format!(
        "temp-message = {}",
        expand_plain_variables(&inline_named_argument_variables(expression, args))
    );
    let temp_resource = FluentResource::try_new(temp_ftl).map_err(|(_, errors)| {
        let error_vec: Vec<FluentError> =
            errors.into_iter().map(FluentError::ParserError).collect();
        anyhow::anyhow!("Failed to parse expression: {}", format_fluent_errors(&error_vec))
    })?;

    if let Err(errors) = bundle.add_resource(Arc::new(temp_resource)) {
        return Err(anyhow::anyhow!(
            "Failed to add temporary message to bundle: {}",
            format_fluent_errors(&errors)
        ));
    }

    let message = bundle
        .get_message("temp-message")
        .ok_or_else(|| anyhow::anyhow!("Temporary message not found in bundle"))?;

    let pattern = message.value().ok_or_else(|| anyhow::anyhow!("Message has no value"))?;

    let mut errors = Vec::new();
    let formatted = bundle.format_pattern(pattern, Some(args), &mut errors);

    if !errors.is_empty() {
        return Err(anyhow::anyhow!("Fluent formatting errors: {}", format_fluent_errors(&errors)));
    }

    Ok(formatted.into_owned())
}

fn format_fluent_errors(errors: &[FluentError]) -> String {
    errors
        .iter()
        .map(|e| match e {
            FluentError::Overriding { kind, id } => {
                format!("Overriding {kind} id={id}")
            }
            FluentError::ParserError(pe) => format!("Parser error: {pe}"),
            FluentError::ResolverError(re) => format!("Resolver error: {re}"),
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn col_index_to_letter(col: u32) -> String {
    let mut result = String::new();
    let mut n = col + 1;
    while n > 0 {
        n -= 1;
        result.insert(0, char::from(b'A' + (n % 26) as u8));
        n /= 26;
    }
    result
}
