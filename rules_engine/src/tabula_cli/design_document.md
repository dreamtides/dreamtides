# Tabula CLI Design Document

## Overview

The Tabula CLI is a Rust command-line tool for managing Excel spreadsheets in a
Git-friendly way. It converts data between Excel tables and TOML files, enabling
version control of spreadsheet content while preserving Excel-specific features
like formulas, data validation, and formatting.

**Primary Goals:**
- Extract named Excel Table data into human-readable TOML files for Git
- Reconstruct Excel spreadsheets from TOML using the original XLSM as a template
- Strip/rebuild embedded images to reduce Git storage costs
- Validate round-trip conversions produce valid Excel files

## Dependencies

### Workspace Dependencies (from existing Cargo.toml)
- `anyhow` - Error handling with context
- `clap` v4 - Command-line argument parsing with derive macros
- `serde` - Serialization framework
- `convert_case` - Column name normalization (snake_case, kebab-case)
- `chrono` - Date/time handling
- `sha2` - Image hash computation for caching
- `zip` - Low-level XLSM manipulation for image stripping

### New Dependencies to Add
- `calamine` 0.32 - Reading Excel files ([docs](https://docs.rs/calamine/0.32.0/calamine/))
- `umya-spreadsheet` - Writing/modifying Excel files with formulas
- `toml` - TOML serialization/deserialization
- `reqwest` (with blocking feature) - HTTP client for image downloading
- `tempfile` - Temporary files for atomic writes and validation

## Project Structure

```
src/tabula_cli/
├── Cargo.toml
├── design_document.md
├── src/
│   ├── main.rs                   # CLI entry point, clap command dispatch
│   ├── lib.rs                    # Public API, re-exports
│   ├── core/
│   │   ├── mod.rs
│   │   ├── excel_reader.rs       # Calamine wrapper for reading tables
│   │   ├── excel_writer.rs       # Umya-spreadsheet wrapper for writing
│   │   ├── toml_data.rs          # TOML data structures and serialization
│   │   ├── column_names.rs       # Name normalization utilities
│   │   └── paths.rs              # Default path resolution
│   └── commands/
│       ├── mod.rs
│       ├── build_toml.rs         # XLS -> TOML conversion
│       ├── build_xls.rs          # TOML -> XLS conversion
│       ├── validate.rs           # Round-trip validation
│       ├── strip_images.rs       # Image placeholder replacement
│       ├── rebuild_images.rs     # Image restoration from URLs
│       └── git_setup.rs          # Git hook installation

tests/tabula_cli_tests/
├── Cargo.toml
├── src/lib.rs
├── tests/
│   ├── lib.rs
│   └── tabula_tests/
│       ├── mod.rs
│       ├── build_toml_tests.rs
│       ├── build_xls_tests.rs
│       └── roundtrip_tests.rs
└── test_data/
    ├── simple_table.xlsx
    └── formula_table.xlsx
```

## Command-Line Interface

```bash
# Convert Excel tables to TOML files
tabula build-toml [XLSM_PATH] [OUTPUT_DIR]

# Update Excel from TOML files (requires original XLSM as template)
tabula build-xls [TOML_DIR] [XLSM_PATH]
tabula build-xls --dry-run [TOML_DIR] [XLSM_PATH]

# Validate round-trip conversion
tabula validate [TOML_DIR]
tabula validate --applescript [TOML_DIR]
tabula validate --strip-images [TOML_DIR]

# Manage embedded images
tabula strip-images [XLSM_PATH]
tabula strip-images --output <OUTPUT_XLSM> [XLSM_PATH]
tabula rebuild-images [XLSM_PATH]

# Git integration
tabula git-setup
```

### Default Paths

When paths are omitted, the CLI finds the git root and defaults to:
- Spreadsheet: `client/Assets/StreamingAssets/Tabula.xlsm`
- TOML directory: `client/Assets/StreamingAssets/Tabula/`

## Core Design: No Schema Files

**Key design decision:** We do NOT generate separate metadata or schema files
for formulas, styles, or data validation. Instead:

1. **build-toml** extracts only raw data values into TOML files
2. **build-xls** requires the original XLSM to exist, uses it as a template,
   and updates only the data cells

This approach:
- Keeps version-controlled files minimal (just data)
- Preserves any formatting changes made to the spreadsheet
- Avoids versioning Excel internal details (style IDs, formula syntax variations)
- Ensures VBA macros are always preserved

### TOML Output Format

Each named Excel Table generates one TOML file: `{TableName}.toml`

```toml
[[card]]
name = "Immolate"
id = "d8fe4b2a-088c-4a92-aeb7-d6d4d22fda1a"
energy-cost = 2
rules-text = "{Dissolve} an enemy character."
prompts = "Choose an enemy character to {dissolve}."
card-type = "Event"
subtype = "Fire"
is-fast = true
image-number = 1907487244

[[card]]
name = "Abolish"
id = "d07ac4fa-cc3b-4bb8-8018-de7dc1760f35"
energy-cost = 2
rules-text = "{Prevent} a played enemy card."
prompts = "Choose an enemy card to {prevent}."
card-type = "Event"
is-fast = true
image-number = 1282908322
```

**Column handling:**
- Raw data columns → included in TOML with normalized names (spaces→hyphens, special characters removed)
- Formula columns (detected via `=` prefix or calamine's DataType::Formula) → skipped
- IMAGE columns (detected via `=IMAGE()` pattern) → skipped
- Empty cells → omitted from that row's TOML entry
- Column order in the TOML output follows the Excel table’s data column order (formula/image columns are skipped)
- Single-column tables that match their column name (e.g., `Predicate Types`) are serialized as a simple array under an underscore key (`predicate_types = ["A", "B"]`) instead of array-of-tables

## Technical Strategy: Round-Trip Conversion

### The Template Approach

Rather than regenerating Excel files from scratch (which risks corruption), we
use the **original XLSM as a template**:

1. **build-xls** opens the original `Tabula.xlsm`
2. For each table, it locates the data range
3. It writes only the data cell values from the TOML files
4. Formula columns are left untouched
5. The modified file is saved atomically (write to temp, then rename)

This preserves:
- All Excel internal structure (`.rels` files, content types, etc.)
- Style definitions and conditional formatting
- Data validation rules
- Table structure and calculated column formulas
- VBA macros (`vbaProject.bin`)

### Formula Recalculation Behavior

Excel tables have **calculated columns** where a single formula applies to the
entire column. These are stored in the table XML as `<calculatedColumnFormula>`.

**Important:** When we modify data cells, we must NOT touch formula cells.
Excel will recalculate these columns automatically when:
1. The file is opened in Excel
2. The `calcChain.xml` triggers recalculation

**Verification required during Milestone 2:** We need to test that after
modifying data cells with umya-spreadsheet:
- The table structure remains valid
- Opening the file in Excel triggers formula recalculation
- No "repair file" warnings appear

If this doesn't work automatically, we may need to:
- Clear cached values in formula cells
- Update the `calcChain.xml` to mark cells dirty
- Or use a different strategy (detailed in Observations if needed)

### Column Classification

During `build-toml`, each column is classified:

| Classification | Detection Method | TOML Behavior |
|---------------|------------------|---------------|
| Data | Cell contains string/number/boolean directly | Include in TOML |
| Formula | DataType::Formula from calamine | Skip entirely |
| Image | Formula contains `IMAGE(` | Skip entirely |
| Empty | All cells empty | Skip entirely |

During `build-xls`, we use the same classification from the template:
- Data columns: overwrite with TOML values
- Formula/Image columns: do not modify (preserve formulas)

## Image Handling

### strip-images Command

Replicates `../client/scripts/xlsm_manager.py` functionality. XLSM files are ZIP archives:

1. Extract XLSM to temp directory
2. Find images in `xl/media/` (JPEG, PNG, GIF)
3. Replace each with 1x1 JPEG placeholder
4. Cache originals by SHA-256 hash in `.git/xlsm_image_cache/`
5. Record mapping in `_xlsm_manifest.json` saved alongside the cache (`.git/xlsm_image_cache/_xlsm_manifest.json`, outside the ZIP)
6. Repack ZIP preserving file order and compression

**Critical ZIP details:**
- Preserve exact file ordering (from manifest)
- Images: `ZIP_STORED`, XML: `ZIP_DEFLATED`
- Timestamps: `1980-01-01 00:00:00`
- No zip64
- Do not add manifest or other auxiliary files to the archive

### rebuild-images Command

Restores images, either by:

- Loading them from the image cache in the .git/ directory, or
- Evaluating `=IMAGE("http://prefix"&G2&".jpg")` type formulas:

1. Read table schema to find IMAGE columns
2. Parse formula to extract URL pattern (e.g., `"prefix"&H2&".jpg"`)
3. For each row, substitute cell values to compute full URL
4. Download image via HTTP
5. Replace placeholder in XLSM ZIP structure

**Formula parsing:** Use regex to identify string literals and cell references
in the pattern. This is sufficient for the known `=IMAGE()` patterns in this
project.

## Validate Command Details

The `validate` command performs a round-trip conversion and checks the result:

```bash
tabula validate [TOML_DIR]
```

**Basic operation:**
1. Run `build-xls` to generate a new XLSM in a temp location
2. Run `build-toml` on the generated XLSM to a temp directory
3. Compare the newly-extracted TOML with the original TOML files
4. Report any differences

### --applescript Flag (macOS only)

```bash
tabula validate --applescript [TOML_DIR]
```

Uses AppleScript to verify Excel can open the file without corruption warnings:

1. Generate test XLSM via round-trip
2. Execute AppleScript that:
   - Opens Microsoft Excel
   - Opens the generated XLSM file
   - Waits briefly for any "repair" dialogs
   - Checks if a repair dialog appeared
   - Quits Excel
3. Return exit code 1 if repair dialog was detected

This catches subtle corruption that passes ZIP validation but triggers Excel's
internal consistency checks.

### --strip-images Flag

```bash
tabula validate --strip-images [TOML_DIR]
```

Includes image stripping in the validation workflow:
1. Run `strip-images` on the original XLSM
2. Perform standard round-trip validation
3. Run `rebuild-images`
4. Verify the final XLSM is valid

## Error Handling

All commands use `anyhow` for error context chains:

**build-toml:**
- `Cannot open spreadsheet at {path}: {err}`
- `No named Excel Tables found in {path}. Tables are distinct from worksheets.`
- `Cannot write to output directory {path}: {err}`

**build-xls:**
- `Original XLSM not found at {path}. This file is required as a template.`
- `Table '{name}' not found in original XLSM`
- `TOML file for table '{name}' not found at {path}`
- `Unexpected TOML table '{name}' (not present in template)`
- `Row {n}: column '{col}' does not match any writable column in '{table}'`
- `Row count mismatch for '{name}': TOML has {toml_rows}, template has {template_rows}`
- `Row {n}: column '{col}' value cannot be parsed: {err}`

**validate:**
- `Round-trip failed: TOML differs at table '{name}', row {n}`
- `Excel reported file corruption (detected via AppleScript)`
- `Files differ at byte offset {offset}`

**strip-images:**
- `File {path} is not a valid XLSM archive: {err}`
- `No embedded images found`

**rebuild-images:**
- `Failed to download image from {url}: {err}`
- `Cannot parse URL from formula: {formula}`

## Git Integration

The `tabula git-setup` command configures the repository for the tabula workflow:

1. **Git LFS**: Configures `*.xlsm` files for Git Large File Storage
2. **Pre-commit hook**: Installs a hook that:
   - Creates a backup of Tabula.xlsm in `.git/excel-backups/`
   - Runs `tabula build-toml` to extract data
   - Runs `tabula strip-images` to replace images with placeholders
   - Fails if any TOML file has a newer timestamp than the spreadsheet
3. **Image cache**: Creates `.git/xlsm_image_cache/` for stripped images

The hook ensures the workflow is always: edit spreadsheet → extract to TOML,
never the reverse (which could overwrite spreadsheet changes).


## Key Library APIs

### Calamine (Reading)

Calamine is read-only. Key types and methods for this project:

**Opening a workbook:**
```rust
use calamine::{open_workbook, Xlsx, Reader, Data};

let mut workbook: Xlsx<_> = open_workbook("Tabula.xlsm")?;
```

**Listing sheet names:**
```rust
let sheet_names: Vec<String> = workbook.sheet_names().to_owned();
```

**Reading a worksheet range:**
```rust
if let Ok(range) = workbook.worksheet_range("Sheet1") {
    for row in range.rows() {
        for cell in row {
            match cell {
                Data::Empty => { /* skip */ }
                Data::String(s) => { /* string value */ }
                Data::Float(f) => { /* numeric value */ }
                Data::Int(i) => { /* integer value */ }
                Data::Bool(b) => { /* boolean */ }
                Data::DateTime(dt) => { /* Excel datetime */ }
                Data::Error(e) => { /* cell error like #REF! */ }
            }
        }
    }
}
```

**Accessing named tables:**

The `Table` struct represents an Excel Table (not just a worksheet). Tables have
names, column headers, and defined data ranges.

```rust
use calamine::Table;

let table_names = workbook.table_names();
for name in table_names {
    if let Some(table) = workbook.table_by_name(&name)? {
        let columns: &[String] = table.columns();
        let data: &Range<Data> = table.data();
    }
}
```

**Getting formulas (separate from values):**
```rust
if let Ok(formulas) = workbook.worksheet_formula("Sheet1") {
    for row in formulas.rows() {
        for formula in row {
            if !formula.is_empty() {
                println!("Formula: {formula}");
            }
        }
    }
}
```

### Umya-Spreadsheet (Writing)

Umya-spreadsheet can read AND write xlsx/xlsm files, preserving existing content.

**Reading an existing file:**
```rust
use umya_spreadsheet::reader::xlsx;

let path = std::path::Path::new("Tabula.xlsm");
let mut book = xlsx::read(path)?;
```

**Accessing worksheets:**
```rust
let sheet = book.get_sheet_by_name_mut("Cards")?;
```

**Reading/writing cell values:**
```rust
let cell = sheet.get_cell_mut("B5");
cell.set_value("New Value");

let cell = sheet.get_cell_mut((2, 5));  // (col, row) 1-indexed
cell.set_value(42.0);
cell.set_value(true);
```

**Getting cell value without modification:**
```rust
let cell = sheet.get_cell("A1")?;
let value: &str = cell.get_value();
```

**Saving the workbook:**
```rust
use umya_spreadsheet::writer::xlsx;

let output = std::path::Path::new("output.xlsm");
xlsx::write(&book, output)?;
```

**Important for this project:** When we modify cell values, umya-spreadsheet
preserves all other content including:
- Table definitions and calculated column formulas
- VBA macros (vbaProject.bin)
- Styles, conditional formatting, data validation
- Relationships between sheets and other components

We rely on this preservation behavior for our template-based approach.

### TOML Serialization

**Defining data structures:**
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct CardRow {
    name: String,
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    energy_cost: Option<i64>,
    rules_text: Option<String>,
}
```

**Writing TOML:**
```rust
let cards = vec![CardRow { ... }, CardRow { ... }];
let toml_string = toml::to_string_pretty(&cards)?;
std::fs::write("Cards.toml", toml_string)?;
```

**Reading TOML:**
```rust
let content = std::fs::read_to_string("Cards.toml")?;
let cards: Vec<CardRow> = toml::from_str(&content)?;
```

## Testing Strategy

### Principles

1. **Black-box testing** - Test command-line behavior, not internal functions
2. **Fast execution** - All tests complete in < 3 seconds
3. **Synthetic test data** - Don't depend on production Tabula.xlsm structure
4. **Deterministic** - No randomness, fixed test inputs

### Test Spreadsheets

Create minimal XLSX files programmatically or check in small test files:

- `simple_table.xlsx`: Basic table with 3 columns, 5 rows, no formulas
- `formula_table.xlsx`: Table with a calculated column
- `validation_table.xlsx`: Table with dropdown data validation

### Test Location

Following existing project patterns in `tests/tabula_cli_tests/`.

## Milestone Breakdown

### Milestone 1: Project Setup
**Scope:** Crate structure, Cargo.toml, clap CLI skeleton
- Create `src/tabula_cli/Cargo.toml` with all dependencies
- Set up `main.rs` with clap subcommand stubs
- Create directory structure under `src/`
- Create test crate `tests/tabula_cli_tests/`
- Verify `cargo check` passes

### Milestone 2: Excel Reading with Calamine
**Scope:** Read named tables and classify columns
- Implement `core/excel_reader.rs` using calamine 0.32
- Enumerate named tables via `Reader` trait
- Read cell data and distinguish DataType variants
- Classify columns as data/formula/image
- **Critical:** Verify we can detect calculated column formulas
- Unit tests with synthetic spreadsheet

### Milestone 3: build-toml Command
**Scope:** Convert tables to TOML
- Implement `commands/build_toml.rs`
- Column name normalization (spaces → hyphens)
- Omit empty cells and formula columns
- Create backup in `.git/excel-backups/`
- Integration tests

### Milestone 4: strip-images Command
**Scope:** Replace images with placeholders
- Implement `commands/strip_images.rs`
- ZIP manipulation with correct ordering/compression
- Image caching by SHA-256
- Manifest file generation
- Tests with image-containing spreadsheet

### Milestone 5: build-xls Command (Basic)
**Scope:** Write data cells to template
- Implement `commands/build_xls.rs` using umya-spreadsheet with calamine metadata for table layout
- Require original XLSM as template; default paths match `build-toml`
- Strict table/column alignment: write data columns only, error on missing/extra tables, skip formula/image columns, no row resizing yet
- Convert TOML scalars to cell values, reject unsupported types, treat absent cells as empty
- Support `--dry-run` validation and atomic writes via temp file + rename
- Integration tests for data writes, preserved formulas, and validation errors

### Milestone 6: build-xls Row Handling
**Scope:** Handle row additions/deletions
- Detect when TOML has more/fewer rows than template
- Add new rows at end of table (copy formatting from last row)
- Delete excess rows if TOML has fewer
- Preserve table boundaries and auto-filter

### Milestone 7: validate Command
**Scope:** Round-trip validation
- Implement `commands/validate.rs`
- Orchestrate build-xls and build-toml
- Implement `--strip-images` flag
- Clear error reporting

### Milestone 8: validate --applescript
**Scope:** macOS Excel validation
- Implement AppleScript execution (conditional compilation)
- Detect repair dialog appearance
- Timeout handling
- Skip gracefully on non-macOS

### Milestone 9: rebuild-images Command
**Scope:** Restore images from URLs *or* from .git image cache
- Implement `commands/rebuild_images.rs`
- Restore image from .git cache when requested via flag
- Parse IMAGE formula patterns
- HTTP download via reqwest
- Replace placeholders in ZIP
- Network error handling

### Milestone 10: Git Integration and Polish
**Scope:** git-setup command, final validation
- Implement `commands/git_setup.rs`
- Generate pre-commit hook script
- Configure Git LFS
- Test with production Tabula.xlsm
- Run `just review`

## Workflow Instructions for Implementing Agent

### Validation Process

After each milestone:
1. Run `just fmt` to apply rustfmt
2. Run `just check` to verify type checking
3. Run `just clippy` to check for lint warnings
4. Run `cargo test -p tabula_cli_tests` to execute tests
5. Run `just review` for complete validation before committing

### Style Rules

Follow project `AGENT.md` rules:
- No inline comments in code
- No `pub use` re-exports in lib.rs; use full module paths instead
- Functions qualified with one module: `excel_reader::extract_tables()`
- Do not import functions directly; call them via a module qualifier (e.g., `paths::git_root_for()`)
- Structs unqualified: `TableData`
- Enums one level: `ColumnType::Formula`
- Public items at top of file, private below
- Cargo.toml dependencies alphabetized
- Prefer inline expressions over intermediate `let` bindings

### Recording Context

Create/update `src/tabula_cli/observations.md` for:
- Implementation decisions that differ from this document
- Issues encountered and solutions
- API quirks in calamine or umya-spreadsheet
- Important source files to read for project context
- Test results for critical behaviors (formula recalculation)

Mark completed milestones at the top of this design document.

### Getting More Information

1. Search codebase: `grep` for similar patterns
2. Read crate docs: `cargo doc --open`
3. Examine `xlsm_manager.py` for behavior reference
4. Look at `client/Assets/StreamingAssets/Tabula.xlsm.d/` structure
5. Ask for clarification rather than assuming

# Tabula CLI Design Document

## Milestone Status
- [x] Milestone 1: Project Setup
- [x] Milestone 2: Excel Reading with Calamine
- [x] Milestone 3: build-toml Command
- [x] Milestone 4: strip-images Command
- [x] Milestone 5: build-xls Command (Basic)

## Milestone 5 Plan (build-xls)

### Goals
- Update the XLSM template using TOML data while leaving formulas/images untouched.
- Validate inputs before writing; no row insertion/deletion until Milestone 6.
- Keep writes atomic and preserve workbook structure/macros.

### CLI Behavior
- `tabula build-xls [TOML_DIR] [XLSM_PATH]` uses the same defaults as `build-toml` when paths are omitted and overwrites the template in place.
- `--dry-run` runs all validations (table presence, column mapping, row counts, TOML type checks) without writing output; failures return a non-zero exit code.
- Writes are atomic: save to a temp file in the destination directory, then rename to the final path.

### Template Metadata and TOML Loading
- Reuse `excel_reader` on the template to capture table layout: table name, sheet name, data start cell (1-based col,row), data row count, and ColumnType classification for each column.
- Add a writer helper that pairs raw column names with normalized names (via `column_names`) so TOML keys map back to Excel columns; retain column order from the template.
- Parse TOML directory into a map keyed by normalized table names; accept the array-of-tables format and the single-column array format emitted by `build-toml`.
- Reject TOML rows containing nested tables/arrays; allow only strings, integers, floats, booleans, or empty/missing cells.

### Validation Rules
- Every template table must have a corresponding TOML file; extra TOML files are errors.
- Only data columns are writable; TOML keys that do not match a data column cause an error. Missing data cells are written as empty.
- Row counts must match the template’s data row count; if TOML has more or fewer rows, return an error (row resizing moves to Milestone 6).
- Formula and IMAGE columns are never written, even if TOML includes values for them.

### Write Behavior
- Open the template with umya-spreadsheet once; compute cell coordinates from the calamine metadata and write data column cells row by row.
- Convert TOML scalars to the closest Excel cell type: bool → bool, integer → i64, float → f64, string → string. Empty/missing becomes an empty string.
- After all tables are updated, persist via the atomic save path; ensure existing ZIP structure, relationships, and macros remain intact.

### Testing Focus
- Integration tests that: rewrite a simple table; verify formula columns remain unchanged; confirm `--dry-run` leaves the file untouched; error on row-count mismatches; error on extra/missing tables or unknown columns; reject unsupported TOML types.
- Add a test workbook with at least one formula column to confirm we skip calculated columns while writing adjacent data.

### Follow-Ups
- Confirm Excel recalculates formulas when opening the modified file; this will be exercised further during `validate`/AppleScript work.
