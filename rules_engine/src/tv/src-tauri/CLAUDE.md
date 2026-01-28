# TV (TOML Viewer) AI Guidance

This document provides architecture guidance for AI assistants working on TV.


## Key Invariants

**State Manager Independence**: Each `State<T>` in lib.rs manages an independent domain:
- `SortStateManager`: Row ordering and index translation
- `FilterStateManager`: Row visibility and hidden rows
- `ComputeExecutorState`: Async derived column computation
- `ImageFetcherState`: Image caching and fetch operations
- `SyncStateMachineState`: File synchronization state
- `PermissionRecoveryState`: Write permission error recovery

These managers never directly communicate. The frontend coordinates by calling
appropriate commands in sequence.

**Generation Tracking Contract**: Derived column results are rejected if the row
generation changed between request and response. This ensures stale async results
never overwrite newer data. The `GenerationTracker` uses a global atomic counter
so generations always increase monotonically across all rows.

**Atomic State Transitions**: The sync state machine enforces strict transitions:
- Idle -> Saving -> Idle (success) or Error (failure)
- Idle -> Loading -> Idle (success) or Error (failure)

Only one operation per file at a time. Concurrent operations return
`InvalidStateTransition` error.


## Architecture Layers

**Layer 1 - TOML Layer** (`toml/`): File I/O using `toml_edit` for structure
preservation. Handles parsing, atomic writes, and metadata extraction.

**Layer 2 - Application Layer**: State managers, derived computation, validation,
image caching. Coordinates between file operations and UI.

**Layer 3 - UI Layer** (`src/`): TypeScript/React frontend using Univer
spreadsheet library. Manages user interaction and display.


## Adding New Commands

1. Create Request/Response types in appropriate module (e.g., `toml/types.rs`)
2. Implement the command function with this signature pattern:
```rust
#[tauri::command]
pub fn my_command(
    app_handle: AppHandle,  // If you need state or events
    param: String,
) -> Result<ResponseType, TvError> {
    // Implementation
}
```

3. Register in `lib.rs` under `invoke_handler`
4. Add IPC wrapper in `src/ipc_bridge.ts`

**Tracing Conventions**:
- Entry: `tracing::debug!(component = "tv.commands.my_cmd", ...)`
- Completion: `tracing::info!(component = "tv.commands.my_cmd", "Operation completed")`
- Errors: Use `TvError` variants, never panic in commands


## State Manager Details

**SortStateManager** (`sort/sort_state.rs`): Maintains bidirectional index
mappings (display-to-original and original-to-display). Frontend calls
`translate_row_index` before saving to convert display indices to TOML indices.

**FilterStateManager** (`filter/filter_state.rs`): Tracks 4 separate maps:
- `states`: Active `FilterState` per file/table
- `visibility`: Per-row boolean visibility vector
- `filter_states`: Active `ColumnFilterState` conditions
- `hidden_rows`: Indices of rows hidden by filters

The distinction between `visibility` and `hidden_rows` allows different query
patterns: `is_row_visible` uses both, while `get_hidden_rows` returns the index list.


## Complex Patterns

**`_with_fs` Pattern**: Functions that perform file I/O have two variants:
```rust
pub fn load_toml_document(path: &str, table: &str) -> Result<Data, TvError> {
    load_toml_document_with_fs(&RealFileSystem, path, table)
}

pub fn load_toml_document_with_fs(
    fs: &dyn FileSystem,
    path: &str,
    table: &str,
) -> Result<Data, TvError> {
    // Actual implementation using fs.read_to_string(), fs.write(), etc.
}
```
This allows tests to inject a fake filesystem without touching the real disk.

**Generation Tracking** (`derived/generation_tracker.rs`): Each row has a
generation counter incremented on edit. Computation requests capture the
generation at request time. Results include this generation; the tracker
validates it matches current before applying. Stale results are silently
discarded.

**Atomic File Writes** (`toml/document_writer.rs`): All saves use temp files:
1. Write to `.tv_save_<pid>_<timestamp>.toml.tmp`
2. fsync the file
3. Atomic rename to target path
4. Orphaned temp files cleaned up on startup


## Frontend Data Flow

The `UniverSpreadsheet.tsx` component maintains 20+ refs for state management:
- `univerRef`, `univerAPIRef`: Univer instance and facade API
- `headersMapRef`: Column headers per sheet
- `columnMappingRef`: Data column to visual column mapping
- `enumRulesRef`: Dropdown validation rules per sheet
- `isRestoringSortRef`, `isRestoringFilterRef`: Suppress events during restore

Data flows:
1. Backend loads TOML, returns `TomlTableData`
2. Frontend builds workbook via `workbook_builder.ts`
3. Cell edits debounced (500ms), then `save_cell` or `save_batch` called
4. Backend updates TOML atomically, returns result
5. File watcher detects external changes, emits `toml-file-changed`


## Error Handling

Always return `Result<T, TvError>`. Never use `unwrap()` or `expect()` in
command handlers. The `TvError` enum covers all failure modes with structured
information for error reporting.

For I/O errors, use the helper functions:
- `map_io_error_for_read(error, path)` for read operations
- `map_io_error_for_write(error, path)` for write operations


## Testing Approach

Tests use the `_with_fs` pattern with `FakeFileSystem` to avoid disk I/O.
Test files live in `tests/` directory, not inline `mod tests {}` blocks.


## Reference Documentation

- Full technical design: `docs/tv_design_document.md`
- Univer integration details: `docs/appendix_d_univer_integration.md`
