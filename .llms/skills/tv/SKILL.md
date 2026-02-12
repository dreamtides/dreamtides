---
name: tv
description: Work with the TV (TOML Viewer) desktop application for editing TOML files in a spreadsheet format. Use when implementing features for TV, understanding the TV architecture, working with Tauri commands, or debugging TV issues.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
user-invocable: false
---

# TV (TOML Viewer) Desktop Application

TV is a Tauri V2 desktop application for viewing and editing TOML files in a spreadsheet format using the Univer library. It replaces an Excel-based workflow, making TOML files the sole source of truth for game data.

## Quick Reference

```bash
# Run in development mode
cd rules_engine/src/tv && pnpm tauri dev

# Run with specific file
cd rules_engine/src/tv && pnpm tauri dev -- -- path/to/file.toml

# Run with directory (opens all TOML files as sheets)
cd rules_engine/src/tv && pnpm tauri dev -- -- path/to/directory/

# Build for production
cd rules_engine/src/tv && pnpm tauri build
```

## Source Code Location

All TV source code is in `rules_engine/src/tv/`:

```
rules_engine/src/tv/
├── src/                          # TypeScript frontend
│   ├── main.tsx                  # React entry point
│   ├── app_root.tsx              # Main app component, state management
│   ├── spreadsheet_view.tsx      # Univer spreadsheet wrapper
│   ├── error_banner.tsx          # Error display component
│   ├── status_indicator.tsx      # Save/sync status display
│   ├── UniverSpreadsheet.tsx     # Core Univer integration
│   └── ipc_bridge.ts             # Tauri command wrappers
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── main.rs               # Binary entry point
│   │   ├── lib.rs                # Library with Tauri setup
│   │   ├── cli.rs                # CLI argument parsing
│   │   ├── commands/             # Tauri command implementations
│   │   ├── toml/                 # TOML loading/writing
│   │   ├── sync/                 # File watching, state machine
│   │   ├── derived/              # Derived column computation
│   │   ├── validation/           # Data validation rules
│   │   ├── images/               # Image caching
│   │   ├── uuid/                 # UUID generation
│   │   ├── logging/              # JSONL logging
│   │   └── error/                # Error types
│   ├── Cargo.toml                # Rust dependencies
│   └── tauri.conf.json           # Tauri configuration
└── docs/                         # Design documentation
```

## Architecture Overview

TV follows a three-layer architecture:

1. **TOML Layer** (Rust): File I/O, parsing, structure preservation with `toml_edit`
2. **Application Layer** (Rust): State management, derived columns, validation, caching
3. **UI Layer** (TypeScript): Univer spreadsheet rendering, user interaction

### Key Concepts

- **Array-of-Tables Format**: TOML files use `[[table_name]]` format where each entry becomes a row
- **Structure Preservation**: All writes use `toml_edit` to preserve comments and formatting
- **Bidirectional Sync**: Changes sync UI-to-file and file-to-UI via file watching
- **Derived Columns**: Columns computed asynchronously from row data (e.g., rules text preview)
- **Multi-Sheet Support**: Each TOML file becomes a separate sheet tab

## Tauri Commands

Available IPC commands (defined in `src-tauri/src/commands/`):

| Command | File | Description |
|---------|------|-------------|
| `load_toml_table` | load_command.rs:9 | Load TOML file as spreadsheet data |
| `save_toml_table` | save_command.rs:9 | Save entire spreadsheet to TOML |
| `save_cell` | save_command.rs:26 | Save single cell update |
| `save_batch` | save_command.rs:45 | Save multiple cell updates atomically |
| `start_file_watcher` | watch_command.rs | Start watching file for external changes |
| `stop_file_watcher` | watch_command.rs | Stop watching a file |
| `get_app_paths` | lib.rs:28 | Get list of TOML files to display |

## Key Modules

### TOML Module (`src-tauri/src/toml/`)
- **document_loader.rs**: Parse TOML files into `TomlTableData` for frontend
- **document_writer.rs**: Write changes back preserving structure
- **value_converter.rs**: Convert between JSON and TOML value types

### Sync Module (`src-tauri/src/sync/`)
- **file_watcher.rs**: File change detection with debouncing
- **state_machine.rs**: Sync state transitions (Idle, Loading, Saving)
- **Events**: `file_changed`, `sync_state_changed`, `sync_conflict`

### Derived Module (`src-tauri/src/derived/`)
- **derived_types.rs**: Types for derived column definitions
- **rich_text_converter.rs**: Convert styled text to Univer rich text format
- **card_lookup.rs**: Cross-reference card data from other tables

## Data Flow

```
User Edit → Univer Event → ipc_bridge.ts → Tauri Command
    → document_writer.rs → TOML File → file_watcher.rs
    → Event to Frontend → Reload (if external change)
```

## Testing

Tests are in `rules_engine/tests/tv_tests/`:

```bash
# Run TV tests
just test -p tv_tests

# Run specific test
just test -p tv_tests -- test_name
```

## Documentation

Detailed documentation in `rules_engine/src/tv/docs/`:
- [tv_design_document.md](../rules_engine/src/tv/docs/tv_design_document.md) - Main design doc
- [appendix_a_metadata_schema.md](../rules_engine/src/tv/docs/appendix_a_metadata_schema.md) - Metadata format
- [appendix_b_derived_functions.md](../rules_engine/src/tv/docs/appendix_b_derived_functions.md) - Derived columns
- [appendix_c_sync_protocol.md](../rules_engine/src/tv/docs/appendix_c_sync_protocol.md) - Sync behavior
- [appendix_d_univer_integration.md](../rules_engine/src/tv/docs/appendix_d_univer_integration.md) - Univer setup
- [appendix_f_file_layout.md](../rules_engine/src/tv/docs/appendix_f_file_layout.md) - File structure
- [appendix_g_rules_text_preview.md](../rules_engine/src/tv/docs/appendix_g_rules_text_preview.md) - Rules text rendering
- [appendix_h_implementation_plan.md](../rules_engine/src/tv/docs/appendix_h_implementation_plan.md) - Implementation tasks

## Additional Reference

For detailed reference, see:
- [COMMANDS.md](COMMANDS.md) - Detailed Tauri command reference with types and usage
- [ARCHITECTURE.md](ARCHITECTURE.md) - In-depth module architecture and data flow

## Related Skills

- **tauri-v2**: Tauri V2 patterns for commands, events, plugins
- **univer**: Univer spreadsheet API for cells, styling, images, validation

## Common Tasks

### Adding a New Tauri Command

1. Create command in `src-tauri/src/commands/`
2. Register in `lib.rs` via `tauri::generate_handler![]`
3. Add TypeScript wrapper in `src/ipc_bridge.ts`

### Adding a Derived Column Function

1. Implement function in `src-tauri/src/derived/`
2. Register in the function registry
3. Define column in TOML metadata section

### Modifying TOML Structure

Always use `toml_edit::DocumentMut` to preserve comments and formatting. See `document_writer.rs` for patterns.
