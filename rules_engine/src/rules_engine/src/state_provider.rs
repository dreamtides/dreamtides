use std::collections::HashMap;
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};
use std::time::Instant;

use battle_state::battle::battle_state::RequestContext;
use core_data::identifiers::UserId;
use database::database::{Database, DatabaseError};
use database::sqlite_database::{self, SqliteDatabase};
use uuid::Uuid;

/// Trait for injecting stateful dependencies into rules engine code.
pub trait StateProvider: Clone + RefUnwindSafe + UnwindSafe + Send + Sync {
    type DatabaseImpl: Database;

    /// Initializes the database at the given path.
    fn initialize_database(&self, path: &str) -> Result<Self::DatabaseImpl, DatabaseError>;

    /// Returns a database connection for the current thread.
    fn get_database(&self) -> Result<Self::DatabaseImpl, DatabaseError>;

    /// Stores a request context for the given user ID.
    fn store_request_context(&self, user_id: UserId, context: RequestContext);

    /// Retrieves a request context for the given user ID.
    fn get_request_context(&self, user_id: UserId) -> Option<RequestContext>;

    /// Stores a request timestamp for the given request ID.
    fn store_request_timestamp(&self, request_id: Option<Uuid>, timestamp: Instant);

    /// Retrieves and formats the elapsed time for a request, cleaning up old
    /// entries.
    fn get_elapsed_time_message(&self, request_id: Option<Uuid>) -> String;

    /// Returns true if errors should panic the current test, used in test
    /// environments.
    fn should_panic_on_error(&self) -> bool {
        false
    }
}

static REQUEST_CONTEXTS: LazyLock<Mutex<HashMap<UserId, RequestContext>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static REQUEST_TIMESTAMPS: LazyLock<Mutex<HashMap<Option<Uuid>, Instant>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Clone)]
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

    fn store_request_timestamp(&self, request_id: Option<Uuid>, timestamp: Instant) {
        let mut timestamps = REQUEST_TIMESTAMPS.lock().unwrap();
        timestamps.insert(request_id, timestamp);
    }

    fn get_elapsed_time_message(&self, request_id: Option<Uuid>) -> String {
        if let Ok(mut timestamps) = REQUEST_TIMESTAMPS.lock() {
            let now = Instant::now();
            timestamps.retain(|_, &mut timestamp| now.duration_since(timestamp).as_secs() < 300);

            if let Some(start_time) = timestamps.get(&request_id) {
                format!("{}ms", start_time.elapsed().as_millis())
            } else if request_id.is_some() {
                format!("[unknown request ID: {:?}]", request_id)
            } else {
                "[empty request ID]".to_string()
            }
        } else {
            "[mutex lock failed]".to_string()
        }
    }
}
