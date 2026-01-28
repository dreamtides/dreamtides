# Appendix F: Project File Layout

## Main Application (rules_engine/src/tv/)

### Root Configuration
- package.json: NPM package with Univer (23 packages) and Tauri dependencies
- pnpm-lock.yaml: Locked dependency versions
- tsconfig.json: TypeScript compiler configuration
- vite.config.ts: Vite bundler with Tauri integration
- index.html: HTML entry point for Tauri webview

### Frontend Source (src/)

**React Components:**
- main.tsx: React entry point, error boundary setup
- app_root.tsx: Top-level state management, multi-sheet coordination, IPC setup
- spreadsheet_view.tsx: Loading/error state wrapper
- UniverSpreadsheet.tsx: Univer lifecycle, data binding, event handling
- error_banner.tsx: Dismissable error/warning overlay

**IPC and Configuration:**
- ipc_bridge.ts: ~35 Tauri commands, 11 event listeners, TypeScript types
- univer_config.ts: Univer instance creation, 14 plugin registrations
- disabled_menu_items.ts: 118 hidden Univer menu items
- spreadsheet_types.ts: TypeScript type definitions

**Spreadsheet Utilities:**
- workbook_builder.ts: Multi-sheet workbook construction, sheet ordering
- sheet_data_utils.ts: Data population, change detection, row comparison
- derived_column_utils.ts: Column mapping, position calculation
- validation_utils.ts: Checkbox/dropdown validation via Univer API
- table_style_utils.ts: Color schemes, conditional formatting
- header_utils.ts: Header formatting, column letter conversion
- rich_text_utils.ts: Derived result to Univer rich text conversion
- image_cell_renderer.ts: Floating image rendering via direct commands

**Logging:**
- logger_frontend.ts: Frontend logging with backend aggregation

### Frontend Styles (src/styles/)
- app_styles.css: Global application styles

### Documentation (docs/)
- tv_design_document.md: Main technical design document
- appendix_a_metadata_schema.md: Metadata specification
- appendix_b_derived_functions.md: Derived function architecture
- appendix_c_sync_protocol.md: Bidirectional sync details
- appendix_d_univer_integration.md: Univer integration and workarounds
- appendix_e_logging_specification.md: Logging format specification
- appendix_f_file_layout.md: This file layout document
- appendix_g_rules_text_preview.md: Rules text preview pipeline
- appendix_h_implementation_plan.md: Implementation tasks

## Tauri Backend (src-tauri/)

### Configuration
- Cargo.toml: Rust package with dependencies
- Cargo.lock: Locked Rust dependency versions
- tauri.conf.json: Tauri app configuration, asset protocol, CSP
- build.rs: Tauri build script
- capabilities/default.json: Tauri permissions

### Rust Source (src/)

**Entry Points:**
- main.rs: Binary entry point
- lib.rs: Tauri builder with 43 command registrations, state initialization
- cli.rs: Command-line argument parsing, AppPaths

### Commands Module (src/commands/)

**File Operations:**
- load_command.rs: `load_toml_table` with sort/filter state restoration
- save_command.rs: `save_cell`, `save_batch`, `save_toml_table`, `add_row`
- watch_command.rs: `start_file_watcher`, `stop_file_watcher`

**Derived Columns:**
- derived_command.rs: `compute_derived`, `compute_derived_batch`,
  `update_lookup_context`, `increment_row_generation`, `clear_computation_queue`,
  `get_computation_queue_length`, `get_derived_columns_config`

**Sort/Filter:**
- sort_command.rs: `get_sort_state`, `set_sort_state`, `clear_sort_state`,
  `get_sort_row_mapping`, `translate_row_index`
- filter_command.rs: `get_filter_state`, `set_filter_state`, `clear_filter_state`,
  `get_filter_visibility`, `is_row_visible`, `set_hidden_rows`, `get_hidden_rows`

**Validation/Styling:**
- validation_command.rs: `get_validation_rules`, `get_enum_validation_rules`
- style_command.rs: `get_table_style`, `get_available_color_schemes`,
  `get_conditional_formatting`

**Configuration:**
- column_command.rs: `get_column_configs`, `set_column_width`,
  `set_derived_column_width`
- row_command.rs: `get_row_config`

**Images:**
- image_command.rs: `fetch_image` (async)

**Logging/State:**
- log_command.rs: `log_message`, `log_perf`
- view_state_command.rs: `load_view_state`, `save_view_state`
- sheet_order_command.rs: `load_sheet_order`, `save_sheet_order`

### TOML Module (src/toml/)

- document_loader.rs: TOML file reading, array-of-tables extraction, header
  collection from sparse data
- document_writer.rs: Atomic writes via temp file + rename, cell-level updates,
  batch updates, UUID integration, boolean type preservation
- metadata_parser.rs: Parse all metadata categories (~36KB): validation rules,
  derived columns, conditional formatting, table style, column/row configs,
  sort/filter state
- metadata_serializer.rs: Serialize configs to TOML (~30KB): sort state, filter
  state, column widths
- metadata_types.rs: Type definitions for metadata schema v1
- value_converter.rs: JSON<->TOML value conversion with type preservation
- color_schemes.rs: Predefined color scheme definitions (15+ schemes)
- conditional_formatting.rs: Condition evaluation (equals, contains, greater_than,
  less_than, is_empty, not_empty, matches)

### Sync Module (src/sync/)

- file_watcher.rs: notify-debouncer-mini integration, 500ms debounce, per-file
  watcher threads, event aggregation
- state_machine.rs: Idle/Saving/Loading/Error states, atomic transitions,
  conflict detection via mtime comparison

### Derived Module (src/derived/)

**Core Infrastructure:**
- function_registry.rs: Global `OnceLock<FunctionRegistry>`, `DerivedFunction`
  trait, registration and lookup
- compute_executor.rs: Async tokio execution, priority queue (visible rows
  first), result event emission, `ComputeExecutorState`
- generation_tracker.rs: Per-row generation counters for staleness detection
- result_cache.rs: LRU cache for computed values

**Built-in Functions:**
- image_derived.rs: Image fetch with caching, returns local cache path
- image_url.rs: Simple URL construction from image number
- image_lookup.rs: Cross-table card ID resolution with image fetch
- card_lookup.rs: Cross-table card name lookup by UUID
- rules_preview.rs: Fluent template processing with rich text output
- fluent_integration.rs: Fluent locale data loading from rules_engine
- rich_text_converter.rs: Univer rich text structure generation
- style_tag_parser.rs: HTML-like tag parsing for styled text

### Validation Module (src/validation/)

- validation_rules.rs: Rule type enum (Enum, Range, Pattern, Required, Type),
  `ValueType` enum
- validators.rs: `validate()`, `validate_all()`, `first_error()` functions for
  all rule types

### Images Module (src/images/)

- image_cache.rs: SHA-256 content-addressed cache, LRU eviction at 100MB,
  metadata persistence in `cache_metadata.json`, startup integrity validation
- image_fetcher.rs: Async HTTP fetch with reqwest, semaphore concurrency (4),
  30s timeout, format validation via `image` crate, browser-like headers

### Sort Module (src/sort/)

- sort_state.rs: `SortStateManager`, bidirectional index mapping
  (display<->original), comparison logic for mixed types
- sort_types.rs: `SortState`, `SortDirection` (Ascending/Descending),
  `CellValue` enum

### Filter Module (src/filter/)

- filter_state.rs: `FilterStateManager`, visibility computation with AND logic,
  hidden row tracking
- filter_types.rs: `FilterState`, `FilterConditionState` (Contains, Equals,
  Range, Boolean, Values)

### View State Module (src/view_state/)

- view_state_types.rs: `ViewState` struct with `active_sheet_path`, JSON
  persistence to `.tv_view_state.json`
- sheet_order.rs: `SheetOrder` persistence to `sheets.toml`

### UUID Module (src/uuid/)

- uuid_generator.rs: `ensure_uuids()` function, case-insensitive "id" column
  detection, UUIDv4 generation

### Logging Module (src/logging/)

- json_logger.rs: Custom tracing subscriber, JSONL output, Pacific Time
  timestamps, component tags
- log_aggregator.rs: Frontend log aggregation, unified log stream

### Error Module (src/error/)

- error_types.rs: `TvError` enum with variants: FileNotFound, InvalidToml,
  SaveFailed, WatcherError, ValidationFailed, ImageFetchError, ImageCacheError,
  DerivedComputationError, BackendThreadPanic, SyncConflict, InvalidArguments

### Traits Module (src/traits/)

- traits_mod.rs: `FileSystem` trait (for testing with mock filesystem),
  `Clock` trait, `RealFileSystem`, `RealClock` implementations
