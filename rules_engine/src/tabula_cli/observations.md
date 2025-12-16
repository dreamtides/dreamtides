# Tabula CLI Observations

Working context for agents implementing Milestones 3+.

## Project Structure

```
src/tabula_cli/
├── Cargo.toml                    # Dependencies configured
├── src/
│   ├── main.rs                   # CLI entry point with clap subcommands
│   ├── lib.rs                    # Module declarations + re-exports
│   ├── core/
│   │   ├── mod.rs
│   │   ├── column_names.rs       # Empty stub
│   │   ├── excel_reader.rs       # ✓ Implemented - table extraction
│   │   ├── excel_writer.rs       # Empty stub
│   │   ├── paths.rs              # Empty stub
│   │   └── toml_data.rs          # Empty stub
│   └── commands/
│       ├── mod.rs
│       ├── build_toml.rs         # Empty stub
│       ├── build_xls.rs          # Empty stub
│       ├── git_setup.rs          # Empty stub
│       ├── rebuild_images.rs     # Empty stub
│       ├── strip_images.rs       # Empty stub
│       └── validate.rs           # Empty stub
```

## Dependencies Available

From workspace Cargo.toml:
- **calamine** v0.32 - Reading Excel files
- **umya-spreadsheet** v2 - Writing/modifying Excel files
- **toml** v0.8 - TOML serialization
- **reqwest** v0.12 (blocking feature) - HTTP client
- **tempfile** v3 - Temporary files
- **sha2** v0.10 - Hash computation
- **zip** v4 - XLSM ZIP manipulation
- **chrono** v0.4 - Date/time
- **convert_case** v0.8 - Column name normalization

## Important Implementation Notes

### Rust Edition 2024
This project uses `edition = "2024"`. Note that:
- `use` statements in 2024 edition require explicit imports
- Some clippy lints behave differently (e.g., `#[expect(...)]` instead of `#[allow(...)`)

### CLI Structure
The CLI uses clap derive macros with kebab-case subcommands:
- `build-toml` → `Commands::BuildToml`
- `build-xls` → `Commands::BuildXls`
- `strip-images` → `Commands::StripImages`
- `rebuild-images` → `Commands::RebuildImages`
- `git-setup` → `Commands::GitSetup`

Each command currently prints a placeholder message. Actual implementation should:
1. Add logic to the corresponding `commands/*.rs` file
2. Call that function from the match arm in `main.rs`

### Test Crate
The test crate `tests/tabula_cli_tests/` has:
- `src/tabula_cli_test_utils.rs` - Helper to create synthetic XLSX files with tables
- `tests/tabula_cli_tests/basic_tabula_tests.rs` - 4 passing tests for excel_reader

### Verification Commands
```bash
cargo check -p tabula_cli
cargo clippy -p tabula_cli
cargo run -p tabula_cli -- --help
cargo test -p tabula_cli_tests
```

## Milestone 2 Completion Notes

### Excel Reader (`core/excel_reader.rs`)

Implemented types and functions:
- `ColumnType` enum: `Data`, `Formula`, `Image`, `Empty`
- `CellValue` enum: `Empty`, `String`, `Float`, `Int`, `Bool`
- `ColumnInfo` struct: column name + type
- `TableInfo` struct: name, columns, rows
- `extract_tables(path) -> Result<Vec<TableInfo>>` - main extraction function

### Critical Calamine API Details

**MUST call `load_tables()` before accessing table data:**
```rust
let mut workbook: Xlsx<_> = open_workbook(path)?;
workbook.load_tables()?;  // Required!
let table_names = workbook.table_names();
```

**Table data coordinate system:**
- `table.data().start()` returns (row, col) in 0-indexed worksheet coordinates
- The data range includes only data rows, not the header
- `table.columns()` returns column headers from the table definition

**Formula detection limitations:**
- `workbook.worksheet_formula()` returns formulas in a separate Range
- Formulas must be looked up by absolute worksheet coordinates (row, col)
- Not all formula formats are readable (umya-spreadsheet set_formula() may not produce compatible output)
- For real Excel files, formula detection should work correctly

### umya-spreadsheet Table Creation

When creating tables for testing:
```rust
let mut table = Table::default();
table.set_name("TableName");
table.set_display_name("TableName");
table.set_area(("A1", "E3"));  // Tuple format, not "A1:E3"

// Must explicitly add columns:
let mut col = TableColumn::default();
col.set_name("ColumnName".to_string());  // Requires String, not &str
table.add_column(col);
```

## Next Steps for Milestone 3

Milestone 3 focuses on the `build-toml` command:
1. Implement `commands/build_toml.rs`
2. Use `excel_reader::extract_tables()` to get table data
3. Implement column name normalization (spaces → hyphens) in `core/column_names.rs`
4. Serialize to TOML format using dynamic keys
5. Create backup in `.git/excel-backups/` before modifying
6. Integration tests

Key types to reuse from lib.rs exports:
- `extract_tables`, `TableInfo`, `ColumnInfo`, `CellValue`, `ColumnType`
