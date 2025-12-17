# Tabula CLI Observations

- Validate command now lives in `commands/validate/` with orchestration in `mod.rs`, TOML comparison in `toml_compare.rs`, and workbook snapshot checks in `workbook_snapshot.rs`; `--strip-images` strips the original XLSM to a temp copy, runs the TOML round-trip against that stripped template, rebuilds images from the cache, then compares workbook snapshots and xl/media bytes against the original. AppleScript validation still returns "AppleScript validation not implemented yet".
- TOML comparison canonicalizes numeric strings and stops at the first difference unless `--report-all` is set; workbook comparison covers sheet order, table layout, dimensions, validations, conditionals, and alignment.
- `strip-images` writes placeholders plus manifest v1 into `.git/xlsm_image_cache`; `rebuild-images` defaults to cache restoration while `--from-urls` resolves IMAGE formulas through rich data mappings with row/column offset handling.
- Test helpers now include `add_media_entries` for injecting `xl/media` files into generated workbooks when exercising strip/rebuild flows.
