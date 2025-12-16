use std::path::Path;

use anyhow::Result;
use umya_spreadsheet::structs::{Table, TableColumn};
use umya_spreadsheet::writer::xlsx;

fn make_column(name: &str) -> TableColumn {
    let mut col = TableColumn::default();
    col.set_name(name.to_string());
    col
}

pub fn create_test_spreadsheet_with_table(path: &Path) -> Result<()> {
    let mut book = umya_spreadsheet::new_file();
    let sheet = book.get_sheet_mut(&0).expect("Sheet 0 should exist");
    sheet.set_name("TestSheet");

    sheet.get_cell_mut("A1").set_value("Name");
    sheet.get_cell_mut("B1").set_value("Count");
    sheet.get_cell_mut("C1").set_value("Active");
    sheet.get_cell_mut("D1").set_value("Computed");
    sheet.get_cell_mut("E1").set_value("Empty");

    sheet.get_cell_mut("A2").set_value("Alice");
    sheet.get_cell_mut("B2").set_value_number(10);
    sheet.get_cell_mut("C2").set_value_bool(true);
    sheet.get_cell_mut("D2").set_formula("=B2*2");

    sheet.get_cell_mut("A3").set_value("Bob");
    sheet.get_cell_mut("B3").set_value_number(20);
    sheet.get_cell_mut("C3").set_value_bool(false);
    sheet.get_cell_mut("D3").set_formula("=B3*2");

    let mut table = Table::default();
    table.set_name("TestTable");
    table.set_display_name("TestTable");
    table.set_area(("A1", "E3"));

    table.add_column(make_column("Name"));
    table.add_column(make_column("Count"));
    table.add_column(make_column("Active"));
    table.add_column(make_column("Computed"));
    table.add_column(make_column("Empty"));

    sheet.add_table(table);

    xlsx::write(&book, path)?;
    Ok(())
}

pub fn create_simple_spreadsheet(path: &Path) -> Result<()> {
    let mut book = umya_spreadsheet::new_file();
    let sheet = book.get_sheet_mut(&0).expect("Sheet 0 should exist");
    sheet.set_name("DataSheet");

    sheet.get_cell_mut("A1").set_value("Name");
    sheet.get_cell_mut("B1").set_value("Value");

    sheet.get_cell_mut("A2").set_value("Test");
    sheet.get_cell_mut("B2").set_value_number(42);

    xlsx::write(&book, path)?;
    Ok(())
}

pub fn create_special_column_spreadsheet(path: &Path) -> Result<()> {
    let mut book = umya_spreadsheet::new_file();
    let sheet = book.get_sheet_mut(&0).expect("Sheet 0 should exist");
    sheet.set_name("SpecialSheet");

    sheet.get_cell_mut("A1").set_value("Is Fast?");
    sheet.get_cell_mut("B1").set_value("ID");

    sheet.get_cell_mut("A2").set_value_bool(true);
    sheet.get_cell_mut("B2").set_value("abc123");

    let mut table = Table::default();
    table.set_name("Special");
    table.set_display_name("Special");
    table.set_area(("A1", "B2"));

    table.add_column(make_column("Is Fast?"));
    table.add_column(make_column("ID"));

    sheet.add_table(table);

    xlsx::write(&book, path)?;
    Ok(())
}

pub fn create_predicate_types_spreadsheet(path: &Path) -> Result<()> {
    let mut book = umya_spreadsheet::new_file();
    let sheet = book.get_sheet_mut(&0).expect("Sheet 0 should exist");
    sheet.set_name("PredicateTypes");

    sheet.get_cell_mut("A1").set_value("Predicate Types");
    sheet.get_cell_mut("A2").set_value("ThisCard");
    sheet.get_cell_mut("A3").set_value("ForEachTarget");
    sheet.get_cell_mut("A4").set_value("ControllerDeck");

    let mut table = Table::default();
    table.set_name("Predicate Types");
    table.set_display_name("Predicate Types");
    table.set_area(("A1", "A4"));

    table.add_column(make_column("Predicate Types"));

    sheet.add_table(table);

    xlsx::write(&book, path)?;
    Ok(())
}
