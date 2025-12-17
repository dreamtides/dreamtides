# Tabula CLI Observations

- `validate` builds a temp XLSM from TOML plus the template via `build_xls`, extracts tables back to temp TOML with `excel_reader`/`toml_data`, compares TOML, then compares semantic workbook snapshots via `umya-spreadsheet`; `--applescript` and `--strip-images` still bail as unimplemented.
- Snapshots cover sheet order, table placement/area/columns/style/totals-row, column widths/hidden/best-fit, row heights/hidden/custom, data validations, conditional formatting rules, and cell alignment/wrap. Extra column/row dimensions or alignments that exist only in the round-trip are ignored; mismatches are reported only when the template provided explicit values.
- Alignment diffs include horiz/vert/wrap details when `--verbose`; `--verbose` also expands table mismatch details. `--all` reports every finding instead of stopping at the first.
- Column dimension errors now include both the column index and letter plus expected/actual widths; row dimension errors include expected height/hidden/custom state when missing in the output.
- Direct `quick-xml` usage was removed; `umya-spreadsheet` is the primary reader for validation while `calamine` still extracts table data for TOML export.
- Always run `just fmt`, `just check`, `just clippy`, and `just review` before handing off changes.
