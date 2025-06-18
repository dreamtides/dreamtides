use core_data::identifiers::UserId;

use crate::save_file::SaveFile;

/// Message describing why a database error happened.
pub struct DatabaseError(pub String);

/// Trait for database implementations.
pub trait Database {
    /// Fetches a save file from the database by user ID.
    fn fetch_save(&self, user_id: UserId) -> Result<Option<SaveFile>, DatabaseError>;

    /// Writes a save file to the database.
    fn write_save(&self, save: SaveFile) -> Result<(), DatabaseError>;
}
