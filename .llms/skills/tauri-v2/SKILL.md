---
name: tauri-v2
description: Build desktop applications with Tauri V2, TypeScript, and React. Use when creating Tauri projects, writing Rust commands, configuring tauri.conf.json, adding plugins, setting up IPC between frontend and backend, or working with Tauri window APIs.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
---

# Tauri V2 Desktop App Development

Build cross-platform desktop apps using TypeScript/React frontend with Rust backend.

## Quick Start

```bash
# Create new project
pnpm create tauri-app
# Select: TypeScript/JavaScript -> pnpm -> React -> TypeScript

# Development
pnpm tauri dev

# Build for production
pnpm tauri build

# Add plugins
pnpm tauri add store
pnpm tauri add dialog
pnpm tauri add fs
```

## Project Structure

```
├── src/                    # React frontend
├── src-tauri/
│   ├── tauri.conf.json    # Tauri config
│   ├── capabilities/
│   │   └── default.json   # Security permissions
│   └── src/
│       ├── lib.rs         # Main Rust code (edit this)
│       └── main.rs        # Entry point (don't edit)
```

## Core Pattern: Commands

**Rust (src-tauri/src/lib.rs):**
```rust
#[tauri::command]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error running tauri application");
}
```

**Frontend (React):**
```typescript
import { invoke } from '@tauri-apps/api/core';

const result = await invoke<string>('greet', { name: 'World' });
```

## Core Pattern: Events

**Rust -> Frontend:**
```rust
use tauri::{AppHandle, Emitter};

#[tauri::command]
fn start_task(app: AppHandle) {
    app.emit("task-progress", 50).unwrap();
}
```

**Frontend listener:**
```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen<number>('task-progress', (event) => {
  console.log('Progress:', event.payload);
});
```

## Configuration (tauri.conf.json)

```json
{
  "productName": "My App",
  "identifier": "com.example.app",
  "build": {
    "beforeDevCommand": "pnpm dev",
    "beforeBuildCommand": "pnpm build",
    "devUrl": "http://localhost:5173",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [{ "title": "My App", "width": 800, "height": 600 }]
  }
}
```

## Permissions (src-tauri/capabilities/default.json)

```json
{
  "identifier": "default",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "store:default",
    "dialog:default",
    "fs:default"
  ]
}
```

For detailed reference, see:
- [COMMANDS.md](COMMANDS.md) - Rust command patterns, async, error handling, state
- [PLUGINS.md](PLUGINS.md) - Store, Dialog, FS, and other common plugins
- [WINDOWS.md](WINDOWS.md) - Window customization, custom titlebar, multi-window

## Documentation Source

Tauri V2 docs available at: `~/Desktop/tmp/tauri-docs/src/content/docs/`
- start/ - Project setup
- develop/ - Commands, events, configuration
- concept/ - Architecture, IPC, process model
- plugin/ - Plugin reference
- learn/Security/ - Permissions and capabilities
