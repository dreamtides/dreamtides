use std::panic::{RefUnwindSafe, UnwindSafe};
use std::path::PathBuf;

use database::database::{Database, DatabaseError};
use database::sqlite_database::{self, SqliteDatabase};

/// Trait for injecting dependencies into rules engine code.
pub trait StateProvider: RefUnwindSafe + UnwindSafe + Send + Sync {
    type DatabaseImpl: Database;

    /// Initializes the database at the given path.
    fn initialize_database(&self, path: &str) -> Result<Self::DatabaseImpl, DatabaseError>;

    /// Returns a database connection for the current thread.
    fn get_database(&self) -> Result<Self::DatabaseImpl, DatabaseError>;
}

pub struct DefaultStateProvider;

impl StateProvider for DefaultStateProvider {
    type DatabaseImpl = SqliteDatabase;

    fn initialize_database(&self, path: &str) -> Result<Self::DatabaseImpl, DatabaseError> {
        sqlite_database::initialize(PathBuf::from(path))
    }

    fn get_database(&self) -> Result<Self::DatabaseImpl, DatabaseError> {
        sqlite_database::get()
    }
}
