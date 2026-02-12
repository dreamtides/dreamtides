# Tauri V2 Commands Reference

## Basic Command

```rust
#[tauri::command]
fn my_command(arg: String) -> String {
    format!("Received: {}", arg)
}
```

Register in builder:
```rust
.invoke_handler(tauri::generate_handler![my_command, other_command])
```

## Async Commands (Preferred for Heavy Work)

```rust
#[tauri::command]
async fn fetch_data(url: String) -> Result<String, String> {
    // async operations don't block UI
    Ok("data".to_string())
}
```

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Custom: {0}")]
    Custom(String),
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::ser::Serializer {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[tauri::command]
fn risky_op() -> Result<String, Error> {
    std::fs::read_to_string("file.txt")?;
    Ok("success".to_string())
}
```

## State Management

```rust
use std::sync::Mutex;
use tauri::Manager;

#[derive(Default)]
struct AppState {
    counter: u32,
}

#[tauri::command]
fn increment(state: tauri::State<'_, Mutex<AppState>>) -> u32 {
    let mut s = state.lock().unwrap();
    s.counter += 1;
    s.counter
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(Mutex::new(AppState::default()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![increment])
        .run(tauri::generate_context!())
        .unwrap();
}
```

## Accessing Window/AppHandle

```rust
#[tauri::command]
async fn with_window(window: tauri::WebviewWindow) {
    println!("Window: {}", window.label());
}

#[tauri::command]
async fn with_app(app: tauri::AppHandle) {
    let data_dir = app.path().app_data_dir().unwrap();
}
```

## Channels (Streaming Data)

```rust
use tauri::ipc::Channel;

#[tauri::command]
async fn stream_data(channel: Channel<Vec<u8>>) {
    for chunk in data_chunks {
        channel.send(chunk).unwrap();
    }
}
```

Frontend:
```typescript
import { invoke, Channel } from '@tauri-apps/api/core';

const channel = new Channel<Uint8Array>();
channel.onmessage = (data) => console.log('Chunk:', data);
await invoke('stream_data', { channel });
```

## Commands in Separate Module

```rust
// src-tauri/src/commands.rs
#[tauri::command]
pub fn my_command() -> String {
    "hello".to_string()
}

// src-tauri/src/lib.rs
mod commands;

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::my_command])
        // ...
}
```

## Frontend Invocation

```typescript
import { invoke } from '@tauri-apps/api/core';

// Basic
const result = await invoke<string>('greet', { name: 'World' });

// With error handling
try {
    const data = await invoke<Data>('fetch_data', { url });
} catch (error) {
    console.error('Command failed:', error);
}

// Arguments use camelCase (converted to snake_case in Rust)
invoke('my_command', { userName: 'alice' }); // -> user_name in Rust
```
