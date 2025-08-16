use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use core_data::identifiers::UserId;
use core_data::initialization_error::{ErrorCode, InitializationError};
use database::database::Database;
use database::save_file::SaveFile;

#[derive(Clone)]
pub struct TestDatabase {
    storage: Arc<RwLock<HashMap<UserId, SaveFile>>>,
}

impl TestDatabase {
    pub fn new() -> Self {
        Self { storage: Arc::new(RwLock::new(HashMap::new())) }
    }
}

impl Default for TestDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl Database for TestDatabase {
    fn fetch_save(&self, user_id: UserId) -> Result<Option<SaveFile>, Vec<InitializationError>> {
        let storage = self.storage.read().map_err(|e| {
            vec![InitializationError::with_details(
                ErrorCode::MutexLockError,
                "Failed to acquire read lock".to_string(),
                e.to_string(),
            )]
        })?;

        Ok(storage.get(&user_id).cloned())
    }

    fn write_save(&self, save: SaveFile) -> Result<(), Vec<InitializationError>> {
        let mut storage = self.storage.write().map_err(|e| {
            vec![InitializationError::with_details(
                ErrorCode::MutexLockError,
                "Failed to acquire write lock".to_string(),
                e.to_string(),
            )]
        })?;
        storage.insert(save.id(), save);
        Ok(())
    }
}
