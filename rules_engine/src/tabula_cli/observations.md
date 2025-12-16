# Tabula CLI Observations (for upcoming milestones)

## Implemented Commands
- `build-toml`: Extracts named tables, normalizes column/table names (kebab-case, special chars removed), preserves data-column order, skips formula/image columns, emits single-column tables as flat arrays (`predicate_types = ["A"]`), backs up the source XLSM with timestamped copies, and prunes `.git/excel-backups` to the 50 newest files.
- `strip-images`: Replaces `xl/media/*` images with a 1x1 JPEG placeholder, writes `_xlsm_manifest.json` to `.git/xlsm_image_cache/` (outside the archive), and caches originals there. It preserves original ZIP file order from the source archive and retains each entry’s original compression method; timestamps are normalized to 1980-01-01. Manifest includes `version`, `file_order`, `images` (hash/size/original_name), and `source_file`. The command supports `--output` to write to a new XLSM; otherwise operates in place via a temp file.

## Paths and Defaults
- Git root discovered by walking up to `.git`.
- Defaults: XLSM `client/Assets/StreamingAssets/Tabula.xlsm`, TOML dir `client/Assets/StreamingAssets/Tabula`, backup dir `.git/excel-backups`, image cache `.git/xlsm_image_cache`.

## Testing Coverage
- build-toml tests cover column order, special-char stripping, single-column array encoding, backup creation/pruning.
- strip-images tests build a synthetic XLSM ZIP with images and verify placeholder substitution, manifest contents, cache writes, and preserved entry order (plus placeholder data).

## Open Areas
- `build-xls`, `validate`, `rebuild-images`, `git-setup` remain unimplemented.
- No cross-check yet that Excel opens stripped outputs; compression is preserved per-entry to reduce corruption risk. Further validation against real XLSM recommended when working on rebuild-images/validate.*** End Patch
