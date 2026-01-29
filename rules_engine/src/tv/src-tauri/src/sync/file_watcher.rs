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
use crate::error::permission_recovery::{self, PermissionState};
use crate::sync::state_machine;

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
        tracing::info!(component = "tv.sync", "All file watchers stopped");
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
                    let mut collected =
                        debounced_events_clone.lock().unwrap_or_else(|e| e.into_inner());
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

            let is_busy = state_machine::is_busy(&app_handle, &file_path);
            let was_recently_saved = state_machine::was_recently_saved(&app_handle, &file_path);
            let event_timestamp_ms = std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0);

            if is_busy {
                tracing::info!(
                    component = "tv.sync.watcher",
                    file_path = %file_path,
                    event_type = %event_type,
                    event_timestamp_ms = %event_timestamp_ms,
                    "Ignoring file change event during active save/load"
                );
                continue;
            }

            if was_recently_saved {
                tracing::info!(
                    component = "tv.sync.watcher",
                    file_path = %file_path,
                    event_type = %event_type,
                    event_timestamp_ms = %event_timestamp_ms,
                    "Ignoring file change event within 600ms of save completion (self-modification)"
                );
                continue;
            }

            tracing::info!(
                component = "tv.sync.watcher",
                file_path = %file_path,
                event_type = %event_type,
                event_timestamp_ms = %event_timestamp_ms,
                "File watcher event will be emitted"
            );

            if !path.exists() {
                // Check if file was previously marked as deleted - if so, it's still
                // missing, no need to emit another event
                if permission_recovery::is_file_deleted(&app_handle, &file_path) {
                    tracing::debug!(
                        component = "tv.sync.deletion",
                        file_path = %file_path,
                        "File still missing, continuing to monitor for reappearance"
                    );
                    continue;
                }

                tracing::warn!(
                    component = "tv.sync.deletion",
                    file_path = %file_path,
                    "Watched file has been deleted or moved"
                );

                // Mark the file as deleted (not just unreadable - this provides
                // more specific handling and tracks deletion time)
                permission_recovery::mark_file_deleted(&app_handle, &file_path);

                let payload = FileChangedPayload {
                    file_path: file_path.clone(),
                    event_type: "delete".to_string(),
                };
                if let Err(e) = app_handle.emit("toml-file-changed", payload) {
                    tracing::error!(
                        component = "tv.sync.deletion",
                        file_path = %file_path,
                        error = %e,
                        "Failed to emit file delete event"
                    );
                }
                continue;
            }

            // Check if file has reappeared after being deleted
            if permission_recovery::check_file_reappearance(&app_handle, &file_path) {
                tracing::info!(
                    component = "tv.sync.deletion",
                    file_path = %file_path,
                    "File has reappeared after deletion, triggering reload"
                );

                let payload = FileChangedPayload {
                    file_path: file_path.clone(),
                    event_type: "restored".to_string(),
                };
                if let Err(e) = app_handle.emit("toml-file-changed", payload) {
                    tracing::error!(
                        component = "tv.sync.deletion",
                        file_path = %file_path,
                        error = %e,
                        "Failed to emit file restored event"
                    );
                }
                // Don't continue - we want to check permissions and emit the
                // regular change event as well for the reload to happen
            }

            // Check if permissions have changed (file might have been restored or
            // permissions modified)
            let current_perm = permission_recovery::detect_permission_state(path);
            let previous_perm = permission_recovery::get_permission_state(&app_handle, &file_path);

            if current_perm != previous_perm {
                let message =
                    permission_recovery::get_permission_error_message(current_perm, &file_path);
                permission_recovery::set_permission_state(
                    &app_handle,
                    &file_path,
                    current_perm,
                    &message,
                );

                // If permissions were restored to ReadWrite, try to apply pending updates
                if current_perm == PermissionState::ReadWrite
                    && previous_perm != PermissionState::ReadWrite
                {
                    let pending_count =
                        permission_recovery::get_pending_update_count(&app_handle, &file_path);
                    if pending_count > 0 {
                        tracing::info!(
                            component = "tv.sync",
                            file_path = %file_path,
                            pending_count = pending_count,
                            "Permissions restored, attempting to apply pending updates"
                        );
                        // Note: actual retry will be triggered by frontend upon receiving
                        // the permission state change event
                    }
                }
            }

            let payload = FileChangedPayload {
                file_path: file_path.clone(),
                event_type: event_type.to_string(),
            };

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
