use std::fs;
use std::io::{Read, Write};

use calamine::{self, Data};
use roxmltree::Document;
use tabula_cli::commands::build_xls;
use tabula_cli_tests::tabula_cli_test_utils;
use tempfile::TempDir;
use zip::{CompressionMethod, DateTime, ZipArchive, ZipWriter};

#[test]
fn build_xls_writes_data_and_preserves_formulas() {
    let temp_dir = TempDir::new().expect("temp dir");
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir_all(&git_dir).expect("git dir");

    let xlsx_path = temp_dir.path().join("test_table.xlsx");
    tabula_cli_test_utils::create_test_spreadsheet_with_table(&xlsx_path).expect("spreadsheet");

    let toml_dir = temp_dir.path().join("toml");
    fs::create_dir_all(&toml_dir).expect("toml dir");
    let toml = r#"
[[test-table]]
name = "Carol"
count = 5
active = false

[[test-table]]
name = "Dave"
count = 7
active = true
"#;
    fs::write(toml_dir.join("test-table.toml"), toml).expect("write toml");

    build_xls::build_xls(false, Some(toml_dir), Some(xlsx_path.clone()), Some(xlsx_path.clone()))
        .expect("build-xls");

    let mut workbook: calamine::Xlsx<_> = calamine::open_workbook(&xlsx_path).expect("open");
    workbook.load_tables().expect("tables");
    let table = workbook.table_by_name("TestTable").expect("table");
    let data = table.data();

    assert!(matches!(data.get((0, 0)), Some(Data::String(s)) if s == "Carol"));
    assert!(matches!(data.get((1, 0)), Some(Data::String(s)) if s == "Dave"));
    assert!(matches!(data.get((0, 1)), Some(Data::Float(f)) if (*f - 5.0).abs() < f64::EPSILON));
    assert!(matches!(data.get((1, 1)), Some(Data::Float(f)) if (*f - 7.0).abs() < f64::EPSILON));
    assert!(matches!(data.get((0, 2)), Some(Data::Bool(b)) if !b));
    assert!(matches!(data.get((1, 2)), Some(Data::Bool(b)) if *b));

    let book = umya_spreadsheet::reader::xlsx::read(&xlsx_path).expect("read umya");
    let sheet = book.get_sheet_by_name("TestSheet").expect("sheet");
    assert_eq!(sheet.get_cell("D2").map(|c| c.get_formula()).unwrap_or(""), "=B2*2");
    assert_eq!(sheet.get_cell("D3").map(|c| c.get_formula()).unwrap_or(""), "=B3*2");
}

#[test]
fn build_xls_dry_run_leaves_file_unchanged() {
    let temp_dir = TempDir::new().expect("temp dir");
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir_all(&git_dir).expect("git dir");

    let xlsx_path = temp_dir.path().join("test_table.xlsx");
    tabula_cli_test_utils::create_test_spreadsheet_with_table(&xlsx_path).expect("spreadsheet");
    let before = fs::read(&xlsx_path).expect("read before");

    let toml_dir = temp_dir.path().join("toml");
    fs::create_dir_all(&toml_dir).expect("toml dir");
    let toml = r#"
[[test-table]]
name = "Carol"
count = 5
active = false

[[test-table]]
name = "Dave"
count = 7
active = true
"#;
    fs::write(toml_dir.join("test-table.toml"), toml).expect("write toml");

    let result = build_xls::build_xls(
        true,
        Some(toml_dir),
        Some(xlsx_path.clone()),
        Some(xlsx_path.clone()),
    );
    assert!(result.is_ok());

    let after = fs::read(&xlsx_path).expect("read after");
    assert_eq!(before, after);
}

#[test]
fn build_xls_errors_on_unknown_columns() {
    let temp_dir = TempDir::new().expect("temp dir");
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir_all(&git_dir).expect("git dir");

    let xlsx_path = temp_dir.path().join("test_table.xlsx");
    tabula_cli_test_utils::create_test_spreadsheet_with_table(&xlsx_path).expect("spreadsheet");

    let toml_dir = temp_dir.path().join("toml");
    fs::create_dir_all(&toml_dir).expect("toml dir");
    let toml = r#"
[[test-table]]
name = "Carol"
count = 5
active = false
computed = "nope"

[[test-table]]
name = "Dave"
count = 7
active = true
computed = "nope"
"#;
    fs::write(toml_dir.join("test-table.toml"), toml).expect("write toml");

    let result =
        build_xls::build_xls(false, Some(toml_dir), Some(xlsx_path.clone()), Some(xlsx_path));
    assert!(result.unwrap_err().to_string().contains("does not match any writable column"));
}

#[test]
fn build_xls_writes_single_column_arrays() {
    let temp_dir = TempDir::new().expect("temp dir");
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir_all(&git_dir).expect("git dir");

    let xlsx_path = temp_dir.path().join("predicate_types.xlsx");
    tabula_cli_test_utils::create_predicate_types_spreadsheet(&xlsx_path).expect("spreadsheet");

    let toml_dir = temp_dir.path().join("toml");
    fs::create_dir_all(&toml_dir).expect("toml dir");
    let toml = r#"
predicate_types = ["One", "Two", "Three"]
"#;
    fs::write(toml_dir.join("predicate-types.toml"), toml).expect("write toml");

    build_xls::build_xls(false, Some(toml_dir), Some(xlsx_path.clone()), Some(xlsx_path.clone()))
        .expect("build-xls");

    let mut workbook: calamine::Xlsx<_> = calamine::open_workbook(&xlsx_path).expect("open");
    workbook.load_tables().expect("tables");
    let table = workbook.table_by_name("Predicate Types").expect("table");
    let data = table.data();

    assert!(matches!(data.get((0, 0)), Some(Data::String(s)) if s == "One"));
    assert!(matches!(data.get((1, 0)), Some(Data::String(s)) if s == "Two"));
    assert!(matches!(data.get((2, 0)), Some(Data::String(s)) if s == "Three"));
}

#[test]
fn build_xls_ignores_trailing_blank_rows_and_writes_data_rows() {
    let temp_dir = TempDir::new().expect("temp dir");
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir_all(&git_dir).expect("git dir");

    let xlsx_path = temp_dir.path().join("trailing.xlsx");
    tabula_cli_test_utils::create_table_with_trailing_blank_row(&xlsx_path).expect("spreadsheet");

    let toml_dir = temp_dir.path().join("toml");
    fs::create_dir_all(&toml_dir).expect("toml dir");
    let toml = r#"
[[trailing]]
name = "Alpha"
value = 10

[[trailing]]
name = "Beta"
value = 20
"#;
    fs::write(toml_dir.join("trailing.toml"), toml).expect("write toml");

    build_xls::build_xls(false, Some(toml_dir), Some(xlsx_path.clone()), Some(xlsx_path.clone()))
        .expect("build-xls");

    let mut workbook: calamine::Xlsx<_> = calamine::open_workbook(&xlsx_path).expect("open");
    workbook.load_tables().expect("tables");
    let table = workbook.table_by_name("Trailing").expect("table");
    let data = table.data();
    assert!(matches!(data.get((0, 0)), Some(Data::String(s)) if s == "Alpha"));
    assert!(matches!(data.get((1, 0)), Some(Data::String(s)) if s == "Beta"));
    assert!(matches!(data.get((0, 1)), Some(Data::Float(f)) if (*f - 10.0).abs() < f64::EPSILON));
    assert!(matches!(data.get((1, 1)), Some(Data::Float(f)) if (*f - 20.0).abs() < f64::EPSILON));
    assert!(matches!(data.get((2, 0)), Some(Data::Empty)));
}

#[test]
fn build_xls_removes_excess_rows() {
    let temp_dir = TempDir::new().expect("temp dir");
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir_all(&git_dir).expect("git dir");

    let xlsx_path = temp_dir.path().join("trailing.xlsx");
    tabula_cli_test_utils::create_table_with_trailing_blank_row(&xlsx_path).expect("spreadsheet");

    let toml_dir = temp_dir.path().join("toml");
    fs::create_dir_all(&toml_dir).expect("toml dir");
    let toml = r#"
[[trailing]]
name = "Alpha"
value = 10
"#;
    fs::write(toml_dir.join("trailing.toml"), toml).expect("write toml");

    build_xls::build_xls(false, Some(toml_dir), Some(xlsx_path.clone()), Some(xlsx_path.clone()))
        .expect("build-xls");

    let mut workbook: calamine::Xlsx<_> = calamine::open_workbook(&xlsx_path).expect("open");
    workbook.load_tables().expect("tables");
    let table = workbook.table_by_name("Trailing").expect("table");
    let data = table.data();
    assert!(matches!(data.get((0, 0)), Some(Data::String(s)) if s == "Alpha"));
    assert!(matches!(data.get((0, 1)), Some(Data::Float(f)) if (*f - 10.0).abs() < f64::EPSILON));
    assert!(matches!(data.get((1, 0)), Some(Data::Empty)));
    assert!(matches!(data.get((1, 1)), Some(Data::Empty)));
}

#[test]
fn build_xls_preserves_metadata_and_relationships() {
    let temp_dir = TempDir::new().expect("temp dir");
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir_all(&git_dir).expect("git dir");

    let xlsx_path = temp_dir.path().join("metadata.xlsx");
    tabula_cli_test_utils::create_test_spreadsheet_with_table(&xlsx_path).expect("spreadsheet");
    let metadata_bytes =
        br#"<metadata xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"/>"#;

    let file = fs::File::open(&xlsx_path).expect("open");
    let mut archive = ZipArchive::new(file).expect("zip");
    let mut entries = Vec::new();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).expect("entry");
        let mut data = Vec::new();
        entry.read_to_end(&mut data).expect("read");
        entries.push((entry.name().to_string(), data, entry.compression(), entry.is_dir()));
    }
    let mut content_types = String::new();
    let mut workbook_rels = String::new();
    for (name, data, _compression, _is_dir) in &entries {
        if name == "[Content_Types].xml" {
            content_types = String::from_utf8_lossy(data).to_string();
        }
        if name == "xl/_rels/workbook.xml.rels" {
            workbook_rels = String::from_utf8_lossy(data).to_string();
        }
    }
    content_types = content_types.replace(
        "</Types>",
        r#"<Override PartName="/xl/metadata.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheetMetadata+xml"/></Types>"#,
    );
    workbook_rels = workbook_rels.replace(
        "</Relationships>",
        r#"<Relationship Id="rIdMeta" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/sheetMetadata" Target="metadata.xml"/></Relationships>"#,
    );

    let time = DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0).expect("zip time");
    let deflated = zip::write::FileOptions::<()>::default()
        .compression_method(CompressionMethod::Deflated)
        .last_modified_time(time);
    let stored = zip::write::FileOptions::<()>::default()
        .compression_method(CompressionMethod::Stored)
        .last_modified_time(time);
    let rewrite = fs::File::create(temp_dir.path().join("rewrite.xlsx")).expect("create");
    let mut writer = ZipWriter::new(rewrite);
    for (name, data, compression, is_dir) in entries {
        let options = match compression {
            CompressionMethod::Deflated => deflated,
            CompressionMethod::Stored => stored,
            other => zip::write::FileOptions::<()>::default()
                .compression_method(other)
                .last_modified_time(time),
        };
        if is_dir {
            writer.add_directory(name, options).expect("dir");
        } else if name == "[Content_Types].xml" {
            writer.start_file(name, options).expect("types");
            writer.write_all(content_types.as_bytes()).expect("write types");
        } else if name == "xl/_rels/workbook.xml.rels" {
            writer.start_file(name, options).expect("rels");
            writer.write_all(workbook_rels.as_bytes()).expect("write rels");
        } else {
            writer.start_file(name, options).expect("file");
            writer.write_all(&data).expect("write");
        }
    }
    writer.start_file("xl/metadata.xml", deflated).expect("metadata entry");
    writer.write_all(metadata_bytes).expect("write metadata");
    writer.finish().expect("finish");
    fs::rename(temp_dir.path().join("rewrite.xlsx"), &xlsx_path).expect("rename");

    let toml_dir = temp_dir.path().join("toml");
    fs::create_dir_all(&toml_dir).expect("toml dir");
    let toml = r#"
[[test-table]]
name = "Eve"
count = 3
active = true
"#;
    fs::write(toml_dir.join("test-table.toml"), toml).expect("write toml");

    build_xls::build_xls(false, Some(toml_dir), Some(xlsx_path.clone()), Some(xlsx_path.clone()))
        .expect("build-xls");

    let mut archive = ZipArchive::new(fs::File::open(&xlsx_path).expect("open")).expect("zip");
    let mut metadata_out = Vec::new();
    archive
        .by_name("xl/metadata.xml")
        .expect("metadata")
        .read_to_end(&mut metadata_out)
        .expect("read metadata");
    assert_eq!(metadata_out, metadata_bytes);
    let mut types_out = String::new();
    archive
        .by_name("[Content_Types].xml")
        .expect("types")
        .read_to_string(&mut types_out)
        .expect("read types");
    assert!(types_out.contains(r#"PartName="/xl/metadata.xml""#));
    let mut rels_out = String::new();
    archive
        .by_name("xl/_rels/workbook.xml.rels")
        .expect("rels")
        .read_to_string(&mut rels_out)
        .expect("read rels");
    Document::parse(&rels_out).expect("rels xml valid");
    assert!(rels_out.contains(r#"Id="rIdMeta""#));
}

#[test]
fn build_xls_does_not_shift_cells_outside_single_table() {
    let temp_dir = TempDir::new().expect("temp dir");
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir_all(&git_dir).expect("git dir");

    let xlsx_path = temp_dir.path().join("single.xlsx");
    tabula_cli_test_utils::create_single_table_with_note(&xlsx_path).expect("spreadsheet");

    let toml_dir = temp_dir.path().join("toml");
    fs::create_dir_all(&toml_dir).expect("toml dir");
    let toml = r#"
[[single]]
name = "Alpha"
value = 1

[[single]]
name = "Beta"
value = 2

[[single]]
name = "Gamma"
value = 3
"#;
    fs::write(toml_dir.join("single.toml"), toml).expect("write toml");

    build_xls::build_xls(false, Some(toml_dir), Some(xlsx_path.clone()), Some(xlsx_path.clone()))
        .expect("build-xls");

    let mut workbook: calamine::Xlsx<_> = calamine::open_workbook(&xlsx_path).expect("open");
    workbook.load_tables().expect("tables");
    let table = workbook.table_by_name("Single").expect("table");
    let data = table.data();
    let names: Vec<_> = data
        .rows()
        .map(|row| match row.get(0) {
            Some(Data::String(s)) => s.clone(),
            _ => "".to_string(),
        })
        .collect();
    assert_eq!(names, vec!["Alpha".to_string(), "Beta".to_string(), "Gamma".to_string()]);

    let book = umya_spreadsheet::reader::xlsx::read(&xlsx_path).expect("read umya");
    let sheet = book.get_sheet_by_name("Single").expect("sheet");
    assert_eq!(sheet.get_value("A10"), "Note");
}

#[test]
fn build_xls_updates_table_ranges_on_multi_table_sheet() {
    let temp_dir = TempDir::new().expect("temp dir");
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir_all(&git_dir).expect("git dir");

    let xlsx_path = temp_dir.path().join("multi.xlsx");
    tabula_cli_test_utils::create_spreadsheet_with_two_tables(&xlsx_path).expect("spreadsheet");

    let toml_dir = temp_dir.path().join("toml");
    fs::create_dir_all(&toml_dir).expect("toml dir");
    let toml = r#"
[[primary]]
name = "First"
value = 1

[[primary]]
name = "Second"
value = 2

[[primary]]
name = "Third"
value = 3

[[secondary]]
kind = "Alpha"
score = 10

[[secondary]]
kind = "Beta"
score = 20
"#;
    fs::write(toml_dir.join("tables.toml"), toml).expect("write toml");

    build_xls::build_xls(false, Some(toml_dir), Some(xlsx_path.clone()), Some(xlsx_path.clone()))
        .expect("build-xls");

    let mut workbook: calamine::Xlsx<_> = calamine::open_workbook(&xlsx_path).expect("open");
    workbook.load_tables().expect("tables");
    let primary = workbook.table_by_name("Primary").expect("primary table");
    let primary_data = primary.data();
    assert_eq!(primary_data.height(), 3);
    assert!(matches!(primary_data.get((2, 0)), Some(Data::String(s)) if s == "Third"));

    let secondary = workbook.table_by_name("Secondary").expect("secondary table");
    let secondary_data = secondary.data();
    assert_eq!(secondary_data.height(), 2);
    assert!(matches!(secondary_data.get((0, 0)), Some(Data::String(s)) if s == "Alpha"));
    assert!(matches!(secondary_data.get((1, 0)), Some(Data::String(s)) if s == "Beta"));
}

#[test]
fn build_xls_handles_side_by_side_tables_without_shifting() {
    let temp_dir = TempDir::new().expect("temp dir");
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir_all(&git_dir).expect("git dir");

    let xlsx_path = temp_dir.path().join("parallel.xlsx");
    tabula_cli_test_utils::create_side_by_side_tables(&xlsx_path).expect("spreadsheet");

    let toml_dir = temp_dir.path().join("toml");
    fs::create_dir_all(&toml_dir).expect("toml dir");
    let toml = r#"
[[left]]
left-name = "L1"
left-value = 1

[[left]]
left-name = "L2"
left-value = 2

[[right]]
right-name = "R1"
right-score = 10

[[right]]
right-name = "R2"
right-score = 20

[[right]]
right-name = "R3"
right-score = 30

[[right]]
right-name = "R4"
right-score = 40
"#;
    fs::write(toml_dir.join("tables.toml"), toml).expect("write toml");

    build_xls::build_xls(false, Some(toml_dir), Some(xlsx_path.clone()), Some(xlsx_path.clone()))
        .expect("build-xls");

    let mut workbook: calamine::Xlsx<_> = calamine::open_workbook(&xlsx_path).expect("open");
    workbook.load_tables().expect("tables");

    let left = workbook.table_by_name("Left").expect("left table");
    let left_data = left.data();
    assert_eq!(left_data.height(), 2);
    assert!(matches!(left_data.get((0, 0)), Some(Data::String(s)) if s == "L1"));
    assert!(matches!(left_data.get((1, 0)), Some(Data::String(s)) if s == "L2"));

    let right = workbook.table_by_name("Right").expect("right table");
    let right_data = right.data();
    assert_eq!(right_data.height(), 4);
    assert!(matches!(right_data.get((3, 0)), Some(Data::String(s)) if s == "R4"));

    let book = umya_spreadsheet::reader::xlsx::read(&xlsx_path).expect("read umya");
    let sheet = book.get_sheet_by_name("Parallel").expect("sheet");
    assert_eq!(sheet.get_value("A8"), "Note");
}

#[test]
fn build_xls_can_write_to_new_output_path() {
    let temp_dir = TempDir::new().expect("temp dir");
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir_all(&git_dir).expect("git dir");

    let template_path = temp_dir.path().join("test_table.xlsx");
    tabula_cli_test_utils::create_test_spreadsheet_with_table(&template_path).expect("spreadsheet");
    let original_bytes = fs::read(&template_path).expect("read template");

    let toml_dir = temp_dir.path().join("toml");
    fs::create_dir_all(&toml_dir).expect("toml dir");
    let toml = r#"
[[test-table]]
name = "Eve"
count = 9
active = true

[[test-table]]
name = "Frank"
count = 11
active = false
"#;
    fs::write(toml_dir.join("test-table.toml"), toml).expect("write toml");

    let output_path = temp_dir.path().join("updated.xlsx");
    build_xls::build_xls(
        false,
        Some(toml_dir),
        Some(template_path.clone()),
        Some(output_path.clone()),
    )
    .expect("build-xls");

    let mut workbook: calamine::Xlsx<_> =
        calamine::open_workbook(&output_path).expect("open output");
    workbook.load_tables().expect("tables");
    let table = workbook.table_by_name("TestTable").expect("table");
    let data = table.data();
    assert!(matches!(data.get((0, 0)), Some(Data::String(s)) if s == "Eve"));
    assert!(matches!(data.get((1, 0)), Some(Data::String(s)) if s == "Frank"));

    let template_after = fs::read(&template_path).expect("read template after");
    assert_eq!(original_bytes, template_after);
}

#[test]
fn build_xls_marks_workbook_for_recalc() {
    let temp_dir = TempDir::new().expect("temp dir");
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir_all(&git_dir).expect("git dir");

    let xlsx_path = temp_dir.path().join("test_table.xlsx");
    tabula_cli_test_utils::create_test_spreadsheet_with_table(&xlsx_path).expect("spreadsheet");

    let toml_dir = temp_dir.path().join("toml");
    fs::create_dir_all(&toml_dir).expect("toml dir");
    let toml = r#"
[[test-table]]
name = "Carol"
count = 5
active = false
"#;
    fs::write(toml_dir.join("test-table.toml"), toml).expect("write toml");

    build_xls::build_xls(false, Some(toml_dir), Some(xlsx_path.clone()), Some(xlsx_path.clone()))
        .expect("build-xls");

    let file = fs::File::open(&xlsx_path).expect("open output");
    let mut archive = ZipArchive::new(file).expect("zip");
    assert!(archive.by_name("xl/calcChain.xml").is_err());

    {
        let mut workbook_rels =
            archive.by_name("xl/_rels/workbook.xml.rels").expect("workbook rels");
        let mut rels_contents = String::new();
        workbook_rels.read_to_string(&mut rels_contents).expect("read rels");
        assert!(!rels_contents.contains("calcChain"));
    }

    let mut workbook = archive.by_name("xl/workbook.xml").expect("workbook");
    let mut workbook_contents = String::new();
    workbook.read_to_string(&mut workbook_contents).expect("read workbook");
    assert!(workbook_contents.contains("fullCalcOnLoad=\"1\""));
    assert!(workbook_contents.contains("calcMode=\"auto\""));
}
