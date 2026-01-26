# Logging Rework Plan

## Problem Statement

The TV logging system has five issues that need to be addressed together:

**Mixed formats on disk.** The `--jsonl` CLI flag in
[cli.rs](../src-tauri/src/cli.rs) controls whether
[json_logger.rs](../src-tauri/src/logging/json_logger.rs) installs a JSON
subscriber or a compact-text subscriber. Both modes append to the same
date-based file (`tv_YYYY-MM-DD.jsonl`). In practice, test runs use JSONL mode
while interactive launches use compact mode, producing files where 58% of lines
are valid JSON and 42% are ANSI-colored plaintext. The files are unparseable by
`jq` or any JSONL tool.

**ANSI escape codes on disk.** The `DualWriter` in
[json_logger.rs](../src-tauri/src/logging/json_logger.rs) writes identical bytes
to both stdout and the log file. In compact mode, tracing's formatter emits
terminal color codes (`\x1b[32m`, `\x1b[2m`, `\x1b[3m`, etc.) that get
persisted to the `.jsonl` file.

**Frontend console calls bypass log files.** The centralized `Logger` class in
[logger_frontend.ts](../src/logger_frontend.ts) forwards structured log entries
to the backend via the `log_message` Tauri command in
[log_command.rs](../src-tauri/src/commands/log_command.rs), where
[log_aggregator.rs](../src-tauri/src/logging/log_aggregator.rs) ingests them
into the tracing system. However, most frontend code does not use this Logger:

- [app_root.tsx](../src/app_root.tsx): 13 direct `console.error`/`console.log`
  calls
- [image_cell_renderer.ts](../src/image_cell_renderer.ts): custom `logDebug`/
  `logError` helper functions that `JSON.stringify` to `console.debug`/
  `console.error`
- [UniverSpreadsheet.tsx](../src/UniverSpreadsheet.tsx): 2 direct console calls
- [rich_text_utils.ts](../src/rich_text_utils.ts): 1 direct `console.debug`
  call

None of these reach the log file on disk.

**No global error capture.** There are no `window.addEventListener("error",...)`
or `window.addEventListener("unhandledrejection",...)` handlers. Uncaught
JavaScript exceptions and rejected promises are invisible in the log files.

**Excessive log density.** A single day of use produced 18 MB / 76,709 lines.
The top contributors from the actual log files:

| Count  | Level | Message                            | Source File |
|--------|-------|------------------------------------|-------------|
| 14,666 | INFO  | File loaded                        | [document_loader.rs](../src-tauri/src/toml/document_loader.rs) |
| 14,232 | INFO  | Cell saved                         | [document_writer.rs](../src-tauri/src/toml/document_writer.rs) |
| 6,423  | DEBUG | Emitted derived value result       | [compute_executor.rs](../src-tauri/src/derived/compute_executor.rs) |
| 4,618  | INFO  | File saved                         | [document_writer.rs](../src-tauri/src/toml/document_writer.rs) |
| 3,538  | DEBUG | Fetching image (cache miss)        | [image_derived.rs](../src-tauri/src/derived/image_derived.rs) |
| 3,436  | DEBUG | Parsed validation rules            | [metadata_parser.rs](../src-tauri/src/toml/metadata_parser.rs) |
| 2,886  | DEBUG | Image cache hit                    | [image_derived.rs](../src-tauri/src/derived/image_derived.rs) |
| 1,059  | DEBUG | Transitioned to Loading state      | [state_machine.rs](../src-tauri/src/sync/state_machine.rs) |

Additionally, no log cleanup runs on startup, so files accumulate indefinitely.

## Design

### Dual tracing layers

Replace the current single-layer-plus-`DualWriter` architecture in
[json_logger.rs](../src-tauri/src/logging/json_logger.rs) with two independent
tracing subscriber layers composed onto a single `Registry`:

**File layer.** Always uses `.json()` format with `.with_ansi(false)`. Writes
only to the log file via a file-only writer. This layer is unconditional — there
is no mode flag. Every line written to disk is valid JSON.

**Stdout layer.** Uses `.compact()` format with ANSI colors enabled. Writes only
to `std::io::stdout`. This provides the human-readable terminal output during
development.

Each layer gets its own `EnvFilter` so that file and terminal verbosity can
differ. The `DualWriter` and `FileAndStdout` structs are deleted entirely.

The `--jsonl` CLI flag in [cli.rs](../src-tauri/src/cli.rs) is removed. The
`jsonl: bool` parameter is removed from `initialize()` in
[json_logger.rs](../src-tauri/src/logging/json_logger.rs) and from `run()` in
[lib.rs](../src-tauri/src/lib.rs).

### Log level adjustments

Downgrade high-frequency routine messages so that INFO represents notable
state changes rather than per-cell operations:

| Message | Current | New | File |
|---------|---------|-----|------|
| File loaded | INFO | DEBUG | [document_loader.rs](../src-tauri/src/toml/document_loader.rs):129 |
| Cell saved | INFO | DEBUG | [document_writer.rs](../src-tauri/src/toml/document_writer.rs):605 |
| File saved | INFO | DEBUG | [document_writer.rs](../src-tauri/src/toml/document_writer.rs):237 |
| Batch saved | INFO | DEBUG | [document_writer.rs](../src-tauri/src/toml/document_writer.rs):495 |
| Row added | INFO | DEBUG | [document_writer.rs](../src-tauri/src/toml/document_writer.rs):751 |
| Row deleted | INFO | DEBUG | [document_writer.rs](../src-tauri/src/toml/document_writer.rs):851 |
| New row appended during save | INFO | DEBUG | [document_writer.rs](../src-tauri/src/toml/document_writer.rs):180 |
| Removing excess row during save | INFO | DEBUG | [document_writer.rs](../src-tauri/src/toml/document_writer.rs):207 |
| Removing empty row during save | INFO | DEBUG | [document_writer.rs](../src-tauri/src/toml/document_writer.rs):217 |
| Starting batch save | INFO | DEBUG | [document_writer.rs](../src-tauri/src/toml/document_writer.rs):351 |
| Sort config updated in metadata | INFO | DEBUG | [metadata_serializer.rs](../src-tauri/src/toml/metadata_serializer.rs):66 |
| Filter config updated in metadata | INFO | DEBUG | [metadata_serializer.rs](../src-tauri/src/toml/metadata_serializer.rs):131 |
| Metadata saved | INFO | DEBUG | [metadata_serializer.rs](../src-tauri/src/toml/metadata_serializer.rs):181 |
| Restored sort state from metadata | INFO | DEBUG | [load_command.rs](../src-tauri/src/commands/load_command.rs):107 |
| Restored filter state from metadata | INFO | DEBUG | [load_command.rs](../src-tauri/src/commands/load_command.rs):164 |
| Emitted derived value result | DEBUG | TRACE | [compute_executor.rs](../src-tauri/src/derived/compute_executor.rs):379 |
| Image cache hit | DEBUG | TRACE | [image_derived.rs](../src-tauri/src/derived/image_derived.rs):51 |
| Cache hit | DEBUG | TRACE | [result_cache.rs](../src-tauri/src/derived/result_cache.rs):75 |
| Cache miss | DEBUG | TRACE | [result_cache.rs](../src-tauri/src/derived/result_cache.rs):87 |
| Inserted cache entry | DEBUG | TRACE | [result_cache.rs](../src-tauri/src/derived/result_cache.rs):128 |
| Skipping stale computation | DEBUG | TRACE | [compute_executor.rs](../src-tauri/src/derived/compute_executor.rs):278 |
| Discarding stale computation result | DEBUG | TRACE | [compute_executor.rs](../src-tauri/src/derived/compute_executor.rs):352 |
| Generation initialized | DEBUG | TRACE | [generation_tracker.rs](../src-tauri/src/derived/generation_tracker.rs):57 |
| Generation incremented | DEBUG | TRACE | [generation_tracker.rs](../src-tauri/src/derived/generation_tracker.rs):78 |
| Generation retrieved | DEBUG | TRACE | [generation_tracker.rs](../src-tauri/src/derived/generation_tracker.rs):98 |
| Queued visible row computation | DEBUG | TRACE | [compute_executor.rs](../src-tauri/src/derived/compute_executor.rs):182 |
| Queued offscreen row computation | DEBUG | TRACE | [compute_executor.rs](../src-tauri/src/derived/compute_executor.rs):193 |

After these changes, INFO is reserved for app lifecycle events (logging
initialized, executor created/stopped, watcher started/stopped, function
registry initialized) and warnings/errors. Per-operation messages move to DEBUG,
and per-cell hot-path messages move to TRACE.

### External crate filtering

Add default filters for noisy third-party crates to the `EnvFilter` in
[json_logger.rs](../src-tauri/src/logging/json_logger.rs). The base filter
string becomes something like `info,hyper=warn,reqwest=warn,tao=warn,wry=warn`
for the file layer and similar for stdout, rather than a bare level. The
`TV_LOG_LEVEL` environment variable continues to override when set.

### Frontend logging unification

Replace all direct `console.*` calls and custom JSON-stringify logging patterns
with the existing `Logger` class from
[logger_frontend.ts](../src/logger_frontend.ts). Each file gets a Logger
instance with an appropriate component name:

| File | Component | Changes |
|------|-----------|---------|
| [app_root.tsx](../src/app_root.tsx) | `tv.ui.app` | Replace 13 `console.error`/`console.log` calls |
| [image_cell_renderer.ts](../src/image_cell_renderer.ts) | `tv.ui.images` | Delete custom `logDebug`/`logError` helpers, replace with Logger |
| [UniverSpreadsheet.tsx](../src/UniverSpreadsheet.tsx) | `tv.ui.spreadsheet` | Replace 2 `console.debug`/`console.error` calls |
| [rich_text_utils.ts](../src/rich_text_utils.ts) | `tv.ui.richtext` | Replace 1 `console.debug` call |

After this change, every frontend log entry flows through the `log_message`
Tauri command to [log_aggregator.rs](../src-tauri/src/logging/log_aggregator.rs)
and into the unified tracing subscriber, appearing in the JSON log file with
`frontend_ts` and `component` fields.

### Global error capture

Add two global event listeners at app startup in
[main.tsx](../src/main.tsx):

- `window.addEventListener("error", ...)` to catch uncaught synchronous
  exceptions, forwarding to a Logger instance with component `tv.ui.global`.
  The handler logs the message, filename, line/column numbers, and stack trace.

- `window.addEventListener("unhandledrejection", ...)` to catch unhandled
  promise rejections, forwarding the reason and stack trace to the same Logger.

These fire before React's error boundaries and capture errors that escape all
try/catch blocks.

### Logger robustness

Update [logger_frontend.ts](../src/logger_frontend.ts):

- Add `.catch(() => {})` to the `invoke("log_message", ...)` call so that IPC
  failures during app shutdown do not produce cascading console errors.

- Consider suppressing console mirror output in production builds (when
  `import.meta.env.PROD` is true) since the file is the authoritative log
  destination. This avoids redundant output in the Tauri webview console.

### Log file cleanup on startup

Add a `cleanup_old_logs()` function in
[json_logger.rs](../src-tauri/src/logging/json_logger.rs), called at the end of
`initialize()`. It reads the log directory, parses dates from filenames matching
`tv_YYYY-MM-DD.jsonl`, and deletes files older than 7 days. Deletion failures
are logged at WARN level. The retention period can be overridden via a
`TV_LOG_RETENTION_DAYS` environment variable.

## Implementation Order

### Step 1: Dual tracing layers

Split the subscriber in
[json_logger.rs](../src-tauri/src/logging/json_logger.rs). Remove the
`DualWriter`/`FileAndStdout` structs. Remove the `--jsonl` flag from
[cli.rs](../src-tauri/src/cli.rs) and the `jsonl` parameter from
[lib.rs](../src-tauri/src/lib.rs). This is the foundational change that
ensures all disk output is clean JSON going forward.

### Step 2: Log cleanup

Add `cleanup_old_logs()` in
[json_logger.rs](../src-tauri/src/logging/json_logger.rs). Natural to
implement alongside Step 1 since it touches the same file.

### Step 3: Log level adjustments and external crate filtering

Change `tracing::info!` to `tracing::debug!` and `tracing::debug!` to
`tracing::trace!` in the files listed in the level adjustments table above.
Update the `EnvFilter` construction in
[json_logger.rs](../src-tauri/src/logging/json_logger.rs) to include
third-party crate suppressions.

### Step 4: Frontend logging unification

Replace console calls in
[app_root.tsx](../src/app_root.tsx),
[image_cell_renderer.ts](../src/image_cell_renderer.ts),
[UniverSpreadsheet.tsx](../src/UniverSpreadsheet.tsx), and
[rich_text_utils.ts](../src/rich_text_utils.ts) with Logger instances.

### Step 5: Global error capture

Add `window.addEventListener` handlers in [main.tsx](../src/main.tsx).

### Step 6: Logger robustness

Update [logger_frontend.ts](../src/logger_frontend.ts) with failure
suppression and optional console suppression.

## Validation

After all changes, verify:

- Log files contain only valid JSON: `jq . < tv_*.jsonl > /dev/null`
- No ANSI escape codes on disk: `grep -P '\x1b\[' tv_*.jsonl` returns nothing
- Frontend log entries appear in the file with `component` values starting
  with `tv.ui.`
- Triggering an uncaught JS error produces an entry with component
  `tv.ui.global`
- Daily log volume under normal use is under 2-3 MB rather than 18 MB
- Files older than 7 days are deleted on startup
- Terminal output remains human-readable with color formatting

## Appendix E Update

After implementation, update
[appendix_e_logging_specification.md](appendix_e_logging_specification.md) to
reflect:

- Removal of the `--jsonl` flag and dual-mode behavior
- The two-layer architecture (file always JSON, stdout always compact)
- Per-layer `EnvFilter` configuration
- Updated level assignments for the messages that changed
- External crate filter defaults
- `TV_LOG_RETENTION_DAYS` environment variable
- Global error capture in the frontend logging section
