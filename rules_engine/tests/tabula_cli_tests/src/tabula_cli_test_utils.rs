use std::fs::File;
use std::io::Write;
use std::path::Path;

use anyhow::Result;
use umya_spreadsheet::structs::{Table, TableColumn};
use umya_spreadsheet::writer::xlsx;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipWriter};

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

pub fn create_xlsm_with_images(path: &Path) -> Result<(Vec<String>, Vec<u8>, Vec<u8>)> {
    let file = File::create(path)?;
    let mut writer = ZipWriter::new(file);
    let time = zip::DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0).expect("valid zip time");
    let deflated = FileOptions::<()>::default()
        .compression_method(CompressionMethod::Deflated)
        .last_modified_time(time);
    let stored = FileOptions::<()>::default()
        .compression_method(CompressionMethod::Stored)
        .last_modified_time(time);

    writer.start_file("[Content_Types].xml", deflated)?;
    writer.write_all(b"<Types/>")?;

    let img1 = b"first-image-bytes";
    let img2 = b"second-image-bytes";
    writer.start_file("xl/media/image1.jpg", stored)?;
    writer.write_all(img1)?;
    writer.start_file("xl/media/image2.png", stored)?;
    writer.write_all(img2)?;

    writer.finish()?;

    Ok((
        vec![
            "[Content_Types].xml".to_string(),
            "xl/media/image1.jpg".to_string(),
            "xl/media/image2.png".to_string(),
        ],
        img1.to_vec(),
        img2.to_vec(),
    ))
}
