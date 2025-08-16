use std::collections::HashMap;
use std::fs::File;
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, Mutex, RwLock};
use std::time::Instant;

use battle_state::battle::battle_state::RequestContext;
use core_data::identifiers::UserId;
use database::database::{Database, DatabaseError};
use database::sqlite_database::{self, SqliteDatabase};
use display_data::command::CommandSequence;
use display_data::request_data::{PollResponseType, RequestId};
use serde_json;
use tabula_data::localized_strings::LanguageId;
use tabula_data::tabula::{self, Tabula, TabulaBuildContext, TabulaRaw};
use uuid::Uuid;

use crate::display_state_provider::{DisplayState, DisplayStateProvider};

#[derive(Debug, Clone)]
pub struct PollResult {
    pub commands: CommandSequence,
    pub request_id: Option<RequestId>,
    pub response_type: PollResponseType,
}

pub trait StateProvider:
    Clone + RefUnwindSafe + UnwindSafe + Send + Sync + DisplayStateProvider
{
    type DatabaseImpl: Database;

    fn initialize(
        &self,
        persistent_data_path: &str,
        streaming_assets_path: &str,
    ) -> Result<Self::DatabaseImpl, DatabaseError>;

    fn get_database(&self) -> Result<Self::DatabaseImpl, DatabaseError>;

    fn store_request_context(&self, user_id: UserId, context: RequestContext);

    fn get_request_context(&self, user_id: UserId) -> Option<RequestContext>;

    fn store_request_timestamp(&self, request_id: Option<Uuid>, timestamp: Instant);

    fn get_elapsed_time_message(&self, request_id: Option<Uuid>) -> String;

    fn store_last_response_version(&self, user_id: UserId, version: Uuid);

    fn get_last_response_version(&self, user_id: UserId) -> Option<Uuid>;

    fn start_processing(&self, user_id: UserId) -> bool;

    fn finish_processing(&self, user_id: UserId);

    fn is_processing(&self, user_id: UserId) -> bool;

    fn append_poll_result(&self, user_id: UserId, result: PollResult);

    fn take_next_poll_result(&self, user_id: UserId) -> Option<PollResult>;

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

static TABULA_DATA: LazyLock<RwLock<Option<Arc<Tabula>>>> = LazyLock::new(|| RwLock::new(None));

#[derive(Clone)]
pub struct DefaultStateProvider;

impl StateProvider for DefaultStateProvider {
    type DatabaseImpl = SqliteDatabase;

    fn initialize(
        &self,
        persistent_data_path: &str,
        streaming_assets_path: &str,
    ) -> Result<Self::DatabaseImpl, DatabaseError> {
        let db = sqlite_database::initialize(PathBuf::from(persistent_data_path))?;
        let tabula_path = format!("{streaming_assets_path}/tabula.json");
        let ctx = TabulaBuildContext { current_language: LanguageId::EnglishUnitedStates };
        let tabula = File::open(&tabula_path)
            .ok()
            .and_then(|file| serde_json::from_reader::<_, TabulaRaw>(file).ok())
            .map(|raw| tabula::build(&ctx, &raw))
            .ok_or(DatabaseError("Failed to load tabula.json".to_string()))?;
        let mut guard = TABULA_DATA.write().unwrap();
        *guard = Some(Arc::new(tabula));
        Ok(db)
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
                format!("[unknown request ID: {request_id:?}]")
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

    fn tabula(&self) -> Arc<Tabula> {
        let guard = TABULA_DATA.read().unwrap();
        guard.clone().unwrap_or_else(|| panic!("Tabula not initialized"))
    }
}
