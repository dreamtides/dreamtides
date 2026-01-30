use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use std::time::Instant;

use battle_state::battle::battle_state::{BattleState, RequestContext};
use core_data::identifiers::{BattleId, UserId};
use core_data::initialization_error::{ErrorCode, InitializationError};
use core_data::types::PlayerName;
use database::save_file::SaveFile;
use tabula_data::tabula::{Tabula, TabulaSource};
use uuid::Uuid;

use crate::display_state_provider::{DisplayState, DisplayStateProvider};
use crate::state_provider::{PollResult, SpeculativeSearchState, StateProvider};

#[derive(Clone)]
pub struct TestStateProvider {
    inner: Arc<TestStateProviderInner>,
}

struct TestStateProviderInner {
    save_files: Mutex<HashMap<UserId, SaveFile>>,
    request_contexts: Mutex<HashMap<UserId, RequestContext>>,
    request_timestamps: Mutex<HashMap<Option<Uuid>, Instant>>,
    last_response_versions: Mutex<HashMap<UserId, Uuid>>,
    processing_users: Mutex<HashMap<UserId, bool>>,
    pending_updates: Mutex<HashMap<UserId, Vec<PollResult>>>,
    display_states: Mutex<HashMap<UserId, DisplayState>>,
    tabula: RwLock<Option<Arc<Tabula>>>,
    undo_stacks: Mutex<HashMap<BattleId, Vec<(PlayerName, BattleState)>>>,
    speculative_searches: Mutex<HashMap<BattleId, SpeculativeSearchState>>,
}

impl TestStateProvider {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(TestStateProviderInner {
                save_files: Mutex::new(HashMap::new()),
                request_contexts: Mutex::new(HashMap::new()),
                request_timestamps: Mutex::new(HashMap::new()),
                last_response_versions: Mutex::new(HashMap::new()),
                processing_users: Mutex::new(HashMap::new()),
                pending_updates: Mutex::new(HashMap::new()),
                display_states: Mutex::new(HashMap::new()),
                undo_stacks: Mutex::new(HashMap::new()),
                speculative_searches: Mutex::new(HashMap::new()),
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
    fn initialize(
        &self,
        _persistent_data_path: &str,
        streaming_assets_path: &str,
    ) -> Result<(), Vec<InitializationError>> {
        // Cache tabula value between tests to improve test performance
        // substantially.
        static GLOBAL_TABULA: OnceLock<Arc<Tabula>> = OnceLock::new();
        if let Some(global) = GLOBAL_TABULA.get() {
            if let Ok(mut guard) = self.inner.tabula.write() {
                *guard = Some(global.clone());
            }
            return Ok(());
        }
        let load_tabula = |dir: &Path| -> Result<Tabula, Vec<InitializationError>> {
            Tabula::load(TabulaSource::Test, dir).map_err(|errors| {
                errors
                    .into_iter()
                    .map(|e| {
                        InitializationError::with_details(
                            ErrorCode::IOError,
                            "Tabula loading error",
                            format!("{e}"),
                        )
                    })
                    .collect()
            })
        };
        let tabula_dir = Path::new(streaming_assets_path).join("Tabula");
        let built = match load_tabula(&tabula_dir) {
            Ok(t) => t,
            Err(_) => load_tabula(Path::new("tabula"))?,
        };
        let arc = GLOBAL_TABULA.get_or_init(|| Arc::new(built));
        if let Ok(mut guard) = self.inner.tabula.write() {
            *guard = Some(arc.clone());
        }
        Ok(())
    }

    fn read_save_file(
        &self,
        user_id: UserId,
    ) -> Result<Option<SaveFile>, Vec<InitializationError>> {
        Ok(self
            .inner
            .save_files
            .lock()
            .map_err(|e| {
                vec![InitializationError::with_details(
                    ErrorCode::MutexLockError,
                    "Failed to acquire lock".to_string(),
                    e.to_string(),
                )]
            })?
            .get(&user_id)
            .cloned())
    }

    fn write_save_file(&self, save: SaveFile) -> Result<(), Vec<InitializationError>> {
        self.inner
            .save_files
            .lock()
            .map_err(|e| {
                vec![InitializationError::with_details(
                    ErrorCode::MutexLockError,
                    "Failed to acquire lock".to_string(),
                    e.to_string(),
                )]
            })?
            .insert(save.id(), save);
        Ok(())
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
        if let Ok(mut updates) = self.inner.pending_updates.lock()
            && let Some(user_updates) = updates.get_mut(&user_id)
            && !user_updates.is_empty()
        {
            return Some(user_updates.remove(0));
        }
        None
    }

    fn set_speculative_search(&self, battle_id: BattleId, search: SpeculativeSearchState) {
        if let Ok(mut searches) = self.inner.speculative_searches.lock() {
            searches.insert(battle_id, search);
        }
    }

    fn take_speculative_search(&self, battle_id: BattleId) -> Option<SpeculativeSearchState> {
        if let Ok(mut searches) = self.inner.speculative_searches.lock() {
            searches.remove(&battle_id)
        } else {
            None
        }
    }

    fn push_undo_entry(&self, battle_id: BattleId, player: PlayerName, state: BattleState) {
        if let Ok(mut stacks) = self.inner.undo_stacks.lock() {
            stacks.entry(battle_id).or_default().push((player, state));
        }
    }

    fn pop_undo_entry(&self, battle_id: BattleId, player: PlayerName) -> Option<BattleState> {
        if let Ok(mut stacks) = self.inner.undo_stacks.lock()
            && let Some(stack) = stacks.get_mut(&battle_id)
        {
            for i in (0..stack.len()).rev() {
                if stack[i].0 == player {
                    let entry = stack.remove(i);
                    return Some(entry.1);
                }
            }
        }
        None
    }

    fn clear_undo_stack(&self, battle_id: BattleId) {
        if let Ok(mut stacks) = self.inner.undo_stacks.lock() {
            stacks.remove(&battle_id);
        }
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

    fn can_undo(&self, battle_id: BattleId, player: PlayerName) -> bool {
        let stacks = self.inner.undo_stacks.lock().unwrap();
        stacks
            .get(&battle_id)
            .map(|stack| stack.iter().any(|(entry_player, _)| *entry_player == player))
            .unwrap_or(false)
    }
}
