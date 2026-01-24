# Appendix C: Bidirectional Sync Protocol

## State Machine

The sync system maintains one of four states per file:
- Idle: No pending operations, file and UI are synchronized
- Saving: A save operation is in progress
- Loading: A load operation is in progress
- Error: An error occurred, displaying last known good state

State transitions are guarded by atomic flags to prevent race conditions
between file watcher events and user edits.

## Save Operation Sequence

1. User edits a cell in Univer
2. Univer emits sheet.mutation.set-range-values command
3. Frontend extracts changed cell coordinates and new value
4. Frontend calls save_cell command with file path, row index, column key, value
5. Backend acquires file lock (advisory on Unix, exclusive on Windows)
6. Backend reads current file content
7. Backend parses with toml_edit::DocumentMut
8. Backend locates the specific value using table index and key
9. Backend updates value in place via mutable reference
10. Backend writes to temporary file in same directory
11. Backend renames temp file over original (atomic on POSIX)
12. Backend releases file lock
13. Backend returns success with any generated values (e.g., UUID)
14. Frontend updates cell display if backend returned generated values
15. State returns to Idle

## Load Operation Sequence

1. File watcher detects file modification event
2. Watcher debounces events, waiting 500ms for additional changes
3. Watcher emits toml-file-changed event with file path
4. Frontend checks saving flag; if true, ignores event
5. Frontend calls load_file command with file path and table name
6. Backend reads file content
7. Backend parses with toml::from_str for read-only access
8. Backend extracts headers from all table entries
9. Backend builds row data with JSON values for each cell
10. Backend extracts metadata section for configuration
11. Backend returns TomlTableData with headers, rows, metadata
12. Frontend replaces Univer data source with new data
13. Frontend applies metadata configuration to Univer settings
14. State returns to Idle

## Debouncing Strategy

File watcher uses notify-debouncer-mini with 500ms window. Multiple file
system events within the window coalesce into a single notification. The
debounce timer resets on each new event, delaying notification until changes
settle.

Frontend applies separate 500ms debounce to user edits before saving. This
batches rapid keystrokes into single save operations. The debounce timer
resets on each keystroke.

## Conflict Detection

After save completes, the file watcher may immediately trigger due to the
save itself. The saving flag prevents this reload. The flag is cleared only
after save success.

If external changes occur during save, the file content after save may differ
from what was written. The watcher detects this and triggers a reload. The
reload overwrites any further pending changes, implementing last-write-wins.

## Atomic Write Protocol

Writing uses a temp-file-then-rename pattern:
1. Generate temp file name with random suffix in same directory
2. Write complete content to temp file
3. Sync temp file to ensure durability
4. Rename temp file over target file
5. Directory is synced on platforms that support it

This ensures the target file is never in a partial state. Crashes during write
leave only orphaned temp files which are cleaned up on next startup.

## Lock Strategy

File locking prevents concurrent writes from multiple TV instances or external
editors. Locks are advisory on Unix, meaning cooperating processes respect them
but non-cooperating processes can still write.

Lock acquisition uses non-blocking try-lock. If lock fails, the save retries
with exponential backoff up to 5 attempts. If all retries fail, an error is
reported to the user.

## Error Recovery

Parse errors during load trigger Error state. The frontend displays last known
good data with error banner. The file watcher continues monitoring. When the
file next parses successfully, normal operation resumes.

Write errors during save preserve the pending change in memory. The change is
retried on the next user action or after a timeout. Persistent write failures
are surfaced to the user with suggested remediation.

## Event Types

Backend-to-frontend events:
- toml-file-changed: File content changed externally, reload needed
- derived-value-computed: A derived column value is ready
- save-completed: Async save finished with status
- error-occurred: An error requires user notification

Frontend-to-backend commands:
- load_file: Load or reload file content
- save_cell: Save single cell change
- save_batch: Save multiple cell changes atomically
- start_watcher: Begin watching file for changes
- stop_watcher: Stop watching file

## Multi-File Coordination

Each file has independent sync state. File watchers run in parallel on
separate notify instances. State flags are per-file, allowing concurrent
operations on different files. The active sheet determines which file receives
user edits; background sheets are read-only until activated.
