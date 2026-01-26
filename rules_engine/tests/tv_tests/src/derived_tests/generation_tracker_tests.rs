use tv_lib::derived::generation_tracker::{
    GenerationTracker, RowKey, TaggedComputationRequest, TaggedComputationResult,
};

#[test]
fn test_initial_generation_is_zero() {
    let tracker = GenerationTracker::new();
    let key = RowKey::new("/test/file.toml", "cards", 0);
    assert_eq!(tracker.get_generation(&key), 0);
}

#[test]
fn test_increment_returns_positive_value() {
    let tracker = GenerationTracker::new();
    let key = RowKey::new("/test/file.toml", "cards", 0);

    let gen = tracker.increment_generation(key.clone());
    assert!(gen > 0, "Generation should be positive after increment");
}

#[test]
fn test_increment_updates_stored_value() {
    let tracker = GenerationTracker::new();
    let key = RowKey::new("/test/file.toml", "cards", 0);

    let gen = tracker.increment_generation(key.clone());
    assert_eq!(tracker.get_generation(&key), gen);
}

#[test]
fn test_multiple_increments_always_increase() {
    let tracker = GenerationTracker::new();
    let key = RowKey::new("/test/file.toml", "cards", 0);

    let gen1 = tracker.increment_generation(key.clone());
    let gen2 = tracker.increment_generation(key.clone());
    let gen3 = tracker.increment_generation(key.clone());

    assert!(gen2 > gen1);
    assert!(gen3 > gen2);
}

#[test]
fn test_different_rows_have_independent_generations() {
    let tracker = GenerationTracker::new();
    let key1 = RowKey::new("/test/file.toml", "cards", 0);
    let key2 = RowKey::new("/test/file.toml", "cards", 1);

    let gen1 = tracker.increment_generation(key1.clone());
    assert_eq!(tracker.get_generation(&key1), gen1);
    assert_eq!(tracker.get_generation(&key2), 0, "Unmodified row should be 0");
}

#[test]
fn test_different_files_have_independent_generations() {
    let tracker = GenerationTracker::new();
    let key1 = RowKey::new("/test/file1.toml", "cards", 0);
    let key2 = RowKey::new("/test/file2.toml", "cards", 0);

    let gen1 = tracker.increment_generation(key1.clone());
    assert_eq!(tracker.get_generation(&key1), gen1);
    assert_eq!(tracker.get_generation(&key2), 0, "Different file should be 0");
}

#[test]
fn test_different_tables_have_independent_generations() {
    let tracker = GenerationTracker::new();
    let key1 = RowKey::new("/test/file.toml", "cards", 0);
    let key2 = RowKey::new("/test/file.toml", "effects", 0);

    let gen1 = tracker.increment_generation(key1.clone());
    assert_eq!(tracker.get_generation(&key1), gen1);
    assert_eq!(tracker.get_generation(&key2), 0, "Different table should be 0");
}

#[test]
fn test_is_generation_current_returns_true_for_current() {
    let tracker = GenerationTracker::new();
    let key = RowKey::new("/test/file.toml", "cards", 0);

    let gen = tracker.increment_generation(key.clone());
    assert!(tracker.is_generation_current(&key, gen));
}

#[test]
fn test_is_generation_current_returns_false_for_stale() {
    let tracker = GenerationTracker::new();
    let key = RowKey::new("/test/file.toml", "cards", 0);

    let old_gen = tracker.increment_generation(key.clone());
    let _new_gen = tracker.increment_generation(key.clone());

    assert!(!tracker.is_generation_current(&key, old_gen));
}

#[test]
fn test_is_generation_current_returns_false_for_untracked() {
    let tracker = GenerationTracker::new();
    let key = RowKey::new("/test/file.toml", "cards", 0);

    // Generation 42 should not match untracked row (generation 0)
    assert!(!tracker.is_generation_current(&key, 42));
}

#[test]
fn test_clear_generation_resets_to_zero() {
    let tracker = GenerationTracker::new();
    let key = RowKey::new("/test/file.toml", "cards", 0);

    tracker.increment_generation(key.clone());
    assert!(tracker.get_generation(&key) > 0);

    tracker.clear_generation(&key);
    assert_eq!(tracker.get_generation(&key), 0);
}

#[test]
fn test_clear_generation_does_not_affect_other_rows() {
    let tracker = GenerationTracker::new();
    let key1 = RowKey::new("/test/file.toml", "cards", 0);
    let key2 = RowKey::new("/test/file.toml", "cards", 1);

    tracker.increment_generation(key1.clone());
    let gen2 = tracker.increment_generation(key2.clone());

    tracker.clear_generation(&key1);

    assert_eq!(tracker.get_generation(&key1), 0);
    assert_eq!(tracker.get_generation(&key2), gen2);
}

#[test]
fn test_clear_file_generations() {
    let tracker = GenerationTracker::new();

    let key1 = RowKey::new("/test/file1.toml", "cards", 0);
    let key2 = RowKey::new("/test/file1.toml", "cards", 1);
    let key3 = RowKey::new("/test/file2.toml", "cards", 0);

    tracker.increment_generation(key1.clone());
    tracker.increment_generation(key2.clone());
    let gen3 = tracker.increment_generation(key3.clone());

    tracker.clear_file_generations("/test/file1.toml");

    assert_eq!(tracker.get_generation(&key1), 0);
    assert_eq!(tracker.get_generation(&key2), 0);
    assert_eq!(tracker.get_generation(&key3), gen3, "Other file should be unaffected");
}

#[test]
fn test_clear_table_generations() {
    let tracker = GenerationTracker::new();

    let key1 = RowKey::new("/test/file.toml", "cards", 0);
    let key2 = RowKey::new("/test/file.toml", "cards", 1);
    let key3 = RowKey::new("/test/file.toml", "effects", 0);

    tracker.increment_generation(key1.clone());
    tracker.increment_generation(key2.clone());
    let gen3 = tracker.increment_generation(key3.clone());

    tracker.clear_table_generations("/test/file.toml", "cards");

    assert_eq!(tracker.get_generation(&key1), 0);
    assert_eq!(tracker.get_generation(&key2), 0);
    assert_eq!(tracker.get_generation(&key3), gen3, "Other table should be unaffected");
}

#[test]
fn test_tagged_computation_request_creation() {
    let key = RowKey::new("/test/file.toml", "cards", 5);
    let request = TaggedComputationRequest::new(key.clone(), "card_lookup", 42);

    assert_eq!(request.row_key, key);
    assert_eq!(request.function_name, "card_lookup");
    assert_eq!(request.generation, 42);
}

#[test]
fn test_tagged_computation_result_creation() {
    let key = RowKey::new("/test/file.toml", "cards", 5);
    let result = TaggedComputationResult::new(key.clone(), "card_lookup", 42, "Dragon Knight");

    assert_eq!(result.row_key, key);
    assert_eq!(result.function_name, "card_lookup");
    assert_eq!(result.generation, 42);
    assert_eq!(result.result, "Dragon Knight");
}

#[test]
fn test_tagged_result_from_request() {
    let key = RowKey::new("/test/file.toml", "cards", 5);
    let request = TaggedComputationRequest::new(key.clone(), "card_lookup", 42);
    let result = TaggedComputationResult::from_request(&request, "Dragon Knight");

    assert_eq!(result.row_key, request.row_key);
    assert_eq!(result.function_name, request.function_name);
    assert_eq!(result.generation, request.generation);
    assert_eq!(result.result, "Dragon Knight");
}

#[test]
fn test_tagged_result_is_current() {
    let tracker = GenerationTracker::new();
    let key = RowKey::new("/test/file.toml", "cards", 0);

    let gen = tracker.increment_generation(key.clone());
    let result = TaggedComputationResult::new(key.clone(), "card_lookup", gen, "Test");

    assert!(result.is_current(&tracker));
}

#[test]
fn test_tagged_result_is_stale_after_row_edit() {
    let tracker = GenerationTracker::new();
    let key = RowKey::new("/test/file.toml", "cards", 0);

    let old_gen = tracker.increment_generation(key.clone());
    let result = TaggedComputationResult::new(key.clone(), "card_lookup", old_gen, "Test");

    // Simulate row edit
    tracker.increment_generation(key.clone());

    assert!(!result.is_current(&tracker), "Result should be stale after row edit");
}

#[test]
fn test_concurrent_usage_simulation() {
    // Simulate concurrent edits followed by result arrival
    let tracker = GenerationTracker::new();
    let key = RowKey::new("/test/file.toml", "cards", 0);

    // First edit - computation starts
    let gen1 = tracker.increment_generation(key.clone());
    let request1 = TaggedComputationRequest::new(key.clone(), "card_lookup", gen1);

    // Second edit before first computation finishes
    let gen2 = tracker.increment_generation(key.clone());
    let request2 = TaggedComputationRequest::new(key.clone(), "card_lookup", gen2);

    // First result arrives (stale)
    let result1 = TaggedComputationResult::from_request(&request1, "Old Value");
    assert!(!result1.is_current(&tracker), "First result should be stale");

    // Second result arrives (current)
    let result2 = TaggedComputationResult::from_request(&request2, "New Value");
    assert!(result2.is_current(&tracker), "Second result should be current");
}

#[test]
fn test_rapid_edits_only_latest_is_current() {
    let tracker = GenerationTracker::new();
    let key = RowKey::new("/test/file.toml", "cards", 0);

    // Simulate rapid editing
    let generations: Vec<u64> =
        (0..10).map(|_| tracker.increment_generation(key.clone())).collect();

    // Only the last generation should be current
    for (i, &gen) in generations.iter().enumerate() {
        let is_last = i == generations.len() - 1;
        assert_eq!(
            tracker.is_generation_current(&key, gen),
            is_last,
            "Only generation at index {} should be current, checking index {i}",
            generations.len() - 1
        );
    }
}

#[test]
fn test_row_key_equality() {
    let key1 = RowKey::new("/test/file.toml", "cards", 0);
    let key2 = RowKey::new("/test/file.toml", "cards", 0);
    let key3 = RowKey::new("/test/file.toml", "cards", 1);

    assert_eq!(key1, key2);
    assert_ne!(key1, key3);
}

#[test]
fn test_row_key_clone() {
    let key1 = RowKey::new("/test/file.toml", "cards", 0);
    let key2 = key1.clone();

    assert_eq!(key1, key2);
}

#[test]
fn test_generation_tracker_default() {
    let tracker = GenerationTracker::default();
    let key = RowKey::new("/test/file.toml", "cards", 0);
    assert_eq!(tracker.get_generation(&key), 0);
}

#[test]
fn test_global_counter_ensures_unique_generations() {
    let tracker = GenerationTracker::new();

    let key1 = RowKey::new("/test/file.toml", "cards", 0);
    let key2 = RowKey::new("/test/file.toml", "cards", 1);
    let key3 = RowKey::new("/other/file.toml", "effects", 0);

    let gen1 = tracker.increment_generation(key1);
    let gen2 = tracker.increment_generation(key2);
    let gen3 = tracker.increment_generation(key3);

    // All generations should be unique and increasing
    assert!(gen2 > gen1);
    assert!(gen3 > gen2);
}
