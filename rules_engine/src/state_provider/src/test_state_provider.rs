use std::collections::HashMap;
use std::fs::File;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;

use battle_state::battle::battle_state::RequestContext;
use core_data::identifiers::UserId;
use core_data::initialization_error::{ErrorCode, InitializationError};
use serde_json;
use tabula_data::localized_strings::LanguageId;
use tabula_data::tabula::{self, Tabula, TabulaBuildContext, TabulaRaw};
use uuid::Uuid;

use crate::display_state_provider::{DisplayState, DisplayStateProvider};
use crate::state_provider::{PollResult, StateProvider};
use crate::test_database::TestDatabase;

#[derive(Clone)]
pub struct TestStateProvider {
    inner: Arc<TestStateProviderInner>,
}

struct TestStateProviderInner {
    databases: Mutex<HashMap<String, TestDatabase>>,
    request_contexts: Mutex<HashMap<UserId, RequestContext>>,
    request_timestamps: Mutex<HashMap<Option<Uuid>, Instant>>,
    last_response_versions: Mutex<HashMap<UserId, Uuid>>,
    processing_users: Mutex<HashMap<UserId, bool>>,
    pending_updates: Mutex<HashMap<UserId, Vec<PollResult>>>,
    display_states: Mutex<HashMap<UserId, DisplayState>>,
    tabula: RwLock<Option<Arc<Tabula>>>,
}

impl TestStateProvider {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(TestStateProviderInner {
                databases: Mutex::new(HashMap::new()),
                request_contexts: Mutex::new(HashMap::new()),
                request_timestamps: Mutex::new(HashMap::new()),
                last_response_versions: Mutex::new(HashMap::new()),
                processing_users: Mutex::new(HashMap::new()),
                pending_updates: Mutex::new(HashMap::new()),
                display_states: Mutex::new(HashMap::new()),
                tabula: RwLock::new(None),
            }),
        }
    }
}

impl Default for TestStateProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl StateProvider for TestStateProvider {
    type DatabaseImpl = TestDatabase;

    fn initialize(
        &self,
        persistent_data_path: &str,
        streaming_assets_path: &str,
    ) -> Result<Self::DatabaseImpl, Vec<InitializationError>> {
        let tabula_path = format!("{streaming_assets_path}/tabula.json");
        let ctx = TabulaBuildContext { current_language: LanguageId::EnglishUnitedStates };
        let tabula = match File::open(&tabula_path) {
            Ok(file) => {
                let raw: TabulaRaw = serde_json::from_reader(file).map_err(|e| {
                    vec![InitializationError::with_details(
                        ErrorCode::JsonError,
                        "Failed to parse tabula.json",
                        e.to_string(),
                    )]
                })?;
                tabula::build(&ctx, &raw)?
            }
            Err(_) => {
                let file = File::open("tabula.json").map_err(|e| {
                    vec![InitializationError::with_details(
                        ErrorCode::IOError,
                        "Failed to open tabula.json",
                        e.to_string(),
                    )]
                })?;
                let raw: TabulaRaw = serde_json::from_reader(file).map_err(|e| {
                    vec![InitializationError::with_details(
                        ErrorCode::JsonError,
                        "Failed to parse tabula.json",
                        e.to_string(),
                    )]
                })?;
                tabula::build(&ctx, &raw)?
            }
        };

        if let Ok(mut guard) = self.inner.tabula.write() {
            *guard = Some(Arc::new(tabula));
        }

        let mut databases = self.inner.databases.lock().map_err(|e| {
            vec![InitializationError::with_details(
                ErrorCode::MutexLockError,
                "Failed to acquire lock".to_string(),
                e.to_string(),
            )]
        })?;

        if let Some(existing_db) = databases.get(persistent_data_path) {
            Ok(existing_db.clone())
        } else {
            let db = TestDatabase::new();
            databases.insert(persistent_data_path.to_string(), db.clone());
            Ok(databases.get(persistent_data_path).unwrap().clone())
        }
    }

    fn get_database(&self) -> Result<Self::DatabaseImpl, Vec<InitializationError>> {
        let databases = self.inner.databases.lock().map_err(|e| {
            vec![InitializationError::with_details(
                ErrorCode::MutexLockError,
                "Failed to acquire lock".to_string(),
                e.to_string(),
            )]
        })?;
        databases.values().next().cloned().ok_or_else(|| {
            vec![InitializationError::with_name(
                ErrorCode::NotInitializedError,
                "No database initialized".to_string(),
            )]
        })
    }

    fn store_request_context(&self, user_id: UserId, context: RequestContext) {
        if let Ok(mut contexts) = self.inner.request_contexts.lock() {
            contexts.insert(user_id, context);
        }
    }

    fn get_request_context(&self, user_id: UserId) -> Option<RequestContext> {
        if let Ok(contexts) = self.inner.request_contexts.lock() {
            contexts.get(&user_id).cloned()
        } else {
            None
        }
    }

    fn store_request_timestamp(&self, request_id: Option<Uuid>, timestamp: Instant) {
        if let Ok(mut timestamps) = self.inner.request_timestamps.lock() {
            timestamps.insert(request_id, timestamp);
        }
    }

    fn get_elapsed_time_message(&self, request_id: Option<Uuid>) -> String {
        if let Ok(mut timestamps) = self.inner.request_timestamps.lock() {
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
        if let Ok(mut versions) = self.inner.last_response_versions.lock() {
            versions.insert(user_id, version);
        }
    }

    fn get_last_response_version(&self, user_id: UserId) -> Option<Uuid> {
        if let Ok(versions) = self.inner.last_response_versions.lock() {
            versions.get(&user_id).copied()
        } else {
            None
        }
    }

    fn start_processing(&self, user_id: UserId) -> bool {
        if let Ok(mut processing) = self.inner.processing_users.lock() {
            if processing.get(&user_id).copied().unwrap_or(false) {
                false
            } else {
                processing.insert(user_id, true);
                true
            }
        } else {
            false
        }
    }

    fn finish_processing(&self, user_id: UserId) {
        if let Ok(mut processing) = self.inner.processing_users.lock() {
            processing.insert(user_id, false);
        }
    }

    fn is_processing(&self, user_id: UserId) -> bool {
        if let Ok(processing) = self.inner.processing_users.lock() {
            processing.get(&user_id).copied().unwrap_or(false)
        } else {
            false
        }
    }

    fn append_poll_result(&self, user_id: UserId, result: PollResult) {
        if let Ok(mut updates) = self.inner.pending_updates.lock() {
            updates.entry(user_id).or_default().push(result);
        }
    }

    fn take_next_poll_result(&self, user_id: UserId) -> Option<PollResult> {
        if let Ok(mut updates) = self.inner.pending_updates.lock() {
            if let Some(user_updates) = updates.get_mut(&user_id) {
                if !user_updates.is_empty() {
                    return Some(user_updates.remove(0));
                }
            }
        }
        None
    }

    fn should_panic_on_error(&self) -> bool {
        true
    }
}

impl DisplayStateProvider for TestStateProvider {
    fn get_display_state(&self, user_id: UserId) -> DisplayState {
        if let Ok(states) = self.inner.display_states.lock() {
            states.get(&user_id).cloned().unwrap_or_default()
        } else {
            DisplayState::default()
        }
    }

    fn set_display_state(&self, user_id: UserId, state: DisplayState) {
        if let Ok(mut states) = self.inner.display_states.lock() {
            states.insert(user_id, state);
        }
    }

    fn tabula(&self) -> Arc<Tabula> {
        if let Ok(tabula) = self.inner.tabula.read() {
            tabula.clone().unwrap_or_else(|| panic!("Tabula not initialized"))
        } else {
            panic!("Tabula not initialized")
        }
    }
}
