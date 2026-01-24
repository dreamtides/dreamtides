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

INFO: Significant state changes and operation completions. Examples: file
loaded, file saved, watcher started, derived column computed.

DEBUG: Detailed operation tracing for development. Examples: cache hit/miss,
event received, state transition, function entry/exit.

TRACE: Verbose debugging for deep investigation. Examples: byte-level file
operations, per-cell iteration, network packet details.

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

Rust backend uses the tracing crate with a custom JSON subscriber. The
subscriber formats events according to this specification. Spans are
logged as separate enter/exit events with matching span IDs.

Initialization configures log level from environment variable TV_LOG_LEVEL
or defaults to INFO for release builds and DEBUG for debug builds.

## Frontend Logging

TypeScript frontend defines a Logger class wrapping console methods. Each
log call constructs a JSON object and sends to backend via log_message
command. Backend aggregates frontend logs into the unified stream.

Frontend logs include source information from error stacks when available.
Browser console also receives logs for development convenience.

## Log Output Destinations

Logs write to two destinations:
1. Standard output for terminal viewing during development
2. Log file in application data directory for persistent storage

Log file path: ~/Library/Application Support/tv/logs/tv_YYYY-MM-DD.jsonl

Stdout is disabled when running as a background process. Log file is always
enabled.

## Log Rotation

Log files rotate at midnight Pacific Time. Filename includes date for easy
identification. Files older than the retention period are deleted on startup.
Default retention is 7 days, configurable via environment variable.

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
{"ts":"2024-01-15T14:30:45.123-08:00","level":"INFO","component":"tv.toml","msg":"File loaded","file_path":"/path/to/cards.toml","rows":523,"duration_ms":45}
{"ts":"2024-01-15T14:30:46.456-08:00","level":"DEBUG","component":"tv.sync","msg":"Watcher event received","file_path":"/path/to/cards.toml","event_type":"modify"}
{"ts":"2024-01-15T14:30:47.789-08:00","level":"ERROR","component":"tv.toml","msg":"Parse failed","file_path":"/path/to/cards.toml","error":"expected string at line 42"}
```

## Log Analysis

Logs are designed for processing with jq. Common queries:
- Errors: jq 'select(.level=="ERROR")'
- Slow operations: jq 'select(.duration_ms > 100)'
- Specific file: jq 'select(.file_path | contains("cards"))'
- Time range: jq 'select(.ts > "2024-01-15T14:00")'
