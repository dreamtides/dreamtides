# Tauri V2 Plugins Reference

## Adding Plugins

```bash
pnpm tauri add <plugin-name>
```

This automatically:
1. Adds Rust dependency to Cargo.toml
2. Initializes plugin in lib.rs
3. Installs JS bindings

## Store Plugin (Persistent Key-Value Storage)

```bash
pnpm tauri add store
```

**Usage:**
```typescript
import { load } from '@tauri-apps/plugin-store';

const store = await load('settings.json', { autoSave: true });

// Set value
await store.set('theme', 'dark');
await store.set('user', { name: 'Alice', id: 123 });

// Get value
const theme = await store.get<string>('theme');
const user = await store.get<{ name: string; id: number }>('user');

// Delete
await store.delete('theme');

// Manual save (if autoSave: false)
await store.save();
```

**Permissions (capabilities/default.json):**
```json
"permissions": ["store:default"]
```

## Dialog Plugin (Native Dialogs)

```bash
pnpm tauri add dialog
```

**Usage:**
```typescript
import { open, save, message, ask, confirm } from '@tauri-apps/plugin-dialog';

// File picker
const file = await open({
    multiple: false,
    directory: false,
    filters: [{ name: 'Images', extensions: ['png', 'jpg'] }]
});

// Directory picker
const dir = await open({ directory: true });

// Save dialog
const savePath = await save({
    defaultPath: 'document.txt',
    filters: [{ name: 'Text', extensions: ['txt'] }]
});

// Message box
await message('Operation complete!', { title: 'Success', kind: 'info' });

// Yes/No dialog
const yes = await ask('Delete this file?', { title: 'Confirm', kind: 'warning' });

// Ok/Cancel dialog
const ok = await confirm('Proceed?', { title: 'Confirm' });
```

**Permissions:**
```json
"permissions": ["dialog:default"]
```

## File System Plugin

```bash
pnpm tauri add fs
```

**Usage:**
```typescript
import {
    readTextFile, writeTextFile, readFile, writeFile,
    readDir, createDir, remove, rename, exists,
    BaseDirectory
} from '@tauri-apps/plugin-fs';

// Read/write text
const content = await readTextFile('config.json', {
    baseDir: BaseDirectory.AppData
});
await writeTextFile('output.txt', 'Hello!', {
    baseDir: BaseDirectory.Download
});

// Read/write binary
const bytes = await readFile('image.png', { baseDir: BaseDirectory.Resource });
await writeFile('copy.png', bytes, { baseDir: BaseDirectory.Download });

// Directory operations
const entries = await readDir('documents', { baseDir: BaseDirectory.Home });
await createDir('my-app/data', { baseDir: BaseDirectory.AppData, recursive: true });

// File operations
await remove('temp.txt', { baseDir: BaseDirectory.Temp });
await rename('old.txt', 'new.txt', { baseDir: BaseDirectory.Download });
const fileExists = await exists('config.json', { baseDir: BaseDirectory.AppConfig });
```

**Base Directories:**
- `AppData` - App-specific data directory
- `AppConfig` - App config directory
- `Download` - User's downloads
- `Home` - User's home directory
- `Resource` - Bundled resources
- `Temp` - Temporary files

**Permissions (scoped access):**
```json
"permissions": [
    "fs:default",
    {
        "identifier": "fs:allow-read-text-file",
        "allow": [{ "path": "$APPDATA/**" }]
    },
    {
        "identifier": "fs:allow-write-text-file",
        "allow": [{ "path": "$DOWNLOAD/**" }]
    }
]
```

## HTTP Plugin

```bash
pnpm tauri add http
```

**Usage:**
```typescript
import { fetch } from '@tauri-apps/plugin-http';

const response = await fetch('https://api.example.com/data', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ key: 'value' })
});
const data = await response.json();
```

## Shell Plugin

```bash
pnpm tauri add shell
```

**Usage:**
```typescript
import { Command } from '@tauri-apps/plugin-shell';

// Run command
const output = await Command.create('echo', ['Hello']).execute();
console.log(output.stdout);

// Open URL/file with default app
import { open } from '@tauri-apps/plugin-shell';
await open('https://tauri.app');
await open('/path/to/file.pdf');
```

## SQL Plugin (SQLite)

```bash
pnpm tauri add sql
```

**Usage:**
```typescript
import Database from '@tauri-apps/plugin-sql';

const db = await Database.load('sqlite:app.db');

await db.execute(`
    CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL
    )
`);

await db.execute('INSERT INTO users (name) VALUES (?)', ['Alice']);

const users = await db.select<{ id: number; name: string }[]>(
    'SELECT * FROM users WHERE name = ?', ['Alice']
);
```

## Other Common Plugins

| Plugin | Use Case |
|--------|----------|
| `updater` | Auto-update functionality |
| `notification` | System notifications |
| `clipboard` | Clipboard read/write |
| `global-shortcut` | Global keyboard shortcuts |
| `process` | Process info, exit, restart |
| `os` | OS information |
| `window-state` | Persist window size/position |
