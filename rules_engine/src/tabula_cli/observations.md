# Tabula CLI Observations

- `rebuild-images` restores stripped XLSM media from `.git/xlsm_image_cache/_xlsm_manifest.json`, using cached hashes, preserving manifest ZIP order, and failing on missing cache files, manifest entries, or size mismatches.
- `strip-images` swaps `xl/media/*` with a 1x1 JPEG while caching originals by SHA-256 in `.git/xlsm_image_cache/` and writing manifest version 1 alongside the cache.
- `validate` still lacks `--applescript` and `--strip-images`; the flow builds XLSM from TOML, re-extracts TOML, and compares workbook snapshots via `umya-spreadsheet`.
- Snapshot diffs cover sheet order, table layout/style, column widths/hidden/best-fit, row heights/hidden/custom, validations, conditional formatting, and alignment/wrap; extra dimensions or alignments absent in the template are ignored, and `--verbose`/`--all` expand reporting.
- Always run `just fmt`, `just check`, `just clippy`, and `just review` before handing off changes.
