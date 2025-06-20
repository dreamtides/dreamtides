use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use core_data::identifiers::UserId;
use database::database::{Database, DatabaseError};
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
    fn fetch_save(&self, user_id: UserId) -> Result<Option<SaveFile>, DatabaseError> {
        let storage = self
            .storage
            .read()
            .map_err(|e| DatabaseError(format!("Failed to acquire read lock: {}", e)))?;

        Ok(storage.get(&user_id).cloned())
    }

    fn write_save(&self, save: SaveFile) -> Result<(), DatabaseError> {
        let mut storage = self
            .storage
            .write()
            .map_err(|e| DatabaseError(format!("Failed to acquire write lock: {}", e)))?;
        storage.insert(save.id(), save);
        Ok(())
    }
}
