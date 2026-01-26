# TV (TOML Viewer) Technical Design Document

## Executive Summary

TV is a desktop application for viewing and editing TOML files in a spreadsheet
format, built with Tauri V2 and the Univer spreadsheet library. It replaces the
Excel-based workflow currently managed by tabula_cli, making TOML files the sole
source of truth for game data. TV provides bidirectional synchronization between
the spreadsheet UI and TOML files on disk, with emphasis on robustness, error
recovery, and format preservation.

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
document structure. Exposes Tauri commands for loading, saving, and watching
files. Manages the metadata section parsing and serialization.

### Layer 2: Application Layer (Rust Backend)
Manages application state, coordinates between file operations and UI, handles
derived column computation, image caching, UUID generation, and data validation.
Maintains an in-memory representation of the current document state and handles
conflict resolution when external changes occur.

### Layer 3: UI Layer (TypeScript Frontend)
Renders the Univer spreadsheet with TOML data, handles user interactions,
forwards cell changes to the backend, and displays error states. Manages
sheet navigation for multi-file views.

### IPC Communication
Frontend and backend communicate via Tauri commands (request-response) and
events (backend-to-frontend notifications). Commands handle data operations
while events notify the frontend of file changes. All IPC uses strongly-typed
JSON serialization with serde.

## Prototype Migration Strategy

The existing prototype in rules_engine/src/tv/ provides the foundation for the
production implementation. Migration proceeds incrementally while keeping the
application functional at each step.

### Current Prototype Assets
The prototype includes a working Tauri V2 setup with React frontend, Univer
spreadsheet integration, basic TOML loading and saving via toml_edit, file
watching with debouncing, and a simple bidirectional sync flow. These assets
are preserved and extended rather than rewritten.

### Migration Phases

Phase 1 involves restructuring the Rust backend from flat files into the module
hierarchy defined in the file layout appendix. The existing toml_loader.rs
splits into the toml/ module with separate files for loading, writing, and
metadata parsing. The file_watcher.rs moves into the sync/ module. New module
files are created as stubs with the existing logic migrated in.

Phase 2 extracts the frontend components from the monolithic App.tsx into
separate files: app_root.tsx for state management, spreadsheet_view.tsx for
Univer wrapper, and new components for error display and status. The
UniverSpreadsheet.tsx component is preserved with minimal changes.

Phase 3 adds the test crate rules_engine/tests/tv_tests/ with fixtures and initial integration
tests validating the restructured code matches prototype behavior. Tests
verify TOML round-trip preservation, file watching, and sync behavior.

Phase 4 implements new features incrementally: metadata parsing, derived
columns, image rendering, validation rules, and multi-file support. Each
feature is developed behind the existing working prototype functionality.

### Preserving Working State
At each migration step, the application remains runnable. Refactoring commits
are separate from feature commits. The prototype's hardcoded file path is
replaced with command-line argument parsing early in migration.

## TOML Structure and Metadata

### Array of Tables Format
TV exclusively handles TOML files in array-of-tables format where a single
array key contains all rows. Each table in the array represents one spreadsheet
row, with table keys becoming column headers. Headers are collected from all
tables in the array to support sparse data where not all rows have all fields.

### Metadata Section
Each TOML file may contain an optional [metadata] table as the final entry.
This table stores spreadsheet configuration that persists across sessions.
The metadata section is never displayed in the spreadsheet grid.

Metadata categories include:
- Column configuration: widths, alignment, text wrapping, frozen state
- Display formatting: number formats, date formats, color schemes
- Data validation rules: allowed values, type constraints, dropdown lists
- Derived column definitions: function names and parameters
- Conditional formatting rules: cell styling based on values
- Table styling: color scheme selection, header formatting
- Sort and filter state: default sort column and direction

### Local View State
TV maintains a lightweight `.tv_view_state.json` file alongside the TOML data
files for persisting local UI state across sessions. This is distinct from the
per-file TOML metadata which stores spreadsheet configuration (column widths,
sort state, etc.) within each TOML file.

The view state file stores cross-file UI preferences such as the most recently
viewed sheet. The file is placed in the parent directory of the opened TOML
files, derived from the first entry in AppPaths. The JSON schema contains an
`active_sheet_path` field holding the absolute path of the last active sheet.
On restore, the path is matched against available sheets; if the path no longer
exists in the available sheets, the first sheet is used as a fallback.

### Structure Preservation Strategy
All file writes use toml_edit::DocumentMut to parse and modify the document
in place. The save operation reads the current file, parses it with toml_edit,
applies only the changed values via mutable access, and writes back the
modified document. This preserves comments attached to values, blank lines
between sections, inline tables versus expanded tables, key ordering within
tables, and any custom formatting the user has applied externally.

## Bidirectional Synchronization

### Write Path (UI to File)
When the user edits a cell in Univer, the spreadsheet emits a command
execution event. The frontend extracts the changed cell coordinates and value,
then sends an update command to the backend. The backend locates the specific
value in the TOML document and updates it in place using toml_edit, then
writes the file atomically via a temp file and rename.

### Read Path (File to UI)
A file watcher monitors the TOML file for external changes using the notify
crate with 500ms debouncing. When changes are detected, the backend emits an
event to the frontend. The frontend reloads the data via a load command,
overwriting any pending local changes that conflict with the file.

### Conflict Resolution
The file on disk always wins in conflict scenarios. A saving flag prevents
reload during active saves to avoid immediate overwrite of user changes.
After a save completes, the watcher may trigger a reload if external changes
occurred during the save window. The UI displays the current sync status to
inform users when their changes are being saved or when external changes were
loaded.

### Cell-Level Updates
Rather than saving the entire spreadsheet on each change, the backend tracks
which specific cell was modified and updates only that value in the TOML file.
This minimizes the window for conflicts and reduces file churn. Batch edits
from operations like paste are collected and applied in a single write.

## Derived Asynchronous Columns

### Registration System
Derived columns are defined by name in the metadata section, specifying a
Rust function identifier. Functions are registered at application startup
via a registry pattern that maps names to function implementations. Each
function receives the complete row data and returns a computed value.

### Computation Model
Derived column values are computed asynchronously on the Rust side. When row
data changes, the backend queues a computation task for each derived column
in that row. Results are sent to the frontend via events as they complete.
Cells display blank while computation is pending.

### Eventually Correct Semantics
Multiple rapid edits to a row may cause in-flight computations to become stale.
The backend tracks a generation counter for each row and discards computation
results that arrive for outdated generations. This ensures the UI eventually
shows the correct value without race conditions.

### Built-in Derived Functions
Initial derived functions include image URL generation from image number
fields, rules text preview rendering with Fluent substitution, and card name
lookups from UUID references. Additional functions can be registered by
extending the function registry.

## Image Rendering

### Univer Image Types
Univer supports two types of images in spreadsheets:
- **Floating Images**: Positioned over cells, can be placed anywhere on the
  sheet with pixel-level positioning
- **Cell Images**: Embedded directly within cells (no mixed text/image layout)

TV uses floating images positioned at cell locations for displaying card
artwork. This provides flexibility for sizing while maintaining cell alignment.

### Required Packages
Image support requires these additional Univer packages beyond the base setup:
- @univerjs/drawing
- @univerjs/drawing-ui
- @univerjs/sheets-drawing
- @univerjs/sheets-drawing-ui

Or use the preset: @univerjs/preset-sheets-drawing

### Univer Image API
The Facade API provides multiple methods for inserting images:

**Floating Images (FWorksheet methods):**
- `insertImage(url, column, row, offsetX?, offsetY?)` - Insert from URL string
- `insertImage(blobSource, column, row, offsetX?, offsetY?)` - Insert from
  IFBlobSource (File/Blob converted to base64 internally)
- `newOverGridImage()` - Builder pattern with chainable `.setSource()`,
  `.setColumn()`, `.setRow()`, `.setWidth()`, `.setHeight()`, etc.

**Cell Images (FRange method):**
- `insertCellImageAsync(file: File | string)` - Insert from File object or URL

**Supported Image Sources:**
- HTTP/HTTPS URL strings (passed directly to Univer)
- Base64 data URIs (`data:image/png;base64,...`)
- File objects (from file inputs)
- IFBlobSource objects (converted to base64 internally by Univer)

**Not supported:** Direct `file://` URLs are not accepted by Univer's API.

### Local Filesystem Images via Tauri Asset Protocol
For local filesystem images, Tauri's asset protocol converts local paths to
URLs loadable by the webview. This avoids base64 encoding overhead.

**Configuration required in tauri.conf.json:**
```json
"app": {
  "security": {
    "csp": "default-src 'self' ipc: http://ipc.localhost; img-src 'self' asset: http://asset.localhost",
    "assetProtocol": {
      "enable": true,
      "scope": {
        "requireLiteralLeadingDot": false,
        "allow": ["$APPDATA/**/*", "$CACHE/**/*"]
      }
    }
  }
}
```

**Usage:**
```typescript
import { convertFileSrc } from '@tauri-apps/api/core';
const assetUrl = convertFileSrc('/path/to/cached/image.png');
// Returns: http://asset.localhost/path/to/cached/image.png
```

The asset URL can then be passed directly to Univer's insertImage method.

### Image Sources
Images come from two sources:
- **Remote URLs**: HTTP/HTTPS URLs for card artwork hosted externally
- **Local cache**: Previously-fetched images stored in the app data directory

The image source is determined by a derived column function that constructs
the URL from row data such as an image number field.

### Caching Strategy
Remote images are cached locally in a content-addressed store:
- Cache key: SHA-256 hash of the source URL
- Storage: Binary files in the application cache directory
- Expiration: Never (cache persists across restarts)
- Eviction: LRU when cache size exceeds configurable limit

For cached images, the Rust backend returns the local cache path. The frontend
uses Tauri's `convertFileSrc()` to create an asset protocol URL, then passes
this URL to Univer's insertImage method.

### Rendering Pipeline
1. Cell with image reference becomes visible
2. Frontend requests image via derived column system
3. Backend checks cache for URL hash
4. Cache hit: Return local file path
5. Cache miss: Fetch from remote, validate, store in cache, return path
6. Frontend converts path to asset URL via `convertFileSrc()`
7. Frontend calls `insertImage(assetUrl, column, row)` on Univer

### Error Handling
Failed image loads display a placeholder icon in the cell with a tooltip
describing the error. Error types include:
- Network timeouts (configurable timeout duration)
- HTTP errors (404, 403, 500, etc.)
- Invalid image format (not decodable by image crate)
- Corrupt cache entries (deleted and refetched)

Image errors are isolated and don't affect other cells or operations.

### Performance Considerations
Using the asset protocol instead of base64 avoids the ~33% size overhead of
base64 encoding. The webview loads images directly from the filesystem via
the asset protocol, which is efficient for large images.

For remote images, the cache ensures each image is fetched only once. Cache
files are stored as original binary format without re-encoding.

## UUID Generation

### Automatic ID Population
When a row is created or edited, the backend checks if an "id" column exists
and if the cell is empty. If both conditions are true, a new UUIDv4 is
generated and inserted into that cell. This matches existing tabula_cli
behavior for seamless transition.

### Detection Logic
Column detection is case-insensitive and handles variations like "ID", "Id",
and "id". The check runs on any cell edit within a row, not just on row
creation, to handle cases where the ID was accidentally cleared.

### Write Integration
UUID generation occurs during the save operation before the TOML file is
written. The generated UUID is returned to the frontend as part of the
save response so the UI can update the cell display without a reload.

## Rules Text Preview

### Fluent Integration
Rules text columns containing template placeholders are processed through the
Fluent localization system. Variables in curly braces are substituted with
values from a companion variables column in the same row. The formatted result
is displayed in a derived preview column.

### Styling Application
The preview rendering supports rich text styling including bold, italic,
underline, and colored spans. Styling is specified via inline tags in the
Fluent output. Univer rich text cells display the formatted result with
appropriate visual styling.

### Parser Integration
The existing ability parser from rules_engine is integrated to validate rules
text syntax. Parse errors are displayed as styling in the preview cell,
highlighting the error location. Successfully parsed rules show the formatted
preview with all variable substitutions applied.

## Data Validation

### Validation Rule Types
Supported validation rules include type constraints requiring numeric or
boolean values, enumeration constraints with allowed value lists, range
constraints for numeric bounds, pattern constraints with regex matching,
and required field constraints preventing empty cells.

### Dropdown Support
Enumeration constraints render as dropdown selectors in the cell editor.
The dropdown lists all allowed values with filtering as the user types.
Invalid values are rejected on commit with an error message.

### Checkbox Cells
Boolean columns render as interactive checkboxes within cells. Clicking the
checkbox toggles the value and triggers a save. The checkbox state reflects
the current TOML boolean value.

### Validation Feedback
Cells failing validation display a red border and icon. Hovering shows the
validation error message. Invalid values can be typed but are not saved until
corrected, with the cell remaining in an error state.

## Sorting and Filtering

### Visual-Only Sort
Sorting changes the display order in the spreadsheet without modifying the
TOML file. The original file order is preserved and restored when sort is
cleared. Sort state can optionally be stored in metadata for persistence
across sessions.

### Column Sort
Clicking a column header sorts by that column ascending. Clicking again
reverses to descending. A third click clears the sort. Sort indicators in
the header show current sort state.

### Filter Interface
The filter panel allows filtering rows by column values. Filter types include
text substring matching, exact value matching for enums, boolean checkbox
matching, and numeric range matching. Multiple filters combine with AND logic.

### Univer Integration
Sorting and filtering leverage the built-in UniverSheetsSortPlugin and
UniverSheetsFilterPlugin. The backend provides the data and the frontend
configures the plugins according to column metadata.

## Table Coloring and Styling

### Color Schemes
TV provides a selection of predefined color schemes matching Excel table
styles. Each scheme defines a header background color, alternating row colors,
and accent colors for selection and focus states. The selected scheme is
stored in metadata.

### Alternating Row Colors
Rows alternate between two background colors to improve readability. The
alternation follows the visual sort order, not the underlying data order,
so the pattern remains consistent when rows are reordered.

### Conditional Formatting
Rules can be defined to apply styling based on cell values. Conditions include
value comparisons, text content matching, and empty/non-empty checks. Styles
include background color, font color, bold, and italic. Rules are stored in
metadata and evaluated on each render.

## Multiple File Support

### Sheet Organization
Each TOML file in the target directory becomes a separate sheet in Univer.
Sheet tabs at the bottom of the window allow navigation between files.
The active sheet determines which file receives edits and is watched for
changes.

### File Discovery
When launched without arguments, TV scans the tabula directory for all TOML
files matching the array-of-tables format. Files are sorted alphabetically
for consistent ordering. New files appearing in the directory are detected
and added as sheets.

### Isolated Watchers
Each file has its own file watcher instance for external change detection.
Changes to one file do not affect other sheets. The active sheet receives
immediate updates while background sheets queue updates for when activated.

## Command Line Interface

### Launch Modes
Running tv with no arguments opens all TOML files in rules_engine/tabula/.
Running tv with a path argument opens that specific file or directory.
The path can be absolute or relative to the current working directory.

### Arguments
The path argument accepts a single file for single-sheet mode or a directory
for multi-sheet mode scanning all TOML files within. Invalid paths display
an error dialog and exit. Non-TOML files are ignored when scanning directories.

### Integration
TV can be launched from other tools via subprocess. Exit codes indicate
success or failure for scripting integration. Standard output captures
logging in JSONL format for external log aggregation.

## Error Scenarios and Mitigation

### Scenario 1: TOML Parse Error
When the TOML file cannot be parsed due to syntax errors, TV displays the
last successfully parsed state with a prominent red error banner showing the
parse error message and line number. The spreadsheet remains read-only until
the external syntax error is fixed and the file parses successfully again.

### Scenario 2: File Deleted
If the watched file is deleted, TV retains the last known data in a read-only
state with an error banner indicating the file no longer exists. If the file
reappears, normal operation resumes after automatic reload.

### Scenario 3: Permission Denied on Read
When the file cannot be read due to permissions, TV shows the last known
state if available or an empty state with an error message. The file watcher
continues monitoring for permission restoration.

### Scenario 4: Permission Denied on Write
Write failures display a save error indicator without losing the pending
change data. The user can retry saving or the system retries automatically
when the permission issue resolves. Changes queue in memory until saveable.

### Scenario 5: File Moved or Renamed
File rename or move is detected as a delete event. TV enters the file-deleted
state. If the user provides the new path via a recovery dialog, the watcher
is reattached to the new location.

### Scenario 6: Disk Full
Write failures due to insufficient disk space are caught before corrupting
the file via atomic write with temp file. The error message indicates disk
space as the issue. Pending changes remain in memory.

### Scenario 7: Network Drive Disconnect
For files on network drives, disconnection appears as permission errors or
file-not-found errors. TV enters a read-only state preserving last known
data. Reconnection triggers automatic reload.

### Scenario 8: File Locked by Another Process
Lock contention on Windows causes write failures. TV retries with exponential
backoff for a reasonable duration before reporting failure. Reading typically
succeeds even when another process holds a write lock.

### Scenario 9: Corrupted Metadata Section
If the metadata section fails to parse while the main data parses successfully,
TV proceeds with default metadata settings and logs a warning. A metadata
reset option allows the user to clear corrupt metadata.

### Scenario 10: Invalid UTF-8 Content
Non-UTF-8 content in the TOML file results in a parse error. The error message
indicates the encoding issue. TV does not attempt encoding detection or
conversion.

### Scenario 11: Extremely Large Files
Files exceeding reasonable size limits trigger a warning about potential
performance impact. TV proceeds with loading but may experience degraded
responsiveness for very large files.

### Scenario 12: Rapid External Changes
Debouncing coalesces rapid external changes into single reload events. If
changes occur faster than the debounce window, some intermediate states
may be skipped, always landing on the latest file state.

### Scenario 13: Concurrent Save and External Change
The saving flag prevents immediate reload of external changes during save.
After the save completes, if the saved content differs from the current file
content, a conflict resolution reload occurs.

### Scenario 14: Invalid Derived Function Reference
Metadata referencing a nonexistent derived function name results in an error
displayed in cells of that column. The function registry lookup failure is
logged with the invalid name for debugging.

### Scenario 15: Derived Function Panic
Panics within derived functions are caught at the task boundary. The cell
displays an error state and the panic is logged. Other derived functions
continue executing normally.

### Scenario 16: Image Cache Corruption
Corrupted cached images fail validation when loaded. Corrupt entries are
deleted and refetched on next access. Cache integrity checks run at startup.

### Scenario 17: Memory Pressure
High memory usage triggers cache eviction for least-recently-used images.
Critical application state is preserved. Out-of-memory conditions are caught
and result in an error dialog rather than a crash.

### Scenario 18: Backend Thread Panic
Panics in background threads are caught by panic hooks and logged. The main
thread remains responsive and displays an error state. Recovery may require
application restart depending on the failed operation.

## Logging Strategy

### Log Format
All logs are emitted in JSONL format with one JSON object per line. Each log
entry includes a timestamp in Pacific Time, log level, component identifier,
message text, and optional structured fields for context.

### Timestamp Formatting
Timestamps use ISO 8601 format with explicit Pacific Time offset for local
debugging convenience. The chrono crate with chrono-tz handles timezone
conversion.

### Log Levels
ERROR for failures requiring user attention, WARN for recoverable issues,
INFO for significant state changes, DEBUG for detailed operation tracing,
TRACE for verbose debugging. Production builds default to INFO level.

### Component Tags
Each log entry includes a component tag indicating the source module. Tags
follow a hierarchical naming scheme like "toml.loader", "sync.watcher",
"ui.spreadsheet" for filtering and aggregation.

### Frontend Logging
TypeScript frontend logs serialize to JSON and send to the backend via a
logging command. Backend aggregates frontend and backend logs into a single
unified stream written to both stdout and a log file.

### Log Rotation
Log files rotate daily with a configurable retention period. Old logs are
compressed and eventually deleted. The log directory lives within the
application data folder.

## Testing Strategy

### Integration Test Architecture
All tests are integration tests against public APIs. No inline module tests
or unit tests of internal functions. Tests live in a separate test crate
with access to the library's public interface.

### Test Categories
File operation tests verify TOML loading, saving, and structure preservation.
Sync tests verify bidirectional sync behavior including conflict scenarios.
Derived column tests verify function registration and computation. Validation
tests verify all validation rule types. UI tests verify Univer integration
via headless rendering.

### Test Data
Test fixtures include sample TOML files covering edge cases like empty files,
files with comments, sparse data, deeply nested structures, and large files.
Fixtures are version controlled and documented.

## File Layout

### Source Directory Structure
```
rules_engine/src/tv/
├── docs/
│   └── tv_design_document.md
├── src/
│   ├── main.tsx
│   ├── app_root.tsx
│   ├── spreadsheet_view.tsx
│   ├── error_banner.tsx
│   ├── status_indicator.tsx
│   └── styles/
│       └── app_styles.css
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── commands/
│   │   │   ├── commands_mod.rs
│   │   │   ├── load_command.rs
│   │   │   ├── save_command.rs
│   │   │   ├── watch_command.rs
│   │   │   └── compute_command.rs
│   │   ├── toml/
│   │   │   ├── toml_mod.rs
│   │   │   ├── document_loader.rs
│   │   │   ├── document_writer.rs
│   │   │   ├── metadata_parser.rs
│   │   │   └── structure_preserver.rs
│   │   ├── sync/
│   │   │   ├── sync_mod.rs
│   │   │   ├── file_watcher.rs
│   │   │   ├── change_tracker.rs
│   │   │   └── conflict_resolver.rs
│   │   ├── derived/
│   │   │   ├── derived_mod.rs
│   │   │   ├── function_registry.rs
│   │   │   ├── compute_executor.rs
│   │   │   ├── image_url_function.rs
│   │   │   ├── rules_preview_function.rs
│   │   │   └── card_lookup_function.rs
│   │   ├── validation/
│   │   │   ├── validation_mod.rs
│   │   │   ├── rule_parser.rs
│   │   │   ├── type_validator.rs
│   │   │   └── enum_validator.rs
│   │   ├── images/
│   │   │   ├── images_mod.rs
│   │   │   ├── image_cache.rs
│   │   │   ├── image_fetcher.rs
│   │   │   └── image_encoder.rs
│   │   ├── uuid/
│   │   │   ├── uuid_mod.rs
│   │   │   └── uuid_generator.rs
│   │   ├── logging/
│   │   │   ├── logging_mod.rs
│   │   │   ├── json_logger.rs
│   │   │   └── log_aggregator.rs
│   │   └── error/
│   │       ├── error_mod.rs
│   │       ├── error_types.rs
│   │       └── recovery_handler.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
├── tsconfig.json
└── vite.config.ts
```

## Transition from tabula_cli

### TOML Compatibility
Existing TOML files exported by tabula_cli are directly compatible with TV.
No format conversion is required. The files work immediately with TV, and
metadata sections are added automatically on first configuration change.

### Feature Parity
TV replicates core tabula_cli features: UUID generation, Fluent rules text
processing, and spreadsheet display. Excel-specific features like macros and
VBA are not needed as TV operates directly on TOML.

### Deprecation Path
After TV reaches production readiness, tabula_cli commands related to Excel
become deprecated. Git hooks are updated to no longer require Excel. The
build-toml and build-xls commands are eventually removed.

## Dependencies

### Rust Backend
- tauri 2.x: Application framework and window management
- toml_edit 0.22: Structure-preserving TOML editing
- toml 0.8: TOML parsing for read operations
- notify 6.x: File system watching
- notify-debouncer-mini 0.4: Debounced file events
- serde 1.x: Serialization framework
- serde_json 1.x: JSON serialization for IPC
- uuid 1.x: UUID generation
- fluent: Localization and template processing
- tokio 1.x: Async runtime for derived columns
- chrono with chrono-tz: Timestamp formatting
- tracing: Structured logging
- reqwest: HTTP client for image fetching
- image: Image decoding and validation

### TypeScript Frontend
- @tauri-apps/api 2.x: Tauri JavaScript bindings
- @univerjs/core and related packages 0.15.x: Spreadsheet library
- @univerjs/sheets-drawing-ui: Cell image support
- React 19.x: UI framework
- RxJS 7.x: Reactive event handling
