use std::collections::HashMap;
use std::fs::File;
use std::io::{Cursor, Read};
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, LazyLock, Mutex, RwLock};
use std::time::Instant;

use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::{BattleState, RequestContext};
use core_data::identifiers::{BattleId, UserId};
use core_data::initialization_error::{ErrorCode, InitializationError};
use core_data::types::PlayerName;
use database::save_file::SaveFile;
use database::save_file_io;
use display_data::command::CommandSequence;
use display_data::request_data::{PollResponseType, RequestId};
use serde_json;
use tabula_data::localized_strings::LanguageId;
use tabula_data::tabula::{self, Tabula, TabulaBuildContext, TabulaRaw};
use tracing::instrument;
use uuid::Uuid;
use zip::ZipArchive;

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
    fn initialize(
        &self,
        persistent_data_path: &str,
        streaming_assets_path: &str,
    ) -> Result<(), Vec<InitializationError>>;

    fn read_save_file(&self, user_id: UserId)
    -> Result<Option<SaveFile>, Vec<InitializationError>>;

    fn write_save_file(&self, save: SaveFile) -> Result<(), Vec<InitializationError>>;

    fn store_request_context(&self, user_id: UserId, context: RequestContext);

    fn get_request_context(&self, user_id: UserId) -> Option<RequestContext>;

    fn store_request_timestamp(&self, request_id: Option<Uuid>, timestamp: Instant);

    fn get_elapsed_time_message(&self, request_id: Option<Uuid>) -> String;

    fn store_last_response_version(&self, user_id: UserId, version: Uuid);

    fn get_last_response_version(&self, user_id: UserId) -> Option<Uuid>;

    fn start_processing(&self, user_id: UserId) -> bool;

    fn finish_processing(&self, user_id: UserId);

    fn push_undo_entry(&self, battle_id: BattleId, player: PlayerName, state: BattleState);

    fn pop_undo_entry(&self, battle_id: BattleId, player: PlayerName) -> Option<BattleState>;

    fn clear_undo_stack(&self, battle_id: BattleId);

    fn is_processing(&self, user_id: UserId) -> bool;

    fn append_poll_result(&self, user_id: UserId, result: PollResult);

    fn take_next_poll_result(&self, user_id: UserId) -> Option<PollResult>;

    fn set_speculative_search(&self, battle_id: BattleId, search: SpeculativeSearchState);

    fn take_speculative_search(&self, battle_id: BattleId) -> Option<SpeculativeSearchState>;

    fn should_panic_on_error(&self) -> bool {
        false
    }

    fn stored_initialization_error(&self) -> Option<String> {
        None
    }

    fn set_initialization_error(&self, _error: String) {}
}

static REQUEST_CONTEXTS: LazyLock<Mutex<HashMap<UserId, RequestContext>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static REQUEST_TIMESTAMPS: LazyLock<Mutex<HashMap<Option<Uuid>, Instant>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static LAST_RESPONSE_VERSIONS: LazyLock<Mutex<HashMap<UserId, Uuid>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static INITIALIZATION_ERROR: LazyLock<Mutex<Option<String>>> = LazyLock::new(|| Mutex::new(None));

#[derive(Clone)]
struct UndoEntry {
    player: PlayerName,
    state: BattleState,
}

static UNDO_STACKS: LazyLock<Mutex<HashMap<BattleId, Vec<UndoEntry>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static PROCESSING_USERS: LazyLock<Mutex<HashMap<UserId, bool>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static PENDING_UPDATES: LazyLock<Mutex<HashMap<UserId, Vec<PollResult>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// Fast path counter for total pending poll results across all users. This lets
// callers check for any pending updates with a single atomic read instead of
// locking the PENDING_UPDATES map. Maintained in append_poll_result /
// take_next_poll_result.
static TOTAL_PENDING_UPDATES: LazyLock<AtomicUsize> = LazyLock::new(|| AtomicUsize::new(0));

static DISPLAY_STATES: LazyLock<Mutex<HashMap<UserId, DisplayState>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Clone)]
pub struct SpeculativeSearchState {
    pub assumed_action: BattleAction,
    pub result: Arc<(Mutex<Option<BattleAction>>, Condvar)>,
}

static SPECULATIVE_SEARCHES: LazyLock<Mutex<HashMap<BattleId, SpeculativeSearchState>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static TABULA_DATA: LazyLock<RwLock<Option<Arc<Tabula>>>> = LazyLock::new(|| RwLock::new(None));

static PERSISTENT_DATA_DIR: LazyLock<Mutex<Option<PathBuf>>> = LazyLock::new(|| Mutex::new(None));

#[derive(Clone)]
pub struct DefaultStateProvider;

impl DefaultStateProvider {
    // Fast check used by the rules engine to determine if any poll results are
    // pending.
    pub fn has_pending_updates(&self) -> bool {
        TOTAL_PENDING_UPDATES.load(Ordering::Acquire) > 0
    }
}

/// Loads tabula.json from an APK jar style URL on Android.
///
/// Example: jar:file:///.../base.apk!/assets/tabula.json
///
/// This is almostly certainly the wrong way to do this. I hope one day to learn
/// the correct method of fetching resources on android.
fn load_tabula_raw_android(jar_url: &str) -> Result<TabulaRaw, Vec<InitializationError>> {
    let without_prefix = jar_url.strip_prefix("jar:file:").ok_or_else(|| {
        vec![InitializationError::with_details(
            ErrorCode::IOError,
            "Android jar URL missing jar:file: prefix",
            jar_url.to_string(),
        )]
    })?;
    let bang_index = without_prefix.find("!/").ok_or_else(|| {
        vec![InitializationError::with_details(
            ErrorCode::IOError,
            "Malformed Android jar URL for tabula.json",
            jar_url.to_string(),
        )]
    })?;
    let (apk_path, entry_path_with_slash) = without_prefix.split_at(bang_index);
    let entry_path = &entry_path_with_slash[2..];
    let mut file = File::open(apk_path).map_err(|e| {
        vec![InitializationError::with_details(
            ErrorCode::IOError,
            "Failed to open base.apk while reading tabula.json",
            e.to_string(),
        )]
    })?;
    let mut zip = ZipArchive::new(&mut file).map_err(|e| {
        vec![InitializationError::with_details(
            ErrorCode::IOError,
            "Failed to read APK zip while reading tabula.json",
            e.to_string(),
        )]
    })?;
    let mut zip_file = zip.by_name(entry_path).map_err(|e| {
        vec![InitializationError::with_details(
            ErrorCode::IOError,
            "tabula.json not found inside APK",
            e.to_string(),
        )]
    })?;
    let mut buf = Vec::new();
    zip_file.read_to_end(&mut buf).map_err(|e| {
        vec![InitializationError::with_details(
            ErrorCode::IOError,
            "Failed to read tabula.json from APK",
            e.to_string(),
        )]
    })?;
    serde_json::from_reader(Cursor::new(buf)).map_err(|e| {
        vec![InitializationError::with_details(
            ErrorCode::JsonError,
            "Failed to parse tabula.json",
            e.to_string(),
        )]
    })
}

/// Loads tabula.json directly from the filesystem (non-Android platforms)
fn load_tabula_raw_filesystem(path: &str) -> Result<TabulaRaw, Vec<InitializationError>> {
    let file = File::open(path).map_err(|e| {
        vec![InitializationError::with_details(
            ErrorCode::IOError,
            "Failed to open tabula.json",
            e.to_string(),
        )]
    })?;
    serde_json::from_reader(file).map_err(|e| {
        vec![InitializationError::with_details(
            ErrorCode::JsonError,
            "Failed to parse tabula.json",
            e.to_string(),
        )]
    })
}

fn load_tabula_raw(tabula_path: &str) -> Result<TabulaRaw, Vec<InitializationError>> {
    if tabula_path.starts_with("jar:file:") {
        load_tabula_raw_android(tabula_path)
    } else {
        load_tabula_raw_filesystem(tabula_path)
    }
}

impl StateProvider for DefaultStateProvider {
    fn initialize(
        &self,
        persistent_data_path: &str,
        streaming_assets_path: &str,
    ) -> Result<(), Vec<InitializationError>> {
        let mut dir_guard = PERSISTENT_DATA_DIR.lock().unwrap();
        *dir_guard = Some(PathBuf::from(persistent_data_path));
        let tabula_path = format!("{streaming_assets_path}/tabula.json");
        let ctx = TabulaBuildContext { current_language: LanguageId::EnglishUnitedStates };
        let raw: TabulaRaw = match load_tabula_raw(&tabula_path) {
            Ok(r) => r,
            Err(e) => {
                let formatted = e.iter().map(|e| e.format()).collect::<Vec<_>>().join("\n");
                if let Ok(mut guard) = INITIALIZATION_ERROR.lock() {
                    *guard = Some(formatted.clone());
                }
                return Err(e);
            }
        };
        let tabula = tabula::build(&ctx, &raw)?;
        let mut guard = TABULA_DATA.write().unwrap();
        *guard = Some(Arc::new(tabula));
        if let Ok(mut guard) = INITIALIZATION_ERROR.lock() {
            *guard = None;
        }
        Ok(())
    }

    #[instrument(skip_all, level = "debug")]
    fn read_save_file(
        &self,
        user_id: UserId,
    ) -> Result<Option<SaveFile>, Vec<InitializationError>> {
        let dir = {
            let guard = PERSISTENT_DATA_DIR.lock().unwrap();
            guard.clone().ok_or_else(|| {
                vec![InitializationError::with_name(
                    ErrorCode::NotInitializedError,
                    "Data directory not initialized. Call initialize() first.".to_string(),
                )]
            })?
        };
        save_file_io::read_save_from_dir(&dir, user_id)
    }

    #[instrument(skip_all, level = "debug")]
    fn write_save_file(&self, save: SaveFile) -> Result<(), Vec<InitializationError>> {
        let dir = {
            let guard = PERSISTENT_DATA_DIR.lock().unwrap();
            guard.clone().ok_or_else(|| {
                vec![InitializationError::with_name(
                    ErrorCode::NotInitializedError,
                    "Data directory not initialized. Call initialize() first.".to_string(),
                )]
            })?
        };
        save_file_io::write_save_to_dir(&dir, &save)
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
        TOTAL_PENDING_UPDATES.fetch_add(1, Ordering::Release);
    }

    fn push_undo_entry(&self, battle_id: BattleId, player: PlayerName, state: BattleState) {
        let mut stacks = UNDO_STACKS.lock().unwrap();
        stacks.entry(battle_id).or_default().push(UndoEntry { player, state });
    }

    fn pop_undo_entry(&self, battle_id: BattleId, player: PlayerName) -> Option<BattleState> {
        let mut stacks = UNDO_STACKS.lock().unwrap();
        let stack = stacks.get_mut(&battle_id)?;
        if stack.is_empty() {
            return None;
        }
        let mut index = None;
        for i in (0..stack.len()).rev() {
            if stack[i].player == player {
                index = Some(i);
                break;
            }
        }
        let idx = index?;
        let drained: Vec<UndoEntry> = stack.drain(idx..).collect();
        drained.into_iter().next().map(|e| e.state)
    }

    fn clear_undo_stack(&self, battle_id: BattleId) {
        let mut stacks = UNDO_STACKS.lock().unwrap();
        stacks.remove(&battle_id);
    }

    fn take_next_poll_result(&self, user_id: UserId) -> Option<PollResult> {
        let mut updates = PENDING_UPDATES.lock().unwrap();
        if let Some(user_updates) = updates.get_mut(&user_id) {
            if !user_updates.is_empty() {
                let result = user_updates.remove(0);
                TOTAL_PENDING_UPDATES.fetch_sub(1, Ordering::AcqRel);
                return Some(result);
            }
        }
        None
    }

    fn set_speculative_search(&self, battle_id: BattleId, search: SpeculativeSearchState) {
        if let Ok(mut searches) = SPECULATIVE_SEARCHES.lock() {
            searches.insert(battle_id, search);
        }
    }

    fn take_speculative_search(&self, battle_id: BattleId) -> Option<SpeculativeSearchState> {
        if let Ok(mut searches) = SPECULATIVE_SEARCHES.lock() {
            searches.remove(&battle_id)
        } else {
            None
        }
    }

    fn stored_initialization_error(&self) -> Option<String> {
        INITIALIZATION_ERROR.lock().ok().and_then(|g| g.clone())
    }

    fn set_initialization_error(&self, error: String) {
        if let Ok(mut guard) = INITIALIZATION_ERROR.lock() {
            *guard = Some(error);
        }
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
        let guard = TABULA_DATA.read().expect("Failed to lock tabula data");
        guard.clone().expect("Tabula not initialized")
    }

    fn can_undo(&self, battle_id: BattleId, player: PlayerName) -> bool {
        let stacks = UNDO_STACKS.lock().unwrap();
        stacks
            .get(&battle_id)
            .map(|stack| stack.iter().any(|entry| entry.player == player))
            .unwrap_or(false)
    }
}
