use std::collections::HashMap;
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};
use std::time::Instant;

use battle_state::battle::battle_state::RequestContext;
use core_data::identifiers::UserId;
use database::database::{Database, DatabaseError};
use database::sqlite_database::{self, SqliteDatabase};
use state_provider::display_state_provider::{DisplayState, DisplayStateProvider};
use uuid::Uuid;

use crate::engine::PollResult;

/// Trait for injecting stateful dependencies into rules engine code.
pub trait StateProvider:
    Clone + RefUnwindSafe + UnwindSafe + Send + Sync + DisplayStateProvider
{
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

    /// Stores the last response version sent to a user.
    fn store_last_response_version(&self, user_id: UserId, version: Uuid);

    /// Gets the last response version sent to a user.
    fn get_last_response_version(&self, user_id: UserId) -> Option<Uuid>;

    /// Marks that we're starting to process a request for a user.
    fn start_processing(&self, user_id: UserId) -> bool;

    /// Marks that we've finished processing a request for a user.
    fn finish_processing(&self, user_id: UserId);

    /// Returns true if we're currently processing a request for a user.
    fn is_processing(&self, user_id: UserId) -> bool;

    /// Appends a poll result for the given user.
    fn append_poll_result(&self, user_id: UserId, result: PollResult);

    /// Takes the next poll result for the given user, removing it from the
    /// queue.
    fn take_next_poll_result(&self, user_id: UserId) -> Option<PollResult>;

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

static LAST_RESPONSE_VERSIONS: LazyLock<Mutex<HashMap<UserId, Uuid>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static PROCESSING_USERS: LazyLock<Mutex<HashMap<UserId, bool>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static PENDING_UPDATES: LazyLock<Mutex<HashMap<UserId, Vec<PollResult>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static DISPLAY_STATES: LazyLock<Mutex<HashMap<UserId, DisplayState>>> =
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

    fn store_last_response_version(&self, user_id: UserId, version: Uuid) {
        let mut versions = LAST_RESPONSE_VERSIONS.lock().unwrap();
        versions.insert(user_id, version);
    }

    fn get_last_response_version(&self, user_id: UserId) -> Option<Uuid> {
        let versions = LAST_RESPONSE_VERSIONS.lock().unwrap();
        versions.get(&user_id).copied()
    }

    fn start_processing(&self, user_id: UserId) -> bool {
        let mut processing = PROCESSING_USERS.lock().unwrap();
        if processing.get(&user_id).copied().unwrap_or(false) {
            false
        } else {
            processing.insert(user_id, true);
            true
        }
    }

    fn finish_processing(&self, user_id: UserId) {
        let mut processing = PROCESSING_USERS.lock().unwrap();
        processing.insert(user_id, false);
    }

    fn is_processing(&self, user_id: UserId) -> bool {
        let processing = PROCESSING_USERS.lock().unwrap();
        processing.get(&user_id).copied().unwrap_or(false)
    }

    fn append_poll_result(&self, user_id: UserId, result: PollResult) {
        let mut updates = PENDING_UPDATES.lock().unwrap();
        updates.entry(user_id).or_default().push(result);
    }

    fn take_next_poll_result(&self, user_id: UserId) -> Option<PollResult> {
        let mut updates = PENDING_UPDATES.lock().unwrap();
        if let Some(user_updates) = updates.get_mut(&user_id) {
            if !user_updates.is_empty() {
                return Some(user_updates.remove(0));
            }
        }
        None
    }
}

impl DisplayStateProvider for DefaultStateProvider {
    fn get_display_state(&self, user_id: UserId) -> DisplayState {
        let states = DISPLAY_STATES.lock().unwrap();
        states.get(&user_id).cloned().unwrap_or_default()
    }

    fn set_display_state(&self, user_id: UserId, state: DisplayState) {
        let mut states = DISPLAY_STATES.lock().unwrap();
        states.insert(user_id, state);
    }
}
