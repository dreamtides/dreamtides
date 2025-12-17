# Tabula CLI Validation Deep Dive

## Purpose and Scope
- Define a CLI strategy to decide whether two Excel workbooks (primarily Tabula.xlsm round-trips) are visually identical to a user.
- Exit codes: 0 when no visual discrepancies are detected, 1 when any difference is found or validation cannot be completed.
- Output: human-friendly description of which validations failed

## What “Visually Identical” Means
- Workbook chrome: sheet order, sheet names, hidden/veryHidden flags, tab colors, active sheet/view, freeze panes.
- Grid layout: column widths, row heights, hidden rows/columns, merged cells, print areas, page breaks that affect rendered layout.
- Cell surface: displayed values, formulas (for display and error states), number formats, alignment, fonts, fills, borders, protection flags, hyperlinks, comments/notes.
- Conditional styling: conditional formatting rules (icons, color scales, data bars, custom formulas) and their applied ranges.
- Data entry constraints: data validation rules (type, operators, allowed values, in-cell dropdowns, error alert configuration).
- Structured features: tables (names, ranges, header/total rows, column order, styles, filters), pivot tables (not present today), named ranges that drive validation or lookups.
- Visual objects: images, shapes, charts, slicers, and their positions/sizes on the canvas; presence of drawing layers and rels to `xl/media`.
- VBA presence is not “visual” but a missing/extra `vbaProject.bin` should still fail validation because it can alter runtime behavior.
- Out-of-scope for visual parity: authorship metadata, creation/mod timestamps, recalculation chains, relationship IDs, CRC ordering inside the ZIP, and other non-rendered metadata.

## Command Shape
```
tabula validate [TOML_DIR]
```

- Visual parity check; ignores known non-visual noise.
- Invokes `build-toml` and then `build-xls`, outputs the new XLSM file to a temp
  location, then runs a visual diff.

## High-Level Pipeline
1. **Load inputs**: accept input `.xlsm` file, create new `xlsm` file via
   `build-toml` and then `build-xls`
2. **Normalize noise**: drop or canonicalize components that never affect visuals:
   - Ignore `docProps/core.xml`, `docProps/app.xml`, `[Content_Types].xml` default ordering differences, `xl/calcChain.xml`, `_rels/.rels`, `.rels` internal IDs, embedded thumbnails.
   - Normalize ZIP metadata (modtime, extra fields, compression flags) by reading the stream contents only.
3. **Build a canonical visual snapshot** for each workbook:
   - Parse `xl/workbook.xml` and workbook rels to map sheet names ↔ worksheet parts, sheet order, visibility flags, active sheet, defined names (keep only those that impact visuals such as `_xlnm.Print_Area`, `_xlnm.Print_Titles`, and any ranges referenced by validation/CF).
   - Parse `xl/styles.xml` to materialize resolved styles: for every `xf`, inline the referenced font/fill/border/numFmt/alignment into a stable struct; use `xl/theme/theme1.xml` for theme colors.
   - Parse `xl/sharedStrings.xml` so string cells can be compared by value, not index.
   - For each worksheet (`xl/worksheets/sheetN.xml`):
     - Capture sheet properties: gridlines shown, headings, tab color, freeze panes, zoom, page setup if it changes print layout.
     - Column definitions: custom widths, hidden flags, style overrides.
     - Row definitions: custom heights, hidden flags, style overrides.
     - Merged cells: normalized ranges.
     - Cells: coordinate, type (value vs formula), raw value, shared string resolution, formula text (including array vs non-array), and resolved style struct. Cached formula results are ignored unless a later render step is enabled.
     - Data validation: type, operator, formulas/values, dropdown flag, alert config, and target ranges.
     - Conditional formatting: rule type, formulas, dxf payload (resolved to full style), priority/stopIfTrue, and target ranges.
     - Hyperlinks and comments/notes (vml or modern comments).
     - Drawing relationships: anchors for images/shapes/charts with size and z-order; link to `xl/media/*` filenames for presence checks.
   - Tables (`xl/tables/tableN.xml`): name, sheet reference, ref range, header/total row flags, column list/order, styles, filter settings, totals row labels/formulas, and data range size.
   - Charts and shapes: record type/name and bounding boxes; do not diff internal series data unless needed for Tabula.
4. **Diff snapshots**:
   - Workbook-level mismatches: missing/extra sheets, visibility changes, different tab colors/active sheet, missing defined names tied to print areas.
   - Sheet-level layout mismatches: row/column size or hidden state changes, different merge ranges, freeze panes.
   - Cell-level mismatches: value or formula text differences, style differences (font/color/fill/border/numFmt/alignment/protection), hyperlink/notes differences.
   - Validation/CF mismatches: different ranges, rule types, operators, or dxf payloads.
   - Table mismatches: range size, header/total flags, column order/names, styles, filter state.
   - Object mismatches: missing/extra images or shapes or chart anchors; image filenames/content hash differences.
   - Report each mismatch as `Sheet!Cell: expected …, actual …` or `Sheet: data validation range A2:A50 differs`.
6. **Exit codes**: 0 on zero mismatches; 1 on any mismatch or if parsing/rendering fails.

## Implementation Notes (Rust)
- Parsing: avoid full in-memory `umya` load for speed; prefer streaming XML parsing with `quick-xml` and targeted structs. Reuse existing ZIP handling from `strip-images` to read entries without caring about ZIP metadata.
- Style resolution: build canonical structs keyed by `cellXfs` index that inline `font/fill/border/numFmt/alignment`. Normalize colors to ARGB after applying theme and tint; normalize number format IDs to strings so built-in vs custom IDs compare correctly.
- Cell values: compare by logical type (bool, number, string, inlineStr, rich text plain text). Treat empty as absent. For formulas, compare formula text and cell type; ignore cached `v` unless render step is on.
- Ranges: normalize to uppercase A1 notation and sort lists before comparison to reduce noise from ordering in XML.
- Drawings/images: parse `xl/drawings/drawing*.xml` anchors; hash referenced `xl/media/*` files (raw bytes) for presence checks without needing to decode images.
- Performance: build snapshots sheet-by-sheet; bail out early after first N
  mismatches unless `--all` is requested.
- CLI UX: default to showing the first 1 mismatch, with `--all` to show
  everything and `--quiet` to suppress descriptions (exit code only).

## Noise to Ignore Explicitly
- ZIP timestamps, CRC ordering, and file ordering.
- `docProps/*`, `_rels/*.rels`, `[Content_Types].xml` default/override ordering.
- `xl/calcChain.xml`, `calcPr` iteration counts, `fullCalcOnLoad` flags.
- Cached formula values (`<v>` under formula cells) unless render step is requested.
- External workbook refs not used for visuals, PowerPivot caches, pivot cache records (Tabula does not use these).

## Current Round-Trip Differences (Tabula.xlsm vs build-xls output)
- Scope of comparison: unzipped originals at `~/Documents/GoogleDrive/dreamtides/client/Assets/StreamingAssets/tmp/` vs round-tripped output at `~/Documents/GoogleDrive/dreamtides/tmp/`.
  - Should not trigger validation error; defines inputs only.
- Metadata: `docProps/core.xml` loses creator/lastModifiedBy and tweaks modified timestamps; `docProps/app.xml` adds an empty Manager field; `.DS_Store` exists only in the generated `xl/`.
  - Should not trigger visual validation error; these are non-visual and expected noise.
- Workbook: sheetIds renumbered (Strings now sheetId=1 instead of 3), activeTab shifts from Card Effects to Test Cards, `lowestEdited` lowered to 4, `filterPrivacy` enabled, `forceFullCalc` added, and `_xlpm.p`/`_xlpm.s` lose `xlm="1"`. Calc chain contents reordered/reduced.
  - ActiveTab change is debatable but generally should not fail; calc flags and sheetId renumbering should not fail visual validation but calcChain replacement should be tolerated unless we require identical recalculation behavior.
- Styles/theme: `xl/styles.xml` drops all `dxfs` (94 → 0) and one font; table definitions (`xl/tables/table1.xml`–`table10.xml`) lose headerRowDxf/dataDxf IDs and have new placeholder ids/xr:uid/autoFilter uids. `xl/theme/theme1.xml` drops the themeFamily extension.
  - Should trigger validation error; losing dxfs removes conditional formatting, and table styling metadata changes affect visual rendering.
- Shared strings and cell storage: shared strings shrink 660 → 634 unique values with 26 values removed; cells stay value-identical but 21 cells in `sheet2` (Cards), 21 in `sheet3` (Dreamwell), 10 in `sheet5` (Card Effects), 80 in `sheet6` (Test Cards), and 23 in `sheet7` (Test Dreamwell) switch from shared-string storage (`t="s"`) to inline number strings (`t` omitted).
  - Should not trigger validation error if rendered text matches, but a stricter visual engine might flag type changes when they imply number vs text formatting differences.
- Metadata part: `xl/metadata.xml` removes the XLDAPR dynamic array metadata and keeps only XLRICHVALUE entries.
  - Should trigger validation error only if dynamic arrays are required for visuals; otherwise can be ignored.
- Media and VBA: four images (`xl/media/image6.jpg`, `image10.jpg`, `image12.jpg`, `image18.jpg`) have different bytes but identical sizes; `vbaProject.bin` shrinks (106,496 → 74,752 bytes) with a different hash.
  - Should trigger validation error: media byte changes are visually significant; VBA size change risks functional differences that may impact visible behavior.
- Shared strings, styles, table IDs, theme, and macro changes are all visually observable risks; image byte deltas indicate non-verbatim media retention.
  - Should trigger validation errors: these cumulative differences mean the output is not visually identical.
