# Appendix E: Logging Specification

## Log Entry Format

Each log line is a self-contained JSON object. Fields are ordered for
readability when viewing raw logs: timestamp, level, component, message,
then additional context fields.

Required fields:
- ts: ISO 8601 timestamp with Pacific Time offset
- level: Uppercase log level string
- component: Dot-separated component path
- msg: Human-readable message string

Optional fields:
- file_path: Relevant file path when applicable
- cell: Cell reference in A1 notation for cell-related logs
- row: Row index for row-related operations
- duration_ms: Operation duration for performance logging
- error: Error message or type for error logs
- stack: Stack trace for error logs when available

## Timestamp Format

Timestamps use RFC 3339 format with explicit Pacific Time offset. Example:
"2024-01-15T14:30:45.123-08:00" for PST or "-07:00" for PDT. The chrono-tz
crate provides America/Los_Angeles timezone conversion.

Timestamps include millisecond precision for correlating rapid sequences.
The system clock is used directly without NTP adjustment.

## Log Levels

ERROR: Unrecoverable errors requiring user attention. Application may continue
in degraded state. Examples: file write failure, TOML parse error, backend
thread panic.

WARN: Recoverable issues that may indicate problems. Examples: deprecated
metadata version, slow operation, retried operation.

INFO: App lifecycle events and significant state changes. Examples: logging
initialized, executor created/stopped, watcher started/stopped, function
registry initialized.

DEBUG: Per-operation tracing for development. Examples: file loaded, file
saved, cell saved, row added/deleted, sort/filter state changes, event
received, state transition.

TRACE: Hot-path and per-cell debugging. Examples: cache hit/miss, derived
value emitted, generation tracking, stale computation skipped, queue
operations.

## Component Hierarchy

Components follow a hierarchical naming convention:
- tv.toml: TOML file operations (load, save, parse)
- tv.sync: Synchronization (watcher, conflict, state)
- tv.derived: Derived columns (registry, executor, functions)
- tv.image: Image handling (cache, fetch, encode)
- tv.validation: Data validation (rules, checks)
- tv.uuid: UUID generation
- tv.ui: Frontend operations (render, event, input)
- tv.ipc: IPC communication (command, event)
- tv.error: Error handling and recovery

## Backend Logging

Rust backend uses the tracing crate with a dual-layer subscriber
architecture composed onto a single Registry:

**File layer.** Always uses JSON format with ANSI disabled. Writes only to
the log file. Every line on disk is valid JSON, parseable by jq.

**Stdout layer.** Uses compact format with ANSI colors enabled. Writes only
to stdout. Provides human-readable terminal output during development.

Each layer has its own EnvFilter. The default filter string includes
suppressions for noisy third-party crates:
`info,hyper=warn,reqwest=warn,tao=warn,wry=warn,tauri=warn` (release) or
`debug,...` (debug builds). The TV_LOG_LEVEL environment variable overrides
the default when set.

## Frontend Logging

TypeScript frontend uses a centralized Logger class from logger_frontend.ts.
Each module creates a Logger with a component name (e.g., `tv.ui.app`,
`tv.ui.images`, `tv.ui.spreadsheet`, `tv.ui.richtext`). Log calls send
structured entries to the backend via the log_message Tauri command, where
log_aggregator.rs ingests them into the tracing system.

Console mirror output is suppressed in production builds
(`import.meta.env.PROD`). In development, logs are mirrored to the browser
console for convenience.

Global error capture is installed at app startup in main.tsx:
- `window.addEventListener("error", ...)` catches uncaught synchronous
  exceptions with message, filename, line/column, and stack trace.
- `window.addEventListener("unhandledrejection", ...)` catches unhandled
  promise rejections with reason and stack trace.
Both handlers use a Logger with component `tv.ui.global`.

## Log Output Destinations

Two independent tracing layers write to separate destinations:

1. **File layer** — JSON format, no ANSI codes. Writes to the log file.
   This layer is unconditional and always active.
2. **Stdout layer** — Compact format with ANSI colors. Writes to stdout
   for human-readable terminal output during development.

Log file path: ~/Library/Application Support/tv/logs/tv_YYYY-MM-DD.jsonl

## Log Rotation

Log files rotate at midnight Pacific Time. Filename includes date for easy
identification. The `cleanup_old_logs()` function runs at the end of
`initialize()` and deletes files older than the retention period. Default
retention is 7 days, configurable via the `TV_LOG_RETENTION_DAYS` environment
variable. Deletion failures are logged at WARN level.

Old log files are not compressed to allow easy grep and jq processing. Disk
space is traded for accessibility.

## Sensitive Data

File paths are logged in full for debugging. Cell values are truncated to
100 characters to avoid log bloat. No credential-like fields are expected
in TOML content, but validation names are logged rather than values.

## Correlation

Related log entries share a request_id field for correlation. Request IDs
are generated at IPC command entry and propagated through all operations.
Frontend commands include the request_id in the command payload.

## Performance Logging

Operations exceeding threshold durations log at WARN level with duration_ms.
Thresholds: file load 500ms, file save 200ms, derived compute 1000ms, image
fetch 5000ms. Performance logs enable identifying slow operations.

## Example Log Entries

```json
{"ts":"2024-01-15T14:30:45.123-08:00","level":"INFO","component":"tv.logging","msg":"Logging initialized","log_file":"/path/to/logs/tv_2024-01-15.jsonl"}
{"ts":"2024-01-15T14:30:45.456-08:00","level":"DEBUG","component":"tv.toml","msg":"File loaded","file_path":"/path/to/cards.toml","rows":523,"duration_ms":45}
{"ts":"2024-01-15T14:30:46.789-08:00","level":"ERROR","component":"tv.toml","msg":"Parse failed","file_path":"/path/to/cards.toml","error":"expected string at line 42"}
{"ts":"2024-01-15T14:30:47.012-08:00","level":"ERROR","component":"tv.ui.global","msg":"Uncaught exception","frontend_ts":"2024-01-15T14:30:47.010-08:00","message":"Cannot read properties of undefined","filename":"app_root.tsx","lineno":42}
```

## Log Analysis

Logs are designed for processing with jq. Common queries:
- Errors: jq 'select(.level=="ERROR")'
- Slow operations: jq 'select(.duration_ms > 100)'
- Specific file: jq 'select(.file_path | contains("cards"))'
- Time range: jq 'select(.ts > "2024-01-15T14:00")'
