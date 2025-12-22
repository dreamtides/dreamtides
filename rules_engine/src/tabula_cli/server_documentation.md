# Tabula Server Documentation

## Overview

The Tabula Server is a local HTTP server (`tabula server` command) that coordinates automatic updates between an open Excel workbook and Rust-based spreadsheet analysis. The server reads the saved XLSM file, runs listeners to compute changes, and returns instructions to a VBA macro that applies updates inside Excel.

**Key Properties:**
- Never writes to XLSM files directly (all writes via VBA)
- Fast round trips for immediate user feedback
- Deterministic with strict failure handling
- Localhost-only (no remote access)

## Architecture

### Components

```
Excel Workbook (VBA)
    ↓ (save event triggers HTTP POST)
Tabula Server (Rust)
    ↓ (reads saved file)
Listeners (analyze & generate changes)
    ↓ (returns line-based protocol)
VBA Handlers (apply changes to cells)
```

### Request Flow

1. User saves the workbook
2. `Workbook_AfterSave` VBA event triggers
3. VBA POSTs metadata to `http://127.0.0.1:3030/notify`
4. Server reads XLSM file from disk using `calamine`
5. Listeners analyze workbook and produce changes
6. Server serializes changes to line-based format
7. VBA parses response and applies changes to Excel cells
8. Changes applied with events disabled to prevent recursion

### File Structure

```
src/tabula_cli/
├── src/
│   ├── commands/server.rs          # CLI command entry point
│   ├── server/
│   │   ├── http.rs                 # HTTP server (POST /notify)
│   │   ├── model.rs                # Request/Response/Change types
│   │   ├── serialization.rs        # Protocol parsing & encoding
│   │   ├── server_workbook_snapshot.rs  # Read Excel files
│   │   ├── listener_runner.rs      # Execute listeners
│   │   └── listeners/
│   │       ├── conditional_formatting.rs
│   │       ├── partial_formatting.rs
│   │       ├── ensure_uuid.rs
│   │       └── boxicons.rs
└── vba/
    ├── tabula_server.bas           # HTTP client via AppleScriptTask
    ├── tabula_server_parser.bas    # Protocol parser with UTF-8 support
    ├── tabula_server_change_handlers.bas  # Apply changes to cells
    └── tabula_server_events.bas    # Workbook event hooks
```

## Protocol Specification

### Line-Based Format

Both requests and responses use UTF-8 text with `\n` line separators. String fields are percent-encoded (space=%20, newline=%0A, percent=%25).

### Request Format

```
TABULA/1
REQUEST_ID <uuid>
WORKBOOK_PATH <percent-encoded-path>
WORKBOOK_MTIME <unix-seconds>
WORKBOOK_SIZE <bytes>
CHANGED_RANGE <sheet> <range>    # Optional, A1 notation
```

### Response Format

```
TABULA/1
REQUEST_ID <uuid>
STATUS ok|error
RETRY_AFTER_MS <ms>              # Optional, with STATUS error
WARNING <message>                # Optional, percent-encoded
CHANGE <type> <args...>          # One per change
CHANGESET_ID <hash>              # Optional, for idempotency
```

### Change Types

All changes specify sheet and cell. Spans use 1-based indexing for VBA compatibility.

**Value Changes:**
- `set_value <sheet> <cell> <value>` - Set cell text
- `clear_value <sheet> <cell>` - Clear cell contents

**Whole-Cell Formatting:**
- `set_bold <sheet> <cell> <0|1>`
- `set_italic <sheet> <cell> <0|1>`
- `set_underline <sheet> <cell> <0|1>`
- `set_font_color <sheet> <cell> <RRGGBB>`
- `set_font_size <sheet> <cell> <points>`
- `set_fill_color <sheet> <cell> <RRGGBB>`
- `set_number_format <sheet> <cell> <format>`
- `set_horizontal_alignment <sheet> <cell> <left|center|right>`

**Character-Level Formatting (Spans):**
- `set_bold_spans <sheet> <cell> <0|1> <spans>`
- `set_italic_spans <sheet> <cell> <0|1> <spans>`
- `set_underline_spans <sheet> <cell> <0|1> <spans>`
- `set_font_color_spans <sheet> <cell> <RRGGBB> <spans>`
- `set_font_name_spans <sheet> <cell> <font-name> <spans>`
- `set_font_size_spans <sheet> <cell> <points> <spans>`
- `set_subscript_spans <sheet> <cell> <0|1> <spans>`

Spans format: `start:length,start:length` (e.g., `1:3,10:5` for chars 1-3 and 10-14)

## Listener System

### Listener Interface

```rust
pub trait Listener: Send + Sync {
    fn name(&self) -> &str;
    fn run(&self, snapshot: &WorkbookSnapshot, context: &ListenerContext)
        -> Result<ListenerResult>;
}
```

Listeners receive:
- `WorkbookSnapshot` - Read-only view of all sheets, cells, and tables
- `ListenerContext` - Request metadata including optional `changed_range` hint

Listeners return:
- `Vec<Change>` - Changes to apply
- `Vec<String>` - Warnings (non-fatal errors)

### Execution & Conflict Resolution

1. Listeners execute in registration order
2. Failed listeners produce warnings but don't stop others
3. Conflicts resolved deterministically:
   - Value changes (set_value/clear_value) applied first
   - Last writer wins for conflicting value changes
   - Formatting changes merged where possible

### Implemented Listeners

**ConditionalFormattingListener**
- Scans `Cards` sheet for cells containing "pineapple" (case-insensitive)
- Emits `set_bold` for matching cells

**PartialFormattingListener**
- Scans all sheets for "jackalope" substring (case-insensitive)
- Emits `set_font_color_spans` to make only "jackalope" orange (#FFA500)
- Preserves formatting of surrounding text

**EnsureUuidListener**
- Finds Excel tables via `ListObject` structures
- Locates `ID` column (case-insensitive, normalizes non-breaking spaces)
- Generates UUID v4 for rows with content but no ID
- Clears IDs from otherwise-empty rows

**BoxiconsListener**
- Scans for pattern text: `{x}`, `{e}`, `{fast}`, `{p}`
- Creates formatted version in adjacent cell to the right (A1 → B1)
- Replaces patterns with boxicons Unicode characters
- Applies boxicons font, size 20, subscript formatting to icons
- Prepends Left-to-Right Mark (U+200E) to prevent RTL rendering issues
- Original cell remains unchanged

Pattern mappings:
- `{x}` → U+FEFC (turn down icon)
- `{e}` → U+F407 (energy icon)
- `{fast}` → U+F93A (fast icon)
- `{p}` → U+FC6A (points icon)

## VBA Integration

### Mac HTTP Strategy

Uses `AppleScriptTask` invoking `curl` for HTTP requests on Excel 2024 for Mac:
- Requests written to temp file, passed to AppleScript
- Responses written to temp file, read by VBA
- Synchronous from VBA perspective
- Requires AppleScript in `~/Library/Application Scripts/com.microsoft.Excel/`
- Install via `tabula server-install` command

### Parser Implementation

**VBA Percent Decoder (`TabulaServerParser.bas`):**
- Handles multi-byte UTF-8 sequences correctly
- Manual UTF-8 decoder (no ADODB.Stream dependency for macOS compatibility)
- Decodes percent-encoded bytes into byte array
- Converts UTF-8 byte sequences to Unicode strings
- Critical for boxicons characters (3-byte UTF-8 sequences)

**Change Handlers (`TabulaServerChange Handlers.bas`):**
- `ApplyChanges()` - Main entry point, disables events during application
- Individual handler for each change type
- Span handlers use `cell.Characters(start, length)` for partial formatting
- `ApplySetFontNameSpans` - Sets `cell.ReadingOrder = xlLTR` to prevent RTL issues

### Event Hooks

**`Workbook_AfterSave`:**
- Triggers server notification after Excel completes save
- Safest hook (file is fully written)
- Includes retry logic for transient read failures

**Event Safety:**
- `Application.EnableEvents = False` during change application
- Prevents recursive save triggers
- Always restored in cleanup/error handlers

## Caching & Idempotency

### Server-Side Cache

- Key: `(workbook_path, workbook_mtime, workbook_size)`
- Value: Serialized response string
- Returns cached response if workbook unchanged
- Avoids redundant file reads and listener execution

### Client-Side Idempotency

- VBA includes `request_id` with each request
- Server computes `changeset_id` from workbook metadata + changes
- VBA stores last applied `changeset_id`
- Ignores duplicate changesets on retry

## Excel File Handling

### Lock Behavior

- Excel opens workbooks with write lock (creates `~$` lock file)
- Other processes can usually read but not write
- Excel saves via temp file then atomic replace
- File may be briefly unavailable during save operation

### Server Read Strategy

1. Capture file size and modified time
2. Open file as ZIP archive (XLSM format)
3. Parse with `calamine`
4. Re-check metadata after read (ensure no mid-read changes)
5. Retry with exponential backoff on transient failures
6. Return error with `retry_after_ms` if read ultimately fails

## Testing

### Unit Tests (`tests/tabula_cli_tests/`)

- Black-box protocol tests using `--once` mode
- Small XLSX/XLSM fixtures in `test_data/server/`
- Create workbook, run listener, validate changes
- No server threads in tests (synchronous execution)
- Examples: pineapple detection, jackalope spans, UUID generation, boxicons

### Running Tests

```bash
cargo test --package tabula_cli_tests --test lib
```

## Command-Line Interface

**Start Server:**
```bash
cargo run --bin tabula -- server [OPTIONS]
```

Options:
- `--port <PORT>` - Default 3030
- `--host <HOST>` - Default 127.0.0.1
- `--once` - Single-request mode for tests
- `--max-payload-bytes <BYTES>` - Request size limit
- `--log-level <LEVEL>` - Logging verbosity

**Install AppleScript Helper (macOS):**
```bash
cargo run --bin tabula -- server-install
```

## Adding New Listeners

1. Create new module in `src/tabula_cli/src/server/listeners/`
2. Implement `Listener` trait
3. Register in `build_listeners()` in `http.rs`
4. Add change types to `model.rs` if needed
5. Add serialization in `serialization.rs` if needed
6. Add VBA handler in `tabula_server_change_handlers.bas` if needed
7. Write black-box tests in `tabula_server_tests.rs`

See `docs/adding_new_effects.md` for detailed guidance.

## Performance Considerations

- Parse only needed workbook regions (use `changed_range` hint)
- In-memory cache for unchanged workbooks
- Bounded retries with exponential backoff
- Hard timeout per request (prevents Excel blocking)
- Limit total changes per response (prevent runaway behavior)

## Error Handling

**Server:**
- Fail closed (never risk workbook corruption)
- Failed listeners produce warnings but don't stop others
- Unreadable workbooks return error with `retry_after_ms`
- All errors logged with structured tracing

**VBA:**
- Respect `retry_after_ms` from server
- Bounded retry attempts (default MAX_RETRIES = 3)
- Always restore `Application.EnableEvents` state
- Clean up temp files on all paths (success and error)
