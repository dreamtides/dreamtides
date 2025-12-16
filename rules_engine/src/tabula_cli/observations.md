# Tabula CLI Observations

Working context for agents implementing Milestones 4+.

## Project Structure

```
src/tabula_cli/
├── Cargo.toml
├── src/
│   ├── main.rs                   # CLI dispatch; only build-toml implemented
│   ├── lib.rs
│   ├── core/
│   │   ├── column_names.rs       # kebab-case normalization for table/column names
│   │   ├── excel_reader.rs       # Table extraction and column classification
│   │   ├── excel_writer.rs       # Empty stub
│   │   ├── paths.rs              # Git root discovery and default paths
│   │   └── toml_data.rs          # TableInfo to TOML serialization
│   └── commands/
│       ├── build_toml.rs         # Implements build-toml conversion + backups
│       ├── build_xls.rs          # Empty stub
│       ├── git_setup.rs          # Empty stub
│       ├── rebuild_images.rs     # Empty stub
│       ├── strip_images.rs       # Empty stub
│       └── validate.rs           # Empty stub
└── tests/tabula_cli_tests/
    ├── Cargo.toml
    ├── src/lib.rs
    ├── src/tabula_cli_test_utils.rs
    └── tests/
        ├── lib.rs
        └── tabula_cli_tests/
            ├── basic_tabula_tests.rs
            └── build_toml_tests.rs
```

## Dependencies Available

Workspace dependencies include calamine, umya-spreadsheet, toml, serde, convert_case, tempfile, sha2, zip, reqwest (blocking), chrono, clap, anyhow.

## Important Implementation Notes

- CLI dispatch now returns errors for unimplemented commands; only build-toml executes real work.
- Paths: `paths::git_root` and `git_root_for` search upward for a `.git` directory; defaults derive spreadsheet path (`client/Assets/StreamingAssets/Tabula.xlsm`) and TOML directory (`client/Assets/StreamingAssets/Tabula`). Backups live under `git_root/.git/excel-backups`.
- Column naming: `normalize_column_name` and `normalize_table_name` convert to kebab-case via convert_case and strip special characters (collapse punctuation to single hyphens, drop trailing hyphens). Single-column tables where the column matches the table name are emitted as a flat array keyed by the table name with hyphens replaced by underscores (e.g., `predicate_types = ["ThisCard", ...]`). Backups are pruned to keep only the 50 most recent files by name.
- Excel reader: `extract_tables` still requires `workbook.load_tables()`. `classify_column` inspects worksheet formulas using relative coordinates from `worksheet_formula` plus a fallback for strings starting with `=`; IMAGE formulas detected case-insensitively. Non-data columns are skipped when building rows. Floats with no fractional component are stored as `CellValue::Int`.
- TOML serialization: `toml_data::table_to_toml` builds an array-of-tables keyed by the normalized table name. Empty cells are omitted. Floats serialize as integers when possible. Column order in TOML follows the source Excel table’s data column order (non-data columns are skipped), enabled by the `toml` crate’s `preserve_order` feature.
- build-toml command: resolves paths from CLI args or defaults, creates the backup directory if needed, writes timestamped backups named `{timestamp}-{original_name}` into `.git/excel-backups`, then emits `{normalized_table}.toml` files into the output directory (auto-created). Errors when writing TOML are surfaced as `Cannot write to output directory {path}: ...`.

## Tests

- `basic_tabula_tests.rs` covers table discovery, column classification (including empty column), cell value extraction, and the error on missing named tables.
- `build_toml_tests.rs` verifies TOML output skips formula/empty columns, preserves data values, preserves column order, and that backups are written to `.git/excel-backups`. Tests create a `.git` directory in a temp location so git-root discovery succeeds.
- Helpers in `src/tabula_cli_test_utils.rs` build synthetic spreadsheets with a named table plus formula columns.

## Verification Commands

Preferred workflow remains: `just fmt`, `just check`, `just clippy`, `cargo test -p tabula_cli_tests`, `just review`.

## Next Steps Toward Milestone 4

- Implement `commands/build_xls.rs` using umya-spreadsheet, reusing column normalization and classification from the template XLSM.
- Extend tests for TOML to XLSM writing and ensure formula columns remain untouched.
