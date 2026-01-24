# Appendix H: Implementation Plan

## Overview

This appendix breaks the TV implementation into granular tasks suitable for
incremental development. Tasks are numbered and list dependencies to enable
parallel work while avoiding merge conflicts. Setup and infrastructure tasks
come first, followed by features in dependency order.

## Phase 1: Project Setup and Infrastructure

### Task 1: Create tv_tests Crate Structure
Create the test crate at rules_engine/src/tv_tests/ with Cargo.toml, src/lib.rs,
and empty module directories. Add the crate to the workspace Cargo.toml. Create
the fixtures/ directory with a single simple_table.toml test fixture. Verify
the crate compiles and can import tv_lib as a dependency.

Dependencies: None

### Task 2: Create Backend Module Directory Structure
Create empty module directories under src-tauri/src/ for commands/, toml/,
sync/, derived/, validation/, images/, uuid/, logging/, and error/. Add mod.rs
files (named as *_mod.rs per naming convention) with empty module declarations.
Update lib.rs to declare all top-level modules. Verify compilation succeeds.

Dependencies: None

### Task 3: Create Error Types Module
Implement error/error_types.rs with a comprehensive TvError enum covering all
failure modes: TomlParseError, FileNotFound, PermissionDenied, WriteError,
ValidationError, DerivedFunctionError, ImageFetchError, etc. Implement Display
and Error traits. Add From implementations for common error types.

Dependencies: Task 2

### Task 4: Create Logging Infrastructure
Implement logging/json_logger.rs with a custom tracing subscriber that outputs
JSONL format. Include Pacific Time timestamps using chrono-tz. Implement
logging/log_aggregator.rs with a Tauri command for frontend log ingestion.
Add initialization function called from lib.rs during app startup.

Dependencies: Task 2, Task 3

### Task 5: Add Log Rotation Support
Implement logging/log_rotation.rs with daily log file rotation. Create log
files in the application data directory. Add cleanup of old log files based
on configurable retention period. Integrate with the json_logger to write
to both stdout and rotating log files.

Dependencies: Task 4

### Task 6: Create Command Line Argument Parsing
Add clap dependency to Cargo.toml. Implement argument parsing in main.rs for
optional file/directory path argument. Store parsed arguments in application
state accessible to commands. Remove hardcoded file path from the prototype.
Default to rules_engine/tabula/ when no argument provided.

Dependencies: Task 2

### Task 7: Implement Recovery Handler
Implement error/recovery_handler.rs with strategies for recovering from
various error states. Define RecoveryAction enum with variants like Retry,
ShowError, EnterReadOnly, ReloadFile. Create function to map TvError to
appropriate RecoveryAction. Add panic hook for backend thread panics.

Dependencies: Task 3

## Phase 2: TOML Layer

### Task 8: Extract Document Loader
Move TOML loading logic from toml_loader.rs to toml/document_loader.rs.
Create TomlDocument struct wrapping toml_edit::DocumentMut. Implement load
function taking file path, returning Result<TomlDocument, TvError>. Add
method to extract table by name as TomlTableData.

Dependencies: Task 3

### Task 9: Extract Document Writer
Move TOML saving logic to toml/document_writer.rs. Implement atomic write
using temp file and rename. Create save_cell function taking document, row
index, column key, and new value. Return Result indicating success or error.
Preserve file structure using toml_edit mutation.

Dependencies: Task 8

### Task 10: Implement Structure Preserver
Create toml/structure_preserver.rs with utilities for toml_edit operations.
Implement find_value_mut to locate a specific cell value by table index and
key. Implement update_value to change a value while preserving formatting.
Add helper to detect and preserve inline table vs expanded table format.

Dependencies: Task 8

### Task 11: Implement Value Converter
Create toml/value_converter.rs with bidirectional conversion between JSON
and TOML types. Handle all primitive types: string, integer, float, boolean,
null. Return None for complex types (arrays, objects) that cannot be cell
values. Add unit tests in tv_tests for all conversion cases.

Dependencies: Task 1, Task 8

### Task 12: Implement Metadata Parser
Create toml/metadata_parser.rs to parse the [metadata] section from TOML
documents. Define Metadata struct with fields for all configuration options.
Implement parse function returning Option<Metadata>. Handle missing metadata
gracefully with defaults. Add validation for schema version.

Dependencies: Task 8

### Task 13: Add Metadata Serialization
Extend metadata_parser.rs with serialization back to TOML. Implement function
to update metadata section in TomlDocument. Create metadata section if it
does not exist. Preserve existing metadata fields not being updated. Add
tests for round-trip metadata preservation.

Dependencies: Task 12

### Task 14: Implement Load Command
Create commands/load_command.rs with the load_toml_table Tauri command.
Use document_loader to read file. Extract headers and rows. Parse metadata.
Return TomlTableData struct to frontend. Add proper error handling with
TvError conversion to user-friendly messages.

Dependencies: Task 8, Task 12

### Task 15: Implement Save Command
Create commands/save_command.rs with save_cell Tauri command. Load document,
apply cell update via structure_preserver, write via document_writer. Return
success with any generated values. Handle concurrent access with file locking
on Windows. Add debounce coordination with frontend.

Dependencies: Task 9, Task 10, Task 11

## Phase 3: Synchronization Layer

### Task 16: Extract File Watcher
Move file watching logic to sync/file_watcher.rs. Create FileWatcher struct
managing notify watcher instance. Implement start method taking AppHandle
and file path. Emit toml-file-changed events on modifications. Add stop
method for cleanup. Handle watcher errors gracefully.

Dependencies: Task 3

### Task 17: Implement Watch Command
Create commands/watch_command.rs with start_file_watcher Tauri command.
Store active watchers in application state. Prevent duplicate watchers for
same file. Return watcher ID for later reference. Add stop_file_watcher
command to terminate watching.

Dependencies: Task 16

### Task 18: Implement Change Tracker
Create sync/change_tracker.rs to track pending cell changes. Store changes
in memory until save completes. Implement generation counter per row for
staleness detection. Provide method to get pending changes for a file.
Clear pending changes after successful save.

Dependencies: Task 3

### Task 19: Implement Sync State Machine
Create sync/state_machine.rs with SyncState enum: Idle, Saving, Loading,
Error. Implement state transitions with guards. Use atomic flags to prevent
race conditions. Provide query methods for current state. Emit state change
events for frontend status display.

Dependencies: Task 18

### Task 20: Implement Conflict Resolver
Create sync/conflict_resolver.rs with logic for handling concurrent edits.
Detect when file changed during save window. Implement file-wins strategy
discarding local changes on conflict. Queue reload after conflict detection.
Log conflicts for debugging.

Dependencies: Task 19

### Task 21: Integrate Sync Components
Update load and save commands to use sync state machine. Coordinate file
watcher events with sync state. Prevent reload during active save. Trigger
reload after external change when idle. Update frontend event handling to
respect sync state.

Dependencies: Task 14, Task 15, Task 17, Task 19, Task 20

## Phase 4: Frontend Restructuring

### Task 22: Extract App Root Component
Create src/app_root.tsx extracting state management from App.tsx. Move
useState hooks for data, error, and status. Move useEffect for initialization.
Move IPC event listeners. Keep App.tsx as thin wrapper rendering AppRoot.
Verify existing functionality unchanged.

Dependencies: None (frontend)

### Task 23: Extract Spreadsheet View Component
Create src/spreadsheet_view.tsx moving Univer-related code from App.tsx.
Accept data and onChange as props. Encapsulate Univer initialization and
cleanup. Export TomlTableData interface. Verify spreadsheet renders and
edits propagate correctly.

Dependencies: Task 22

### Task 24: Create IPC Bridge Module
Create src/ipc_bridge.ts with TypeScript wrappers for all Tauri commands.
Define typed interfaces for command parameters and responses. Add error
handling with typed error responses. Create event subscription helpers
with proper cleanup. Replace raw invoke calls with bridge functions.

Dependencies: Task 22

### Task 25: Create Error Banner Component
Create src/error_banner.tsx for displaying error states. Accept error
message and optional details as props. Style with red background and
warning icon. Add dismiss button for recoverable errors. Position at
top of viewport overlaying spreadsheet.

Dependencies: Task 22

### Task 26: Create Status Indicator Component
Create src/status_indicator.tsx showing sync status. Display Saving,
Saved, or Error states with appropriate icons. Auto-dismiss Saved state
after timeout. Position in corner of viewport. Subscribe to sync state
events from backend.

Dependencies: Task 22, Task 24

### Task 27: Create Frontend Logger
Create src/logger_frontend.ts with Logger class. Format log entries as
JSON objects. Send logs to backend via log_message command. Include
source file and line from stack traces. Mirror logs to browser console
for development. Configure log level from environment.

Dependencies: Task 4, Task 24

### Task 28: Integrate Frontend Components
Update app_root.tsx to use new components: SpreadsheetView, ErrorBanner,
StatusIndicator. Wire up IPC bridge for all commands. Connect frontend
logger. Verify complete application flow from load through edit to save.
Remove deprecated code from prototype.

Dependencies: Task 23, Task 24, Task 25, Task 26, Task 27

## Phase 5: UUID Generation

### Task 29: Implement UUID Generator
Create uuid/uuid_generator.rs with ID column detection and UUID generation.
Implement find_id_column searching for "id" column case-insensitively.
Generate UUIDv4 strings. Return generated UUID or None if column missing
or cell non-empty.

Dependencies: Task 3

### Task 30: Integrate UUID with Save Command
Update save_command to call UUID generator before writing. Check if edited
row needs UUID generation. Include generated UUID in save response. Update
frontend to apply generated UUID to cell display without full reload.

Dependencies: Task 15, Task 29

## Phase 6: Derived Columns Infrastructure

### Task 31: Define Derived Function Trait
Create derived/derived_mod.rs with DerivedFunction trait definition. Define
methods: name(), input_keys(), compute(), is_async(). Create DerivedResult
enum with variants: Text, Number, Boolean, Image, RichText, Error. Document
trait contract and usage.

Dependencies: Task 3

### Task 32: Implement Function Registry
Create derived/function_registry.rs with FunctionRegistry struct. Store
functions in HashMap by name. Implement register and lookup methods.
Create global registry instance initialized at startup. Add error for
duplicate registration.

Dependencies: Task 31

### Task 33: Implement Compute Executor
Create derived/compute_executor.rs with async computation infrastructure.
Create tokio thread pool for async functions. Implement task queuing with
priority for visible rows. Track generation counters to discard stale
results. Send results via Tauri events.

Dependencies: Task 31, Task 32

### Task 34: Implement Compute Command
Create commands/compute_command.rs with Tauri command to trigger derived
computation. Accept row index and column name. Look up function in registry.
Execute computation and emit result event. Handle errors gracefully.

Dependencies: Task 33

### Task 35: Implement Generation Tracker
Create derived/generation_tracker.rs to track row generations. Increment
generation on any row edit. Store current generation with queued tasks.
Compare generation when result arrives. Discard results for outdated
generations.

Dependencies: Task 33

### Task 36: Add Frontend Derived Column Support
Update spreadsheet_view.tsx to handle derived columns. Subscribe to
derived-value-computed events. Update cell display when results arrive.
Show blank cells while computation pending. Display error styling for
failed computations.

Dependencies: Task 28, Task 34

## Phase 7: Image Support

### Task 37: Implement Image Cache
Create images/image_cache.rs with content-addressed cache. Store images
by URL hash in application data directory. Implement get and put methods.
Return cached image or None. Add cache size tracking for LRU eviction.

Dependencies: Task 3

### Task 38: Implement Image Fetcher
Create images/image_fetcher.rs with async HTTP image download. Use reqwest
with timeout configuration. Validate response content type. Decode image
to verify validity. Return image bytes or error.

Dependencies: Task 37

### Task 39: Implement Image Encoder
Create images/image_encoder.rs with base64 encoding for frontend transport.
Encode image bytes to data URI format. Include content type in URI. Handle
common formats: PNG, JPEG, GIF, WebP.

Dependencies: Task 38

### Task 40: Implement Image URL Derived Function
Create derived/functions/image_url_function.rs implementing DerivedFunction.
Read image_number from row data. Construct URL using configurable template.
Fetch via image_fetcher, cache result, return base64-encoded image.
Register in function registry.

Dependencies: Task 32, Task 38, Task 39

### Task 41: Add Frontend Image Cell Support
Add @univerjs/sheets-drawing-ui package if not present. Update spreadsheet
view to call insertImage for image-type derived results. Handle image
load events. Display placeholder during load. Show error icon on failure.

Dependencies: Task 36, Task 40

## Phase 8: Rules Text Preview

### Task 42: Implement Fluent Resource Loader
Create derived/functions/fluent_loader.rs to load and cache strings.ftl.
Parse into FluentResource at startup. Store in static or registry. Handle
parse errors gracefully with descriptive messages.

Dependencies: Task 32

### Task 43: Implement Style Tag Parser
Create derived/functions/style_tag_parser.rs with HTML-like tag parsing.
Parse bold, italic, underline, color tags. Track nested style state.
Generate StyledRun list with character positions. Handle malformed tags
as literal text.

Dependencies: Task 3

### Task 44: Implement Rich Text Builder
Create derived/functions/rich_text_builder.rs to build Univer rich text.
Convert StyledRun list to Univer paragraph structure. Generate ICellData-
compatible JSON. Handle all style combinations.

Dependencies: Task 43

### Task 45: Implement Rules Preview Function
Create derived/functions/rules_preview_function.rs implementing DerivedFunction.
Parse variables column to FluentArgs. Format through Fluent. Parse style
tags. Build rich text. Return RichText result for Univer rendering.

Dependencies: Task 42, Task 43, Task 44

### Task 46: Add Frontend Rich Text Support
Update spreadsheet view to handle RichText derived results. Set cell content
using Univer rich text API. Apply paragraph structure with styled runs.
Verify formatting renders correctly.

Dependencies: Task 36, Task 45

## Phase 9: Validation and Data Entry

### Task 47: Implement Validation Rule Parser
Create validation/rule_parser.rs to parse validation rules from metadata.
Define ValidationRule struct with type, constraints, error message. Parse
enum lists, numeric ranges, patterns. Return list of rules per column.

Dependencies: Task 12

### Task 48: Implement Type Validator
Create validation/type_validator.rs with type checking functions. Validate
string, integer, float, boolean constraints. Return validation result with
error message if failed. Handle empty cells according to required flag.

Dependencies: Task 47

### Task 49: Implement Enum Validator
Create validation/enum_validator.rs for enumeration validation. Check value
against allowed list. Generate error message listing valid options. Support
case-insensitive matching option.

Dependencies: Task 47

### Task 50: Integrate Validation with Save
Update save_command to validate cell value before writing. Reject invalid
values with descriptive error. Return validation errors to frontend. Keep
cell in error state until corrected.

Dependencies: Task 15, Task 48, Task 49

### Task 51: Add Frontend Dropdown Support
Configure Univer data validation for enum columns. Read enum values from
metadata. Set up dropdown UI. Apply validation rule on cell edit. Display
validation error styling for rejected values.

Dependencies: Task 28, Task 47

### Task 52: Add Frontend Checkbox Support
Configure Univer data validation for boolean columns. Render checkbox UI.
Toggle value on click. Trigger save on toggle. Apply validation for
boolean-only columns.

Dependencies: Task 28, Task 50

## Phase 10: Multi-File Support

### Task 53: Implement File Discovery
Create function to scan directory for TOML files. Filter to array-of-tables
format by attempting parse. Sort alphabetically. Return list of valid file
paths. Handle symlinks appropriately.

Dependencies: Task 8

### Task 54: Create Sheet Tabs Component
Create src/sheet_tabs.tsx with tab bar UI. Display tab for each file.
Highlight active tab. Handle tab click to switch files. Show file name
without extension as label.

Dependencies: Task 22

### Task 55: Implement Multi-Sheet State
Update app_root to manage multiple sheets. Load all files on startup.
Track active sheet for editing. Maintain separate sync state per file.
Switch data source when tab clicked.

Dependencies: Task 21, Task 53, Task 54

### Task 56: Add Directory Watcher
Extend file watcher to monitor directory for new files. Emit event when
TOML file added or removed. Update sheet tabs when directory changes.
Handle file rename as remove plus add.

Dependencies: Task 16, Task 55

## Phase 11: Styling and Formatting

### Task 57: Implement Table Color Schemes
Create color scheme definitions matching Excel table styles. Store scheme
selection in metadata. Apply header and alternating row colors on load.
Provide UI or metadata way to select scheme.

Dependencies: Task 12

### Task 58: Implement Conditional Formatting
Create validation/conditional_format.rs to evaluate formatting rules.
Parse rules from metadata. Evaluate conditions against cell values.
Return style overrides for matching cells. Integrate with Univer
conditional formatting plugin.

Dependencies: Task 12, Task 28

### Task 59: Implement Column Width Persistence
Detect column width changes via Univer events. Update metadata with new
widths. Apply widths from metadata on load. Handle auto-size for columns
without explicit width.

Dependencies: Task 13, Task 28

### Task 60: Implement Frozen Panes
Read freeze configuration from metadata. Apply to Univer on load. Detect
freeze changes and update metadata. Default to frozen header row and ID
column when not specified.

Dependencies: Task 13, Task 28

## Phase 12: Testing

### Task 61: Create TOML Test Fixtures
Create comprehensive test fixtures in tv_tests/fixtures/. Include files
with comments, sparse data, metadata, unicode, large row counts, and
intentional syntax errors. Document purpose of each fixture.

Dependencies: Task 1

### Task 62: Write TOML Loading Tests
Create toml_tests/load_tests.rs with integration tests. Test loading
simple tables, sparse data, files with metadata. Verify header extraction.
Test error handling for missing files and parse errors.

Dependencies: Task 14, Task 61

### Task 63: Write TOML Preservation Tests
Create toml_tests/preservation_tests.rs testing structure preservation.
Load file, modify cell, save, reload. Verify comments preserved. Verify
whitespace preserved. Verify key ordering preserved.

Dependencies: Task 15, Task 61

### Task 64: Write Sync Tests
Create sync_tests/ with watcher and conflict tests. Test file change
detection. Test debouncing behavior. Test conflict resolution. Use mock
filesystem for isolation.

Dependencies: Task 21, Task 61

### Task 65: Write Derived Function Tests
Create derived_tests/ with registry and executor tests. Test function
registration. Test async computation. Test generation tracking and stale
result discard. Test error handling.

Dependencies: Task 35, Task 61

### Task 66: Write Validation Tests
Create validation_tests/ testing all rule types. Test type validation.
Test enum validation. Test range validation. Test integration with save
command rejection.

Dependencies: Task 50, Task 61

### Task 67: Write End-to-End Tests
Create integration_tests/end_to_end_tests.rs with full workflow tests.
Test load, edit, save cycle. Test external file change handling. Test
error recovery. Test multi-file scenarios.

Dependencies: All previous tasks

## Task Dependency Summary

Tasks with no dependencies (can start immediately):
- Task 1, Task 2, Task 22

Frontend tasks (independent track after Task 22):
- Task 22 → Task 23 → Task 28
- Task 22 → Task 24 → Task 27, Task 28
- Task 22 → Task 25, Task 26 → Task 28

Backend infrastructure (early chain):
- Task 2 → Task 3 → Task 4 → Task 5
- Task 2 → Task 6
- Task 3 → Task 7

TOML layer (depends on infrastructure):
- Task 3 → Task 8 → Task 9, Task 10, Task 11, Task 12
- Task 12 → Task 13

Sync layer (depends on TOML):
- Task 8 → Task 16 → Task 17
- Task 16 → Task 18 → Task 19 → Task 20 → Task 21

Critical path to working prototype:
Task 2 → Task 3 → Task 8 → Task 14, Task 15 → Task 21 → Task 28
