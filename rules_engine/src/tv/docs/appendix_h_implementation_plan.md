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

### Task 2: Add Command Line Argument Parsing
Add clap dependency. Implement argument parsing in main.rs for optional
file/directory path argument. Remove hardcoded file path. Default to
rules_engine/tabula/ when no argument provided. Verify launching with different
paths works correctly.

Dependencies: None

### Task 3: Stabilize File Watcher
Review the existing file_watcher.rs. Ensure it handles watcher errors gracefully
(file deleted, permission changes). Verify debouncing works correctly. Add
proper cleanup on window close. Test external file modification triggers reload.

Dependencies: Task 1

### Task 4: Extract Frontend State Management
Create src/app_root.tsx extracting state management from App.tsx. Move useState
hooks for data and error state. Move useEffect for initialization. Move IPC
event listeners. Keep App.tsx as thin wrapper. Verify read-only display still
works.

Dependencies: None (frontend)

### Task 5: Create Error Display Component
Create src/error_banner.tsx for displaying error states from the backend. Accept
error message as prop. Style with appropriate warning colors. Position at top
of viewport. Update app_root to display errors from load failures.

Dependencies: Task 4

### Task 6: Create IPC Bridge Module
Create src/ipc_bridge.ts with TypeScript wrappers for Tauri commands. Define
typed interfaces for TomlTableData and error responses. Create event subscription
helpers with proper cleanup. Replace raw invoke calls.

Dependencies: Task 4

### Task 7: MVP 1 Integration Test
Create a manual testing checklist and verify: launch with no args loads default
directory, launch with file path loads that file, launch with invalid path shows
error, external file modification triggers reload, TOML parse error shows error
banner.

Dependencies: Tasks 1-6

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

### Task 13: MVP 2 Integration Test
Create a manual testing checklist and verify: cell edit triggers save, TOML
file reflects change, comments in TOML preserved after edit, concurrent external
edit handled gracefully, save error displays without data loss.

Dependencies: Task 12

## Phase 3: Error Robustness

These tasks ensure the application handles error scenarios gracefully.

### Task 14: Create Error Types Module
Create error/error_types.rs with TvError enum covering failure modes:
TomlParseError, FileNotFound, PermissionDenied, WriteError. Implement Display
and Error traits. Add From implementations for common error types.

Dependencies: Task 13

### Task 15: Implement Error Recovery
Define error recovery strategies: which errors allow continued editing in
read-only mode, which require reload, which show permanent error state. Implement
recovery logic in save and load paths.

Dependencies: Task 14

### Task 16: Handle File Deletion
When watched file is deleted, enter read-only state with error banner. Preserve
last known data. Monitor for file reappearance. Resume normal operation when
file returns.

Dependencies: Task 15

### Task 17: Handle Permission Errors
When file becomes unreadable, preserve last known data and show error. When
file becomes unwritable, allow viewing but reject saves with error message.
Queue pending changes for retry when permissions restore.

Dependencies: Task 15

## Phase 4: Multi-File Support

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

## Phase 5: UUID Generation

### Task 22: Implement UUID Generator
Create uuid_generator.rs with ID column detection (case-insensitive). Generate
UUIDv4 for empty ID cells. Return generated UUID or None if column missing or
cell non-empty.

Dependencies: Task 13

### Task 23: Integrate UUID with Save
Update save command to call UUID generator before writing. Include generated
UUID in save response. Update frontend to apply generated UUID to display.

Dependencies: Task 22

## Phase 6: Metadata Support

### Task 24: Define Metadata Schema
Design the [metadata] section schema. Define TypeScript and Rust types for
metadata structure. Document all supported fields, types, and defaults.

Dependencies: Task 13

### Task 25: Implement Metadata Parser
Create metadata_parser.rs to parse [metadata] section. Return Option<Metadata>
with defaults for missing fields. Handle missing section gracefully.

Dependencies: Task 24

### Task 26: Implement Metadata Serialization
Add serialization back to TOML. Update metadata section in document. Create
section if missing. Preserve unrecognized fields.

Dependencies: Task 25

### Task 27: Apply Column Widths from Metadata
Read column widths from metadata on load. Apply to Univer. Detect width changes
via Univer events. Update metadata. Save on change.

Dependencies: Task 26

## Phase 7: Derived Columns

### Task 28: Define Derived Function Trait
Create DerivedFunction trait with name(), input_keys(), compute() methods.
Create DerivedResult enum with Text, Number, Error variants.

Dependencies: Task 13

### Task 29: Implement Function Registry
Create FunctionRegistry storing functions by name. Implement register and lookup.
Initialize global registry at startup.

Dependencies: Task 28

### Task 30: Implement Compute Executor
Create async computation infrastructure with tokio. Queue computations. Send
results via Tauri events. Handle errors gracefully.

Dependencies: Task 29

### Task 31: Add Generation Tracking
Track row generation counters. Increment on row edit. Discard stale computation
results. Ensure eventual correctness.

Dependencies: Task 30

### Task 32: Add Frontend Derived Column Support
Subscribe to derived-value-computed events. Update cell display when results
arrive. Show blank while pending. Show error styling on failure.

Dependencies: Task 31

## Phase 8: Image Support

### Task 33: Implement Image Cache
Create content-addressed image cache. Store by URL hash. Implement get/put.
Add LRU eviction.

Dependencies: Task 13

### Task 34: Implement Image Fetcher
Create async HTTP image download with reqwest. Validate response. Decode to
verify validity.

Dependencies: Task 33

### Task 35: Implement Image Derived Function
Create function implementing DerivedFunction. Construct URL from row data.
Fetch, cache, return base64-encoded image.

Dependencies: Tasks 34, 29

### Task 36: Add Frontend Image Cell Support
Use Univer sheets-drawing-ui for cell images. Call insertImage for image
results. Display placeholder during load.

Dependencies: Task 35

## Phase 9: Validation

### Task 37: Implement Validation Rule Parser
Parse validation rules from metadata. Define ValidationRule struct with type,
constraints, error message. Support enum, range, pattern rules.

Dependencies: Task 25

### Task 38: Implement Validators
Create type and enum validators. Return validation result with error message.
Handle required vs optional fields.

Dependencies: Task 37

### Task 39: Integrate Validation with Save
Validate cell values before writing. Reject invalid values with error. Return
validation errors to frontend.

Dependencies: Task 38

### Task 40: Add Frontend Dropdown Support
Configure Univer data validation for enum columns. Read values from metadata.
Set up dropdown UI.

Dependencies: Task 37

## Phase 10: Rules Text Preview

### Task 41: Implement Fluent Integration
Load strings.ftl. Parse into FluentResource. Store for template processing.

Dependencies: Task 29

### Task 42: Implement Style Tag Parser
Parse HTML-like styling tags. Generate styled run list with character positions.

Dependencies: Task 13

### Task 43: Implement Rules Preview Function
Create DerivedFunction for rules preview. Parse variables. Format through
Fluent. Parse style tags. Return rich text result.

Dependencies: Tasks 41, 42

### Task 44: Add Frontend Rich Text Support
Handle RichText derived results. Apply Univer rich text formatting.

Dependencies: Task 43

## Phase 11: Styling

### Task 45: Implement Table Color Schemes
Define color scheme presets. Store selection in metadata. Apply header and
alternating row colors.

Dependencies: Task 26

### Task 46: Implement Conditional Formatting
Parse formatting rules from metadata. Evaluate conditions. Return style
overrides. Integrate with Univer.

Dependencies: Task 26

### Task 47: Implement Frozen Panes
Read freeze configuration from metadata. Apply to Univer on load. Persist
changes to metadata.

Dependencies: Task 26

## Phase 12: Logging

### Task 48: Create JSON Logger
Implement tracing subscriber outputting JSONL format. Include Pacific Time
timestamps. Write to stdout and log file.

Dependencies: Task 13

### Task 49: Add Frontend Log Forwarding
Create frontend logger. Format logs as JSON. Send to backend via command.
Mirror to browser console.

Dependencies: Task 48

### Task 50: Add Log Rotation
Implement daily log file rotation. Create logs in app data directory. Clean
old logs based on retention period.

Dependencies: Task 48

---

## Testing Strategy

### Test Crate Structure

Following this project's patterns, create a test crate at:

```
rules_engine/tests/tv_tests/
├── Cargo.toml
├── src/
│   └── lib.rs          (exports test utilities as a library)
└── tests/
    ├── lib.rs          (declares test modules)
    └── tv_tests/
        ├── mod.rs
        ├── load_tests.rs
        ├── save_tests.rs
        ├── sync_tests.rs
        └── preservation_tests.rs
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
}

impl TvTestHarness {
    pub fn new() -> Self { ... }
    pub fn create_toml_file(&self, name: &str, content: &str) -> PathBuf { ... }
    pub fn load_table(&self, path: &Path) -> Result<TomlTableData, TvError> { ... }
    pub fn save_cell(&self, path: &Path, row: usize, col: &str, value: Value) -> Result<(), TvError> { ... }
    pub fn read_file_content(&self, path: &Path) -> String { ... }
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

Create test fixture files at `tests/tv_tests/fixtures/`:
- `simple_table.toml` - Basic array of tables
- `with_comments.toml` - TOML with inline and block comments
- `sparse_data.toml` - Rows with different column sets
- `with_metadata.toml` - File including [metadata] section
- `unicode_content.toml` - Non-ASCII text in values
- `invalid_syntax.toml` - Intentionally malformed for error tests

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

**MVP 1 (Read-Only) - Can start immediately:**
- Tasks 1, 2 (backend, parallel)
- Tasks 4, 5, 6 (frontend, parallel)
- Task 3 depends on Task 1
- Task 7 depends on all above

**MVP 2 (Editable) - After MVP 1:**
- Task 8 depends on Task 7
- Tasks 9, 10 sequential
- Task 11 depends on Task 6
- Task 12 depends on Tasks 10, 11
- Task 13 depends on Task 12

**Post-MVP - After MVP 2:**
- Error Robustness (Tasks 14-17) can start after Task 13
- Multi-File (Tasks 18-21) can start after Task 13
- UUID (Tasks 22-23) can start after Task 13
- Remaining phases depend on their listed prerequisites

**Critical Path to Working Prototype:**
Task 1 → Task 3 → Task 7 → Task 8 → Task 9 → Task 10 → Task 12 → Task 13
