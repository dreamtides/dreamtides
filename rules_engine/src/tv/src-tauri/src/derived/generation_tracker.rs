use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

/// A unique identifier for a row within a file/table combination.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RowKey {
    /// The file path of the TOML file.
    pub file_path: String,
    /// The table name within the file.
    pub table_name: String,
    /// The zero-based row index.
    pub row_index: usize,
}

impl RowKey {
    pub fn new(file_path: impl Into<String>, table_name: impl Into<String>, row_index: usize) -> Self {
        Self { file_path: file_path.into(), table_name: table_name.into(), row_index }
    }
}

/// Tracks generation counters for rows to ensure stale computation results are discarded.
///
/// Each row has a generation counter that is incremented whenever the row is edited.
/// When a derived column computation is started, the current generation is captured.
/// When the result arrives, it is only applied if the generation matches the current
/// value, ensuring that stale results from outdated computations are discarded.
pub struct GenerationTracker {
    generations: RwLock<HashMap<RowKey, u64>>,
    /// Global counter for generating unique generation values.
    /// Using a global counter ensures generations are always increasing,
    /// even across different rows.
    global_counter: AtomicU64,
}

impl GenerationTracker {
    /// Creates a new generation tracker.
    pub fn new() -> Self {
        Self { generations: RwLock::new(HashMap::new()), global_counter: AtomicU64::new(1) }
    }

    /// Gets the current generation for a row.
    ///
    /// Returns 0 if the row has never been tracked.
    pub fn get_generation(&self, key: &RowKey) -> u64 {
        self.generations.read().expect("Generation tracker lock poisoned").get(key).copied().unwrap_or(0)
    }

    /// Increments the generation counter for a row and returns the new value.
    ///
    /// This should be called whenever a row is edited to invalidate any
    /// in-flight computations.
    pub fn increment_generation(&self, key: RowKey) -> u64 {
        let new_gen = self.global_counter.fetch_add(1, Ordering::SeqCst);
        self.generations.write().expect("Generation tracker lock poisoned").insert(key.clone(), new_gen);

        tracing::debug!(
            component = "tv.derived.generation_tracker",
            file_path = %key.file_path,
            table_name = %key.table_name,
            row_index = key.row_index,
            generation = new_gen,
            "Incremented row generation"
        );

        new_gen
    }

    /// Checks if a computation result with the given generation is still valid.
    ///
    /// A result is valid if the generation matches the current generation for the row.
    /// This prevents stale results from overwriting newer computations.
    pub fn is_generation_current(&self, key: &RowKey, generation: u64) -> bool {
        let current = self.get_generation(key);
        let is_current = current == generation;

        if !is_current {
            tracing::debug!(
                component = "tv.derived.generation_tracker",
                file_path = %key.file_path,
                table_name = %key.table_name,
                row_index = key.row_index,
                result_generation = generation,
                current_generation = current,
                "Discarding stale computation result"
            );
        }

        is_current
    }

    /// Clears the generation for a specific row.
    ///
    /// This is useful when a row is deleted.
    pub fn clear_generation(&self, key: &RowKey) {
        self.generations.write().expect("Generation tracker lock poisoned").remove(key);

        tracing::debug!(
            component = "tv.derived.generation_tracker",
            file_path = %key.file_path,
            table_name = %key.table_name,
            row_index = key.row_index,
            "Cleared row generation"
        );
    }

    /// Clears all generations for a file.
    ///
    /// This is useful when a file is closed or reloaded.
    pub fn clear_file_generations(&self, file_path: &str) {
        let mut generations = self.generations.write().expect("Generation tracker lock poisoned");
        let keys_to_remove: Vec<RowKey> =
            generations.keys().filter(|k| k.file_path == file_path).cloned().collect();

        for key in &keys_to_remove {
            generations.remove(key);
        }

        tracing::debug!(
            component = "tv.derived.generation_tracker",
            file_path = %file_path,
            rows_cleared = keys_to_remove.len(),
            "Cleared all generations for file"
        );
    }

    /// Clears all generations for a specific table within a file.
    pub fn clear_table_generations(&self, file_path: &str, table_name: &str) {
        let mut generations = self.generations.write().expect("Generation tracker lock poisoned");
        let keys_to_remove: Vec<RowKey> = generations
            .keys()
            .filter(|k| k.file_path == file_path && k.table_name == table_name)
            .cloned()
            .collect();

        for key in &keys_to_remove {
            generations.remove(key);
        }

        tracing::debug!(
            component = "tv.derived.generation_tracker",
            file_path = %file_path,
            table_name = %table_name,
            rows_cleared = keys_to_remove.len(),
            "Cleared all generations for table"
        );
    }
}

impl Default for GenerationTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// A computation request tagged with its generation.
///
/// This struct captures the generation at the time the computation was
/// requested, allowing the result to be validated when it arrives.
#[derive(Debug, Clone)]
pub struct TaggedComputationRequest {
    /// The row being computed.
    pub row_key: RowKey,
    /// The derived function name to execute.
    pub function_name: String,
    /// The generation of the row when this computation was requested.
    pub generation: u64,
}

impl TaggedComputationRequest {
    /// Creates a new tagged computation request.
    pub fn new(row_key: RowKey, function_name: impl Into<String>, generation: u64) -> Self {
        Self { row_key, function_name: function_name.into(), generation }
    }
}

/// A computation result tagged with its generation.
///
/// This struct pairs a computation result with the generation it was computed for,
/// allowing staleness checking before applying the result.
#[derive(Debug, Clone)]
pub struct TaggedComputationResult<T> {
    /// The row the result is for.
    pub row_key: RowKey,
    /// The derived function that produced this result.
    pub function_name: String,
    /// The generation this result was computed for.
    pub generation: u64,
    /// The actual result.
    pub result: T,
}

impl<T> TaggedComputationResult<T> {
    /// Creates a new tagged computation result.
    pub fn new(row_key: RowKey, function_name: impl Into<String>, generation: u64, result: T) -> Self {
        Self { row_key, function_name: function_name.into(), generation, result }
    }

    /// Creates a tagged result from a request and result.
    pub fn from_request(request: &TaggedComputationRequest, result: T) -> Self {
        Self {
            row_key: request.row_key.clone(),
            function_name: request.function_name.clone(),
            generation: request.generation,
            result,
        }
    }

    /// Checks if this result is still current according to the tracker.
    pub fn is_current(&self, tracker: &GenerationTracker) -> bool {
        tracker.is_generation_current(&self.row_key, self.generation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_key(row: usize) -> RowKey {
        RowKey::new("/test/file.toml", "cards", row)
    }

    #[test]
    fn test_initial_generation_is_zero() {
        let tracker = GenerationTracker::new();
        assert_eq!(tracker.get_generation(&make_key(0)), 0);
    }

    #[test]
    fn test_increment_generation() {
        let tracker = GenerationTracker::new();
        let key = make_key(0);

        let gen1 = tracker.increment_generation(key.clone());
        assert!(gen1 > 0);
        assert_eq!(tracker.get_generation(&key), gen1);

        let gen2 = tracker.increment_generation(key.clone());
        assert!(gen2 > gen1);
        assert_eq!(tracker.get_generation(&key), gen2);
    }

    #[test]
    fn test_different_rows_have_different_generations() {
        let tracker = GenerationTracker::new();
        let key1 = make_key(0);
        let key2 = make_key(1);

        let gen1 = tracker.increment_generation(key1.clone());
        let gen2 = tracker.increment_generation(key2.clone());

        assert_ne!(gen1, gen2);
        assert_eq!(tracker.get_generation(&key1), gen1);
        assert_eq!(tracker.get_generation(&key2), gen2);
    }

    #[test]
    fn test_is_generation_current() {
        let tracker = GenerationTracker::new();
        let key = make_key(0);

        let gen1 = tracker.increment_generation(key.clone());
        assert!(tracker.is_generation_current(&key, gen1));

        let gen2 = tracker.increment_generation(key.clone());
        assert!(!tracker.is_generation_current(&key, gen1));
        assert!(tracker.is_generation_current(&key, gen2));
    }

    #[test]
    fn test_clear_generation() {
        let tracker = GenerationTracker::new();
        let key = make_key(0);

        tracker.increment_generation(key.clone());
        assert!(tracker.get_generation(&key) > 0);

        tracker.clear_generation(&key);
        assert_eq!(tracker.get_generation(&key), 0);
    }

    #[test]
    fn test_clear_file_generations() {
        let tracker = GenerationTracker::new();

        let key1 = RowKey::new("/test/file1.toml", "cards", 0);
        let key2 = RowKey::new("/test/file1.toml", "cards", 1);
        let key3 = RowKey::new("/test/file2.toml", "cards", 0);

        tracker.increment_generation(key1.clone());
        tracker.increment_generation(key2.clone());
        tracker.increment_generation(key3.clone());

        tracker.clear_file_generations("/test/file1.toml");

        assert_eq!(tracker.get_generation(&key1), 0);
        assert_eq!(tracker.get_generation(&key2), 0);
        assert!(tracker.get_generation(&key3) > 0);
    }

    #[test]
    fn test_clear_table_generations() {
        let tracker = GenerationTracker::new();

        let key1 = RowKey::new("/test/file.toml", "cards", 0);
        let key2 = RowKey::new("/test/file.toml", "cards", 1);
        let key3 = RowKey::new("/test/file.toml", "other", 0);

        tracker.increment_generation(key1.clone());
        tracker.increment_generation(key2.clone());
        tracker.increment_generation(key3.clone());

        tracker.clear_table_generations("/test/file.toml", "cards");

        assert_eq!(tracker.get_generation(&key1), 0);
        assert_eq!(tracker.get_generation(&key2), 0);
        assert!(tracker.get_generation(&key3) > 0);
    }

    #[test]
    fn test_tagged_computation_request() {
        let key = make_key(0);
        let request = TaggedComputationRequest::new(key.clone(), "card_lookup", 42);

        assert_eq!(request.row_key, key);
        assert_eq!(request.function_name, "card_lookup");
        assert_eq!(request.generation, 42);
    }

    #[test]
    fn test_tagged_computation_result() {
        let tracker = GenerationTracker::new();
        let key = make_key(0);

        let gen = tracker.increment_generation(key.clone());
        let result = TaggedComputationResult::new(key.clone(), "card_lookup", gen, "Test Card");

        assert!(result.is_current(&tracker));

        tracker.increment_generation(key.clone());
        assert!(!result.is_current(&tracker));
    }

    #[test]
    fn test_tagged_result_from_request() {
        let key = make_key(0);
        let request = TaggedComputationRequest::new(key.clone(), "card_lookup", 42);
        let result = TaggedComputationResult::from_request(&request, "Test Card");

        assert_eq!(result.row_key, key);
        assert_eq!(result.function_name, "card_lookup");
        assert_eq!(result.generation, 42);
        assert_eq!(result.result, "Test Card");
    }

    #[test]
    fn test_generations_always_increase() {
        let tracker = GenerationTracker::new();
        let mut last_gen = 0u64;

        for i in 0..100 {
            let key = make_key(i % 10);
            let gen = tracker.increment_generation(key);
            assert!(gen > last_gen);
            last_gen = gen;
        }
    }
}
