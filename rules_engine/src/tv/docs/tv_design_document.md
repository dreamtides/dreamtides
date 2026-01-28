# TV (TOML Viewer) Technical Design Document

## Executive Summary

TV is a desktop application for viewing and editing TOML files in a spreadsheet
format, built with Tauri V2 and the Univer spreadsheet library. It replaces the
Excel-based workflow previously managed by tabula_cli, making TOML files the
sole source of truth for game data. TV provides bidirectional synchronization
between the spreadsheet UI and TOML files on disk, with emphasis on robustness,
error recovery, and format preservation.

**Example TOML file:** See [dreamwell.toml](../../../tabula/dreamwell.toml) for
a representative example of the array-of-tables format TV displays. Each
`[[dreamwell]]` entry becomes a spreadsheet row, with keys like `name`, `id`,
`energy-produced`, and `rules-text` becoming columns.

## Goals and Non-Goals

### Goals
- Replace tabula_cli Excel integration with a native TOML-first workflow
- Display TOML array-of-tables files in an editable spreadsheet format
- Implement bidirectional sync with immediate writes on cell edits
- Preserve TOML structure including comments, whitespace, and ordering
- Support derived columns computed asynchronously by Rust functions
- Render inline images from URLs or local filesystem within cells
- Provide bulletproof error recovery that never crashes or loses data
- Store spreadsheet configuration in TOML metadata sections
- Support multiple TOML files as separate sheets in a single window

### Non-Goals
- Multi-user collaboration or remote sync
- Excel format import/export
- Mobile or web deployment

## Architecture Overview

TV follows a three-layer architecture with clear separation between file
operations, application state, and UI rendering.

### Layer 1: TOML Layer (Rust Backend)
Handles all file I/O operations, TOML parsing, structure preservation, and file
watching. Uses toml_edit crate exclusively for all write operations to maintain
document structure. Exposes 43 Tauri commands for loading, saving, watching,
sorting, filtering, styling, and derived column computation. Manages metadata
section parsing and serialization.

### Layer 2: Application Layer (Rust Backend)
Manages application state via state managers (SortStateManager,
FilterStateManager, etc.), coordinates between file operations and UI, handles
derived column computation with generation tracking, image caching with LRU
eviction, UUID generation, and data validation. Uses a state machine to
coordinate sync operations and prevent conflicts.

### Layer 3: UI Layer (TypeScript Frontend)
Renders the Univer spreadsheet with TOML data, handles user interactions,
forwards cell changes to the backend, and displays error states. Manages
sheet navigation for multi-file views. Uses 23 Univer packages for spreadsheet
functionality including sorting, filtering, data validation, and drawing.

### IPC Communication
Frontend and backend communicate via Tauri commands (request-response) and
events (backend-to-frontend notifications). Commands handle data operations
while events notify the frontend of file changes, derived computation results,
and sync state. All IPC uses strongly-typed JSON serialization with serde.

**Backend Events:**
- `toml-file-changed`: External file modification detected
- `derived-value-computed`: Async computation result ready
- `save-completed`: Save operation finished
- `sync-conflict-detected`: File modified externally during save
- `autosave-disabled-changed`: Autosave state toggled

## TOML Structure and Metadata

### Array of Tables Format
TV exclusively handles TOML files in array-of-tables format where a single
array key contains all rows. Each table in the array represents one spreadsheet
row, with table keys becoming column headers. Headers are collected from all
tables in the array to support sparse data where not all rows have all fields.

### Metadata Section
Each TOML file may contain an optional [metadata] table storing spreadsheet
configuration that persists across sessions. The metadata section is never
displayed in the spreadsheet grid.

**Implemented metadata categories:**
- Column configuration: widths, alignment (left/center/right), wrap, frozen,
  hidden, bold, format
- Data validation rules: enum (with dropdowns), range, pattern, required, type
- Derived column definitions: function name, position, width, frozen, inputs
- Conditional formatting: equals, contains, greater_than, less_than, is_empty,
  not_empty, matches (regex)
- Table styling: color_scheme, show_row_stripes, show_column_stripes,
  header_bold, header_background
- Sort state: column key, ascending/descending direction
- Filter state: contains, equals, range, boolean, values conditions
- Row configuration: header_height, default_height, frozen_rows, per-row heights

### Local View State
TV maintains a `.tv_view_state.json` file alongside the TOML data files for
persisting local UI state across sessions. This stores the active sheet path.
Sheet ordering is stored in a separate `sheets.toml` file.

### Structure Preservation Strategy
All file writes use toml_edit::DocumentMut to parse and modify the document
in place. This preserves comments attached to values, blank lines between
sections, inline tables versus expanded tables, key ordering within tables,
and any custom formatting the user has applied externally.

## Bidirectional Synchronization

### Write Path (UI to File)
When the user edits a cell, the frontend debounces changes (500ms) then sends
an update command to the backend. Three save strategies exist:
- `save_cell`: Single cell update with validation
- `save_batch`: Multiple cell updates atomically
- `save_toml_table`: Full document replacement

The backend validates the value against any applicable validation rules, locates
the specific value in the TOML document, updates it in place using toml_edit,
and writes the file atomically via a temp file (`.tv_save_*` prefix) and rename.

### Read Path (File to UI)
A file watcher monitors TOML files using notify-debouncer-mini with 500ms
debouncing. When external changes are detected, the backend emits a
`toml-file-changed` event. The frontend ignores changes within 1500ms of a
completed save to prevent self-triggered reload loops.

### Conflict Resolution
The file on disk always wins in conflict scenarios. A state machine
(Idle/Saving/Loading/Error states) prevents concurrent operations. After a save
completes, if the file was modified externally during the save window, a
`sync-conflict-detected` event triggers reload to restore consistency.

### Cell-Level Updates
Rather than saving the entire spreadsheet on each change, the backend tracks
which specific cell was modified and updates only that value. Batch edits from
paste operations are collected and applied in a single atomic write.

## Derived Asynchronous Columns

### Registration System
Derived columns are defined in metadata, specifying a Rust function identifier.
Functions implement the `DerivedFunction` trait and are registered at startup
via a global registry (`OnceLock<FunctionRegistry>`). Each function receives
row data and a lookup context for cross-table references.

### Computation Model
Derived column values are computed asynchronously via tokio. When row data
changes, the frontend calls `incrementRowGeneration()` and queues computation.
Results are sent via `derived-value-computed` events. Visible rows are
prioritized in the computation queue.

### Eventually Correct Semantics
The backend tracks a generation counter for each row. Computation results
include the generation at request time; stale results (generation mismatch)
are discarded. This ensures the UI eventually shows the correct value.

### Built-in Derived Functions
- `image_url`: Constructs image URLs from image number fields
- `image_derived`: Fetches images with caching, returns local cache path
- `image_lookup`: Cross-table card ID resolution with image fetch
- `card_lookup`: Cross-table card name lookup by UUID
- `rules_preview`: Fluent-based rules text rendering with rich text styling

## Image Rendering

### Implementation Note
Univer's facade API methods (`insertImage`, `newOverGridImage`) are broken
under Vite pre-bundling due to class prototype duplication from Univer's
mixin extension system. TV uses direct command execution as a workaround:
```typescript
univerAPI.executeCommand("sheet.command.insert-sheet-image", {
  unitId, drawings: [{ drawingId, source, transform, ... }]
});
```
See `image_cell_renderer.ts` for the full implementation.

### Floating Images
TV uses floating images positioned at cell locations. Default size is 120x120px
with 4px offset from cell origin. When row height is specified, images are
scaled to fit while preserving aspect ratio.

### Caching Strategy
Remote images are cached locally in a content-addressed store:
- Cache location: `$APPDATA/image_cache/`
- Cache key: SHA-256 hash of the source URL
- Max size: 100MB with LRU eviction
- Format validation via `image` crate on fetch
- Integrity validation on startup (orphaned files removed)

### Rendering Pipeline
1. Derived column computation returns cache path
2. Frontend receives `derived-value-computed` event with image result
3. Frontend converts path to asset URL via `convertFileSrc()`
4. ImageCellRenderer waits for drawing commands to be registered (polls 5s)
5. Removes any existing image at cell position
6. Calculates fitted dimensions based on row height
7. Executes `sheet.command.insert-sheet-image` with position data

### Tauri Asset Protocol
Local images require asset protocol configuration in tauri.conf.json:
```json
"security": {
  "csp": { "img-src": "'self' asset: http://asset.localhost blob: data:" },
  "assetProtocol": { "enable": true, "scope": { "allow": ["$APPDATA/**/*"] } }
}
```

## Data Validation

### Validation Rule Types
- **Enum**: Allowlist of string values, renders as dropdown
- **Range**: Min/max numeric bounds (inclusive)
- **Pattern**: Regex string matching
- **Required**: Non-null/non-empty constraint
- **Type**: Data type enforcement (string, integer, float, boolean)

### Frontend Validation UI
Boolean columns are auto-detected and render as checkboxes via Univer's
data validation API. Enum columns render as dropdowns with type-ahead filtering.
Validation rules are parsed from metadata and applied to cell ranges.

### Validation Execution
Validation runs during save operations in the backend. Invalid values are
rejected with error messages. For batch saves, all updates are validated
before any are applied (all-or-nothing semantics).

## Sorting and Filtering

### Sort Implementation
Sorting is visual-only and does not modify the TOML file. The backend maintains
bidirectional index mappings (display-to-original and original-to-display) via
`SortStateManager`. Sort state can be persisted to metadata. Cell updates
translate display indices back to TOML indices before saving.

### Filter Implementation
Filtering supports multiple condition types: contains (case-insensitive
substring), equals (exact match), range (numeric), boolean, and values (set
membership). Multiple filters combine with AND logic. Hidden row tracking is
maintained separately from filter state.

### Univer Integration
Sorting and filtering use UniverSheetsSortPlugin and UniverSheetsFilterPlugin.
State is restored from metadata on load with suppression flags to prevent
persistence events during restore.

## Table Coloring and Styling

### Color Schemes
Predefined color schemes define header background, alternating row stripe
colors, and optional column stripe colors. Schemes are stored in metadata.

### Conditional Formatting
Rules apply styling (background color, font color, bold, italic) based on cell
values. Conditions: equals, contains, greater_than, less_than, is_empty,
not_empty, matches (regex). Later rules override earlier ones for conflicts.

## Multiple File Support

### Sheet Organization
Each TOML file becomes a separate sheet in Univer. Sheet tabs allow navigation.
Sheet ordering is persisted in `sheets.toml`. Active sheet is tracked in
`.tv_view_state.json`.

### Isolated Watchers
Each file has its own file watcher instance. Changes to one file do not affect
other sheets.

## UUID Generation

When a row is created or the "id" column is empty, a new UUIDv4 is generated
during save. Column detection is case-insensitive. Generated UUIDs are returned
in the save response, triggering a frontend reload to display the new value.

## Rules Text Preview

Rules text columns with template placeholders are processed through Fluent.
Variables are substituted from companion columns. The preview supports rich
text styling (bold, italic, underline, color) via inline HTML-like tags parsed
by `style_tag_parser.rs` and converted to Univer rich text structures.

## Error Scenarios and Mitigation

### File Errors
- **TOML Parse Error**: Display error banner with line number, read-only mode
- **File Deleted**: Retain last known data, read-only state
- **Permission Denied**: Show error, continue monitoring for restoration
- **Disk Full**: Atomic write prevents corruption, error message shown

### Sync Errors
- **Concurrent Save and External Change**: Conflict detected, reload triggered
- **Rapid External Changes**: Debouncing coalesces to single reload

### Derived Column Errors
- **Invalid Function Reference**: Error displayed in column cells
- **Function Panic**: Caught at task boundary, logged, other functions continue

### Image Errors
- **Cache Corruption**: Detected on startup, corrupt entries deleted
- **Fetch Failure**: Error displayed in cell with red text

## Logging Strategy

### Log Format
JSONL format with Pacific Time timestamps, log level, component tag, message,
and structured context fields. Component tags follow hierarchical naming
(e.g., `tv.sync.watcher`, `tv.ui.images`).

### Log Location
- macOS: `~/Library/Application Support/tv/logs/`
- Windows: `%APPDATA%\tv\logs\`
- Linux: `~/.local/share/tv/logs/`

Files are named `tv_YYYY-MM-DD.jsonl` with separate `tv_perf_*.jsonl` for
performance timing.

### Frontend Logging
TypeScript logs via `createLogger()` are sent to the backend via `logMessage`
command and unified into the same log files.

## File Layout

### Frontend Source (src/)
```
main.tsx                 # React entry point
app_root.tsx             # State management, IPC setup, multi-sheet coordination
spreadsheet_view.tsx     # Loading/error states wrapper
UniverSpreadsheet.tsx    # Univer lifecycle, data binding, event handling
error_banner.tsx         # Dismissable error/warning overlay
ipc_bridge.ts            # Tauri command/event wrappers (~35 commands, 11 events)
univer_config.ts         # Univer instance creation, plugin registration
workbook_builder.ts      # Multi-sheet workbook construction
sheet_data_utils.ts      # Data population and comparison utilities
derived_column_utils.ts  # Column mapping for derived columns
validation_utils.ts      # Checkbox and dropdown validation
table_style_utils.ts     # Color scheme and conditional formatting
header_utils.ts          # Header formatting utilities
rich_text_utils.ts       # Rich text conversion for derived results
image_cell_renderer.ts   # Floating image rendering via commands
logger_frontend.ts       # Frontend logging with backend aggregation
disabled_menu_items.ts   # 118 hidden Univer menu items
spreadsheet_types.ts     # TypeScript type definitions
```

### Rust Backend (src-tauri/src/)
```
main.rs                  # Binary entry point
lib.rs                   # Tauri builder, 43 command registrations

commands/                # Tauri command implementations
  load_command.rs        # Load file with sort/filter state restoration
  save_command.rs        # save_cell, save_batch, save_toml_table, add_row
  watch_command.rs       # start/stop_file_watcher
  derived_command.rs     # compute_derived, compute_derived_batch, etc.
  sort_command.rs        # get/set/clear_sort_state, row mapping
  filter_command.rs      # get/set/clear_filter_state, visibility
  validation_command.rs  # get_validation_rules, get_enum_validation_rules
  image_command.rs       # fetch_image (async)
  style_command.rs       # get_table_style, get_conditional_formatting
  column_command.rs      # get_column_configs, set_column_width
  row_command.rs         # get_row_config
  log_command.rs         # log_message, log_perf
  view_state_command.rs  # load/save_view_state
  sheet_order_command.rs # load/save_sheet_order

toml/                    # TOML file operations
  document_loader.rs     # TOML parsing, array-of-tables extraction
  document_writer.rs     # Atomic writes, cell-level updates, UUID integration
  metadata_parser.rs     # Parse all metadata categories (36KB)
  metadata_serializer.rs # Serialize sort/filter/column configs (30KB)
  metadata_types.rs      # Type definitions for metadata schema v1
  value_converter.rs     # JSON<->TOML value conversion
  color_schemes.rs       # Predefined color scheme definitions
  conditional_formatting.rs # Condition evaluation logic

sync/                    # File synchronization
  file_watcher.rs        # notify-debouncer-mini integration, 500ms debounce
  state_machine.rs       # Idle/Saving/Loading/Error states, conflict detection

derived/                 # Derived column system
  function_registry.rs   # Global registry, DerivedFunction trait
  compute_executor.rs    # Async execution, priority queue, result events
  generation_tracker.rs  # Per-row generation counters
  result_cache.rs        # LRU cache for computed values
  image_derived.rs       # Image fetch with caching
  image_url.rs           # Simple URL construction
  image_lookup.rs        # Cross-table card lookup with image
  card_lookup.rs         # Cross-table name lookup
  rules_preview.rs       # Fluent template processing
  fluent_integration.rs  # Fluent locale data loading
  rich_text_converter.rs # Univer rich text structure generation
  style_tag_parser.rs    # HTML-like tag parsing

validation/              # Data validation
  validation_rules.rs    # Rule types: Enum, Range, Pattern, Required, Type
  validators.rs          # Validation logic for all rule types

images/                  # Image caching
  image_cache.rs         # SHA-256 content-addressed cache, LRU eviction
  image_fetcher.rs       # HTTP fetch with reqwest, format validation

sort/                    # Sort state management
  sort_state.rs          # SortStateManager, index mapping
  sort_types.rs          # SortState, SortDirection, CellValue

filter/                  # Filter state management
  filter_state.rs        # FilterStateManager, visibility computation
  filter_types.rs        # FilterConditionState variants

view_state/              # UI state persistence
  view_state_types.rs    # ViewState (active sheet path)
  sheet_order.rs         # SheetOrder persistence

uuid/                    # UUID generation
  uuid_generator.rs      # ensure_uuids() for ID column auto-population

logging/                 # Structured logging
  json_logger.rs         # Custom tracing subscriber for JSONL
  log_aggregator.rs      # Frontend log aggregation

error/                   # Error handling
  error_types.rs         # TvError enum with all failure modes

traits/                  # Abstractions for testing
  traits_mod.rs          # FileSystem, Clock traits
```

## Dependencies

### Rust Backend
- tauri 2.x: Application framework
- toml_edit 0.22: Structure-preserving TOML editing
- toml 0.8 (preserve_order): TOML parsing
- notify-debouncer-mini: File watching with debouncing
- serde/serde_json: Serialization
- uuid: UUID generation
- fluent: Template processing
- tokio: Async runtime
- chrono/chrono-tz: Timestamps
- tracing: Structured logging
- reqwest: HTTP client
- image: Image validation

### TypeScript Frontend
- @tauri-apps/api 2.x: Tauri bindings
- @univerjs/* 0.15.3: 23 packages (all same version required)
- React 19.x, RxJS 7.x

**IMPORTANT**: All `@univerjs/*` packages must be pinned to the same exact
version. Vite's dependency pre-bundling duplicates class prototypes, breaking
facade mixins and dependency injection. See
[Appendix D](appendix_d_univer_integration.md) for workarounds.

## Debugging

View recent logs:
```bash
tail -100 ~/Library/Application\ Support/tv/logs/tv_$(date +%Y-%m-%d).jsonl
```

Search by component:
```bash
grep "tv.ui.images" ~/Library/Application\ Support/tv/logs/tv_*.jsonl
grep "tv.sync" ~/Library/Application\ Support/tv/logs/tv_*.jsonl
grep "ERROR" ~/Library/Application\ Support/tv/logs/tv_*.jsonl
```
