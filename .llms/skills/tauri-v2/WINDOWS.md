# Tauri V2 Window Customization

## Window Configuration (tauri.conf.json)

```json
{
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "My App",
        "width": 800,
        "height": 600,
        "minWidth": 400,
        "minHeight": 300,
        "resizable": true,
        "fullscreen": false,
        "decorations": true,
        "transparent": false,
        "alwaysOnTop": false,
        "center": true
      }
    ]
  }
}
```

## Window API (Frontend)

```typescript
import { getCurrentWindow, WebviewWindow } from '@tauri-apps/api/window';

const appWindow = getCurrentWindow();

// Basic operations
await appWindow.setTitle('New Title');
await appWindow.show();
await appWindow.hide();
await appWindow.close();

// Size and position
await appWindow.setSize({ width: 1200, height: 800 });
await appWindow.setMinSize({ width: 400, height: 300 });
await appWindow.setPosition({ x: 100, y: 100 });
await appWindow.center();

// State
await appWindow.minimize();
await appWindow.maximize();
await appWindow.unmaximize();
await appWindow.toggleMaximize();
await appWindow.setFullscreen(true);

// Properties
await appWindow.setAlwaysOnTop(true);
await appWindow.setDecorations(false);
await appWindow.setResizable(false);

// Query state
const isMaximized = await appWindow.isMaximized();
const isMinimized = await appWindow.isMinimized();
const isFullscreen = await appWindow.isFullscreen();
```

## Creating New Windows

```typescript
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

const settingsWindow = new WebviewWindow('settings', {
    url: '/settings',
    title: 'Settings',
    width: 400,
    height: 300,
    resizable: false,
    center: true
});

// Wait for window to be created
settingsWindow.once('tauri://created', () => {
    console.log('Window created');
});

settingsWindow.once('tauri://error', (e) => {
    console.error('Window creation failed:', e);
});
```

## Custom Titlebar

### 1. Disable Decorations

```json
{
  "app": {
    "windows": [{
      "decorations": false
    }]
  }
}
```

### 2. Add Permissions

```json
{
  "permissions": [
    "core:window:default",
    "core:window:allow-start-dragging"
  ]
}
```

### 3. Create Titlebar Component (React)

```tsx
import { getCurrentWindow } from '@tauri-apps/api/window';

function Titlebar() {
    const appWindow = getCurrentWindow();

    return (
        <div className="titlebar">
            <div data-tauri-drag-region className="titlebar-drag" />
            <div className="titlebar-buttons">
                <button onClick={() => appWindow.minimize()}>−</button>
                <button onClick={() => appWindow.toggleMaximize()}>□</button>
                <button onClick={() => appWindow.close()}>×</button>
            </div>
        </div>
    );
}
```

### 4. Style the Titlebar

```css
.titlebar {
    height: 32px;
    background: #2d2d2d;
    display: flex;
    justify-content: space-between;
    user-select: none;
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    z-index: 9999;
}

.titlebar-drag {
    flex: 1;
    -webkit-app-region: drag;
}

.titlebar-buttons {
    display: flex;
}

.titlebar-buttons button {
    width: 46px;
    height: 32px;
    border: none;
    background: transparent;
    color: white;
}

.titlebar-buttons button:hover {
    background: #3d3d3d;
}

.titlebar-buttons button:last-child:hover {
    background: #e81123;
}

/* Offset main content */
body {
    padding-top: 32px;
}
```

## Window Events

```typescript
import { getCurrentWindow } from '@tauri-apps/api/window';

const appWindow = getCurrentWindow();

// Listen to events
const unlisten = await appWindow.onResized(({ payload: size }) => {
    console.log('Resized:', size.width, size.height);
});

await appWindow.onMoved(({ payload: position }) => {
    console.log('Moved:', position.x, position.y);
});

await appWindow.onCloseRequested(async (event) => {
    // Prevent close and show confirmation
    event.preventDefault();
    const confirmed = await confirm('Really close?');
    if (confirmed) {
        await appWindow.destroy();
    }
});

await appWindow.onFocusChanged(({ payload: focused }) => {
    console.log('Focus:', focused);
});

// Clean up
unlisten();
```

## Creating Windows from Rust

```rust
use tauri::{WebviewUrl, WebviewWindowBuilder, Manager};

#[tauri::command]
async fn open_settings(app: tauri::AppHandle) {
    let _window = WebviewWindowBuilder::new(
        &app,
        "settings",
        WebviewUrl::App("settings.html".into())
    )
    .title("Settings")
    .inner_size(400.0, 300.0)
    .resizable(false)
    .center()
    .build()
    .unwrap();
}
```

## macOS Transparent Titlebar

```rust
use tauri::{TitleBarStyle, WebviewUrl, WebviewWindowBuilder};

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let win = WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::default()
            )
            .title("My App")
            .inner_size(800.0, 600.0);

            #[cfg(target_os = "macos")]
            let win = win.title_bar_style(TitleBarStyle::Transparent);

            win.build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .unwrap();
}
```

## Window State Plugin (Persist Size/Position)

```bash
pnpm tauri add window-state
```

Automatically saves and restores window dimensions between app launches.

```json
"permissions": ["window-state:default"]
```
