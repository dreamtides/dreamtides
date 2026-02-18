# Logging Subsystem

The rules engine uses the `tracing` ecosystem for structured, hierarchical
logging with two output channels: human-readable tree-formatted logs and
machine-readable JSON trace files. The `logging` crate
(rules_engine/src/logging/) centralizes all initialization and configuration.

## Tracing Stack

Initialization happens once per process via `logging::maybe_initialize()`, which
uses a `Once` guard for idempotency. The tracing subscriber registry is composed
of:

- **ForestLayer** (`tracing-forest`): Produces tree-structured output where
  tracing spans create visual nesting. Uses a custom `PrettyPrinter` with
  emoji-coded tags based on log level and module target.
- **ErrorLayer** (`tracing-error`): Captures error context and backtraces for
  `SpanTrace` integration.
- **EnvFilter**: Reads `RUST_LOG` (defaults to `"debug"` if unset).

When a log directory is provided, a custom `DualMakeWriter` writes
simultaneously to stdout and `dreamtides.log`. Without a directory, output goes
to stdout only.

## Emoji Tag System

The forest layer's tag parser assigns emoji icons for visual categorization in
log output:

- Error/warn: error and warning indicators
- Module-specific: icons for `battle_queries`, `battle_mutations`,
  `client_logging`, `rules_engine`, `ai`, and `macros` modules
- Level-based: distinct icons for trace, debug, and info levels

The `LOG_FILTER_EMOJIS` constant exports the filter set used by the in-game log
viewer panel (display crate) to let developers filter logs by category.

## Initialization Paths

**Dev server** (dev_server.rs): Initializes at server startup with file logging
enabled, AI diagram logging on, and action legality checks on. Log directory is
the project root.

**Plugin/FFI** (plugin.rs): Initializes on first `connect` call. Log directory
comes from the client's persistent data path. AI diagram logging is disabled for
performance. Also writes to Android logcat via `android_logging::write_to_logcat()`
on Android builds.

**Tests**: Do not explicitly initialize logging. Battle tracing is controlled
independently via the `BattleState.tracing` field.

## Battle Trace Macro

The `battle_trace!` macro (battle_queries/src/macros/battle_trace.rs) is the
primary logging entry point during battle execution. It has three forms:

- Message only: `battle_trace!("message", battle)`
- Variable capture: `battle_trace!("message", battle, player, card_id)`
- Expression capture: `battle_trace!("message", battle, card_id = some_expr)`

The macro is conditional on `battle.tracing.is_some()` and performs two
operations when active:

- **Tracing event**: Emits a `tracing::debug!()` event that flows through the
  forest layer into the tree-formatted log output.
- **JSON trace event**: Calls `write_tracing_event::write_battle_event()` which
  serializes a `BattleTraceEvent` to `dreamtides.json`. Each event contains the
  message, captured variable values as a `BTreeMap<String, String>`, a full
  debug snapshot of the `BattleState`, and a timestamp.

The `BattleTracing` field on `BattleState` is marked `#[serde(skip)]` so it is
never persisted. It defaults to `None` and must be explicitly enabled when
tracing is desired.

## Output Files

When file logging is active, two files are produced in the log directory:

- **dreamtides.log**: Human-readable tree-structured output from
  `tracing-forest`, with emoji tags and hierarchical span nesting. Used for
  real-time development debugging.
- **dreamtides.json**: Machine-readable JSON array of `BattleTraceEvent`
  records, each containing the message, variable values, full battle state
  snapshot, and timestamp. Used for post-game analysis and crash debugging. The
  `write_tracing_event` module also captures animation and command sequence data,
  and records panic snapshots.

File paths are accessible via `logging::trace_json_path()` and
`logging::log_file_path()`.

## Public API

The `logging` crate exports:

- `maybe_initialize(request_context)` — idempotent subscriber setup
- `trace_json_path()` / `log_file_path()` — file path accessors
- `create_forest_layer(env_filter)` — create a configured forest layer for
  custom subscriber setups
- `get_developer_mode_project_directory()` /
  `get_developer_mode_streaming_assets_path()` — path resolution utilities used
  by tests and tooling
- `android_logging` module — platform-specific Android logcat integration
