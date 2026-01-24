# Appendix F: Project File Layout

## Main Application (rules_engine/src/tv/)

### Root Configuration
- package.json: NPM package definition with Univer and Tauri dependencies
- pnpm-lock.yaml: Locked dependency versions for reproducible builds
- pnpm-workspace.yaml: Workspace configuration for pnpm
- tsconfig.json: TypeScript compiler configuration for frontend
- tsconfig.node.json: TypeScript configuration for Node tooling
- vite.config.ts: Vite bundler configuration with Tauri integration
- index.html: HTML entry point loaded by Tauri webview

### Frontend Source (src/)
- main.tsx: React application entry point, mounts root component
- app_root.tsx: Top-level component with state management and IPC setup
- spreadsheet_view.tsx: Univer spreadsheet wrapper with data binding
- sheet_tabs.tsx: Multi-file tab navigation component
- error_banner.tsx: Error display overlay component
- status_indicator.tsx: Save/sync status display component
- cell_renderers.tsx: Custom cell renderers for images, checkboxes, rich text
- ipc_bridge.ts: Tauri command and event wrappers with TypeScript types
- logger_frontend.ts: Frontend logging utilities with backend aggregation

### Frontend Styles (src/styles/)
- app_styles.css: Global application styles
- spreadsheet_overrides.css: Univer styling customizations
- error_styles.css: Error banner and indicator styles
- tabs_styles.css: Sheet tab styling

### Documentation (docs/)
- tv_design_document.md: Main technical design document
- appendix_a_metadata_schema.md: Metadata specification
- appendix_b_derived_functions.md: Derived function architecture
- appendix_c_sync_protocol.md: Bidirectional sync details
- appendix_d_univer_integration.md: Univer integration details
- appendix_e_logging_specification.md: Logging format specification
- appendix_f_file_layout.md: This file layout document
- appendix_g_rules_text_preview.md: Rules text preview pipeline
- appendix_h_implementation_plan.md: Granular implementation tasks

### Tauri Backend (src-tauri/)

#### Configuration
- Cargo.toml: Rust package definition with dependencies
- Cargo.lock: Locked Rust dependency versions
- tauri.conf.json: Tauri application configuration
- build.rs: Tauri build script
- capabilities/default.json: Tauri permissions configuration

#### Rust Source (src/)
- main.rs: Binary entry point, calls lib run function
- lib.rs: Library entry with Tauri builder and command registration

#### Commands Module (src/commands/)
- commands_mod.rs: Module declarations and re-exports
- load_command.rs: Load file Tauri command implementation
- save_command.rs: Save cell Tauri command implementation
- watch_command.rs: Start watcher Tauri command implementation
- compute_command.rs: Trigger derived computation command
- log_command.rs: Frontend log aggregation command

#### TOML Module (src/toml/)
- toml_mod.rs: Module declarations and public types
- document_loader.rs: TOML file reading and parsing
- document_writer.rs: TOML file atomic writing
- metadata_parser.rs: Metadata section extraction and validation
- structure_preserver.rs: toml_edit operations for format preservation
- value_converter.rs: JSON to TOML value conversion

#### Sync Module (src/sync/)
- sync_mod.rs: Module declarations and sync state types
- file_watcher.rs: notify-based file watching with debouncing
- change_tracker.rs: Track pending changes and generation counters
- conflict_resolver.rs: Handle concurrent edit conflicts
- state_machine.rs: Sync state transitions and guards

#### Derived Module (src/derived/)
- derived_mod.rs: Module declarations and trait definitions
- function_registry.rs: Function registration and lookup
- compute_executor.rs: Async computation task execution
- generation_tracker.rs: Row generation counter management
- result_cache.rs: Computed value caching with LRU eviction

#### Derived Functions (src/derived/functions/)
- functions_mod.rs: Built-in function exports
- image_url_function.rs: Image URL derivation from image number
- rules_preview_function.rs: Fluent-based rules text rendering
- style_tag_parser.rs: HTML-like style tag parsing for rich text
- rich_text_builder.rs: Univer rich text structure generation
- card_lookup_function.rs: Cross-table card name lookup

#### Validation Module (src/validation/)
- validation_mod.rs: Module declarations and rule types
- rule_parser.rs: Parse validation rules from metadata
- type_validator.rs: Type constraint checking
- enum_validator.rs: Enumeration constraint checking
- range_validator.rs: Numeric range checking
- pattern_validator.rs: Regex pattern matching

#### Images Module (src/images/)
- images_mod.rs: Module declarations and image types
- image_cache.rs: Content-addressed image cache management
- image_fetcher.rs: HTTP image download with reqwest
- image_encoder.rs: Base64 encoding for frontend transport
- cache_cleanup.rs: LRU eviction and startup integrity check

#### UUID Module (src/uuid/)
- uuid_mod.rs: Module declarations
- uuid_generator.rs: UUID v4 generation and ID column detection

#### Logging Module (src/logging/)
- logging_mod.rs: Module declarations and initialization
- json_logger.rs: Custom tracing subscriber for JSONL output
- log_aggregator.rs: Frontend log aggregation and formatting
- log_rotation.rs: Date-based log file rotation

#### Error Module (src/error/)
- error_mod.rs: Module declarations and error types
- error_types.rs: Custom error enum with variants for all failure modes
- recovery_handler.rs: Error recovery strategies and user notification

## Test Suite (rules_engine/tests/tv_tests/)

### Configuration
- Cargo.toml: Test crate dependencies including tv as dependency

### Test Fixtures (fixtures/)
- simple_table.toml: Basic array of tables for happy path testing
- with_comments.toml: File with inline and standalone comments
- sparse_data.toml: Tables with missing fields in some entries
- with_metadata.toml: File with complete metadata section
- large_table.toml: 1000+ row table for performance testing
- invalid_syntax.toml: Intentionally malformed for error testing
- unicode_content.toml: Non-ASCII content for encoding testing

### Test Source (src/)
- lib.rs: Test crate root with shared fixtures loading

### TOML Tests (src/toml_tests/)
- toml_tests_mod.rs: Module organization
- load_tests.rs: File loading and parsing tests
- save_tests.rs: File writing and atomic save tests
- preservation_tests.rs: Comment and whitespace preservation tests
- metadata_tests.rs: Metadata parsing and serialization tests

### Sync Tests (src/sync_tests/)
- sync_tests_mod.rs: Module organization
- watcher_tests.rs: File watcher behavior tests
- conflict_tests.rs: Concurrent edit conflict resolution tests
- state_machine_tests.rs: Sync state transition tests

### Derived Tests (src/derived_tests/)
- derived_tests_mod.rs: Module organization
- registry_tests.rs: Function registration and lookup tests
- executor_tests.rs: Async computation execution tests
- generation_tests.rs: Generation counter and staleness tests

### Validation Tests (src/validation_tests/)
- validation_tests_mod.rs: Module organization
- rule_tests.rs: All validation rule type tests
- dropdown_tests.rs: Enum dropdown behavior tests

### Integration Tests (src/integration_tests/)
- integration_tests_mod.rs: Module organization
- end_to_end_tests.rs: Full workflow integration tests
- multi_file_tests.rs: Multiple sheet scenario tests
- recovery_tests.rs: Error recovery scenario tests

### Test Utilities (test_utils/)
- test_utils_mod.rs: Utility exports
- mock_filesystem.rs: In-memory filesystem for isolated testing
- mock_clock.rs: Controllable clock for time-dependent tests
- fixture_loader.rs: Test fixture file loading helpers
- assertion_helpers.rs: Custom assertion macros for test readability
