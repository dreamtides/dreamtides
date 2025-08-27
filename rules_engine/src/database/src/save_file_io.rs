use std::path::{Path, PathBuf};

use core_data::identifiers::UserId;
use core_data::initialization_error::{ErrorCode, InitializationError};
use serde_json;

use crate::save_file::SaveFile;

/// Returns the path to the save file for the given user.
pub fn save_path(dir: &Path, user_id: UserId) -> PathBuf {
    dir.join(format!("save-{}.json", user_id.0))
}

/// Reads a save file from the given directory.
pub fn read_save_from_dir(
    dir: &Path,
    user_id: UserId,
) -> Result<Option<SaveFile>, Vec<InitializationError>> {
    let file_path = save_path(dir, user_id);
    if !file_path.exists() {
        return Ok(None);
    }
    let data = std::fs::read(&file_path).map_err(|e| {
        vec![InitializationError::with_details(
            ErrorCode::IOError,
            "Failed to read save file",
            e.to_string(),
        )]
    })?;
    let save = serde_json::from_slice(&data).map_err(|e| {
        vec![InitializationError::with_details(
            ErrorCode::JsonError,
            "Failed to parse save file",
            e.to_string(),
        )]
    })?;
    Ok(Some(save))
}

/// Writes a save file to the given directory.
pub fn write_save_to_dir(dir: &Path, save: &SaveFile) -> Result<(), Vec<InitializationError>> {
    std::fs::create_dir_all(dir).map_err(|e| {
        vec![InitializationError::with_details(
            ErrorCode::IOError,
            "Failed to create save directory",
            e.to_string(),
        )]
    })?;
    let file_path = save_path(dir, save.id());
    let file = std::fs::File::create(&file_path).map_err(|e| {
        vec![InitializationError::with_details(
            ErrorCode::IOError,
            "Failed to create save file",
            e.to_string(),
        )]
    })?;
    serde_json::to_writer_pretty(file, &save).map_err(|e| {
        vec![InitializationError::with_details(
            ErrorCode::JsonError,
            "Failed to serialize save file",
            e.to_string(),
        )]
    })?;
    Ok(())
}
