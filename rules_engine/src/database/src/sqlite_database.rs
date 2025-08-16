use std::cell::OnceCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::OnceLock;

use core_data::identifiers::UserId;
use core_data::initialization_error::{ErrorCode, InitializationError};
use rusqlite::{Connection, Error, OptionalExtension};
use serde_json::{self, ser};
use tracing::{debug, instrument};

use crate::database::Database;
use crate::save_file::SaveFile;

static DATABASE_PATH: OnceLock<PathBuf> = OnceLock::new();
thread_local! {
    static DATABASE: OnceCell<SqliteDatabase> = const { OnceCell::new() };
}

/// Sets the database path which will be used by all threads for their database
/// connections, then returns a [Database] connection for the current thread.
pub fn initialize(path: PathBuf) -> Result<SqliteDatabase, Vec<InitializationError>> {
    // Try to set the database path
    if DATABASE_PATH.set(path.clone()).is_err() {
        // Path is already set, check if it's the same
        if let Some(existing_path) = DATABASE_PATH.get() {
            if existing_path != &path {
                return Err(vec![InitializationError::with_details(
                    ErrorCode::AlreadyInitializedWithDifferentPath,
                    "Database already initialized with a different path".to_string(),
                    format!("existing: {existing_path:?}, requested: {path:?}"),
                )]);
            }
        }
    }

    // Create a database for the current thread if needed
    DATABASE.with(|cell| {
        if let Some(db) = cell.get() {
            Ok(db.clone())
        } else {
            match SqliteDatabase::new(path) {
                Ok(db) => {
                    if cell.set(db.clone()).is_err() {
                        Err(vec![InitializationError::with_name(
                            ErrorCode::NotInitializedError,
                            "Failed to store database in thread-local storage.".to_string(),
                        )])
                    } else {
                        Ok(db)
                    }
                }
                Err(e) => Err(e),
            }
        }
    })
}

/// Returns the database connection for the current thread.
///
/// This creates a new thread-local connection if one has not previously been
/// created. It uses the database path set in `initialize` and returns an error
/// if initialization has not happened yet.
#[instrument(name = "get_sqlite_database", level = "debug")]
pub fn get() -> Result<SqliteDatabase, Vec<InitializationError>> {
    DATABASE.with(|cell| {
        if let Some(db) = cell.get() {
            Ok(db.clone())
        } else {
            match DATABASE_PATH.get() {
                Some(path) => match SqliteDatabase::new(path.clone()) {
                    Ok(db) => {
                        if cell.set(db.clone()).is_err() {
                            Err(vec![InitializationError::with_name(
                                ErrorCode::NotInitializedError,
                                "Failed to store database in thread-local storage.".to_string(),
                            )])
                        } else {
                            Ok(db)
                        }
                    }
                    Err(e) => Err(e),
                },
                None => Err(vec![InitializationError::with_name(
                    ErrorCode::NotInitializedError,
                    "Database not initialized. Call initialize() first.".to_string(),
                )]),
            }
        }
    })
}

/// SQLite database connection.
///
/// This struct is used to fetch data from & mutate the database. It operates as
/// a smart pointer, so calling .clone() is inexpensive and this is the expected
/// way to pass the connection between callers.
#[derive(Debug, Clone)]
pub struct SqliteDatabase {
    connection: Rc<Connection>,
}

impl SqliteDatabase {
    pub fn new(directory: PathBuf) -> Result<Self, Vec<InitializationError>> {
        let path = directory.join("saves.sqlite");
        debug!(?path, "Opening new database connection");
        let connection = match Connection::open(path) {
            Ok(c) => c,
            Err(e) => return Err(to_error_vec(e, "opening connection")),
        };

        if let Err(e) = connection.pragma_update(None, "foreign_keys", true) {
            return Err(to_error_vec(e, "setting foreign_keys pragma"));
        }

        if let Err(e) = connection.execute(
            "CREATE TABLE IF NOT EXISTS saves (
                   id BLOB PRIMARY KEY,
                   data BLOB
                ) STRICT;",
            (),
        ) {
            return Err(to_error_vec(e, "creating saves table"));
        }

        Ok(Self { connection: Rc::new(connection) })
    }
}

impl Database for SqliteDatabase {
    #[instrument(name = "fetch_save_file", level = "debug", skip(self))]
    fn fetch_save(&self, user_id: UserId) -> Result<Option<SaveFile>, Vec<InitializationError>> {
        debug!(?user_id, "Fetching save file");
        let data: Option<Vec<u8>> = match self
            .connection
            .query_row("SELECT data FROM saves WHERE id = ?1", [&user_id.0], |row| row.get(0))
            .optional()
        {
            Ok(opt) => opt,
            Err(e) => return Err(to_error_vec(e, &format!("querying save for user {user_id:?}"))),
        };

        match data {
            Some(bytes) => {
                let save = match serde_json::from_slice(&bytes) {
                    Ok(s) => s,
                    Err(e) => {
                        return Err(vec![InitializationError::with_details(
                            ErrorCode::JsonError,
                            "Error deserializing save file".to_string(),
                            format!("user: {user_id:?}, error: {e:?}"),
                        )]);
                    }
                };
                Ok(Some(save))
            }
            None => Ok(None),
        }
    }

    #[instrument(name = "write_save_file", level = "debug", skip(self, save))]
    fn write_save(&self, save: SaveFile) -> Result<(), Vec<InitializationError>> {
        let save_id = save.id();
        debug!(?save_id, "Writing save file to database");
        let data = match ser::to_vec(&save) {
            Ok(v) => v,
            Err(e) => {
                return Err(vec![InitializationError::with_details(
                    ErrorCode::JsonError,
                    "Error serializing save file".to_string(),
                    format!("save_id: {save_id:?}, error: {e:?}"),
                )]);
            }
        };

        if let Err(e) = self.connection.execute(
            "INSERT INTO saves (id, data)
                VALUES (?1, ?2)
                ON CONFLICT(id) DO UPDATE SET data = ?2",
            (&save.id().0, &data),
        ) {
            return Err(to_error_vec(e, &format!("writing save file {save_id:?}")));
        }

        Ok(())
    }
}

fn to_error_vec(error: Error, context: &str) -> Vec<InitializationError> {
    vec![InitializationError::with_details(
        ErrorCode::DatabaseError,
        format!("Database Error ({context})"),
        error.to_string(),
    )]
}
