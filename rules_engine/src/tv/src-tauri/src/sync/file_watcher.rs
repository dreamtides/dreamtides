use std::path::Path;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Clone, Serialize)]
pub struct FileChangedPayload {
    pub path: String,
}

/// Starts a file watcher in a background thread for the given file path.
pub fn start_watcher(app_handle: AppHandle, file_path: String) {
    thread::spawn(move || {
        watch_file(app_handle, file_path);
    });
}

fn watch_file(app_handle: AppHandle, file_path: String) {
    let (tx, rx) = channel();
    let mut debouncer = match new_debouncer(Duration::from_millis(500), tx) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to create file watcher: {e}");
            return;
        }
    };

    let path = Path::new(&file_path);
    if let Err(e) = debouncer.watcher().watch(path, notify::RecursiveMode::NonRecursive) {
        eprintln!("Failed to watch file {file_path}: {e}");
        return;
    }

    println!("Watching file: {file_path}");

    for result in rx {
        match result {
            Ok(events) => {
                for event in events {
                    if event.kind == DebouncedEventKind::Any {
                        let payload = FileChangedPayload { path: file_path.clone() };
                        if let Err(e) = app_handle.emit("toml-file-changed", payload) {
                            eprintln!("Failed to emit file change event: {e}");
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("File watch error: {e:?}");
            }
        }
    }
}
