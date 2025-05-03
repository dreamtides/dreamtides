use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

use core_data::identifiers::UserId;
use rusqlite::{Connection, Error, OptionalExtension};
use serde_json::{self, ser};

use crate::save_file::SaveFile;

/// Message describing why a database error happened.
pub struct DatabaseError(pub String);

/// Converts a rusqlite Error into a human-readable DatabaseError.
///
/// The context parameter provides information about where the error occurred.
pub fn to_database_error(error: Error, context: &str) -> DatabaseError {
    DatabaseError(format!("Database Error ({}): {}", context, error))
}

/// SQLite database connection.
///
/// This struct is used to fetch data from & mutate the database. It operates as
/// a smart pointer, so calling .clone() is inexpensive and this is the expected
/// way to pass the connection between callers.
#[derive(Clone, Debug)]
pub struct Database {
    pub connection: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(directory: PathBuf) -> Result<Self, DatabaseError> {
        let connection = Connection::open(directory.join("game.sqlite"))
            .map_err(|e| to_database_error(e, "opening connection"))?;

        connection
            .pragma_update(None, "foreign_keys", true)
            .map_err(|e| to_database_error(e, "setting foreign_keys pragma"))?;

        connection
            .execute(
                "CREATE TABLE IF NOT EXISTS saves (
                   id BLOB PRIMARY KEY,
                   data BLOB
                ) STRICT;",
                (),
            )
            .map_err(|e| to_database_error(e, "creating saves table"))?;

        Ok(Self { connection: Arc::new(Mutex::new(connection)) })
    }

    /// Fetches a save file from the database by user ID.
    pub fn fetch_save(&self, user_id: UserId) -> Result<Option<SaveFile>, DatabaseError> {
        let db = self.db()?;

        let data: Option<Vec<u8>> = db
            .query_row("SELECT data FROM saves WHERE id = ?1", [&user_id.0], |row| row.get(0))
            .optional()
            .map_err(|e| to_database_error(e, &format!("querying save for user {:?}", user_id)))?;

        match data {
            Some(bytes) => {
                let save = serde_json::from_slice(&bytes).map_err(|e| {
                    DatabaseError(format!(
                        "Error deserializing save file for user {:?}: {:?}",
                        user_id, e
                    ))
                })?;
                Ok(Some(save))
            }
            None => Ok(None),
        }
    }

    /// Writes a save file to the database.
    pub fn write_save(&self, save: SaveFile) -> Result<(), DatabaseError> {
        let data = ser::to_vec(&save).map_err(|e| {
            DatabaseError(format!("Error serializing save file {:?}: {:?}", save.id(), e))
        })?;

        self.db()?
            .execute(
                "INSERT INTO saves (id, data)
                VALUES (?1, ?2)
                ON CONFLICT(id) DO UPDATE SET data = ?2",
                (&save.id().0, &data),
            )
            .map_err(|e| to_database_error(e, &format!("writing save file {:?}", save.id())))?;

        Ok(())
    }

    fn db(&self) -> Result<MutexGuard<Connection>, DatabaseError> {
        self.connection.lock().map_err(|e| {
            DatabaseError(format!("Error getting database lock, did a writer panic? {:?}", e))
        })
    }
}
