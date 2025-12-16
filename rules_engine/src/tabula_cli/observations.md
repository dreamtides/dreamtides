# Tabula CLI Observations

Working context for agents implementing Milestones 2+.

## Project Structure

```
src/tabula_cli/
├── Cargo.toml                    # Dependencies configured
├── src/
│   ├── main.rs                   # CLI entry point with clap subcommands
│   ├── lib.rs                    # Module declarations
│   ├── core/
│   │   ├── mod.rs
│   │   ├── column_names.rs       # Empty stub
│   │   ├── excel_reader.rs       # Empty stub
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
- Some clippy lints behave differently (e.g., `#[expect(...)]` instead of `#[allow(...)]`)

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
The test crate `tests/tabula_cli_tests/` is set up but has empty stub files:
- `src/tabula_cli_test_utils.rs` - Utility functions for tests
- `tests/tabula_cli_tests/basic_tabula_tests.rs` - Actual tests

Tests should follow the pattern from `design_document.md`:
- Black-box testing of command-line behavior
- Fast execution (<3 seconds)
- Use synthetic test spreadsheets

### Verification Commands
```bash
cargo check -p tabula_cli
cargo clippy -p tabula_cli
cargo run -p tabula_cli -- --help
cargo test -p tabula_cli_tests
```

## Next Steps for Milestone 2

Milestone 2 focuses on Excel reading with Calamine:
1. Implement `core/excel_reader.rs`
2. Use calamine 0.32 to enumerate named tables
3. Classify columns as data/formula/image
4. Create unit tests with synthetic spreadsheets

Key calamine APIs to use:
- `open_workbook()` - Open XLSM file
- `workbook.table_names()` - List named tables
- `workbook.table_by_name()` - Get table data
- `workbook.worksheet_formula()` - Get formulas (separate from values)
