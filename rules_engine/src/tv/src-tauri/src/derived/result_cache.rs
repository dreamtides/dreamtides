use std::collections::HashMap;

use crate::derived::derived_types::DerivedResult;
use crate::derived::generation_tracker::{GenerationTracker, RowKey};

/// A cache key combining row identity, function name, and generation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    row_key: RowKey,
    function_name: String,
    generation: u64,
}

/// An entry in the result cache, containing the cached result and access
/// ordering metadata for LRU eviction.
struct CacheEntry {
    result: DerivedResult,
    access_order: u64,
}

/// Caches computed derived column values with LRU eviction.
///
/// Cache entries are keyed by (row_key, function_name, generation). When a row's
/// generation changes, all cached results for that row are automatically stale
/// because lookups use the current generation as part of the key. The cache uses
/// LRU eviction when the number of entries exceeds the configured capacity.
pub struct ResultCache {
    entries: HashMap<CacheKey, CacheEntry>,
    capacity: usize,
    access_counter: u64,
}

impl ResultCache {
    /// Creates a new result cache with the specified maximum capacity.
    pub fn new(capacity: usize) -> Self {
        Self { entries: HashMap::new(), capacity, access_counter: 0 }
    }

    /// Returns the maximum number of entries this cache can hold.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the current number of entries in the cache.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Looks up a cached result for the given row, function, and generation.
    ///
    /// Returns the cached result if present, or None if no entry exists. Updates
    /// the access order for LRU tracking on hit.
    pub fn get(
        &mut self,
        row_key: &RowKey,
        function_name: &str,
        generation: u64,
    ) -> Option<&DerivedResult> {
        let key = CacheKey {
            row_key: row_key.clone(),
            function_name: function_name.to_string(),
            generation,
        };

        if self.entries.contains_key(&key) {
            self.access_counter += 1;
            let entry = self.entries.get_mut(&key).expect("Key confirmed present");
            entry.access_order = self.access_counter;

            tracing::debug!(
                component = "tv.derived.result_cache",
                file_path = %row_key.file_path,
                table_name = %row_key.table_name,
                row_index = row_key.row_index,
                function_name = %function_name,
                generation = generation,
                "Cache hit"
            );

            Some(&self.entries[&key].result)
        } else {
            tracing::debug!(
                component = "tv.derived.result_cache",
                file_path = %row_key.file_path,
                table_name = %row_key.table_name,
                row_index = row_key.row_index,
                function_name = %function_name,
                generation = generation,
                "Cache miss"
            );
            None
        }
    }

    /// Inserts a computed result into the cache.
    ///
    /// If the cache is at capacity, the least recently used entry is evicted
    /// before insertion.
    pub fn put(
        &mut self,
        row_key: RowKey,
        function_name: String,
        generation: u64,
        result: DerivedResult,
    ) {
        let key = CacheKey { row_key: row_key.clone(), function_name: function_name.clone(), generation };

        if self.entries.contains_key(&key) {
            self.access_counter += 1;
            let entry = self.entries.get_mut(&key).expect("Key confirmed present");
            entry.result = result;
            entry.access_order = self.access_counter;
            return;
        }

        if self.entries.len() >= self.capacity {
            self.evict_lru();
        }

        self.access_counter += 1;
        self.entries.insert(key, CacheEntry { result, access_order: self.access_counter });

        tracing::debug!(
            component = "tv.derived.result_cache",
            file_path = %row_key.file_path,
            table_name = %row_key.table_name,
            row_index = row_key.row_index,
            function_name = %function_name,
            generation = generation,
            cache_size = self.entries.len(),
            "Inserted cache entry"
        );
    }

    /// Invalidates all cached results for a specific row.
    ///
    /// This removes all entries matching the row key regardless of function name
    /// or generation. Call this when a row is edited.
    pub fn invalidate_row(&mut self, row_key: &RowKey) {
        let keys_to_remove: Vec<CacheKey> = self
            .entries
            .keys()
            .filter(|k| k.row_key == *row_key)
            .cloned()
            .collect();

        let count = keys_to_remove.len();
        for key in keys_to_remove {
            self.entries.remove(&key);
        }

        if count > 0 {
            tracing::debug!(
                component = "tv.derived.result_cache",
                file_path = %row_key.file_path,
                table_name = %row_key.table_name,
                row_index = row_key.row_index,
                entries_removed = count,
                "Invalidated row cache entries"
            );
        }
    }

    /// Invalidates all cached results for a specific file.
    ///
    /// This removes all entries whose row key matches the given file path.
    pub fn invalidate_file(&mut self, file_path: &str) {
        let keys_to_remove: Vec<CacheKey> = self
            .entries
            .keys()
            .filter(|k| k.row_key.file_path == file_path)
            .cloned()
            .collect();

        let count = keys_to_remove.len();
        for key in keys_to_remove {
            self.entries.remove(&key);
        }

        if count > 0 {
            tracing::debug!(
                component = "tv.derived.result_cache",
                file_path = %file_path,
                entries_removed = count,
                "Invalidated file cache entries"
            );
        }
    }

    /// Invalidates all cached results for a specific table within a file.
    pub fn invalidate_table(&mut self, file_path: &str, table_name: &str) {
        let keys_to_remove: Vec<CacheKey> = self
            .entries
            .keys()
            .filter(|k| k.row_key.file_path == file_path && k.row_key.table_name == table_name)
            .cloned()
            .collect();

        let count = keys_to_remove.len();
        for key in keys_to_remove {
            self.entries.remove(&key);
        }

        if count > 0 {
            tracing::debug!(
                component = "tv.derived.result_cache",
                file_path = %file_path,
                table_name = %table_name,
                entries_removed = count,
                "Invalidated table cache entries"
            );
        }
    }

    /// Removes all entries from the cache.
    pub fn clear(&mut self) {
        let count = self.entries.len();
        self.entries.clear();
        self.access_counter = 0;

        tracing::debug!(
            component = "tv.derived.result_cache",
            entries_removed = count,
            "Cleared result cache"
        );
    }

    /// Looks up a cached result using the current generation from the tracker.
    ///
    /// This is a convenience method that combines generation lookup with cache
    /// retrieval. Returns None if the row has no tracked generation or if no
    /// cached result exists for the current generation.
    pub fn get_current(
        &mut self,
        row_key: &RowKey,
        function_name: &str,
        tracker: &GenerationTracker,
    ) -> Option<&DerivedResult> {
        let generation = tracker.get_generation(row_key);
        if generation == 0 {
            return None;
        }
        self.get(row_key, function_name, generation)
    }

    /// Evicts the least recently used entry from the cache.
    fn evict_lru(&mut self) {
        let lru_key = self
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.access_order)
            .map(|(key, _)| key.clone());

        if let Some(key) = lru_key {
            tracing::debug!(
                component = "tv.derived.result_cache",
                file_path = %key.row_key.file_path,
                table_name = %key.row_key.table_name,
                row_index = key.row_key.row_index,
                function_name = %key.function_name,
                generation = key.generation,
                "Evicting LRU cache entry"
            );
            self.entries.remove(&key);
        }
    }
}

impl Default for ResultCache {
    fn default() -> Self {
        Self::new(10_000)
    }
}
