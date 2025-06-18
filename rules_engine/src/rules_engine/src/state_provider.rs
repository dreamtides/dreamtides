use std::collections::HashMap;
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

use battle_state::battle::battle_state::RequestContext;
use core_data::identifiers::UserId;
use database::database::{Database, DatabaseError};
use database::sqlite_database::{self, SqliteDatabase};

/// Trait for injecting dependencies into rules engine code.
pub trait StateProvider: RefUnwindSafe + UnwindSafe + Send + Sync {
    type DatabaseImpl: Database;

    /// Initializes the database at the given path.
    fn initialize_database(&self, path: &str) -> Result<Self::DatabaseImpl, DatabaseError>;

    /// Returns a database connection for the current thread.
    fn get_database(&self) -> Result<Self::DatabaseImpl, DatabaseError>;

    /// Stores a request context for the given user ID.
    fn store_request_context(&self, user_id: UserId, context: RequestContext);

    /// Retrieves a request context for the given user ID.
    fn get_request_context(&self, user_id: UserId) -> Option<RequestContext>;
}

static REQUEST_CONTEXTS: LazyLock<Mutex<HashMap<UserId, RequestContext>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub struct DefaultStateProvider;

impl StateProvider for DefaultStateProvider {
    type DatabaseImpl = SqliteDatabase;

    fn initialize_database(&self, path: &str) -> Result<Self::DatabaseImpl, DatabaseError> {
        sqlite_database::initialize(PathBuf::from(path))
    }

    fn get_database(&self) -> Result<Self::DatabaseImpl, DatabaseError> {
        sqlite_database::get()
    }

    fn store_request_context(&self, user_id: UserId, context: RequestContext) {
        let mut contexts = REQUEST_CONTEXTS.lock().unwrap();
        contexts.insert(user_id, context);
    }

    fn get_request_context(&self, user_id: UserId) -> Option<RequestContext> {
        let contexts = REQUEST_CONTEXTS.lock().unwrap();
        contexts.get(&user_id).cloned()
    }
}
