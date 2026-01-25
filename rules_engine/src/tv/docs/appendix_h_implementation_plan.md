# Appendix H: Implementation Plan (MVP-First)

## Overview

This appendix breaks the TV implementation into granular tasks using an MVP-first
strategy. The goal is to have a working, usable application as early as possible,
then layer on additional features incrementally.

MVP 1: Read-only TOML viewer (load and display)
MVP 2: Editable viewer (write changes back to TOML)
Post-MVP: All other features (metadata, derived columns, images, etc.)

## Phase 1: MVP Read-Only Viewer

These tasks produce a working read-only TOML viewer. The prototype already has
most of this functionality; these tasks clean up and solidify it.

### Task 1: Stabilize Prototype Load Path
Review and clean up the existing load_toml_table command. Ensure it handles
errors gracefully (file not found, parse errors, permission denied). Add proper
error types with Display implementations. Verify the frontend displays errors
rather than crashing.

Dependencies: None

### Task 1a: Restructure Backend Modules
Restructure the Rust backend from flat files into the module hierarchy defined
in the design document file layout. Split toml_loader.rs into the toml/ module
with separate files: document_loader.rs, document_writer.rs, toml_mod.rs. Move
file_watcher.rs into the sync/ module with sync_mod.rs. Create stub files for
other modules (commands/, derived/, validation/, images/, uuid/, logging/,
error/) with module declarations. Keep the application functional at each step;
separate refactoring commits from feature commits.

Dependencies: Task 1

### Task 2: Add Command Line Argument Parsing
Add clap dependency. Implement argument parsing in main.rs for optional
file/directory path argument. Remove hardcoded file path. Default to
rules_engine/tabula/ when no argument provided. Display error dialog and exit
with non-zero exit code for invalid paths. Define exit codes: 0 for success,
1 for invalid arguments, 2 for file errors. Support JSONL stdout logging for
external log aggregation via --jsonl flag. Verify launching with different
paths works correctly.

Dependencies: None

### Task 3: Stabilize File Watcher
Review the existing file_watcher.rs (now in sync/ module). Ensure it handles
watcher errors gracefully (file deleted, permission changes). Verify debouncing
works correctly. Add proper cleanup on window close. Test external file
modification triggers reload.

Dependencies: Task 1a

### Task 4: Extract Frontend State Management
Create src/app_root.tsx extracting state management from App.tsx. Move useState
hooks for data and error state. Move useEffect for initialization. Move IPC
event listeners. Create src/spreadsheet_view.tsx as a thin wrapper around the
Univer spreadsheet component, accepting data and event handlers as props. Keep
App.tsx as thin wrapper rendering AppRoot. Verify read-only display still works.

Dependencies: None (frontend)

### Task 4a: Create CSS Styles Module
Create src/styles/app_styles.css with base styling for the application. Define
CSS variables for theming (colors, spacing, typography). Style the main layout
container, error states, and status indicators. Import in main.tsx. Keep styles
minimal initially; expand as components are added.

Dependencies: Task 4

### Task 4b: Configure Univer Plugins
Review and configure Univer plugin initialization per Appendix D. Verify all
required plugins are loaded in correct order: UniverRenderEnginePlugin through
UniverSheetsConditionalFormattingUIPlugin. Create src/univer_config.ts with
plugin configuration and initialization helper. Verify spreadsheet renders
correctly with all plugins active. Document which plugins are deferred to
later phases (drawing plugins for images in Phase 9).

Dependencies: Task 4

### Task 4c: Create Univer Style Overrides
Create src/styles/spreadsheet_overrides.css for Univer-specific styling
customizations. Override default Univer colors to match application theme.
Style header row appearance, cell borders, and selection highlights.
Ensure styles don't conflict with Univer's internal CSS.

Dependencies: Task 4a

### Task 5: Create Error Display Component
Create src/error_banner.tsx for displaying error states from the backend. Accept
error message as prop. Style with appropriate warning colors. Position at top
of viewport. Update app_root to display errors from load failures.

Dependencies: Task 4

### Task 6: Create IPC Bridge Module
Create src/ipc_bridge.ts with TypeScript wrappers for Tauri commands. Define
typed interfaces matching the sync protocol (Appendix C):
- TomlTableData: headers array, rows array, metadata object
- CellUpdate: file path, row index, column key, new value
- SyncState: Idle, Saving, Loading, Error
- Event types: toml-file-changed, derived-value-computed, save-completed, error-occurred
- Command types: load_file, save_cell, save_batch, start_watcher, stop_watcher
Create event subscription helpers with proper cleanup using disposable pattern.
Replace raw invoke calls throughout frontend.

Dependencies: Task 4

### Task 7: Set Up Test Crate
Create the tv_tests crate at rules_engine/tests/tv_tests/, following the existing
test crate pattern (see rules_engine/tests/parser_v2_tests/ for reference).
Implement TvTestHarness with temp file creation, load_table, save_cell, and
read_file_content methods. Define the FileSystem and Clock traits in the main
TV library for dependency injection. Implement RealFileSystem for production use.
Create initial test fixtures: simple_table.toml, with_comments.toml, sparse_data.toml.
Add just commands to parent justfile: `just tv-test` to run tests, `just tv-dev`
to start development server, `just tv-build` to build release binary.

Dependencies: Task 1a

### Task 7a: Implement Load Tests
Write integration tests for the load path using TvTestHarness:
- test_load_simple_table: Load basic array-of-tables, verify headers and row data
- test_load_sparse_data: Load file with varying columns per row
- test_load_file_not_found: Verify TvError::FileNotFound returned
- test_load_parse_error: Verify TvError::TomlParseError with invalid_syntax.toml
- test_load_unicode: Load unicode_content.toml, verify non-ASCII preserved

Dependencies: Task 7

### Task 7b: MVP 1 Manual Testing
Create a manual testing checklist and verify: launch with no args loads default
directory, launch with file path loads that file, launch with invalid path shows
error, external file modification triggers reload, TOML parse error shows error
banner.

Dependencies: Tasks 1-6, 7a

## Phase 2: MVP Editable Viewer

These tasks add the ability to edit cells and write changes back to TOML.

### Task 8: Review Save Path
Review and clean up the existing save_toml_table command. Ensure atomic write
using temp file and rename. Verify toml_edit preserves document structure
(comments, whitespace, ordering). Add proper error handling for write failures.

Dependencies: Task 7

### Task 9: Implement Cell-Level Save
Modify save command to update only the changed cell rather than rewriting the
entire document. Use toml_edit's mutable access to locate and update specific
values. Verify structure preservation with targeted updates.

Dependencies: Task 8

### Task 10: Add Save Coordination
Add a saving flag to prevent file watcher reload during active saves. After
save completes, check if file changed externally during save window. Implement
file-wins conflict resolution. Emit sync state events to frontend.

Dependencies: Task 9

### Task 10a: Implement Sync State Machine
Create sync/state_machine.rs implementing the four-state sync model from
Appendix C: Idle, Saving, Loading, Error. Use atomic flags for thread-safe
state transitions. Implement guards preventing invalid transitions (e.g.,
cannot start save while loading). Emit state change events to frontend for
status indicator updates.

Dependencies: Task 10

### Task 10b: Implement Value Converter
Create toml/value_converter.rs for bidirectional JSON-TOML value conversion.
Convert JavaScript values (string, number, boolean, null) to appropriate TOML
types. Handle edge cases: NaN, Infinity become strings; arrays become TOML
arrays; nested objects become inline tables. Preserve type information through
round-trip conversion.

Dependencies: Task 9

### Task 11: Create Status Indicator Component
Create src/status_indicator.tsx showing sync status (Saving, Saved, Error).
Display appropriate icons for each state. Auto-dismiss Saved state after
timeout. Position in corner of viewport. Subscribe to sync state events.

Dependencies: Task 6

### Task 12: Wire Up Frontend Editing
Ensure spreadsheet cell edits trigger the save command. Handle save responses
including errors. Update status indicator based on save state. Display save
errors without losing pending changes.

Dependencies: Tasks 10, 11

### Task 13: Implement Save Tests
Write integration tests for the save path using TvTestHarness:
- test_save_cell_updates_value: Edit cell, verify TOML file contains new value
- test_save_preserves_comments: Edit cell in with_comments.toml, verify comments remain
- test_save_preserves_whitespace: Verify blank lines and formatting preserved
- test_save_preserves_key_order: Verify keys remain in original order after save

Add a MockFileSystem implementation for error injection tests:
- test_save_permission_denied: MockFileSystem returns permission error, verify TvError
- test_save_disk_full: MockFileSystem returns disk full error, verify TvError

Dependencies: Task 12

### Task 13a: Implement Preservation Property Tests
Add proptest or quickcheck dependency. Write property-based tests:
- prop_roundtrip_preserves_data: Generate random TOML, load, save, reload, verify equal
- prop_comments_survive_edits: Generate TOML with comments, edit cells, verify comments
- prop_whitespace_stable: Multiple save cycles don't accumulate whitespace changes

Dependencies: Task 13

### Task 13b: MVP 2 Manual Testing
Create a manual testing checklist and verify: cell edit triggers save, TOML
file reflects change, comments in TOML preserved after edit, concurrent external
edit handled gracefully, save error displays without data loss.

Dependencies: Tasks 13, 13a

## Phase 3: Error Robustness

These tasks ensure the application handles error scenarios gracefully. The design
document specifies 18 error scenarios; these tasks implement handling for all of them.

**Testing approach:** Error scenarios use MockFileSystem for injection. Each error
type should have at least one test verifying the error is detected, the correct
TvError variant is returned, and any recovery behavior works as specified.

### Task 14: Create Error Types Module
Create error/error_types.rs with TvError enum covering all failure modes:
TomlParseError, FileNotFound, PermissionDenied, WriteError, DiskFull,
FileLocked, InvalidUtf8, MetadataCorrupt, DerivedFunctionError, ImageCacheError,
MemoryPressure. Implement Display and Error traits. Add From implementations
for common error types. Include user-friendly error messages for each variant.

Dependencies: Task 13

### Task 15: Implement Error Recovery
Define error recovery strategies for each error type: which errors allow
continued editing in read-only mode (parse errors, permission denied on write),
which require reload (external file changes), which show permanent error state
(file deleted). Implement recovery logic in save and load paths. Add retry
with exponential backoff for transient errors (file locked, network issues).

Dependencies: Task 14

### Task 16: Handle File Deletion and Movement
When watched file is deleted or moved, enter read-only state with error banner.
Preserve last known data in memory. Monitor for file reappearance at original
path. Add recovery dialog allowing user to specify new file path if file was
renamed/moved. Resume normal operation when file returns or new path provided.

Dependencies: Task 15

### Task 17: Handle Permission Errors
When file becomes unreadable, preserve last known data and show error. When
file becomes unwritable, allow viewing but reject saves with error message.
Queue pending changes for retry when permissions restore. Handle network drive
disconnection as permission/not-found error with appropriate messaging.

Dependencies: Task 15

### Task 17a: Handle Disk Full and File Lock Errors
Detect disk full errors during atomic write (temp file creation fails). Display
specific error message about disk space. Preserve pending changes in memory.
Handle file lock contention on Windows with retry and exponential backoff.
Report failure after reasonable retry duration.

Dependencies: Task 15

### Task 17b: Handle Encoding and Large File Errors
Detect invalid UTF-8 content during TOML parse and report encoding error with
line/byte offset. Add configurable size limit warning for extremely large files.
Display performance warning dialog before loading files exceeding threshold.
Allow user to proceed or cancel.

Dependencies: Task 15

### Task 17c: Handle Corrupted Metadata
When [metadata] section fails to parse but main data section parses successfully,
proceed with default metadata settings. Log warning with parse error details.
Add "Reset Metadata" action in error banner to clear corrupt metadata section
and write clean defaults.

Dependencies: Task 15

### Task 17d: Handle Derived Function Errors
Catch panics within derived function execution at the task boundary using
catch_unwind. Display error state in affected cells without crashing. Log
panic details including function name and row data. Handle invalid function
name references in metadata by displaying error in column cells and logging
the unrecognized name.

Dependencies: Task 32

### Task 17e: Handle Image Cache Corruption
Validate cached images on startup by checking file headers. Delete corrupt
entries and log warnings. Handle corrupt entries discovered during load by
deleting and refetching. Add cache integrity verification at application start.

Dependencies: Task 35

### Task 17f: Handle Memory Pressure
Monitor memory usage during large file operations. Implement LRU eviction for
image cache when memory threshold exceeded. Catch out-of-memory conditions
and display error dialog with recovery options. Preserve critical application
state during memory pressure events.

Dependencies: Task 35

### Task 17g: Handle Backend Thread Panic
Install panic hooks at application startup to catch panics in background threads.
Log panic details with backtrace information. Keep the main UI thread responsive
when background threads panic. Display error state in the UI indicating which
operation failed. For critical panics (e.g., file watcher thread), attempt
automatic restart of the failed component. For unrecoverable panics, display
error dialog suggesting application restart.

Dependencies: Task 15

## Phase 4: Multi-File Support

**Testing approach:** Use real temp directories with multiple TOML files to test
discovery and filtering. Test watcher behavior by creating/deleting files in temp
directories and verifying events are emitted correctly.

### Task 18: Implement File Discovery
Create function to scan directory for TOML files. Filter to array-of-tables
format by attempting parse. Sort alphabetically. Return list of valid file
paths.

Dependencies: Task 13

### Task 19: Create Sheet Tabs Component
Create src/sheet_tabs.tsx with tab bar UI. Display tab for each file. Highlight
active tab. Handle tab click to switch files. Show file name without extension.

Dependencies: Task 4

### Task 20: Implement Multi-Sheet State
Update app_root to manage multiple sheets. Load all files on startup. Track
active sheet for editing. Maintain separate file watcher per file. Switch
data source when tab clicked.

Dependencies: Tasks 18, 19

### Task 21: Add Directory Watcher
Extend file watcher to monitor directory for new/removed files. Emit events
when TOML files appear or disappear. Update sheet tabs accordingly.

Dependencies: Task 20

### Task 21a: Implement Sync Tests
Write integration tests for file watching and sync behavior:
- test_watcher_detects_external_change: Modify file externally, verify reload event
- test_save_blocks_reload: Start save, modify file, complete save, verify no double-reload
- test_conflict_resolution: External and internal edits concurrent, verify file-wins

Dependencies: Task 21

## Phase 5: Row Operations

These tasks implement adding and deleting rows in the spreadsheet.

### Task 22: Implement Row Addition Backend
Create command to add new row to TOML array. Insert empty table entry at
specified position or end of array. Preserve document structure with toml_edit.
Return new row index to frontend.

Dependencies: Task 13

### Task 22a: Implement Row Deletion Backend
Create command to delete row from TOML array. Remove table entry at specified
index. Shift subsequent rows. Preserve document structure. Handle deletion
of last row gracefully.

Dependencies: Task 13

### Task 22b: Add Frontend Row Operations
Add row insertion UI (button or context menu). Add row deletion UI with
confirmation for non-empty rows. Wire to backend commands. Refresh display
after row operations.

Dependencies: Tasks 22, 22a

### Task 22c: Implement Batch Edit Handling
Collect multiple cell changes from paste operations into single batch. Apply
all changes in one TOML write operation. Emit single sync event after batch
complete. Handle partial batch failure gracefully.

Dependencies: Task 9

### Task 23: Implement Row Operations Tests
Write integration tests for row operations using TvTestHarness:
- test_add_row_at_end: Add row, verify TOML array has new entry
- test_add_row_at_position: Insert row at index, verify ordering preserved
- test_delete_row: Remove row, verify TOML array updated
- test_delete_last_row: Delete when single row, verify empty array handling
- test_batch_paste: Paste multiple cells, verify single atomic write

Dependencies: Task 22c

## Phase 6: UUID Generation

### Task 24: Implement UUID Generator
Create uuid_generator.rs with ID column detection (case-insensitive, handling
"ID", "Id", "id" variations). Generate UUIDv4 for empty ID cells. Check runs
on any cell edit within a row, not just row creation. Return generated UUID
or None if column missing or cell already populated.

Dependencies: Task 13

### Task 25: Integrate UUID with Save
Update save command to call UUID generator before writing. Insert generated
UUID into TOML document. Include generated UUID in save response so frontend
can update cell display without reload.

Dependencies: Task 24

### Task 25a: Implement UUID Tests
Write integration tests for UUID generation using TvTestHarness:
- test_uuid_generated_for_empty_id: Create row with empty id, verify UUID populated
- test_uuid_case_insensitive: Test "ID", "Id", "id" column name variations
- test_uuid_not_overwritten: Edit row with existing UUID, verify UUID unchanged
- test_uuid_in_save_response: Verify generated UUID returned to frontend

Dependencies: Task 25

## Phase 7: Metadata Support

### Task 26: Define Metadata Schema
Design the [metadata] section schema. Define TypeScript and Rust types for
metadata structure. Document all supported fields, types, and defaults.

Dependencies: Task 13

### Task 27: Implement Metadata Parser
Create metadata_parser.rs to parse [metadata] section. Return Option<Metadata>
with defaults for missing fields. Handle missing section gracefully.

Dependencies: Task 26

### Task 28: Implement Metadata Serialization
Add serialization back to TOML. Update metadata section in document. Create
section if missing. Preserve unrecognized fields for forward compatibility.

Dependencies: Task 27

### Task 29: Apply Column Widths from Metadata
Read column widths from metadata on load. Apply to Univer. Detect width changes
via Univer events. Update metadata. Save on change.

Dependencies: Task 28

### Task 29a: Apply Additional Column Formatting
Read column alignment (left, center, right) from metadata. Read text wrapping
settings per column. Apply to Univer column configuration. Detect changes via
Univer events and persist to metadata.

Dependencies: Task 28

### Task 29b: Apply Number and Date Formats
Read number format strings from metadata (decimal places, thousands separator).
Read date format strings for date columns. Apply to Univer cell formatting.
Allow format customization per column via metadata.

Dependencies: Task 28

### Task 29c: Implement Metadata Tests
Write integration tests for metadata parsing and serialization:
- test_parse_metadata_columns: Load with_metadata.toml, verify column config parsed
- test_parse_metadata_validation: Verify validation rules extracted from metadata
- test_metadata_defaults: Load file without metadata, verify defaults applied
- test_metadata_round_trip: Load, modify column width, save, reload, verify preserved
- test_metadata_forward_compat: Unknown metadata fields preserved on save

Dependencies: Task 29b

## Phase 8: Derived Columns

**Testing approach:** Create test implementations of DerivedFunction trait for
testing the executor. Use deterministic test functions (e.g., one that concatenates
input values) rather than testing production functions directly. Test generation
tracking with a MockClock to control timing. Tests verify compute results, error
handling, and stale result rejection.

### Task 30: Define Derived Function Trait
Create DerivedFunction trait with name(), input_keys(), compute() methods.
Define compute() as async for long-running operations. Create DerivedResult
enum with Text, Number, Image, RichText, Error variants.

Dependencies: Task 13

### Task 31: Implement Function Registry
Create FunctionRegistry storing functions by name. Implement register and lookup.
Initialize global registry at startup. Log registered functions for debugging.

Dependencies: Task 30

### Task 32: Implement Compute Executor
Create async computation infrastructure with tokio thread pool. Queue computations
by row. Send results via Tauri events as they complete. Catch panics at task
boundary. Prioritize visible rows for computation.

Dependencies: Task 31

### Task 33: Add Generation Tracking
Track row generation counters in derived/generation_tracker.rs. Increment
counter on any row edit. Tag computation requests with current generation.
Discard computation results arriving for outdated generations. Ensure UI
eventually shows correct value.

Dependencies: Task 32

### Task 33a: Implement Result Cache
Create derived/result_cache.rs for caching computed derived values. Cache key
combines row index, generation counter, and function name. Implement LRU
eviction when cache exceeds memory threshold. Invalidate entries when row
generation changes. Cache persists only for session duration (not across
restarts).

Dependencies: Task 33

### Task 34: Add Frontend Derived Column Support
Subscribe to derived-value-computed events. Update cell display when results
arrive. Show blank/loading indicator while pending. Show error styling with
tooltip on failure. Handle all DerivedResult variants appropriately.

Dependencies: Task 33

### Task 34a: Implement Derived Column Tests
Write integration tests for derived columns using test function implementations:
- test_registry_register_lookup: Register function, verify lookup by name
- test_registry_duplicate_panics: Verify duplicate registration fails at startup
- test_executor_computes_value: Register test function, trigger compute, verify result
- test_executor_catches_panic: Function that panics, verify error result returned
- test_generation_discards_stale: Edit row twice rapidly, verify only latest result used
Use MockClock to control timing for generation tests.

Dependencies: Task 34

## Phase 9: Image Support

**Testing approach:** Define an HttpClient trait for network requests. MockHttpClient
returns predetermined responses or errors for testing cache behavior, network
timeouts, and invalid image handling. Use real temp files for cache storage tests
to verify file-based operations. Include test images in fixtures/ directory.

### Task 35: Implement Image Cache
Create content-addressed image cache in application data directory. Store by
URL hash as key. Implement get/put with file-based storage. Add LRU eviction
when cache size exceeds configurable limit. Cache never expires; persist
across restarts.

Dependencies: Task 13

### Task 36: Implement Image Fetcher
Create async HTTP image download with reqwest and connection pooling. Validate
response status and content-type. Decode to verify image validity using the
image crate. Support HTTP and HTTPS URLs. For local filesystem paths, read
directly without caching. Handle network timeouts with configurable duration.
Store fetched images in cache directory as binary files.

Dependencies: Task 35

### Task 36a: Configure Tauri Asset Protocol
Configure tauri.conf.json to enable asset protocol for loading cached images.
Add appropriate CSP rules for img-src allowing 'self', asset:, and
http://asset.localhost. Set asset protocol scope to include $APPDATA and
$CACHE directories. Configure capabilities/default.json with required
permissions for file system access, shell commands, and IPC. This allows
the frontend to load cached images directly via asset:// URLs without
base64 encoding overhead.

Dependencies: Task 13

### Task 37: Implement Image Derived Function
Create function implementing DerivedFunction trait. Construct image URL from
row data (e.g., from image number field). Check cache first. Fetch and cache
on miss. Return local cache file path for cached images, or error status.
Frontend will convert path to asset URL.

Dependencies: Tasks 36, 36a, 31

### Task 38: Add Frontend Image Cell Support
Install Univer drawing packages: @univerjs/drawing, @univerjs/drawing-ui,
@univerjs/sheets-drawing, @univerjs/sheets-drawing-ui (or use the preset
@univerjs/preset-sheets-drawing). Use convertFileSrc() from @tauri-apps/api/core
to convert cache paths to asset URLs. Call FWorksheet.insertImage(assetUrl,
column, row) to display floating images positioned at cells. Display loading
placeholder while fetch is pending. Display error placeholder icon for failed
loads (network timeout, 404, invalid format). Show error details in tooltip
on hover. Ensure image errors don't affect other cells.

Dependencies: Task 37

### Task 38a: Implement Image Tests
Write integration tests for image caching and fetching using MockHttpClient:
- test_cache_miss_fetches: Request uncached URL, verify HTTP fetch occurs
- test_cache_hit_no_fetch: Request cached URL twice, verify single HTTP fetch
- test_cache_content_addressed: Same image from different URLs shares cache entry
- test_fetch_timeout_error: MockHttpClient returns timeout, verify error result
- test_fetch_404_error: MockHttpClient returns 404, verify error result
- test_invalid_image_rejected: MockHttpClient returns non-image, verify validation fails
- test_lru_eviction: Fill cache beyond limit, verify oldest entries evicted
Include test images (PNG, JPEG) in fixtures/images/ directory.

Dependencies: Task 38

## Phase 10: Validation

### Task 39: Implement Validation Rule Parser
Parse validation rules from metadata. Define ValidationRule struct with type,
constraints, error message. Support validation types: enum (allowed value list),
range (numeric min/max), pattern (regex), required (non-empty), type (numeric
or boolean).

Dependencies: Task 27

### Task 40: Implement Validators
Create validators for each rule type. Return validation result with error
message. Handle required vs optional fields. Validate cells on edit before
save.

Dependencies: Task 39

### Task 41: Integrate Validation with Save
Validate cell values before writing to TOML. Reject invalid values with
descriptive error message. Return validation errors to frontend. Keep cell
in error state until corrected (red border, error icon).

Dependencies: Task 40

### Task 42: Add Frontend Dropdown Support
Configure Univer data validation for enum columns. Read allowed values from
metadata. Set up dropdown UI with type-ahead filtering. Reject values not in
allowed list on commit.

Dependencies: Task 39

### Task 43: Add Checkbox Cell Support
Detect boolean columns from metadata or data type inference. Render boolean
values as interactive checkboxes within cells using Univer cell rendering.
Handle checkbox click to toggle value and trigger save. Ensure checkbox state
reflects current TOML boolean value.

Dependencies: Task 39

### Task 43a: Implement Validation Tests
Write integration tests for validation rules:
- test_enum_validation_pass: Value in allowed list, verify save succeeds
- test_enum_validation_fail: Value not in list, verify TvError::ValidationFailed
- test_range_validation: Numeric value within/outside bounds
- test_pattern_validation: Text matching/not matching regex
- test_required_validation: Empty value in required field rejected
- test_type_validation_numeric: Non-numeric string in numeric column rejected
- test_multiple_rules_combine: Column with multiple rules, all must pass

Dependencies: Task 43

## Phase 11: Rules Text Preview

### Task 44: Implement Fluent Integration
Load strings.ftl from rules_engine/l10n/. Parse into FluentResource. Store
for template processing. Handle missing translation gracefully.

Dependencies: Task 31

### Task 45: Implement Style Tag Parser
Parse HTML-like styling tags (bold, italic, underline, color spans). Generate
styled run list with character positions for Univer rich text API.

Dependencies: Task 13

### Task 46: Implement Rules Preview Function
Create DerivedFunction for rules preview. Parse variables from companion
variables column in same row. Format through Fluent template system. Parse
style tags for rich text output. Integrate with existing ability parser from
rules_engine to validate syntax. Highlight parse errors with error styling
at error location. Return rich text result with all variable substitutions.

Dependencies: Tasks 44, 45

### Task 47: Implement Card Lookup Function
Create DerivedFunction for card name lookups from UUID references. Accept
cell value containing card UUID. Look up card name from cards.toml data.
Return card name text or "Unknown Card" if UUID not found. Cache lookup
results for performance.

Dependencies: Task 31

### Task 48: Add Frontend Rich Text Support
Handle RichText derived results. Apply Univer rich text formatting with bold,
italic, underline, and colored spans per the styled run list. Display syntax
errors from ability parser with red underline or background styling.

Dependencies: Task 46

### Task 48a: Implement Rules Preview Tests
Write integration tests for rules text preview:
- test_variable_substitution: Simple {$var} replaced with value
- test_fluent_term_expansion: Term reference like {-keyword(k: "Foresee")} expanded
- test_style_tag_bold: <b>text</b> produces bold styled run
- test_style_tag_color: <color=#FF0000>text</color> produces colored run
- test_nested_style_tags: Nested bold/color tags produce combined styling
- test_missing_variable_error: Missing variable shows error in output
- test_invalid_fluent_syntax_error: Malformed expression shows error
Include test cases from Appendix G example flow.

Dependencies: Task 48

## Phase 12: Sorting and Filtering

These tasks implement visual-only sorting and filtering that does not modify
the underlying TOML file order.

### Task 49: Implement Visual-Only Sort Backend
Create sort state management that tracks current sort column and direction.
Apply sort to row display order without modifying TOML array order. Implement
sort comparators for text, numeric, and boolean columns. Optionally persist
default sort state to metadata.

Dependencies: Task 28

### Task 50: Configure Univer Sort Plugin
Integrate UniverSheetsSortPlugin. Handle column header click to cycle through
ascending, descending, and no sort. Display sort indicators in column headers.
Emit sort state changes to backend for optional persistence.

Dependencies: Task 49

### Task 51: Implement Filter Backend
Create filter state management supporting multiple concurrent filters. Implement
filter types: text substring, exact value match for enums, boolean checkbox
match, numeric range. Combine multiple filters with AND logic. Track filtered
row visibility.

Dependencies: Task 28

### Task 52: Configure Univer Filter Plugin
Integrate UniverSheetsFilterPlugin. Display filter UI in column headers. Connect
filter changes to backend filter state. Update visible rows based on filter
results. Optionally persist filter state to metadata.

Dependencies: Task 51

## Phase 13: Styling

### Task 53: Implement Table Color Schemes
Define color scheme presets matching Excel table styles. Each scheme includes
header background, alternating row colors, accent colors for selection/focus.
Store scheme selection in metadata. Apply colors via Univer cell styling API.
Ensure alternating colors follow visual sort order, not data order.

Dependencies: Task 28

### Task 54: Implement Conditional Formatting
Parse conditional formatting rules from metadata. Support conditions: value
comparisons, text content matching, empty/non-empty checks. Styles include
background color, font color, bold, italic. Evaluate rules on each render.
Integrate with Univer cell styling.

Dependencies: Task 28

### Task 55: Implement Frozen Panes
Read freeze configuration from metadata (frozen rows, frozen columns). Apply
to Univer via freeze pane API on load. Detect user changes to freeze state.
Persist changes to metadata. Default to freezing header row.

Dependencies: Task 28

## Phase 14: Logging

### Task 56: Create JSON Logger
Implement tracing subscriber outputting JSONL format. Include Pacific Time
timestamps using chrono with chrono-tz. Include component tags following
hierarchical naming (e.g., "toml.loader", "sync.watcher"). Write to both
stdout and log file. Default to INFO level for production. Support log levels:
ERROR, WARN, INFO, DEBUG, TRACE.

Dependencies: Task 13

### Task 57: Add Frontend Log Forwarding
Create frontend logger module. Format logs as JSON with timestamp, level,
component, message, and context fields. Send to backend via Tauri logging
command. Also mirror to browser console for development. Aggregate with
backend logs into unified stream.

Dependencies: Task 56

### Task 58: Add Log Rotation
Implement daily log file rotation. Create logs in application data directory.
Compress old logs after rotation. Delete logs exceeding configurable retention
period. Verify log directory exists at startup.

Dependencies: Task 56

---

## Testing Strategy

### Test Crate Structure

Following this project's patterns and the design document file layout, create
a test crate at:

```
rules_engine/tests/tv_tests/
├── Cargo.toml
├── fixtures/
│   ├── simple_table.toml
│   ├── with_comments.toml
│   ├── sparse_data.toml
│   ├── with_metadata.toml
│   ├── large_table.toml
│   ├── invalid_syntax.toml
│   ├── unicode_content.toml
│   └── images/
│       ├── test_image.png
│       └── test_image.jpg
├── src/
│   ├── lib.rs
│   ├── toml_tests/
│   │   ├── toml_tests_mod.rs
│   │   ├── load_tests.rs
│   │   ├── save_tests.rs
│   │   ├── preservation_tests.rs
│   │   └── metadata_tests.rs
│   ├── sync_tests/
│   │   ├── sync_tests_mod.rs
│   │   ├── watcher_tests.rs
│   │   ├── conflict_tests.rs
│   │   └── state_machine_tests.rs
│   ├── derived_tests/
│   │   ├── derived_tests_mod.rs
│   │   ├── registry_tests.rs
│   │   ├── executor_tests.rs
│   │   └── generation_tests.rs
│   ├── validation_tests/
│   │   ├── validation_tests_mod.rs
│   │   ├── rule_tests.rs
│   │   └── dropdown_tests.rs
│   ├── image_tests/
│   │   ├── image_tests_mod.rs
│   │   ├── cache_tests.rs
│   │   └── fetch_tests.rs
│   ├── uuid_tests/
│   │   ├── uuid_tests_mod.rs
│   │   └── generator_tests.rs
│   ├── row_operation_tests/
│   │   ├── row_operation_tests_mod.rs
│   │   └── add_delete_tests.rs
│   ├── rules_preview_tests/
│   │   ├── rules_preview_tests_mod.rs
│   │   ├── fluent_tests.rs
│   │   └── style_tag_tests.rs
│   └── integration_tests/
│       ├── integration_tests_mod.rs
│       └── end_to_end_tests.rs
└── test_utils/
    ├── test_utils_mod.rs
    ├── mock_filesystem.rs
    ├── mock_clock.rs
    ├── mock_http_client.rs
    ├── fixture_loader.rs
    └── assertion_helpers.rs
```

### Test Utilities Library

The `src/lib.rs` exports a `TvTestHarness` that provides:
- Methods to create temporary TOML files with specific content
- Methods to call the public TV library functions (load, save)
- Methods to simulate file system events
- Methods to verify TOML file contents after operations

Example:
```rust
pub struct TvTestHarness {
    temp_dir: TempDir,
    fs: Box<dyn FileSystem>,
}

impl TvTestHarness {
    /// Create harness with real file system (most tests)
    pub fn new() -> Self { ... }

    /// Create harness with mock file system (error injection tests)
    pub fn with_mock_fs(mock: MockFileSystem) -> Self { ... }

    pub fn create_toml_file(&self, name: &str, content: &str) -> PathBuf { ... }
    pub fn load_table(&self, path: &Path) -> Result<TomlTableData, TvError> { ... }
    pub fn save_cell(&self, path: &Path, row: usize, col: &str, value: Value) -> Result<(), TvError> { ... }
    pub fn read_file_content(&self, path: &Path) -> String { ... }
}
```

### Mock Implementations

**MockFileSystem** - For error injection testing:
```rust
pub struct MockFileSystem {
    read_result: Option<io::Result<String>>,
    write_result: Option<io::Result<()>>,
}

impl MockFileSystem {
    pub fn failing_read(error: io::Error) -> Self { ... }
    pub fn failing_write(error: io::Error) -> Self { ... }
}
```

**MockHttpClient** - For image fetching tests:
```rust
pub struct MockHttpClient {
    responses: HashMap<String, MockResponse>,
}

impl MockHttpClient {
    pub fn with_response(url: &str, status: u16, body: Vec<u8>) -> Self { ... }
    pub fn with_timeout(url: &str) -> Self { ... }
}
```

**MockClock** - For deterministic timing:
```rust
pub struct MockClock {
    current_time: AtomicU64,
}

impl MockClock {
    pub fn advance(&self, duration: Duration) { ... }
}
```

### Public API Testing

All tests call the public library API (the same functions the Tauri commands
call internally). Tests do NOT test internal implementation functions.

Example test:
```rust
#[test]
fn test_load_simple_table() {
    let harness = TvTestHarness::new();
    let path = harness.create_toml_file("test.toml", r#"
[[cards]]
id = "abc-123"
name = "Test Card"
"#);

    let table = harness.load_table(&path).unwrap();

    assert_eq!(table.headers, vec!["id", "name"]);
    assert_eq!(table.rows.len(), 1);
    assert_eq!(table.rows[0]["id"], "abc-123");
}
```

### Dependency Injection Strategy

If external dependencies need to be abstracted for testing, use trait-based
dependency injection at the library boundary. The TV library defines traits
for external operations:

```rust
pub trait FileSystem {
    fn read_to_string(&self, path: &Path) -> io::Result<String>;
    fn write(&self, path: &Path, content: &str) -> io::Result<()>;
    fn exists(&self, path: &Path) -> bool;
}

pub trait Clock {
    fn now(&self) -> SystemTime;
}
```

The library functions accept these traits as parameters or use a context struct:

```rust
pub fn load_toml_table(
    fs: &impl FileSystem,
    path: &Path,
) -> Result<TomlTableData, TvError> { ... }
```

In production, the Tauri commands pass a `RealFileSystem` implementation.
In tests, the harness can pass a `TestFileSystem` that operates on temp files
or returns predetermined responses for error testing.

### When to Use Fakes vs Real Dependencies

**Use real file system for most tests:** Since TOML file operations are the
core functionality, most tests should use real temp files. This catches issues
with actual file I/O, path handling, and permission behavior.

**Use fakes for:**
- Error injection (simulate permission denied, disk full)
- Deterministic time (if timestamp-dependent behavior needs testing)
- Network operations (image fetching in derived columns)

### Test Fixtures

Test fixture files live in `rules_engine/tests/tv_tests/fixtures/` as shown in
the directory structure above. Each fixture tests specific scenarios:
- `simple_table.toml` - Basic array of tables for happy path testing
- `with_comments.toml` - TOML with inline and block comments for preservation tests
- `sparse_data.toml` - Rows with different column sets for header collection
- `with_metadata.toml` - File including [metadata] section for metadata parsing
- `large_table.toml` - Many rows for performance and memory testing
- `invalid_syntax.toml` - Intentionally malformed for error handling tests
- `unicode_content.toml` - Non-ASCII text in values for encoding tests

### Test Categories

**Load tests** verify:
- Simple table loading
- Sparse data (different columns per row)
- Error handling (file not found, parse error)
- Unicode content
- Large files

**Save tests** verify:
- Cell update modifies correct value
- Comments preserved after save
- Whitespace preserved after save
- Key ordering preserved after save
- Error handling (permission denied, disk full via fake)

**Sync tests** verify:
- File watcher detects external changes
- Save coordination prevents reload during save
- Conflict resolution (file-wins)

**Preservation tests** verify:
- Round-trip: load, save, reload produces identical data
- Comments survive multiple edit cycles
- Inline table format preserved
- Property-based tests with random TOML documents

### Continuous Integration

Tests run via `just tv-test` which:
1. Builds the tv_tests crate
2. Runs all integration tests
3. Reports coverage of public API surface

---

## Task Dependency Summary

**Phase 1 - MVP 1 (Read-Only) - Can start immediately:**
- Tasks 1, 2 (backend, parallel)
- Tasks 4, 5, 6 (frontend, parallel)
- Task 1a (restructure modules) depends on Task 1
- Task 3 depends on Task 1a
- Task 4a (CSS styles) depends on Task 4
- Task 4b (Univer plugins) depends on Task 4
- Task 4c (Univer style overrides) depends on Task 4a
- Task 7 (test crate setup + just commands) depends on Task 1a
- Task 7a (load tests) depends on Task 7
- Task 7b (manual testing) depends on Tasks 1-6, 1a, 4a-4c, 7a

**Phase 2 - MVP 2 (Editable) - After MVP 1:**
- Task 8 depends on Task 7b
- Tasks 9, 10 sequential
- Task 10a (state machine) depends on Task 10
- Task 10b (value converter) depends on Task 9
- Task 11 depends on Task 6
- Task 12 depends on Tasks 10, 10a, 11
- Task 13 (save tests) depends on Task 12
- Task 13a (property tests) depends on Task 13
- Task 13b (manual testing) depends on Tasks 13, 13a

**Phase 3 - Error Robustness - After MVP 2:**
- Tasks 14-17c, 17g can start after Task 13b
- Task 17d depends on Task 32 (derived column executor) - implement after Phase 8
- Tasks 17e, 17f depend on Task 35 (image cache) - implement after Phase 9
- Task 17g (backend thread panic) depends on Task 15
- Each error type should include a test using MockFileSystem
- Note: Tasks 17d, 17e, 17f have cross-phase dependencies; implement them
  after their respective phases complete rather than blocking Phase 3

**Phase 4 - Multi-File Support - After MVP 2:**
- Tasks 18-21 can start after Task 13b
- Task 21a (sync tests) depends on Task 21

**Phase 5 - Row Operations - After MVP 2:**
- Tasks 22-22c can start after Task 13b
- Task 22c depends on Task 9
- Task 23 (row operations tests) depends on Task 22c

**Phase 6 - UUID Generation - After MVP 2:**
- Tasks 24-25 can start after Task 13b
- Task 25a (UUID tests) depends on Task 25

**Phase 7 - Metadata Support - After MVP 2:**
- Tasks 26-29b can start after Task 13b
- Task 29c (metadata tests) depends on Task 29b

**Phase 8-14 - Feature Phases - After Metadata:**
- Derived Columns (Phase 8): Tasks 30-34, 33a, 34a (tests)
- Image Support (Phase 9): Tasks 35-38, 38a (tests), depends on Task 31
- Validation (Phase 10): Tasks 39-43, 43a (tests), depends on Task 27
- Rules Preview (Phase 11): Tasks 44-48, 48a (tests), depends on Task 31
- Sorting/Filtering (Phase 12): Tasks 49-52, depends on Task 28
- Styling (Phase 13): Tasks 53-55, depends on Task 28
- Logging (Phase 14): Tasks 56-58, can start after Task 13b

**Critical Path to Working MVP:**
Task 1 → Task 1a → Task 3 → Task 7 → Task 7a → Task 7b → Task 8 → Task 9 → Task 10 → Task 10a → Task 12 → Task 13 → Task 13b

**Parallel Work Streams After MVP 2:**
1. Error Robustness (Phase 3)
2. Multi-File Support (Phase 4) + Sync Tests (Task 21a)
3. Row Operations (Phase 5)
4. UUID Generation (Phase 6)
5. Metadata Support (Phase 7) - blocks many later phases
6. Logging (Phase 14)

**Testing Principles (see Testing Strategy section):**
- All tests use TvTestHarness calling public library APIs
- Real temp files for most tests; MockFileSystem for error injection
- MockHttpClient for network operations; MockClock for timing
- Property tests verify round-trip preservation
