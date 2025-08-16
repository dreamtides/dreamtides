use core_data::identifiers::UserId;
use core_data::initialization_error::InitializationError;

use crate::save_file::SaveFile;

/// Trait for database implementations.
pub trait Database {
    /// Fetches a save file from the database by user ID.
    fn fetch_save(&self, user_id: UserId) -> Result<Option<SaveFile>, Vec<InitializationError>>;

    /// Writes a save file to the database.
    fn write_save(&self, save: SaveFile) -> Result<(), Vec<InitializationError>>;
}
