# TV Architecture Reference

Detailed architecture documentation for the TV application.

## Module Overview

### TOML Module (`src-tauri/src/toml/`)

Handles all TOML file operations with structure preservation.

#### document_loader.rs

Loads TOML files and converts to frontend-compatible format.

```rust
pub struct TomlTableData {
    pub headers: Vec<String>,
    pub rows: Vec<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

pub fn load_toml_document(
    file_path: &str,
    table_name: &str,
) -> Result<TomlTableData, TvError>;
```

**Key behaviors:**
- Headers collected from all table entries (sparse data support)
- Preserves original key ordering
- Extracts `[metadata]` section separately
- Converts TOML values to JSON for frontend

#### document_writer.rs

Writes changes back preserving TOML structure.

```rust
pub fn save_toml_document(
    file_path: &str,
    table_name: &str,
    data: &TomlTableData,
) -> Result<(), TvError>;

pub fn save_cell(
    file_path: &str,
    table_name: &str,
    update: &CellUpdate,
) -> Result<SaveCellResult, TvError>;

pub fn save_batch(
    file_path: &str,
    table_name: &str,
    updates: &[CellUpdate],
) -> Result<SaveBatchResult, TvError>;

pub fn cleanup_orphaned_temp_files(dir: &str) -> Result<(), TvError>;
```

**Key behaviors:**
- Uses `toml_edit::DocumentMut` for in-place edits
- Atomic writes via temp file + rename
- Preserves comments and formatting
- Auto-generates UUIDs for empty `id` columns

#### value_converter.rs

Converts between JSON and TOML value types.

```rust
pub fn json_to_toml_value(json: &serde_json::Value) -> toml_edit::Value;
pub fn toml_to_json_value(toml: &toml_edit::Item) -> serde_json::Value;
```

### Sync Module (`src-tauri/src/sync/`)

Manages bidirectional synchronization.

#### file_watcher.rs

File change detection using `notify` crate.

```rust
pub struct FileWatcherState {
    watchers: Mutex<HashMap<PathBuf, RecommendedWatcher>>,
}

pub fn start_watcher(
    app_handle: &AppHandle,
    file_path: &str,
) -> Result<(), TvError>;

pub fn stop_watcher(
    app_handle: &AppHandle,
    file_path: &str,
) -> Result<(), TvError>;

pub fn stop_all_watchers(app_handle: &AppHandle);
```

**Key behaviors:**
- 500ms debouncing to coalesce rapid changes
- One watcher per file
- Emits `file_changed` event
- Cleanup on window close

#### state_machine.rs

Sync state management to prevent race conditions.

```rust
pub enum SyncState {
    Idle,
    Loading,
    Saving,
    Error(String),
}

pub fn begin_load(app_handle: &AppHandle, file_path: &str) -> Result<(), TvError>;
pub fn end_load(app_handle: &AppHandle, file_path: &str, success: bool);
pub fn begin_save(app_handle: &AppHandle, file_path: &str) -> Result<(), TvError>;
pub fn end_save(app_handle: &AppHandle, file_path: &str, success: bool) -> Result<(), TvError>;
```

**State transitions:**
```
Idle → Loading → Idle (success) or Error (failure)
Idle → Saving → Idle (success) or Error (failure)
Saving blocks Loading (file changes during save are ignored)
```

### Derived Module (`src-tauri/src/derived/`)

Asynchronous computed columns.

#### derived_types.rs

Types for derived column definitions.

```rust
pub struct DerivedColumnDef {
    pub name: String,
    pub function_name: String,
    pub params: HashMap<String, serde_json::Value>,
}

pub struct DerivedResult {
    pub column: String,
    pub row_index: usize,
    pub value: DerivedValue,
}

pub enum DerivedValue {
    Text(String),
    RichText(UniverRichText),
    Image(String),
    Error(String),
}
```

#### rich_text_converter.rs

Converts styled text to Univer rich text format.

```rust
pub fn convert_to_rich_text(
    text: &str,
    styles: &[TextStyle],
) -> UniverRichText;

pub struct TextStyle {
    pub start: usize,
    pub end: usize,
    pub bold: bool,
    pub italic: bool,
    pub color: Option<String>,
}
```

#### card_lookup.rs

Cross-references card data between tables.

```rust
pub fn lookup_card_name(
    card_id: &str,
    cards_data: &TomlTableData,
) -> Option<String>;
```

### Error Module (`src-tauri/src/error/`)

#### error_types.rs

Centralized error definitions.

```rust
#[derive(Debug, thiserror::Error, serde::Serialize)]
pub enum TvError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Failed to parse TOML: {0}")]
    ParseError(String),

    #[error("Failed to write file: {0}")]
    WriteError(String),

    #[error("Sync error: {0}")]
    SyncError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}
```

## Frontend Architecture

### Component Hierarchy

```
AppRoot (app_root.tsx)
├── ErrorBanner (error_banner.tsx)
├── SpreadsheetView (spreadsheet_view.tsx)
│   └── UniverSpreadsheet (UniverSpreadsheet.tsx)
└── StatusIndicator (status_indicator.tsx)
```

### State Management

State managed in `AppRoot`:

```typescript
interface AppState {
  sheets: SheetInfo[];           // All loaded sheets
  multiSheetData: MultiSheetData | null;  // Sheet data
  activeSheetId: string | null;  // Currently active sheet
  error: string | null;          // Error message
  loading: boolean;              // Loading state
  saveStatus: SyncState;         // Save/sync status
}
```

### Data Flow

1. **Initialization:**
   - `getAppPaths()` → list of TOML files
   - `loadTomlTable()` for each file
   - `startFileWatcher()` for each file

2. **User Edit:**
   - Univer `SheetValueChanged` event
   - Debounced `save_cell` or `save_batch` call
   - Status indicator shows "Saving"

3. **External Change:**
   - `file_changed` event received
   - Check if currently saving (ignore if yes)
   - `loadTomlTable()` to refresh
   - Update Univer workbook data

### Univer Integration

Setup in `UniverSpreadsheet.tsx`:

```typescript
const { univerAPI } = createUniver({
  locale: LocaleType.EN_US,
  theme: defaultTheme,
  presets: [
    UniverSheetsCorePreset({ container: containerRef.current }),
  ],
});

// Listen for value changes
univerAPI.addEvent(univerAPI.Event.SheetValueChanged, (params) => {
  // Extract changes and forward to backend
});
```

## Configuration

### tauri.conf.json

Key settings in `src-tauri/tauri.conf.json`:

```json
{
  "productName": "tv",
  "identifier": "com.dreamtides.tv",
  "build": {
    "beforeDevCommand": "pnpm dev",
    "beforeBuildCommand": "pnpm build",
    "devUrl": "http://localhost:5173",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [{
      "title": "TV - TOML Viewer",
      "width": 1200,
      "height": 800
    }]
  }
}
```

### Cargo.toml Dependencies

Key Rust dependencies:

```toml
[dependencies]
tauri = { version = "2", features = [] }
toml_edit = "0.22"
toml = "0.8"
notify = "6"
notify-debouncer-mini = "0.4"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4"] }
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
thiserror = "1"
```

## File Format

### TOML Array-of-Tables

TV expects files in this format:

```toml
# Optional metadata section (not displayed)
[metadata]
column_widths = { name = 200, id = 100 }
derived_columns = [{ name = "preview", function = "rules_preview" }]

# Main data as array of tables
[[cards]]
id = "abc123"
name = "Example Card"
energy_cost = 3

[[cards]]
id = "def456"
name = "Another Card"
energy_cost = 5
```

Each `[[cards]]` entry becomes a spreadsheet row. Keys become column headers.
