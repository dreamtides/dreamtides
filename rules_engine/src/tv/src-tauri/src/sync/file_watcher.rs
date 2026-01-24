use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::error::error_types::TvError;

#[derive(Clone, Serialize)]
pub struct FileChangedPayload {
    pub file_path: String,
    pub event_type: String,
}

enum WatcherCommand {
    Stop,
}

struct WatcherHandle {
    stop_tx: Sender<WatcherCommand>,
}

pub struct FileWatcherState {
    watchers: Mutex<HashMap<PathBuf, WatcherHandle>>,
}

impl Default for FileWatcherState {
    fn default() -> Self {
        Self { watchers: Mutex::new(HashMap::new()) }
    }
}

impl FileWatcherState {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Starts watching a file for external changes with 500ms debouncing.
pub fn start_watcher(app_handle: AppHandle, file_path: String) -> Result<(), TvError> {
    let path = PathBuf::from(&file_path);
    let state = app_handle.state::<FileWatcherState>();

    {
        let watchers = state.watchers.lock().unwrap_or_else(|e| e.into_inner());
        if watchers.contains_key(&path) {
            tracing::debug!(
                component = "tv.sync",
                file_path = %file_path,
                "Watcher already active for file"
            );
            return Ok(());
        }
    }

    let (stop_tx, stop_rx) = mpsc::channel::<WatcherCommand>();

    let handle = WatcherHandle { stop_tx };

    {
        let mut watchers = state.watchers.lock().unwrap_or_else(|e| e.into_inner());
        watchers.insert(path.clone(), handle);
    }

    let app_handle_clone = app_handle.clone();
    let file_path_clone = file_path.clone();
    let path_clone = path.clone();

    thread::spawn(move || {
        if let Err(e) = run_watcher(app_handle_clone, file_path_clone, stop_rx) {
            tracing::error!(
                component = "tv.sync",
                error = %e,
                "Watcher thread terminated with error"
            );
        }

        if let Some(state) = app_handle.try_state::<FileWatcherState>() {
            let mut watchers = state.watchers.lock().unwrap_or_else(|e| e.into_inner());
            watchers.remove(&path_clone);
            tracing::debug!(
                component = "tv.sync",
                file_path = %path_clone.display(),
                "Watcher removed from state"
            );
        }
    });

    Ok(())
}

/// Stops watching a file for changes.
pub fn stop_watcher(app_handle: AppHandle, file_path: String) -> Result<(), TvError> {
    let path = PathBuf::from(&file_path);
    let state = app_handle.state::<FileWatcherState>();

    let mut watchers = state.watchers.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(handle) = watchers.remove(&path) {
        let _ = handle.stop_tx.send(WatcherCommand::Stop);
        tracing::info!(
            component = "tv.sync",
            file_path = %file_path,
            "Watcher stop requested"
        );
    } else {
        tracing::debug!(
            component = "tv.sync",
            file_path = %file_path,
            "No active watcher to stop"
        );
    }

    Ok(())
}

/// Stops all active file watchers.
pub fn stop_all_watchers(app_handle: &AppHandle) {
    if let Some(state) = app_handle.try_state::<FileWatcherState>() {
        let mut watchers = state.watchers.lock().unwrap_or_else(|e| e.into_inner());
        for (path, handle) in watchers.drain() {
            let _ = handle.stop_tx.send(WatcherCommand::Stop);
            tracing::debug!(
                component = "tv.sync",
                file_path = %path.display(),
                "Watcher stopped during cleanup"
            );
        }
        tracing::info!(
            component = "tv.sync",
            "All file watchers stopped"
        );
    }
}

fn run_watcher(
    app_handle: AppHandle,
    file_path: String,
    stop_rx: mpsc::Receiver<WatcherCommand>,
) -> Result<(), TvError> {
    let (tx, rx) = mpsc::channel();

    let mut debouncer = new_debouncer(Duration::from_millis(500), tx).map_err(|e| {
        tracing::error!(
            component = "tv.sync",
            error = %e,
            "Failed to create file watcher"
        );
        TvError::WatcherCreationFailed { message: e.to_string() }
    })?;

    let path = Path::new(&file_path);
    debouncer.watcher().watch(path, notify::RecursiveMode::NonRecursive).map_err(|e| {
        tracing::error!(
            component = "tv.sync",
            file_path = %file_path,
            error = %e,
            "Failed to watch file"
        );
        TvError::WatchPathFailed { path: file_path.clone(), message: e.to_string() }
    })?;

    tracing::info!(
        component = "tv.sync",
        file_path = %file_path,
        "Watcher started"
    );

    let debounced_events = Arc::new(Mutex::new(Vec::new()));
    let debounced_events_clone = Arc::clone(&debounced_events);

    thread::spawn(move || {
        for result in rx {
            match result {
                Ok(events) => {
                    let mut collected = debounced_events_clone.lock().unwrap_or_else(|e| e.into_inner());
                    collected.extend(events);
                }
                Err(e) => {
                    tracing::warn!(
                        component = "tv.sync",
                        error = ?e,
                        "Watcher received error event, continuing to watch"
                    );
                }
            }
        }
    });

    loop {
        match stop_rx.recv_timeout(Duration::from_millis(100)) {
            Ok(WatcherCommand::Stop) => {
                tracing::debug!(
                    component = "tv.sync",
                    file_path = %file_path,
                    "Watcher received stop command"
                );
                break;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                tracing::debug!(
                    component = "tv.sync",
                    file_path = %file_path,
                    "Watcher stop channel disconnected"
                );
                break;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }

        let events: Vec<_> = {
            let mut collected = debounced_events.lock().unwrap_or_else(|e| e.into_inner());
            collected.drain(..).collect()
        };

        for event in events {
            let event_type = match event.kind {
                DebouncedEventKind::Any => "modify",
                DebouncedEventKind::AnyContinuous => "modify",
                _ => "modify",
            };

            tracing::debug!(
                component = "tv.sync",
                file_path = %file_path,
                event_type = %event_type,
                "Watcher event received"
            );

            if !path.exists() {
                tracing::warn!(
                    component = "tv.sync",
                    file_path = %file_path,
                    "Watched file no longer exists"
                );
                let payload =
                    FileChangedPayload { file_path: file_path.clone(), event_type: "delete".to_string() };
                if let Err(e) = app_handle.emit("toml-file-changed", payload) {
                    tracing::error!(
                        component = "tv.sync",
                        file_path = %file_path,
                        error = %e,
                        "Failed to emit file delete event"
                    );
                }
                continue;
            }

            let payload =
                FileChangedPayload { file_path: file_path.clone(), event_type: event_type.to_string() };

            if let Err(e) = app_handle.emit("toml-file-changed", payload) {
                tracing::error!(
                    component = "tv.sync",
                    file_path = %file_path,
                    error = %e,
                    "Failed to emit file change event"
                );
            }
        }
    }

    tracing::debug!(
        component = "tv.sync",
        file_path = %file_path,
        "Watcher stopped"
    );

    Ok(())
}
