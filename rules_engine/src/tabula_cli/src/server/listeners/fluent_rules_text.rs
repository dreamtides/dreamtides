use std::sync::Arc;

use anyhow::Result;
use fluent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentError, FluentResource};

use super::super::listener_runner::{Listener, ListenerContext, ListenerResult};
use super::super::model::Change;
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

                match format_fluent_expression(&self.resource, input_text) {
                    Ok(formatted) => {
                        changes.push(Change::SetValue {
                            sheet: sheet.name.clone(),
                            cell: output_cell_ref,
                            value: formatted,
                        });
                    }
                    Err(e) => {
                        changes.push(Change::SetFillColor {
                            sheet: sheet.name.clone(),
                            cell: input_cell_ref.clone(),
                            rgb: "FFE0E0".to_string(),
                        });
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

fn format_fluent_expression(resource: &Arc<FluentResource>, expression: &str) -> Result<String> {
    let mut bundle: FluentBundle<Arc<FluentResource>> = FluentBundle::default();
    bundle.set_use_isolating(false);
    if let Err(errors) = bundle.add_resource(Arc::clone(resource)) {
        return Err(anyhow::anyhow!(
            "Failed to add Fluent resource to bundle: {}",
            format_fluent_errors(&errors)
        ));
    }

    let temp_ftl = format!("temp-message = {expression}");
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

    let args = FluentArgs::new();
    let mut errors = Vec::new();
    let formatted = bundle.format_pattern(pattern, Some(&args), &mut errors);

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
