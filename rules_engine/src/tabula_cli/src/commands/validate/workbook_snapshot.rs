use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{Context, Result};
use umya_spreadsheet::helper::coordinate::string_from_column_index;
use umya_spreadsheet::reader::xlsx;
use umya_spreadsheet::structs::{
    ConditionalFormattingRule, DataValidation, DataValidationOperatorValues,
    HorizontalAlignmentValues, VerticalAlignmentValues,
};
use umya_spreadsheet::{DataValidationValues, Worksheet};

use super::runner::{ValidateConfig, record_error};

#[derive(Clone)]
pub(super) struct WorkbookSnapshot {
    sheets: Vec<String>,
    tables: BTreeMap<String, TableSnapshot>,
    sheet_snapshots: BTreeMap<String, SheetSnapshot>,
}

#[derive(Clone)]
struct TableSnapshot {
    name: String,
    sheet: String,
    area: String,
    totals_row: bool,
    columns: Vec<String>,
    style_name: Option<String>,
}

#[derive(Clone)]
struct SheetSnapshot {
    name: String,
    columns: BTreeMap<u32, ColumnDimensionSnapshot>,
    rows: BTreeMap<u32, RowDimensionSnapshot>,
    data_validations: BTreeSet<DataValidationSnapshot>,
    conditionals: BTreeSet<ConditionalFormattingSnapshot>,
    cell_alignment: BTreeMap<String, CellAlignmentSnapshot>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ColumnDimensionSnapshot {
    width: String,
    hidden: bool,
    best_fit: bool,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RowDimensionSnapshot {
    height: String,
    hidden: bool,
    custom: bool,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct DataValidationSnapshot {
    sqref: String,
    r#type: DataValidationValues,
    operator: DataValidationOperatorValues,
    allow_blank: bool,
    show_input: bool,
    show_error: bool,
    prompt_title: String,
    prompt: String,
    error_title: String,
    error_message: String,
    formula1: String,
    formula2: String,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ConditionalFormattingSnapshot {
    sqref: String,
    rules: Vec<ConditionalRuleSnapshot>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ConditionalRuleSnapshot {
    r#type: String,
    operator: String,
    priority: i32,
    stop_if_true: bool,
    formula: Vec<String>,
    text: String,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CellAlignmentSnapshot {
    horizontal: Option<HorizontalAlignmentValues>,
    vertical: Option<VerticalAlignmentValues>,
    wrap_text: bool,
}

pub(super) fn compare_workbooks(
    original: &Path,
    roundtrip: &Path,
    config: ValidateConfig,
    errors: &mut Vec<String>,
) -> Result<()> {
    let original_snapshot = build_snapshot(original)?;
    let roundtrip_snapshot = build_snapshot(roundtrip)?;
    compare_sheet_order(&original_snapshot, &roundtrip_snapshot, config, errors)?;
    compare_tables(&original_snapshot, &roundtrip_snapshot, config, errors)?;
    compare_sheet_snapshots(&original_snapshot, &roundtrip_snapshot, config, errors)?;
    Ok(())
}

fn build_snapshot(path: &Path) -> Result<WorkbookSnapshot> {
    let book = xlsx::read(path)
        .with_context(|| format!("Cannot open spreadsheet at {}", path.display()))?;
    let sheets: Vec<String> =
        book.get_sheet_collection().iter().map(|s| s.get_name().to_string()).collect();

    let mut tables = BTreeMap::new();
    let mut sheet_snapshots = BTreeMap::new();
    for sheet in book.get_sheet_collection() {
        for table in sheet.get_tables() {
            let area = table.get_area();
            let start_col = string_from_column_index(area.0.get_col_num());
            let start_row = area.0.get_row_num();
            let end_col = string_from_column_index(area.1.get_col_num());
            let end_row = area.1.get_row_num();
            let area_str = format!("{start_col}{start_row}:{end_col}{end_row}");
            let style_name = table.get_style_info().map(|style| style.get_name().to_string());
            let columns: Vec<String> =
                table.get_columns().iter().map(|c| c.get_name().to_string()).collect();
            tables.insert(table.get_name().to_string(), TableSnapshot {
                name: table.get_name().to_string(),
                sheet: sheet.get_name().to_string(),
                area: area_str,
                totals_row: *table.get_totals_row_shown(),
                columns,
                style_name,
            });
        }
        sheet_snapshots.insert(sheet.get_name().to_string(), SheetSnapshot {
            name: sheet.get_name().to_string(),
            columns: collect_columns(sheet),
            rows: collect_rows(sheet),
            data_validations: collect_data_validations(sheet),
            conditionals: collect_conditionals(sheet),
            cell_alignment: collect_alignment(sheet),
        });
    }

    Ok(WorkbookSnapshot { sheets, tables, sheet_snapshots })
}

fn compare_sheet_order(
    expected: &WorkbookSnapshot,
    actual: &WorkbookSnapshot,
    config: ValidateConfig,
    errors: &mut Vec<String>,
) -> Result<()> {
    if expected.sheets.len() != actual.sheets.len()
        && record_error(
            errors,
            config.report_all,
            format!(
                "Sheet count differs: expected {}, found {}",
                expected.sheets.len(),
                actual.sheets.len()
            ),
        )
    {
        return Ok(());
    }
    let max = std::cmp::max(expected.sheets.len(), actual.sheets.len());
    for idx in 0..max {
        let expected_name = expected.sheets.get(idx);
        let actual_name = actual.sheets.get(idx);
        if expected_name != actual_name {
            let message = format!(
                "Sheet order differs at position {}: expected {:?}, found {:?}",
                idx + 1,
                expected_name,
                actual_name
            );
            if record_error(errors, config.report_all, message) {
                return Ok(());
            }
        }
    }
    Ok(())
}

fn compare_tables(
    expected: &WorkbookSnapshot,
    actual: &WorkbookSnapshot,
    config: ValidateConfig,
    errors: &mut Vec<String>,
) -> Result<()> {
    for (name, table) in &expected.tables {
        let Some(actual_table) = actual.tables.get(name) else {
            if record_error(errors, config.report_all, format!("Missing table '{}'", table.name)) {
                return Ok(());
            }
            continue;
        };
        if table.sheet != actual_table.sheet
            && record_error(
                errors,
                config.report_all,
                format!(
                    "Table '{}' on sheet '{}' but found on sheet '{}'",
                    table.name, table.sheet, actual_table.sheet
                ),
            )
        {
            return Ok(());
        }
        if table.area != actual_table.area
            && record_error(
                errors,
                config.report_all,
                format!(
                    "Table '{}' range differs: expected {}, found {}",
                    table.name, table.area, actual_table.area
                ),
            )
        {
            return Ok(());
        }
        if table.totals_row != actual_table.totals_row
            && record_error(
                errors,
                config.report_all,
                format!(
                    "Table '{}' totals row presence differs (expected {}, found {})",
                    table.name, table.totals_row, actual_table.totals_row
                ),
            )
        {
            return Ok(());
        }
        if table.columns != actual_table.columns {
            let mut message = format!("Table '{}' columns differ", table.name);
            if config.verbose {
                message = format!(
                    "{message}: expected {:?}, found {:?}",
                    table.columns, actual_table.columns
                );
            }
            if record_error(errors, config.report_all, message) {
                return Ok(());
            }
        }
        if table.style_name != actual_table.style_name {
            let mut message = format!(
                "Table '{}' style differs: expected {:?}, found {:?}",
                table.name, table.style_name, actual_table.style_name
            );
            if !config.verbose {
                message = format!("Table '{}' style differs", table.name);
            }
            if record_error(errors, config.report_all, message) {
                return Ok(());
            }
        }
    }
    for name in actual.tables.keys() {
        if !expected.tables.contains_key(name)
            && record_error(errors, config.report_all, format!("Unexpected table '{name}'"))
        {
            return Ok(());
        }
    }
    Ok(())
}

fn compare_sheet_snapshots(
    expected: &WorkbookSnapshot,
    actual: &WorkbookSnapshot,
    config: ValidateConfig,
    errors: &mut Vec<String>,
) -> Result<()> {
    for (name, sheet) in &expected.sheet_snapshots {
        let Some(actual_sheet) = actual.sheet_snapshots.get(name) else {
            if record_error(errors, config.report_all, format!("Missing sheet snapshot '{name}'")) {
                return Ok(());
            }
            continue;
        };
        compare_column_dimensions(sheet, actual_sheet, config, errors)?;
        compare_row_dimensions(sheet, actual_sheet, config, errors)?;
        compare_data_validations(sheet, actual_sheet, config, errors)?;
        compare_conditionals(sheet, actual_sheet, config, errors)?;
        compare_alignment(sheet, actual_sheet, config, errors)?;
    }
    for name in actual.sheet_snapshots.keys() {
        if !expected.sheet_snapshots.contains_key(name)
            && record_error(
                errors,
                config.report_all,
                format!("Unexpected sheet snapshot '{name}'"),
            )
        {
            return Ok(());
        }
    }
    Ok(())
}

fn collect_columns(sheet: &Worksheet) -> BTreeMap<u32, ColumnDimensionSnapshot> {
    let mut result = BTreeMap::new();
    for column in sheet.get_column_dimensions() {
        let col_num = *column.get_col_num();
        result.insert(col_num, ColumnDimensionSnapshot {
            width: column.get_width().to_string(),
            hidden: *column.get_hidden(),
            best_fit: *column.get_best_fit(),
        });
    }
    result
}

fn collect_rows(sheet: &Worksheet) -> BTreeMap<u32, RowDimensionSnapshot> {
    let mut result = BTreeMap::new();
    for row in sheet.get_row_dimensions() {
        let row_num = *row.get_row_num();
        result.insert(row_num, RowDimensionSnapshot {
            height: row.get_height().to_string(),
            hidden: *row.get_hidden(),
            custom: *row.get_custom_height(),
        });
    }
    result
}

fn collect_data_validations(sheet: &Worksheet) -> BTreeSet<DataValidationSnapshot> {
    let mut set = BTreeSet::new();
    if let Some(validations) = sheet.get_data_validations() {
        for validation in validations.get_data_validation_list() {
            set.insert(validation_snapshot(validation));
        }
    }
    set
}

fn validation_snapshot(validation: &DataValidation) -> DataValidationSnapshot {
    DataValidationSnapshot {
        sqref: validation.get_sequence_of_references().get_sqref().clone(),
        r#type: validation.get_type().clone(),
        operator: validation.get_operator().clone(),
        allow_blank: *validation.get_allow_blank(),
        show_input: *validation.get_show_input_message(),
        show_error: *validation.get_show_error_message(),
        prompt_title: validation.get_prompt_title().to_string(),
        prompt: validation.get_prompt().to_string(),
        error_title: validation.get_error_title().to_string(),
        error_message: validation.get_error_message().to_string(),
        formula1: validation.get_formula1().to_string(),
        formula2: validation.get_formula2().to_string(),
    }
}

fn collect_conditionals(sheet: &Worksheet) -> BTreeSet<ConditionalFormattingSnapshot> {
    let mut set = BTreeSet::new();
    for cf in sheet.get_conditional_formatting_collection() {
        let rules: Vec<ConditionalRuleSnapshot> =
            cf.get_conditional_collection().iter().map(rule_snapshot).collect();
        set.insert(ConditionalFormattingSnapshot {
            sqref: cf.get_sequence_of_references().get_sqref().clone(),
            rules,
        });
    }
    set
}

fn rule_snapshot(rule: &ConditionalFormattingRule) -> ConditionalRuleSnapshot {
    let formula =
        rule.get_formula().map(umya_spreadsheet::Formula::get_address_str).unwrap_or_default();
    ConditionalRuleSnapshot {
        r#type: format!("{:?}", rule.get_type()),
        operator: format!("{:?}", rule.get_operator()),
        priority: *rule.get_priority(),
        stop_if_true: *rule.get_stop_if_true(),
        formula: if formula.is_empty() { Vec::new() } else { vec![formula] },
        text: rule.get_text().to_string(),
    }
}

fn collect_alignment(sheet: &Worksheet) -> BTreeMap<String, CellAlignmentSnapshot> {
    let mut map = BTreeMap::new();
    let mut cells: Vec<_> = sheet.get_collection_to_hashmap().iter().collect();
    cells.sort_by_key(|((col, row), _)| (*row, *col));
    for ((col, row), cell) in cells {
        if let Some(alignment) = cell.get_style().get_alignment() {
            let coordinate = format!("{}{}", string_from_column_index(col), row);
            map.insert(coordinate, CellAlignmentSnapshot {
                horizontal: Some(alignment.get_horizontal().clone()),
                vertical: Some(alignment.get_vertical().clone()),
                wrap_text: *alignment.get_wrap_text(),
            });
        }
    }
    map
}

fn compare_column_dimensions(
    expected: &SheetSnapshot,
    actual: &SheetSnapshot,
    config: ValidateConfig,
    errors: &mut Vec<String>,
) -> Result<()> {
    for (col, dim) in &expected.columns {
        let Some(actual_dim) = actual.columns.get(col) else {
            if record_error(
                errors,
                config.report_all,
                format!(
                    "Sheet '{}' missing column dimension {} ({}): expected width={}, hidden={}, best_fit={}",
                    expected.name,
                    col,
                    string_from_column_index(col),
                    dim.width,
                    dim.hidden,
                    dim.best_fit
                ),
            ) {
                return Ok(());
            }
            continue;
        };
        if dim.width != actual_dim.width
            && record_error(
                errors,
                config.report_all,
                format!(
                    "Sheet '{}' column {} ({}) width differs: expected {}, found {}",
                    expected.name,
                    col,
                    string_from_column_index(col),
                    dim.width,
                    actual_dim.width
                ),
            )
        {
            return Ok(());
        }
        if dim.hidden != actual_dim.hidden
            && record_error(
                errors,
                config.report_all,
                format!(
                    "Sheet '{}' column {} hidden differs (expected {}, found {})",
                    expected.name, col, dim.hidden, actual_dim.hidden
                ),
            )
        {
            return Ok(());
        }
        if dim.best_fit != actual_dim.best_fit
            && record_error(
                errors,
                config.report_all,
                format!(
                    "Sheet '{}' column {} best-fit differs (expected {}, found {})",
                    expected.name, col, dim.best_fit, actual_dim.best_fit
                ),
            )
        {
            return Ok(());
        }
    }
    Ok(())
}

fn compare_row_dimensions(
    expected: &SheetSnapshot,
    actual: &SheetSnapshot,
    config: ValidateConfig,
    errors: &mut Vec<String>,
) -> Result<()> {
    for (row, dim) in &expected.rows {
        let Some(actual_dim) = actual.rows.get(row) else {
            if record_error(
                errors,
                config.report_all,
                format!(
                    "Sheet '{}' missing row dimension {}: expected height={}, hidden={}, custom={}",
                    expected.name, row, dim.height, dim.hidden, dim.custom
                ),
            ) {
                return Ok(());
            }
            continue;
        };
        if dim.height != actual_dim.height
            && record_error(
                errors,
                config.report_all,
                format!(
                    "Sheet '{}' row {} height differs: expected {}, found {}",
                    expected.name, row, dim.height, actual_dim.height
                ),
            )
        {
            return Ok(());
        }
        if dim.hidden != actual_dim.hidden
            && record_error(
                errors,
                config.report_all,
                format!(
                    "Sheet '{}' row {} hidden differs (expected {}, found {})",
                    expected.name, row, dim.hidden, actual_dim.hidden
                ),
            )
        {
            return Ok(());
        }
        if dim.custom != actual_dim.custom
            && record_error(
                errors,
                config.report_all,
                format!(
                    "Sheet '{}' row {} custom height differs (expected {}, found {})",
                    expected.name, row, dim.custom, actual_dim.custom
                ),
            )
        {
            return Ok(());
        }
    }
    Ok(())
}

fn compare_data_validations(
    expected: &SheetSnapshot,
    actual: &SheetSnapshot,
    config: ValidateConfig,
    errors: &mut Vec<String>,
) -> Result<()> {
    for validation in &expected.data_validations {
        if !actual.data_validations.contains(validation)
            && record_error(
                errors,
                config.report_all,
                format!(
                    "Sheet '{}' missing data validation at {}",
                    expected.name, validation.sqref
                ),
            )
        {
            return Ok(());
        }
    }
    for validation in &actual.data_validations {
        if !expected.data_validations.contains(validation)
            && record_error(
                errors,
                config.report_all,
                format!(
                    "Sheet '{}' has unexpected data validation at {}",
                    expected.name, validation.sqref
                ),
            )
        {
            return Ok(());
        }
    }
    Ok(())
}

fn compare_conditionals(
    expected: &SheetSnapshot,
    actual: &SheetSnapshot,
    config: ValidateConfig,
    errors: &mut Vec<String>,
) -> Result<()> {
    for conditional in &expected.conditionals {
        if !actual.conditionals.contains(conditional)
            && record_error(
                errors,
                config.report_all,
                format!(
                    "Sheet '{}' missing conditional formatting at {}",
                    expected.name, conditional.sqref
                ),
            )
        {
            return Ok(());
        }
    }
    for conditional in &actual.conditionals {
        if !expected.conditionals.contains(conditional)
            && record_error(
                errors,
                config.report_all,
                format!(
                    "Sheet '{}' has unexpected conditional formatting at {}",
                    expected.name, conditional.sqref
                ),
            )
        {
            return Ok(());
        }
    }
    Ok(())
}

fn compare_alignment(
    expected: &SheetSnapshot,
    actual: &SheetSnapshot,
    config: ValidateConfig,
    errors: &mut Vec<String>,
) -> Result<()> {
    for (coord, style) in &expected.cell_alignment {
        let Some(actual_style) = actual.cell_alignment.get(coord) else {
            let message = if config.verbose {
                format!(
                    "Sheet '{}' missing alignment for {} (expected {})",
                    expected.name,
                    coord,
                    describe_alignment(style)
                )
            } else {
                format!("Sheet '{}' missing alignment for {}", expected.name, coord)
            };
            if record_error(errors, config.report_all, message) {
                return Ok(());
            }
            continue;
        };
        if style != actual_style
            && record_error(
                errors,
                config.report_all,
                if config.verbose {
                    format!(
                        "Sheet '{}' alignment differs at {} (expected {}, found {})",
                        expected.name,
                        coord,
                        describe_alignment(style),
                        describe_alignment(actual_style)
                    )
                } else {
                    format!("Sheet '{}' alignment differs at {}", expected.name, coord)
                },
            )
        {
            return Ok(());
        }
    }
    Ok(())
}

fn describe_alignment(alignment: &CellAlignmentSnapshot) -> String {
    let horiz = alignment
        .horizontal
        .as_ref()
        .map(|h| format!("{h:?}"))
        .unwrap_or_else(|| "None".to_string());
    let vert =
        alignment.vertical.as_ref().map(|v| format!("{v:?}")).unwrap_or_else(|| "None".to_string());
    format!("h={horiz} v={vert} wrap={}", alignment.wrap_text)
}
