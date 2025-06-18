use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use battle_state::battle::battle_state::RequestContext;
use core_data::identifiers::UserId;
use database::database::DatabaseError;
use rules_engine::state_provider::StateProvider;
use uuid::Uuid;

use super::test_database::TestDatabase;

#[derive(Clone)]
pub struct TestStateProvider {
    inner: Arc<TestStateProviderInner>,
}

struct TestStateProviderInner {
    databases: Mutex<HashMap<String, TestDatabase>>,
    request_contexts: Mutex<HashMap<UserId, RequestContext>>,
    request_timestamps: Mutex<HashMap<Option<Uuid>, Instant>>,
}

impl TestStateProvider {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(TestStateProviderInner {
                databases: Mutex::new(HashMap::new()),
                request_contexts: Mutex::new(HashMap::new()),
                request_timestamps: Mutex::new(HashMap::new()),
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

    fn initialize_database(&self, path: &str) -> Result<Self::DatabaseImpl, DatabaseError> {
        let db = TestDatabase::new();
        let mut databases = self
            .inner
            .databases
            .lock()
            .map_err(|e| DatabaseError(format!("Failed to acquire lock: {}", e)))?;
        databases.insert(path.to_string(), db.clone());
        Ok(db)
    }

    fn get_database(&self) -> Result<Self::DatabaseImpl, DatabaseError> {
        let databases = self
            .inner
            .databases
            .lock()
            .map_err(|e| DatabaseError(format!("Failed to acquire lock: {}", e)))?;
        databases
            .values()
            .next()
            .cloned()
            .ok_or_else(|| DatabaseError("No database initialized".to_string()))
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
                format!("[unknown request ID: {:?}]", request_id)
            } else {
                "[empty request ID]".to_string()
            }
        } else {
            "[mutex lock failed]".to_string()
        }
    }
}
