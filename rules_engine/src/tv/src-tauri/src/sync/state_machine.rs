use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Mutex;
use std::time::SystemTime;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::error::error_types::TvError;

const STATE_IDLE: u8 = 0;
const STATE_SAVING: u8 = 1;
const STATE_LOADING: u8 = 2;
const STATE_ERROR: u8 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncState {
    Idle,
    Saving,
    Loading,
    Error,
}

impl SyncState {
    fn to_u8(self) -> u8 {
        match self {
            SyncState::Idle => STATE_IDLE,
            SyncState::Saving => STATE_SAVING,
            SyncState::Loading => STATE_LOADING,
            SyncState::Error => STATE_ERROR,
        }
    }

    fn from_u8(value: u8) -> Self {
        match value {
            STATE_IDLE => SyncState::Idle,
            STATE_SAVING => SyncState::Saving,
            STATE_LOADING => SyncState::Loading,
            STATE_ERROR => SyncState::Error,
            _ => SyncState::Error,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            SyncState::Idle => "idle",
            SyncState::Saving => "saving",
            SyncState::Loading => "loading",
            SyncState::Error => "error",
        }
    }
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

struct FileSyncState {
    state: AtomicU8,
    mtime_before_operation: Mutex<Option<SystemTime>>,
}

impl FileSyncState {
    fn new() -> Self {
        Self { state: AtomicU8::new(STATE_IDLE), mtime_before_operation: Mutex::new(None) }
    }

    fn get_state(&self) -> SyncState {
        SyncState::from_u8(self.state.load(Ordering::SeqCst))
    }

    fn try_transition(&self, from: SyncState, to: SyncState) -> bool {
        self.state
            .compare_exchange(from.to_u8(), to.to_u8(), Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }

    fn force_state(&self, state: SyncState) {
        self.state.store(state.to_u8(), Ordering::SeqCst);
    }
}

pub struct SyncStateMachineState {
    file_states: Mutex<HashMap<PathBuf, FileSyncState>>,
}

impl Default for SyncStateMachineState {
    fn default() -> Self {
        Self { file_states: Mutex::new(HashMap::new()) }
    }
}

impl SyncStateMachineState {
    pub fn new() -> Self {
        Self::default()
    }
}

fn get_or_create_file_state<'a>(
    states: &'a mut HashMap<PathBuf, FileSyncState>,
    path: &Path,
) -> &'a FileSyncState {
    states.entry(path.to_path_buf()).or_insert_with(FileSyncState::new)
}

/// Checks if any operation (save or load) is in progress for the file.
pub fn is_busy(app_handle: &AppHandle, file_path: &str) -> bool {
    let Some(state) = app_handle.try_state::<SyncStateMachineState>() else {
        return false;
    };

    let path = PathBuf::from(file_path);
    let states = state.file_states.lock().unwrap_or_else(|e| e.into_inner());
    states
        .get(&path)
        .map(|s| matches!(s.get_state(), SyncState::Saving | SyncState::Loading))
        .unwrap_or(false)
}

/// Attempts to transition to the Saving state from Idle or Error.
pub fn begin_save(app_handle: &AppHandle, file_path: &str) -> Result<(), TvError> {
    let state = app_handle.state::<SyncStateMachineState>();
    let path = PathBuf::from(file_path);

    let mut states = state.file_states.lock().unwrap_or_else(|e| e.into_inner());
    let file_state = get_or_create_file_state(&mut states, &path);
    let current = file_state.get_state();

    let can_transition = matches!(current, SyncState::Idle | SyncState::Error);
    if !can_transition {
        return Err(TvError::InvalidStateTransition {
            file_path: file_path.to_string(),
            from_state: current.as_str().to_string(),
            to_state: SyncState::Saving.as_str().to_string(),
        });
    }

    if !file_state.try_transition(current, SyncState::Saving) {
        let actual = file_state.get_state();
        return Err(TvError::InvalidStateTransition {
            file_path: file_path.to_string(),
            from_state: actual.as_str().to_string(),
            to_state: SyncState::Saving.as_str().to_string(),
        });
    }

    let mtime = get_file_mtime(&path);
    *file_state.mtime_before_operation.lock().unwrap_or_else(|e| e.into_inner()) = mtime;

    drop(states);

    emit_sync_state(app_handle, file_path, SyncState::Saving);

    tracing::debug!(
        component = "tv.sync.state_machine",
        file_path = %file_path,
        from_state = %current.as_str(),
        "Transitioned to Saving state"
    );

    Ok(())
}

/// Completes a save operation, transitioning to Idle or Error based on success.
pub fn end_save(
    app_handle: &AppHandle,
    file_path: &str,
    success: bool,
) -> Result<bool, TvError> {
    let state = app_handle.state::<SyncStateMachineState>();
    let path = PathBuf::from(file_path);

    let external_change_detected: bool;
    let final_state: SyncState;

    {
        let states = state.file_states.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(file_state) = states.get(&path) {
            let mtime_before =
                *file_state.mtime_before_operation.lock().unwrap_or_else(|e| e.into_inner());

            external_change_detected =
                if success { check_for_external_changes(&path, mtime_before) } else { false };

            final_state = if success { SyncState::Idle } else { SyncState::Error };

            file_state.force_state(final_state);
        } else {
            external_change_detected = false;
            final_state = SyncState::Idle;
        }
    }

    emit_sync_state(app_handle, file_path, final_state);

    if external_change_detected {
        emit_conflict_detected(
            app_handle,
            file_path,
            "File was modified externally during save. Reload recommended.",
        );
        tracing::warn!(
            component = "tv.sync.state_machine",
            file_path = %file_path,
            "External modification detected during save window"
        );
    }

    tracing::debug!(
        component = "tv.sync.state_machine",
        file_path = %file_path,
        success = success,
        external_change = external_change_detected,
        final_state = %final_state.as_str(),
        "Save operation completed"
    );

    Ok(external_change_detected)
}

/// Attempts to transition to the Loading state from Idle or Error.
pub fn begin_load(app_handle: &AppHandle, file_path: &str) -> Result<(), TvError> {
    let state = app_handle.state::<SyncStateMachineState>();
    let path = PathBuf::from(file_path);

    let mut states = state.file_states.lock().unwrap_or_else(|e| e.into_inner());
    let file_state = get_or_create_file_state(&mut states, &path);
    let current = file_state.get_state();

    let can_transition = matches!(current, SyncState::Idle | SyncState::Error);
    if !can_transition {
        return Err(TvError::InvalidStateTransition {
            file_path: file_path.to_string(),
            from_state: current.as_str().to_string(),
            to_state: SyncState::Loading.as_str().to_string(),
        });
    }

    if !file_state.try_transition(current, SyncState::Loading) {
        let actual = file_state.get_state();
        return Err(TvError::InvalidStateTransition {
            file_path: file_path.to_string(),
            from_state: actual.as_str().to_string(),
            to_state: SyncState::Loading.as_str().to_string(),
        });
    }

    drop(states);

    emit_sync_state(app_handle, file_path, SyncState::Loading);

    tracing::debug!(
        component = "tv.sync.state_machine",
        file_path = %file_path,
        from_state = %current.as_str(),
        "Transitioned to Loading state"
    );

    Ok(())
}

/// Completes a load operation, transitioning to Idle or Error based on success.
pub fn end_load(app_handle: &AppHandle, file_path: &str, success: bool) {
    let state = app_handle.state::<SyncStateMachineState>();
    let path = PathBuf::from(file_path);

    let final_state = if success { SyncState::Idle } else { SyncState::Error };

    {
        let states = state.file_states.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(file_state) = states.get(&path) {
            file_state.force_state(final_state);
        }
    }

    emit_sync_state(app_handle, file_path, final_state);

    tracing::debug!(
        component = "tv.sync.state_machine",
        file_path = %file_path,
        success = success,
        final_state = %final_state.as_str(),
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

    let payload = SyncStateChangedPayload { file_path: file_path.to_string(), state, timestamp };

    if let Err(e) = app_handle.emit("sync-state-changed", payload) {
        tracing::error!(
            component = "tv.sync.state_machine",
            file_path = %file_path,
            error = %e,
            "Failed to emit sync state event"
        );
    }
}

fn emit_conflict_detected(app_handle: &AppHandle, file_path: &str, message: &str) {
    let payload =
        ConflictDetectedPayload { file_path: file_path.to_string(), message: message.to_string() };

    if let Err(e) = app_handle.emit("sync-conflict-detected", payload) {
        tracing::error!(
            component = "tv.sync.state_machine",
            file_path = %file_path,
            error = %e,
            "Failed to emit conflict detected event"
        );
    }
}
