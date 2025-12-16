# Tabula CLI Observations

## Implemented Commands
- `build-toml`: Extracts named tables, normalizes names (kebab-case, special chars removed), preserves data-column order, skips formula/image columns, emits single-column tables as flat arrays, backs up the source XLSM with timestamped copies, and prunes `.git/excel-backups` to the 50 newest files.
- `strip-images`: Replaces `xl/media/*` images with a 1x1 JPEG placeholder, writes `_xlsm_manifest.json` to `.git/xlsm_image_cache/`, caches originals there, preserves original ZIP entry order and compression, and normalizes timestamps to 1980-01-01. Supports `--output` to write to a new XLSM or operates in place via a temp file.
- `build-xls`: Loads table layouts via calamine (sheet name, data start cell, data row count, column types), validates TOML against template tables/columns, and writes only data columns with umya-spreadsheet. Requires matching row counts and errors on missing/extra tables or unknown columns. Accepts only scalar TOML values (string/int/float/bool); missing cells become empty strings. Supports `--dry-run` and atomic writes using a temp file that preserves the source extension (prevents umya panics on unknown formats). Formula and IMAGE columns are always skipped.

## Defaults and Paths
- Git root discovered by walking up to `.git`.
- Defaults: XLSM `client/Assets/StreamingAssets/Tabula.xlsm`, TOML dir `client/Assets/StreamingAssets/Tabula`, backup dir `.git/excel-backups`, image cache `.git/xlsm_image_cache`.

## build-xls Behavior Notes
- TOML parsing accepts array-of-tables and the single-column array shape emitted by build-toml (underscored keys are normalized). Duplicate normalized table names in TOML are rejected.
- Validation requires a TOML file for every template table; extra TOML tables fail fast. Row counts must match the template exactly; row resizing is deferred to Milestone 6.
- Writable columns are template data columns only; TOML keys that do not match a writable column produce an error. Data is written in template column order; missing cells are written as empty strings.
- Value conversion: bool→bool, integer/float→number, string→string. Unsupported types (arrays/tables/datetimes) raise `Row {n}: column '{col}' value cannot be parsed: unsupported type`.
- Atomic write: modifies the workbook in memory and writes to a temp file with the original extension in the destination directory, then renames; `--dry-run` skips the write after validation.

## Testing Coverage
- build-toml: column order, special-char stripping, single-column array encoding, backup creation and pruning.
- strip-images: placeholder substitution, manifest contents, cache writes, preserved entry order/compression.
- build-xls: rewrites data columns and leaves formulas intact, dry-run leaves file unchanged, errors on row-count mismatch and unknown columns, supports single-column array input.

## Risks and Follow-Ups
- Row insertion/deletion still unimplemented (Milestone 6).
- Rebuild-images, validate (including AppleScript), and git-setup remain unimplemented.
- Need real-world confirmation that Excel recalculates formulas without warnings on modified files; to verify during validate milestones.
