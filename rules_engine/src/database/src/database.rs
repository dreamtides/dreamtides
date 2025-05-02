use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rusqlite::{Connection, Error};

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
}
