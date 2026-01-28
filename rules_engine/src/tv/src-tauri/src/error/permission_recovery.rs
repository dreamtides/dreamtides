use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::error::error_types::TvError;
use crate::toml::document_writer::CellUpdate;

/// Tracks the permission state for a file (readable and writable).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionState {
    /// File is both readable and writable.
    ReadWrite,
    /// File is readable but not writable (read-only mode).
    ReadOnly,
    /// File is not readable (preserve last known data).
    Unreadable,
}

impl PermissionState {
    fn as_str(self) -> &'static str {
        match self {
            PermissionState::ReadWrite => "read_write",
            PermissionState::ReadOnly => "read_only",
            PermissionState::Unreadable => "unreadable",
        }
    }
}

/// A pending cell update that was rejected due to permission issues.
#[derive(Debug, Clone)]
pub struct PendingUpdate {
    pub table_name: String,
    pub update: CellUpdate,
    pub timestamp: std::time::SystemTime,
}

/// State for a single file's permission tracking.
struct FilePermissionState {
    permission: PermissionState,
    pending_updates: Vec<PendingUpdate>,
}

impl Default for FilePermissionState {
    fn default() -> Self {
        Self { permission: PermissionState::ReadWrite, pending_updates: Vec::new() }
    }
}

/// Manages permission states and pending updates for all watched files.
pub struct PermissionRecoveryState {
    file_states: Mutex<HashMap<PathBuf, FilePermissionState>>,
}

impl Default for PermissionRecoveryState {
    fn default() -> Self {
        Self { file_states: Mutex::new(HashMap::new()) }
    }
}

impl PermissionRecoveryState {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Payload for permission state change events sent to the frontend.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionStateChangedPayload {
    pub file_path: String,
    pub state: PermissionState,
    pub message: String,
    pub pending_update_count: usize,
}

/// Gets the current permission state for a file.
pub fn get_permission_state(app_handle: &AppHandle, file_path: &str) -> PermissionState {
    let Some(state) = app_handle.try_state::<PermissionRecoveryState>() else {
        return PermissionState::ReadWrite;
    };

    let path = PathBuf::from(file_path);
    let states = state.file_states.lock().unwrap_or_else(|e| e.into_inner());
    states.get(&path).map(|s| s.permission).unwrap_or(PermissionState::ReadWrite)
}

/// Sets the permission state for a file and emits an event to the frontend.
pub fn set_permission_state(
    app_handle: &AppHandle,
    file_path: &str,
    new_state: PermissionState,
    message: &str,
) {
    let Some(state) = app_handle.try_state::<PermissionRecoveryState>() else {
        return;
    };

    let path = PathBuf::from(file_path);
    let pending_count: usize;
    let old_state: PermissionState;

    {
        let mut states = state.file_states.lock().unwrap_or_else(|e| e.into_inner());
        let file_state = states.entry(path).or_default();
        old_state = file_state.permission;
        file_state.permission = new_state;
        pending_count = file_state.pending_updates.len();
    }

    if old_state != new_state {
        tracing::info!(
            component = "tv.error.permission",
            file_path = %file_path,
            old_state = %old_state.as_str(),
            new_state = %new_state.as_str(),
            pending_updates = pending_count,
            "Permission state changed"
        );

        emit_permission_state_changed(app_handle, file_path, new_state, message, pending_count);
    }
}

/// Queues a pending update that was rejected due to permission issues.
pub fn queue_pending_update(
    app_handle: &AppHandle,
    file_path: &str,
    table_name: &str,
    update: CellUpdate,
) {
    let Some(state) = app_handle.try_state::<PermissionRecoveryState>() else {
        return;
    };

    let path = PathBuf::from(file_path);
    let pending_update =
        PendingUpdate { table_name: table_name.to_string(), update, timestamp: std::time::SystemTime::now() };

    let pending_count: usize;
    {
        let mut states = state.file_states.lock().unwrap_or_else(|e| e.into_inner());
        let file_state = states.entry(path).or_default();
        file_state.pending_updates.push(pending_update);
        pending_count = file_state.pending_updates.len();
    }

    tracing::debug!(
        component = "tv.error.permission",
        file_path = %file_path,
        table_name = %table_name,
        pending_count = pending_count,
        "Queued pending update"
    );
}

/// Gets the number of pending updates for a file.
pub fn get_pending_update_count(app_handle: &AppHandle, file_path: &str) -> usize {
    let Some(state) = app_handle.try_state::<PermissionRecoveryState>() else {
        return 0;
    };

    let path = PathBuf::from(file_path);
    let states = state.file_states.lock().unwrap_or_else(|e| e.into_inner());
    states.get(&path).map(|s| s.pending_updates.len()).unwrap_or(0)
}

/// Takes all pending updates for a file, clearing the queue.
pub fn take_pending_updates(app_handle: &AppHandle, file_path: &str) -> Vec<PendingUpdate> {
    let Some(state) = app_handle.try_state::<PermissionRecoveryState>() else {
        return Vec::new();
    };

    let path = PathBuf::from(file_path);
    let mut states = state.file_states.lock().unwrap_or_else(|e| e.into_inner());
    states.get_mut(&path).map(|s| std::mem::take(&mut s.pending_updates)).unwrap_or_default()
}

/// Attempts to retry pending updates after permissions are restored.
/// Returns the number of successfully applied updates.
pub fn retry_pending_updates(
    app_handle: &AppHandle,
    file_path: &str,
) -> Result<usize, TvError> {
    let pending = take_pending_updates(app_handle, file_path);
    if pending.is_empty() {
        return Ok(0);
    }

    tracing::info!(
        component = "tv.error.permission",
        file_path = %file_path,
        pending_count = pending.len(),
        "Retrying pending updates after permission restore"
    );

    let mut success_count = 0;
    let mut failed_updates = Vec::new();

    for pending_update in pending {
        let result = crate::toml::document_writer::save_cell(
            file_path,
            &pending_update.table_name,
            &pending_update.update,
        );

        match result {
            Ok(_) => {
                success_count += 1;
                tracing::debug!(
                    component = "tv.error.permission",
                    file_path = %file_path,
                    row = pending_update.update.row_index,
                    column = %pending_update.update.column_key,
                    "Pending update applied successfully"
                );
            }
            Err(e) => {
                tracing::warn!(
                    component = "tv.error.permission",
                    file_path = %file_path,
                    row = pending_update.update.row_index,
                    column = %pending_update.update.column_key,
                    error = %e,
                    "Failed to apply pending update"
                );
                failed_updates.push(pending_update);
            }
        }
    }

    // Re-queue any failed updates
    if !failed_updates.is_empty() {
        let Some(state) = app_handle.try_state::<PermissionRecoveryState>() else {
            return Ok(success_count);
        };

        let path = PathBuf::from(file_path);
        let mut states = state.file_states.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(file_state) = states.get_mut(&path) {
            file_state.pending_updates.extend(failed_updates);
        }
    }

    Ok(success_count)
}

/// Checks if a file is readable by attempting to read its metadata.
pub fn check_file_readable(path: &Path) -> bool {
    std::fs::metadata(path).is_ok()
}

/// Checks if a file is writable by attempting to open it for writing.
pub fn check_file_writable(path: &Path) -> bool {
    std::fs::OpenOptions::new().write(true).append(true).open(path).is_ok()
}

/// Determines the permission state for a file based on filesystem checks.
pub fn detect_permission_state(path: &Path) -> PermissionState {
    if !check_file_readable(path) {
        return PermissionState::Unreadable;
    }
    if !check_file_writable(path) {
        return PermissionState::ReadOnly;
    }
    PermissionState::ReadWrite
}

/// Generates a user-friendly error message for a permission state.
pub fn get_permission_error_message(state: PermissionState, file_path: &str) -> String {
    let file_name =
        Path::new(file_path).file_name().map(|n| n.to_string_lossy()).unwrap_or_default();

    match state {
        PermissionState::ReadWrite => String::new(),
        PermissionState::ReadOnly => {
            format!(
                "Cannot save changes to '{}': file is read-only. Changes will be queued until \
                 write permissions are restored.",
                file_name
            )
        }
        PermissionState::Unreadable => {
            format!(
                "Cannot read '{}': permission denied or file unavailable. The last known data is \
                 preserved. Check file permissions or network connection.",
                file_name
            )
        }
    }
}

fn emit_permission_state_changed(
    app_handle: &AppHandle,
    file_path: &str,
    state: PermissionState,
    message: &str,
    pending_count: usize,
) {
    let payload = PermissionStateChangedPayload {
        file_path: file_path.to_string(),
        state,
        message: message.to_string(),
        pending_update_count: pending_count,
    };

    if let Err(e) = app_handle.emit("permission-state-changed", payload) {
        tracing::error!(
            component = "tv.error.permission",
            file_path = %file_path,
            error = %e,
            "Failed to emit permission state changed event"
        );
    }
}

/// Handles a permission error from a load or save operation.
/// Updates the permission state and returns an appropriate error or message.
pub fn handle_permission_error(
    app_handle: &AppHandle,
    file_path: &str,
    error: &TvError,
) -> PermissionState {
    let new_state = match error {
        TvError::PermissionDenied { operation, .. } => {
            if operation == "read" {
                PermissionState::Unreadable
            } else {
                PermissionState::ReadOnly
            }
        }
        TvError::FileNotFound { .. } => {
            // Treat file not found as unreadable (could be network disconnect)
            PermissionState::Unreadable
        }
        _ => {
            // For other errors, check actual permissions
            detect_permission_state(Path::new(file_path))
        }
    };

    let message = get_permission_error_message(new_state, file_path);
    set_permission_state(app_handle, file_path, new_state, &message);

    new_state
}

/// Clears the permission state for a file (call when file is closed/unwatched).
pub fn clear_permission_state(app_handle: &AppHandle, file_path: &str) {
    let Some(state) = app_handle.try_state::<PermissionRecoveryState>() else {
        return;
    };

    let path = PathBuf::from(file_path);
    let mut states = state.file_states.lock().unwrap_or_else(|e| e.into_inner());
    states.remove(&path);

    tracing::debug!(
        component = "tv.error.permission",
        file_path = %file_path,
        "Permission state cleared"
    );
}
