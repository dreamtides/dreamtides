# Tabula Server Design Document

## Overview

The Tabula Server is a new `tabula server` command in the Tabula CLI. It runs a
local HTTP server that coordinates fast, safe, and deterministic updates between
an open Excel workbook and Rust-based spreadsheet analysis. The server never
writes to the XLSM file directly. It reads the saved file from disk, runs
listeners to compute changes, and returns change instructions to a VBA macro
that applies those updates inside Excel.

**Primary Goals:**
- Modify an open Excel workbook safely without file corruption or locking issues
- Keep round trips fast so user edits feel immediate
- Provide a robust, idempotent protocol for VBA to apply changes
- Maintain deterministic behavior with strict failure handling

**Non-goals:**
- Remote access over a network (localhost only)
- Server-side writing of XLSM files
- Real-time per-cell updates without a save event

## Dependencies

### Rust
- `axum` or `hyper` for the HTTP server (local-only, minimal middleware)
- `tokio` for async runtime and timeouts
- `serde` for shared data structures and configs
- `calamine` for reading XLSM/XLSX values
- `uuid` for UUID generation
- `once_cell` or `parking_lot` for shared state and caching
- `tracing` for structured logs (follow workspace patterns)

### VBA
- HTTP client via `AppleScriptTask` calling `curl`
- A small parser for the line-based response format
- Event handlers in `ThisWorkbook` and optional `Worksheet` modules

## Project Structure

```
src/tabula_cli/
├── tabula_server_design_document.md
├── src/
│   ├── commands/
│   │   ├── mod.rs
│   │   └── server.rs
│   ├── server/
│   │   ├── mod.rs
│   │   ├── http.rs
│   │   ├── model.rs
│   │   ├── serialization.rs
│   │   ├── workbook_snapshot.rs
│   │   ├── listener_runner.rs
│   │   └── listeners/
│   │       ├── mod.rs
│   │       ├── conditional_formatting.rs
│   │       ├── partial_formatting.rs
│   │       └── ensure_uuid.rs
│   └── core/
│       └── excel_reader.rs

src/tabula_cli/
├── vba/
│   ├── tabula_server.bas
│   ├── tabula_server_parser.bas
│   └── tabula_server_events.bas

tests/tabula_cli_tests/
├── test_data/
│   └── server/
│       ├── workbook_small.xlsm
│       ├── workbook_jackalope.xlsx
│       └── workbook_uuid_tables.xlsx
└── tests/
    └── tabula_cli_tests/
        └── tabula_server_tests.rs
```

The `vba/` directory is a design target for where VBA modules should live in the
repo. These modules are not compiled by Rust and must be copy-pasted or imported
into Excel manually, but they are part of the server integration contract.

## Command-Line Interface

`tabula server` starts a local HTTP server that listens for workbook change
notifications.

Key flags:
- `--port` with default `3030`
- `--host` with default `127.0.0.1`
- `--max-payload-bytes` to defend against runaway requests
- `--once` for single-request mode to simplify tests
- `--log-level` to align with workspace logging expectations

Additional commands:
- `tabula server-install` installs the required AppleScriptTask helper into
  `~/Library/Application Scripts/com.microsoft.Excel/` for Excel 2024 on Mac.

## Core Workflow

1. User edits the workbook and saves.
2. A VBA macro runs after save and POSTs a notification to the server.
3. The server reads the saved XLSM file from disk.
4. Listeners analyze workbook state and produce changes.
5. The server aggregates changes into a response payload.
6. The VBA macro applies the changes in Excel.

The server never writes to the workbook. All mutations happen in VBA against
Excel's in-memory state.

## Excel File Locking and Save Semantics

This design relies on Excel's save model and lock behavior:

- When a workbook is open, Excel creates a lock file (typically `~$`) and opens
  the workbook with a write lock. Other processes are prevented from writing but
  can usually read the file.
- Excel saves by writing a temporary file and then replacing the original. The
  file can briefly appear missing or partially written during this operation.
- `Workbook_AfterSave` fires after Excel has completed the save, which makes it
  the safest hook for notifying the server.

Implications for the server:
- The server must never attempt to write to the XLSM file.
- The server must tolerate transient read failures immediately after save.
- The server should retry failed reads for a short, bounded window if the file
  is momentarily unavailable or unreadable as a ZIP archive.
- The server should validate that the file metadata (size, modified time)
  remains stable for the duration of the read to avoid partial snapshots.

Implications for VBA:
- The macro should avoid triggering additional saves while applying server
  changes.
- Use `Application.EnableEvents = False` and restore it, to avoid reentrancy.
- Apply changes quickly, then restore calculation and screen updates.

## VBA Capabilities and Constraints

VBA can:
- Issue HTTP requests synchronously with reliable timeout controls
- Read and write cell values, ranges, and character-level formatting
- Access table metadata via `ListObject`, `ListColumns`, and `DataBodyRange`
- Detect sheet changes with `Workbook_SheetChange` and `Worksheet_Change`

VBA cannot:
- Parse complex payloads without a custom parser module
- Reliably operate if events recursively trigger itself, so guardrails are
  required

The design assumes a small parser module embedded in the workbook or
shipped alongside the VBA integration.

## Mac HTTP Strategy (Excel 2024)

**Definitive choice:** `AppleScriptTask` invoking `curl` for HTTP requests.

Reasons for the choice:
- Excel 2024 for Mac has no COM HTTP objects, so Windows-only APIs are not viable.
- `AppleScriptTask` is the supported bridge for VBA → OS actions on modern Mac Excel.
- `curl` provides predictable timeouts, retries, and HTTP status handling.
- The flow stays synchronous from VBA's perspective without UI-blocking polling.
- It keeps the protocol HTTP-based, matching the Rust server design and tests.

Implementation constraints:
- `AppleScriptTask` requires a script placed in
  `~/Library/Application Scripts/com.microsoft.Excel/`.
- Requests should be written to a temp file and passed by path to avoid quoting
  issues and AppleScript return size limits. This is still a synchronous HTTP
  call, not a file-based mailbox.
- Responses should be written to a temp file and then read by VBA.
- Timeouts and retries should be controlled in `curl` arguments, and VBA should
  treat non-zero exit codes as retryable errors.

Installer requirement:
- Provide a CLI command that copies the AppleScript file into the required
  directory, validates permissions, and reports actionable errors if Excel is
  not installed or the path is unavailable.

Rejected alternative: file-based polling:
- Requires scheduled polling (`Application.OnTime`) to avoid blocking the UI.
- Adds latency and race conditions around stale responses and concurrent saves.
- Shifts complexity to VBA for lifecycle management and cleanup.

## VBA Parsing Strategy (Excel 2024)

The request and response payloads use a line-based protocol to avoid JSON
parsing and minimize VBA code. Parsing relies on `Split` plus a small
percent-decoder.

Approach:
- Parse the response as lines using `vbLf` and ignore empty lines.
- Split each line on spaces, with a fixed positional schema per line type.
- Decode percent-encoded fields for sheet names and cell values.

Integration plan:
- Place the parser in `vba/tabula_server_parser.bas` as the source of truth.
- Import it into the workbook alongside the event and HTTP modules.
- Use a single entry function that returns a list of change records with
  concrete fields instead of dynamic maps.

## Server Architecture

### HTTP Endpoints

- `POST /notify` is the single entry point.
- Response uses a line-based protocol with a deterministic schema and version line.
- No other endpoints are required for the initial implementation.

### Request Payload

The request uses the same line-based format as the response. Required lines are
listed under the protocol section, and any additional lines must be ignored by
the server.

### Response Payload

The response is a line-based format with percent-encoded strings and fixed
positional fields. The message begins with a version line followed by status,
warnings, and change lines.

### Change Types

Changes are granular and designed to map directly to VBA operations:

- `set_bold` for a range
- `set_font_color_spans` for partial formatting inside a single cell
- `set_value` and `clear_value` for ID generation
- `set_font_color` for whole-cell font color
- `set_font_size` for whole-cell font size
- `set_italic` for whole-cell italic toggle
- `set_underline` for whole-cell underline toggle
- `set_fill_color` for cell background color
- `set_number_format` for Excel number format strings
- `set_horizontal_alignment` for cell alignment

Each change includes:
- Sheet name
- Range or cell address
- Payload fields like `bold`, `color_rgb`, and `spans`

The `spans` field for partial formatting is a list of start/length pairs. Index
is 1-based to align with VBA `Characters`.

### Idempotency and Caching

To avoid duplicate updates on retries:
- VBA includes `request_id`.
- The server returns a `changeset_id` computed from workbook metadata plus
  changes content.
- VBA stores the last applied `changeset_id` and ignores duplicates.

The server keeps a small in-memory cache keyed by `workbook_path` plus
`workbook_mtime` and `workbook_size`. If the workbook has not changed, it
returns the cached response immediately.

## Listener Framework

### Listener Interface

Each listener accepts a read-only workbook snapshot and a context object with
request metadata. It returns a list of changes plus diagnostics. Listener
results are merged in a stable order to produce a single response.

### Conflict Resolution

If multiple listeners target the same cell:
- `set_value` and `clear_value` are applied before formatting operations
- Formatting changes are merged where possible
- Conflicts are logged and reported as warnings

Deterministic ordering ensures reproducible outcomes.

## Listener 1: Conditional Formatting (Pineapple)

**Goal:** On the `Cards` sheet, any cell containing the substring `pineapple`
should be bolded.

Design:
- Scan the `Cards` sheet values using `calamine`.
- Consider only string cells, case-insensitive match for `pineapple`.
- For each match, emit a `set_bold` change for that cell.
- Apply bold in VBA using the range `Font.Bold` property.

Performance:
- If `changed_range` is provided and is on `Cards`, scan only that range.
- If no range is provided, scan the full used range of the sheet.

## Listener 2: Partial Formatting (Jackalope)

**Goal:** On any sheet, when a cell contains the substring `jackalope`, only the
word `jackalope` becomes orange and the rest of the cell retains its original
color.

Design:
- Scan all sheets for string cells.
- Case-insensitive search for all non-overlapping `jackalope` occurrences.
- For each occurrence, record a span with 1-based start index and length.
- Emit a `set_font_color_spans` change for that cell with color `#FFA500`.

VBA application:
- Use `Range.Characters(Start, Length).Font.Color` to set only those spans.
- Do not touch other parts of the cell to preserve existing formatting.
- If a cell already has rich text, only the specified spans are modified.

Performance:
- If `changed_range` is provided, scan only affected cells and their containing
  rows to avoid missing partial edits.

## Listener 3: UUID Generation

**Goal:** Automatically fill missing UUIDs in `ID` columns in Excel tables, and
clear UUIDs from rows that are otherwise blank.

Design modeled after `src/tabula_cli/ensure_guid.vba`:
- Iterate all tables (Excel `ListObject` equivalents from the file snapshot).
- Identify the `ID` column by case-insensitive match of column name after
  trimming whitespace and normalizing non-breaking spaces.
- For each row:
  - Define `id_value` as the trimmed cell text in the ID column.
  - Define `has_other_content` as any non-empty, non-error cell in other
    columns.
  - If `id_value` is empty and `has_other_content` is true, emit a `set_value`
    change with a generated UUID v4.
  - If `id_value` is present and `has_other_content` is false, emit a
    `clear_value` change.

Optimization:
- If `changed_range` is provided, only consider rows that intersect that range.
- If no range is provided, use a small row hash cache to avoid re-checking rows
  that are unchanged between requests.

UUID format:
- Lowercase hex with hyphens, matching existing VBA output.

### Listener 4: Boxicon Font Insertion

**Goal:** Automatically insert icons using the "boxicons" font.

When a cell contains specific literal text patterns, replace them with corresponding
Boxicons Unicode characters. The boxicons font is already installed, but needs to be
applied to the characters.

Supported patterns:
- `{x}` → U+FEFC (turn down icon)
- `{e}` → U+F407 (energy icon)
- `{fast}` → U+F93A (fast icon)
- `{p}` → U+FC6A (points icon)

Design:
- Scan all sheets for string cells containing any of the literal patterns.
- Patterns are checked longest-first to avoid partial matches.
- For each occurrence, replace the pattern with a Left-to-Right Mark (LRM, `\u{200E}`)
  followed by the corresponding boxicons character.
- The LRM prevents RTL rendering issues when the icon appears at the start of a cell,
  since many boxicons characters are in the Arabic Presentation Forms-B range.
- All replacements in a cell are sorted by position and applied together.
- Emit a `set_value` change with the modified text containing all icon replacements.
- Emit a `set_font_name_spans` change to apply the "boxicons" font to each LRM+icon
  pair (span length 2 per icon).
- Emit a `set_font_size_spans` change to set the font size to 16 points for each
  LRM+icon pair.
- Emit a `set_subscript_spans` change to set subscript formatting for each LRM+icon
  pair.

VBA application:
- Apply `cell.ReadingOrder = xlLTR` to ensure left-to-right text direction.
- Use `cell.Characters(Start, Length).Font.Name` to set the boxicons font on each
  2-character span (LRM + icon).
- Use `cell.Characters(Start, Length).Font.Size` to set size 16 on each span.
- Use `cell.Characters(Start, Length).Font.Subscript` to enable subscript rendering
  on each span.

*Only* the icon characters (LRM + boxicons) should have the boxicons font, size 16,
and subscript formatting applied. The remaining text should keep its original formatting.

Icons render correctly inline in Excel as subscripts. Examples:
- `=UNICHAR(HEX2DEC("FEFC"))` for the turn down icon
- `=UNICHAR(HEX2DEC("F407"))` for the energy icon
- `=UNICHAR(HEX2DEC("F93A"))` for the fast icon
- `=UNICHAR(HEX2DEC("FC6A"))` for the points icon

## Workbook Snapshot Strategy

The server should read a stable snapshot without touching formulas:
- Use `calamine` to read worksheet values.
- Read table metadata and ranges via `Table` APIs.
- Avoid parsing styles or formulas unless needed for a listener.

Before reading:
- Capture file size and modified time.
- Attempt to open and parse as a ZIP-based XLSM.
- If parse fails, wait briefly and retry a limited number of times.
- Re-check metadata after read to ensure the file did not change mid-read.

## Serialization Format and Round Trip

The protocol is simple, stable, and fast to parse in VBA. Both request and
response use the same line-based format.

Format rules:
- The response is UTF-8 text with `\n` line separators.
- The first non-empty line must be `TABULA/1`.
- Each subsequent line begins with a type token and fixed positional fields.
- Unrecognized line types are ignored.

Percent-encoding:
- Encode space as `%20`, percent as `%25`, and newline as `%0A`.
- The server encodes all string fields, and the VBA parser decodes them.

Request line types:
- `REQUEST_ID <id>`: unique identifier from VBA.
- `WORKBOOK_PATH <path>`: percent-encoded full path.
- `WORKBOOK_MTIME <seconds>`: modified time as Unix seconds.
- `WORKBOOK_SIZE <bytes>`: file size in bytes.
- `CHANGED_RANGE <sheet> <range>`: optional hint, A1 notation range.

Response line types:
- `REQUEST_ID <id>`: echoes the request identifier.
- `STATUS <ok|error>`: overall status.
- `RETRY_AFTER_MS <ms>`: optional, only with `STATUS error`.
- `WARNING <message>`: optional, percent-encoded message.
- `CHANGE set_bold <sheet> <cell> <bold>`: bold is `0` or `1`.
- `CHANGE set_font_color_spans <sheet> <cell> <rgb> <spans>`: `rgb` is `RRGGBB`,
  `spans` is `start:length,start:length` with 1-based indices.
- `CHANGE set_value <sheet> <cell> <value>`: value is percent-encoded.
- `CHANGE clear_value <sheet> <cell>`.
- `CHANGE set_font_color <sheet> <cell> <rgb>`: `rgb` is `RRGGBB`.
- `CHANGE set_font_size <sheet> <cell> <points>`: points is a decimal number.
- `CHANGE set_fill_color <sheet> <cell> <rgb>`: `rgb` is `RRGGBB`.
- `CHANGE set_number_format <sheet> <cell> <format>`: format is percent-encoded.
- `CHANGE set_horizontal_alignment <sheet> <cell> <alignment>`: alignment is one
  of `left`, `center`, or `right`.
- `CHANGE set_font_name_spans <sheet> <cell> <font_name> <spans>`: `font_name` is
  percent-encoded, `spans` is `start:length,start:length` with 1-based indices.
- `CHANGE set_font_size_spans <sheet> <cell> <points> <spans>`: `points` is a decimal
  number, `spans` is `start:length,start:length` with 1-based indices.
- `CHANGE set_subscript_spans <sheet> <cell> <subscript> <spans>`: `subscript` is `0`
  or `1`, `spans` is `start:length,start:length` with 1-based indices.

Round-trip cycle:
- VBA sends request with workbook metadata and optional changed range.
- Server responds with change list and idempotency token.
- VBA applies changes and stores the idempotency token to avoid repeats.

## Fault Tolerance

The server must fail closed and never risk corruption:
- If the workbook cannot be read, return an error with `retry_after_ms`.
- If a listener fails, exclude it and return warnings while continuing others.
- Keep a hard timeout per request to avoid blocking Excel.
- Limit total changes per response to protect against runaway behavior.

The VBA macro should:
- Respect `retry_after_ms` and perform a bounded retry.
- Disable events while applying changes to avoid recursion.
- Restore Excel state on all error paths.

## Performance Considerations

- Parse only the workbook areas that listeners need.
- Use in-memory caching of the last snapshot and response for each workbook.
- Avoid heavy hashing of the full XLSM unless required by a listener.
- Use a single reader thread to avoid disk thrash on repeated saves.

## Testing Strategy

The server should follow the existing `tabula_cli_tests` pattern and remain
black-box and deterministic.

Principles:
- Use the `tabula server --once` mode to avoid background threads in tests.
- Run tests against small XLSX/XLSM fixtures in `tests/tabula_cli_tests/test_data/server`.
- Validate protocol responses and change lists, not internal data structures.
- Keep tests under 500ms by using tiny workbooks and in-memory HTTP clients.

Test cases:
- `pineapple` in Cards sheet returns `set_bold` changes.
- `jackalope` returns `set_font_color_spans` with correct offsets.
- UUID generation fills IDs for rows with other content and clears when empty.
- Retry behavior when workbook read fails (mock failure or use invalid file).
- Idempotency: repeated request with same metadata returns identical response.

## Workflow Instructions for Implementing Agent

- Follow the existing style rules in `AGENTS.md` and avoid inline code comments.
- No inline comments in code
- No `pub use` re-exports in lib.rs; use full module paths instead
- Functions qualified with one module: `excel_reader::extract_tables()`
- Do not import functions directly; call them via a module qualifier (e.g., `paths::git_root_for()`)
- Structs unqualified: `TableData`
- Enums one level: `ColumnType::Formula`
- Public items at top of file, private below
- Cargo.toml dependencies alphabetized
- Prefer inline expressions over intermediate `let` bindings
- Record decisions and API quirks in `src/tabula_cli/observations.md`.
- Use `just review` after changes.

If more information is needed:
- Inspect existing `tabula_cli` command patterns for routing and errors.
- Review `ensure_guid.vba` to match UUID behavior and edge cases.
- Ask for clarification on Excel workbook locations or macro constraints.

# Manual Testing

CRITICAL: After every step is completed, the agent should provide


```

# MANUAL TESTING INSTRUCTIONS

```

Output with a description of how to manually verify the most recently completed
work. Please assume the working directory is /dreamtides/ and all cargo commands
require `--manifest-path rules_engine/Cargo.toml`. Please format all commands
via newline + three backticks for easy copy and paste.

Please use the path to the workbook
`dreamtides/client/Assets/StreamingAssets/Tabula.xlsm` for provided testing
commands.

## Milestone Breakdown

1. AppleScriptTask Proof of Concept
   - Build a minimal AppleScriptTask + `curl` flow that posts to a stub URL.
   - Confirm request/response file paths and error handling on Excel 2024 for Mac.
   - Record any platform quirks in `src/tabula_cli/observations.md`.
   - Implement `tabula server-install` to place the AppleScript helper.

2. Server Skeleton and CLI Wiring
   - Add `tabula server` command with basic argument parsing.
   - Implement `--once` mode for deterministic tests.
   - Build HTTP handler that returns a fixed empty response.

3. Protocol and Parsing Integration
   - Define request and response models with versioning.
   - Implement line-based parsing and serialization for requests and responses.
   - Finalize the Mac-safe VBA parser module and response schema constraints.
   - Add idempotency token handling and caching.

4. Workbook Snapshot and Retry Loop
   - Implement snapshot reader using `calamine`.
   - Add retry behavior for transient read failures.
   - Map table metadata and used ranges into in-memory structures.

5. Listener Framework
   - Add listener trait and runner with deterministic ordering.
   - Build conflict resolution and change aggregation.
   - Add diagnostics and warning propagation to responses.

6. Listener 1: Pineapple Formatting
   - Implement the `Cards` sheet pineapple bold listener.
   - Add black-box tests for `set_bold` change generation.

7. Listener 2: Jackalope Partial Formatting
   - Implement the cross-sheet jackalope span listener.
   - Add black-box tests for `set_font_color_spans` offsets.

8. Listener 3: UUID Generation and End-to-End Tests
   - Implement the table ID generation listener and row pruning rules.
   - Add end-to-end tests for ID creation and clearing behavior.
   - Validate overall response aggregation and VBA expectations.

9. Listener 4: Boxicons font application
   - Replace the text {x} with the boxicons icon
   - Add unit tests