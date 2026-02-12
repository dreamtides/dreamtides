# TV Tauri Commands Reference

Detailed reference for all Tauri IPC commands in TV.

## Load Command

**File**: `src-tauri/src/commands/load_command.rs`

```rust
#[tauri::command]
pub fn load_toml_table(
    app_handle: AppHandle,
    file_path: String,
    table_name: String,
) -> Result<TomlTableData, TvError>
```

Loads a TOML file and returns spreadsheet-compatible data.

**Parameters:**
- `file_path`: Absolute path to the TOML file
- `table_name`: Name of the array-of-tables key (e.g., "cards", "dreamwell")

**Returns:** `TomlTableData` containing:
- `headers`: Column names extracted from all table entries
- `rows`: Array of row data as JSON objects
- `metadata`: Optional metadata section

**Frontend Usage:**
```typescript
import { invoke } from '@tauri-apps/api/core';

const data = await invoke<TomlTableData>('load_toml_table', {
  filePath: '/path/to/file.toml',
  tableName: 'cards'
});
```

## Save Commands

**File**: `src-tauri/src/commands/save_command.rs`

### save_toml_table

```rust
#[tauri::command]
pub fn save_toml_table(
    app_handle: AppHandle,
    file_path: String,
    table_name: String,
    data: TomlTableData,
) -> Result<(), TvError>
```

Saves entire spreadsheet data back to TOML file.

### save_cell

```rust
#[tauri::command]
pub fn save_cell(
    app_handle: AppHandle,
    file_path: String,
    table_name: String,
    row_index: usize,
    column_key: String,
    value: serde_json::Value,
) -> Result<SaveCellResult, TvError>
```

Saves a single cell update. Preferred for individual edits to minimize conflicts.

**Parameters:**
- `row_index`: Zero-based row index
- `column_key`: Column name/key
- `value`: New cell value as JSON

**Returns:** `SaveCellResult` with any auto-generated values (e.g., UUID)

### save_batch

```rust
#[tauri::command]
pub fn save_batch(
    app_handle: AppHandle,
    file_path: String,
    table_name: String,
    updates: Vec<CellUpdate>,
) -> Result<SaveBatchResult, TvError>
```

Saves multiple cell updates in a single atomic write. Used for paste operations.

**CellUpdate structure:**
```rust
pub struct CellUpdate {
    pub row_index: usize,
    pub column_key: String,
    pub value: serde_json::Value,
}
```

## Watch Commands

**File**: `src-tauri/src/commands/watch_command.rs`

### start_file_watcher

```rust
#[tauri::command]
pub async fn start_file_watcher(
    app_handle: AppHandle,
    file_path: String,
) -> Result<(), TvError>
```

Starts watching a file for external changes. Emits `file_changed` event when detected.

### stop_file_watcher

```rust
#[tauri::command]
pub async fn stop_file_watcher(
    app_handle: AppHandle,
    file_path: String,
) -> Result<(), TvError>
```

Stops watching a file.

## Events

Events emitted from backend to frontend:

### file_changed

Emitted when external file changes are detected (after debouncing).

```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen<{ path: string }>('file_changed', (event) => {
  console.log('File changed:', event.payload.path);
  // Trigger reload
});
```

### sync_state_changed

Emitted when sync state transitions.

```typescript
type SyncState = 'idle' | 'loading' | 'saving' | 'saved' | 'error';

await listen<{ state: SyncState }>('sync_state_changed', (event) => {
  console.log('State:', event.payload.state);
});
```

### sync_conflict

Emitted when concurrent edit conflict detected.

```typescript
await listen<{ filePath: string; message: string }>('sync_conflict', (event) => {
  console.log('Conflict:', event.payload.message);
});
```

## Error Handling

All commands return `Result<T, TvError>`. Error types defined in `error/error_types.rs`:

```rust
pub enum TvError {
    FileNotFound(String),
    ParseError(String),
    WriteError(String),
    SyncError(String),
    ValidationError(String),
}
```

Frontend receives errors as rejected promises with error message.

## IPC Bridge

TypeScript wrappers in `src/ipc_bridge.ts`:

```typescript
export async function loadTomlTable(
  filePath: string,
  tableName: string
): Promise<TomlTableData>;

export async function saveTomlTable(
  filePath: string,
  tableName: string,
  data: TomlTableData
): Promise<void>;

export async function saveCell(
  filePath: string,
  tableName: string,
  rowIndex: number,
  columnKey: string,
  value: unknown
): Promise<SaveCellResult>;

export async function saveBatch(
  filePath: string,
  tableName: string,
  updates: CellUpdate[]
): Promise<SaveBatchResult>;

export async function startFileWatcher(filePath: string): Promise<void>;
export async function stopFileWatcher(filePath: string): Promise<void>;
export async function getAppPaths(): Promise<string[]>;

// Event listeners
export function onFileChanged(callback: (payload: { path: string }) => void): Disposable;
export function onSyncStateChanged(callback: (payload: { state: SyncState }) => void): Disposable;
export function onSyncConflict(callback: (payload: { filePath: string; message: string }) => void): Disposable;
```
