use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::error::error_types::TvError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncState {
    Idle,
    Saving,
    Loading,
    Error,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStateChangedPayload {
    pub file_path: String,
    pub state: SyncState,
    pub timestamp: u64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictDetectedPayload {
    pub file_path: String,
    pub message: String,
}

struct FileSaveState {
    is_saving: bool,
    mtime_before_save: Option<SystemTime>,
}

pub struct SaveCoordinatorState {
    file_states: Mutex<HashMap<PathBuf, FileSaveState>>,
}

impl Default for SaveCoordinatorState {
    fn default() -> Self {
        Self { file_states: Mutex::new(HashMap::new()) }
    }
}

impl SaveCoordinatorState {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Checks if a save operation is currently in progress for the given file.
pub fn is_saving(app_handle: &AppHandle, file_path: &str) -> bool {
    let Some(state) = app_handle.try_state::<SaveCoordinatorState>() else {
        return false;
    };

    let path = PathBuf::from(file_path);
    let states = state.file_states.lock().unwrap_or_else(|e| e.into_inner());
    states.get(&path).map(|s| s.is_saving).unwrap_or(false)
}

/// Marks the beginning of a save operation for a file.
pub fn begin_save(app_handle: &AppHandle, file_path: &str) -> Result<(), TvError> {
    let state = app_handle.state::<SaveCoordinatorState>();
    let path = PathBuf::from(file_path);

    let mtime = get_file_mtime(&path);

    {
        let mut states = state.file_states.lock().unwrap_or_else(|e| e.into_inner());
        states.insert(path, FileSaveState { is_saving: true, mtime_before_save: mtime });
    }

    emit_sync_state(app_handle, file_path, SyncState::Saving);

    tracing::debug!(
        component = "tv.sync",
        file_path = %file_path,
        "Save operation started"
    );

    Ok(())
}

/// Marks the completion of a save operation and checks for external modifications.
pub fn end_save(
    app_handle: &AppHandle,
    file_path: &str,
    success: bool,
) -> Result<bool, TvError> {
    let state = app_handle.state::<SaveCoordinatorState>();
    let path = PathBuf::from(file_path);

    let mtime_before = {
        let mut states = state.file_states.lock().unwrap_or_else(|e| e.into_inner());
        let file_state = states.get_mut(&path);
        if let Some(fs) = file_state {
            fs.is_saving = false;
            fs.mtime_before_save
        } else {
            None
        }
    };

    let external_change_detected = if success {
        check_for_external_changes(&path, mtime_before)
    } else {
        false
    };

    let final_state = if !success {
        SyncState::Error
    } else {
        SyncState::Idle
    };

    emit_sync_state(app_handle, file_path, final_state);

    if external_change_detected {
        emit_conflict_detected(
            app_handle,
            file_path,
            "File was modified externally during save. Reload recommended.",
        );
        tracing::warn!(
            component = "tv.sync",
            file_path = %file_path,
            "External modification detected during save window"
        );
    }

    tracing::debug!(
        component = "tv.sync",
        file_path = %file_path,
        success = success,
        external_change = external_change_detected,
        "Save operation completed"
    );

    Ok(external_change_detected)
}

/// Notifies that a load operation is starting.
pub fn begin_load(app_handle: &AppHandle, file_path: &str) {
    emit_sync_state(app_handle, file_path, SyncState::Loading);

    tracing::debug!(
        component = "tv.sync",
        file_path = %file_path,
        "Load operation started"
    );
}

/// Notifies that a load operation has completed.
pub fn end_load(app_handle: &AppHandle, file_path: &str, success: bool) {
    let state = if success { SyncState::Idle } else { SyncState::Error };
    emit_sync_state(app_handle, file_path, state);

    tracing::debug!(
        component = "tv.sync",
        file_path = %file_path,
        success = success,
        "Load operation completed"
    );
}

fn get_file_mtime(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).ok().and_then(|m| m.modified().ok())
}

fn check_for_external_changes(path: &Path, mtime_before: Option<SystemTime>) -> bool {
    let Some(before) = mtime_before else {
        return false;
    };

    let Some(current_mtime) = get_file_mtime(path) else {
        return false;
    };

    current_mtime > before
}

fn emit_sync_state(app_handle: &AppHandle, file_path: &str, state: SyncState) {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let payload = SyncStateChangedPayload {
        file_path: file_path.to_string(),
        state,
        timestamp,
    };

    if let Err(e) = app_handle.emit("sync-state-changed", payload) {
        tracing::error!(
            component = "tv.sync",
            file_path = %file_path,
            error = %e,
            "Failed to emit sync state event"
        );
    }
}

fn emit_conflict_detected(app_handle: &AppHandle, file_path: &str, message: &str) {
    let payload = ConflictDetectedPayload { file_path: file_path.to_string(), message: message.to_string() };

    if let Err(e) = app_handle.emit("sync-conflict-detected", payload) {
        tracing::error!(
            component = "tv.sync",
            file_path = %file_path,
            error = %e,
            "Failed to emit conflict detected event"
        );
    }
}
