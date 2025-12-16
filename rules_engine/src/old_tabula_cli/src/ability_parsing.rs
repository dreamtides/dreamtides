use anyhow::Result;
use core_data::initialization_error::InitializationError;
use parser::{ability_parser, displayed_ability_parser};
use tabula_data::card_definitions::base_card_definition_type::BaseCardDefinitionType;
use tabula_data::tabula_table::Table;

pub fn parse_table_abilities<I, T>(
    sheet_name: &str,
    table: &mut Table<I, T>,
) -> Result<(), Vec<InitializationError>>
where
    T: BaseCardDefinitionType,
{
    let mut errors: Vec<InitializationError> = Vec::new();
    for (row_index, row) in table.iter_mut().enumerate() {
        match ability_parser::parse(row.rules_text_en_us()) {
            Ok(parsed) => {
                *row.abilities_mut() = Some(parsed.clone());
            }
            Err(mut errs) => {
                for e in errs.iter_mut() {
                    if e.tabula_sheet.is_none() {
                        e.tabula_sheet = Some(String::from(sheet_name));
                    }
                    if e.tabula_row.is_none() {
                        e.tabula_row = Some(row_index);
                    }
                    if e.tabula_column.is_none() {
                        e.tabula_column = Some(String::from("rules_text_en_us"));
                    }
                    if e.tabula_id.is_none() {
                        e.tabula_id = Some(row.id_string());
                    }
                }
                errors.extend(errs);
            }
        }
    }
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

pub fn parse_table_displayed_abilities<I, T>(
    sheet_name: &str,
    table: &mut Table<I, T>,
) -> Result<(), Vec<InitializationError>>
where
    T: BaseCardDefinitionType,
{
    let mut errors: Vec<InitializationError> = Vec::new();
    for (row_index, row) in table.iter_mut().enumerate() {
        match displayed_ability_parser::parse_with(
            row.abilities_ref().expect("abilities not present"),
            row.rules_text_en_us(),
        ) {
            Ok(parsed) => {
                *row.displayed_abilities_mut() = Some(parsed.clone());
            }
            Err(mut errs) => {
                for e in errs.iter_mut() {
                    if e.tabula_sheet.is_none() {
                        e.tabula_sheet = Some(String::from(sheet_name));
                    }
                    if e.tabula_row.is_none() {
                        e.tabula_row = Some(row_index);
                    }
                    if e.tabula_column.is_none() {
                        e.tabula_column = Some(String::from("rules_text_en_us"));
                    }
                    if e.tabula_id.is_none() {
                        e.tabula_id = Some(row.id_string());
                    }
                }
                errors.extend(errs);
            }
        }
    }
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

pub fn parse_all_abilities_for_raw_tabula(raw: &mut tabula_data::tabula::TabulaRaw) -> Result<()> {
    parse_table_abilities("test_cards", &mut raw.cards).map_err(|errs| {
        anyhow::anyhow!(errs.into_iter().map(|e| e.format()).collect::<Vec<_>>().join("\n"))
    })?;
    parse_table_displayed_abilities("test_cards", &mut raw.cards).map_err(|errs| {
        anyhow::anyhow!(errs.into_iter().map(|e| e.format()).collect::<Vec<_>>().join("\n"))
    })?;

    parse_table_abilities("dreamwell_cards", &mut raw.dreamwell_cards).map_err(|errs| {
        anyhow::anyhow!(errs.into_iter().map(|e| e.format()).collect::<Vec<_>>().join("\n"))
    })?;
    parse_table_displayed_abilities("dreamwell_cards", &mut raw.dreamwell_cards).map_err(
        |errs| anyhow::anyhow!(errs.into_iter().map(|e| e.format()).collect::<Vec<_>>().join("\n")),
    )?;

    Ok(())
}
