use std::cell::OnceCell;
use std::fmt::Display;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use core_data::identifiers::UserId;
use rusqlite::{Connection, Error, OptionalExtension};
use serde_json::{self, ser};

use crate::save_file::SaveFile;

static DATABASE_PATH: OnceLock<PathBuf> = OnceLock::new();
thread_local! {
    static DATABASE: OnceCell<Database> = OnceCell::new();
}

/// Sets the database path which will be used by all threads for their database
/// connections, then returns a [Database] connection for the current thread.
pub fn initialize(path: PathBuf) -> Result<Database, DatabaseError> {
    // Try to set the database path
    if let Err(_) = DATABASE_PATH.set(path.clone()) {
        // Path is already set, check if it's the same
        if let Some(existing_path) = DATABASE_PATH.get() {
            if existing_path != &path {
                return Err(DatabaseError(format!(
                    "Database already initialized with a different path: {:?}",
                    existing_path
                )));
            }
        }
    }

    // Create a database for the current thread if needed
    DATABASE.with(|cell| {
        if let Some(db) = cell.get() {
            Ok(db.clone())
        } else {
            let db = Database::new(path)?;
            if let Err(_) = cell.set(db.clone()) {
                return Err(DatabaseError(
                    "Failed to store database in thread-local storage.".to_string(),
                ));
            }
            Ok(db)
        }
    })
}

/// Returns the database connection for the current thread.
///
/// This creates a new thread-local connection if one has not previously been
/// created. It uses the database path set in `initialize` and returns an error
/// if initialization has not happened yet.
pub fn get() -> Result<Database, DatabaseError> {
    DATABASE.with(|cell| {
        if let Some(db) = cell.get() {
            Ok(db.clone())
        } else {
            match DATABASE_PATH.get() {
                Some(path) => {
                    let db = Database::new(path.clone())?;
                    if let Err(_) = cell.set(db.clone()) {
                        return Err(DatabaseError(
                            "Failed to store database in thread-local storage.".to_string(),
                        ));
                    }
                    Ok(db)
                }
                None => Err(DatabaseError(
                    "Database not initialized. Call initialize() first.".to_string(),
                )),
            }
        }
    })
}

/// Message describing why a database error happened.
pub struct DatabaseError(pub String);

impl Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// SQLite database connection.
///
/// This struct is used to fetch data from & mutate the database. It operates as
/// a smart pointer, so calling .clone() is inexpensive and this is the expected
/// way to pass the connection between callers.
#[derive(Debug, Clone)]
pub struct Database {
    connection: Arc<Connection>,
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

        Ok(Self { connection: Arc::new(connection) })
    }

    /// Fetches a save file from the database by user ID.
    pub fn fetch_save(&self, user_id: UserId) -> Result<Option<SaveFile>, DatabaseError> {
        let data: Option<Vec<u8>> = self
            .connection
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

        self.connection
            .execute(
                "INSERT INTO saves (id, data)
                VALUES (?1, ?2)
                ON CONFLICT(id) DO UPDATE SET data = ?2",
                (&save.id().0, &data),
            )
            .map_err(|e| to_database_error(e, &format!("writing save file {:?}", save.id())))?;

        Ok(())
    }
}

/// Converts a rusqlite Error into a human-readable DatabaseError.
///
/// The context parameter provides information about where the error occurred.
fn to_database_error(error: Error, context: &str) -> DatabaseError {
    DatabaseError(format!("Database Error ({}): {}", context, error))
}
