use tv_lib::derived::derived_types::DerivedResult;
use tv_lib::derived::generation_tracker::{GenerationTracker, RowKey};
use tv_lib::derived::result_cache::ResultCache;

fn make_key(row: usize) -> RowKey {
    RowKey::new("/test/file.toml", "cards", row)
}

#[test]
fn test_new_cache_is_empty() {
    let cache = ResultCache::new(100);
    assert!(cache.is_empty());
    assert_eq!(cache.len(), 0);
    assert_eq!(cache.capacity(), 100);
}

#[test]
fn test_default_cache_capacity() {
    let cache = ResultCache::default();
    assert_eq!(cache.capacity(), 10_000);
    assert!(cache.is_empty());
}

#[test]
fn test_put_and_get() {
    let mut cache = ResultCache::new(100);
    let key = make_key(0);

    cache.put(key.clone(), "card_lookup".to_string(), 1, DerivedResult::Text("Dragon".to_string()));

    let result = cache.get(&key, "card_lookup", 1);
    assert_eq!(result, Some(&DerivedResult::Text("Dragon".to_string())));
}

#[test]
fn test_get_miss_returns_none() {
    let mut cache = ResultCache::new(100);
    let key = make_key(0);

    assert!(cache.get(&key, "card_lookup", 1).is_none());
}

#[test]
fn test_different_function_names_are_separate() {
    let mut cache = ResultCache::new(100);
    let key = make_key(0);

    cache.put(key.clone(), "card_lookup".to_string(), 1, DerivedResult::Text("Dragon".to_string()));
    cache.put(
        key.clone(),
        "image_url".to_string(),
        1,
        DerivedResult::Image("http://example.com".to_string()),
    );

    assert_eq!(cache.get(&key, "card_lookup", 1), Some(&DerivedResult::Text("Dragon".to_string())));
    assert_eq!(
        cache.get(&key, "image_url", 1),
        Some(&DerivedResult::Image("http://example.com".to_string()))
    );
    assert_eq!(cache.len(), 2);
}

#[test]
fn test_different_generations_are_separate() {
    let mut cache = ResultCache::new(100);
    let key = make_key(0);

    cache.put(key.clone(), "card_lookup".to_string(), 1, DerivedResult::Text("Old".to_string()));
    cache.put(key.clone(), "card_lookup".to_string(), 2, DerivedResult::Text("New".to_string()));

    assert_eq!(cache.get(&key, "card_lookup", 1), Some(&DerivedResult::Text("Old".to_string())));
    assert_eq!(cache.get(&key, "card_lookup", 2), Some(&DerivedResult::Text("New".to_string())));
    assert_eq!(cache.len(), 2);
}

#[test]
fn test_different_rows_are_separate() {
    let mut cache = ResultCache::new(100);
    let key0 = make_key(0);
    let key1 = make_key(1);

    cache.put(key0.clone(), "card_lookup".to_string(), 1, DerivedResult::Text("Row 0".to_string()));
    cache.put(key1.clone(), "card_lookup".to_string(), 1, DerivedResult::Text("Row 1".to_string()));

    assert_eq!(cache.get(&key0, "card_lookup", 1), Some(&DerivedResult::Text("Row 0".to_string())));
    assert_eq!(cache.get(&key1, "card_lookup", 1), Some(&DerivedResult::Text("Row 1".to_string())));
    assert_eq!(cache.len(), 2);
}

#[test]
fn test_put_overwrites_existing_entry() {
    let mut cache = ResultCache::new(100);
    let key = make_key(0);

    cache.put(key.clone(), "card_lookup".to_string(), 1, DerivedResult::Text("Old".to_string()));
    cache.put(
        key.clone(),
        "card_lookup".to_string(),
        1,
        DerivedResult::Text("Updated".to_string()),
    );

    assert_eq!(
        cache.get(&key, "card_lookup", 1),
        Some(&DerivedResult::Text("Updated".to_string()))
    );
    assert_eq!(cache.len(), 1);
}

#[test]
fn test_lru_eviction_at_capacity() {
    let mut cache = ResultCache::new(3);

    cache.put(make_key(0), "f".to_string(), 1, DerivedResult::Text("A".to_string()));
    cache.put(make_key(1), "f".to_string(), 1, DerivedResult::Text("B".to_string()));
    cache.put(make_key(2), "f".to_string(), 1, DerivedResult::Text("C".to_string()));

    assert_eq!(cache.len(), 3);

    // Inserting a 4th entry should evict the least recently used (row 0)
    cache.put(make_key(3), "f".to_string(), 1, DerivedResult::Text("D".to_string()));

    assert_eq!(cache.len(), 3);
    assert!(cache.get(&make_key(0), "f", 1).is_none(), "LRU entry should have been evicted");
    assert!(cache.get(&make_key(1), "f", 1).is_some());
    assert!(cache.get(&make_key(2), "f", 1).is_some());
    assert!(cache.get(&make_key(3), "f", 1).is_some());
}

#[test]
fn test_lru_access_updates_order() {
    let mut cache = ResultCache::new(3);

    cache.put(make_key(0), "f".to_string(), 1, DerivedResult::Text("A".to_string()));
    cache.put(make_key(1), "f".to_string(), 1, DerivedResult::Text("B".to_string()));
    cache.put(make_key(2), "f".to_string(), 1, DerivedResult::Text("C".to_string()));

    // Access row 0 to make it recently used
    cache.get(&make_key(0), "f", 1);

    // Now row 1 is the LRU entry, so it should be evicted
    cache.put(make_key(3), "f".to_string(), 1, DerivedResult::Text("D".to_string()));

    assert_eq!(cache.len(), 3);
    assert!(cache.get(&make_key(0), "f", 1).is_some(), "Recently accessed entry should survive");
    assert!(cache.get(&make_key(1), "f", 1).is_none(), "LRU entry should have been evicted");
    assert!(cache.get(&make_key(2), "f", 1).is_some());
    assert!(cache.get(&make_key(3), "f", 1).is_some());
}

#[test]
fn test_invalidate_row_removes_all_functions() {
    let mut cache = ResultCache::new(100);
    let key = make_key(0);

    cache.put(key.clone(), "card_lookup".to_string(), 1, DerivedResult::Text("Name".to_string()));
    cache.put(key.clone(), "image_url".to_string(), 1, DerivedResult::Image("url".to_string()));
    cache.put(make_key(1), "card_lookup".to_string(), 1, DerivedResult::Text("Other".to_string()));

    assert_eq!(cache.len(), 3);

    cache.invalidate_row(&key);

    assert_eq!(cache.len(), 1);
    assert!(cache.get(&key, "card_lookup", 1).is_none());
    assert!(cache.get(&key, "image_url", 1).is_none());
    assert!(cache.get(&make_key(1), "card_lookup", 1).is_some());
}

#[test]
fn test_invalidate_row_removes_all_generations() {
    let mut cache = ResultCache::new(100);
    let key = make_key(0);

    cache.put(key.clone(), "card_lookup".to_string(), 1, DerivedResult::Text("Old".to_string()));
    cache.put(key.clone(), "card_lookup".to_string(), 2, DerivedResult::Text("New".to_string()));

    cache.invalidate_row(&key);

    assert!(cache.is_empty());
}

#[test]
fn test_invalidate_file_removes_all_entries_for_file() {
    let mut cache = ResultCache::new(100);

    let key1 = RowKey::new("/test/file1.toml", "cards", 0);
    let key2 = RowKey::new("/test/file1.toml", "cards", 1);
    let key3 = RowKey::new("/test/file2.toml", "cards", 0);

    cache.put(key1.clone(), "f".to_string(), 1, DerivedResult::Text("A".to_string()));
    cache.put(key2.clone(), "f".to_string(), 1, DerivedResult::Text("B".to_string()));
    cache.put(key3.clone(), "f".to_string(), 1, DerivedResult::Text("C".to_string()));

    cache.invalidate_file("/test/file1.toml");

    assert_eq!(cache.len(), 1);
    assert!(cache.get(&key1, "f", 1).is_none());
    assert!(cache.get(&key2, "f", 1).is_none());
    assert!(cache.get(&key3, "f", 1).is_some());
}

#[test]
fn test_invalidate_table_removes_entries_for_table() {
    let mut cache = ResultCache::new(100);

    let key1 = RowKey::new("/test/file.toml", "cards", 0);
    let key2 = RowKey::new("/test/file.toml", "cards", 1);
    let key3 = RowKey::new("/test/file.toml", "effects", 0);

    cache.put(key1.clone(), "f".to_string(), 1, DerivedResult::Text("A".to_string()));
    cache.put(key2.clone(), "f".to_string(), 1, DerivedResult::Text("B".to_string()));
    cache.put(key3.clone(), "f".to_string(), 1, DerivedResult::Text("C".to_string()));

    cache.invalidate_table("/test/file.toml", "cards");

    assert_eq!(cache.len(), 1);
    assert!(cache.get(&key1, "f", 1).is_none());
    assert!(cache.get(&key2, "f", 1).is_none());
    assert!(cache.get(&key3, "f", 1).is_some());
}

#[test]
fn test_clear_removes_all_entries() {
    let mut cache = ResultCache::new(100);

    cache.put(make_key(0), "f".to_string(), 1, DerivedResult::Text("A".to_string()));
    cache.put(make_key(1), "f".to_string(), 1, DerivedResult::Text("B".to_string()));
    cache.put(make_key(2), "f".to_string(), 1, DerivedResult::Text("C".to_string()));

    assert_eq!(cache.len(), 3);

    cache.clear();

    assert!(cache.is_empty());
    assert_eq!(cache.len(), 0);
}

#[test]
fn test_clear_empty_cache_is_safe() {
    let mut cache = ResultCache::new(100);
    cache.clear();
    assert!(cache.is_empty());
}

#[test]
fn test_get_current_with_generation_tracker() {
    let mut cache = ResultCache::new(100);
    let tracker = GenerationTracker::new();
    let key = make_key(0);

    let gen = tracker.increment_generation(key.clone());
    cache.put(
        key.clone(),
        "card_lookup".to_string(),
        gen,
        DerivedResult::Text("Dragon".to_string()),
    );

    let result = cache.get_current(&key, "card_lookup", &tracker);
    assert_eq!(result, Some(&DerivedResult::Text("Dragon".to_string())));
}

#[test]
fn test_get_current_returns_none_for_stale_generation() {
    let mut cache = ResultCache::new(100);
    let tracker = GenerationTracker::new();
    let key = make_key(0);

    let old_gen = tracker.increment_generation(key.clone());
    cache.put(
        key.clone(),
        "card_lookup".to_string(),
        old_gen,
        DerivedResult::Text("Old".to_string()),
    );

    // Row is edited, creating a new generation
    tracker.increment_generation(key.clone());

    // get_current uses the new generation, so the old entry won't be found
    let result = cache.get_current(&key, "card_lookup", &tracker);
    assert!(result.is_none(), "Stale generation should not return a result");
}

#[test]
fn test_get_current_returns_none_for_untracked_row() {
    let mut cache = ResultCache::new(100);
    let tracker = GenerationTracker::new();
    let key = make_key(0);

    // Row has generation 0 (untracked), so get_current should return None
    let result = cache.get_current(&key, "card_lookup", &tracker);
    assert!(result.is_none(), "Untracked row should not return a result");
}

#[test]
fn test_invalidate_nonexistent_row_is_safe() {
    let mut cache = ResultCache::new(100);
    cache.put(make_key(0), "f".to_string(), 1, DerivedResult::Text("A".to_string()));

    cache.invalidate_row(&make_key(99));

    assert_eq!(cache.len(), 1);
}

#[test]
fn test_invalidate_nonexistent_file_is_safe() {
    let mut cache = ResultCache::new(100);
    cache.put(make_key(0), "f".to_string(), 1, DerivedResult::Text("A".to_string()));

    cache.invalidate_file("/nonexistent/file.toml");

    assert_eq!(cache.len(), 1);
}

#[test]
fn test_invalidate_nonexistent_table_is_safe() {
    let mut cache = ResultCache::new(100);
    cache.put(make_key(0), "f".to_string(), 1, DerivedResult::Text("A".to_string()));

    cache.invalidate_table("/test/file.toml", "nonexistent");

    assert_eq!(cache.len(), 1);
}

#[test]
fn test_all_derived_result_variants_cacheable() {
    let mut cache = ResultCache::new(100);

    cache.put(make_key(0), "f".to_string(), 1, DerivedResult::Text("text".to_string()));
    cache.put(make_key(1), "f".to_string(), 1, DerivedResult::Number(42.0));
    cache.put(make_key(2), "f".to_string(), 1, DerivedResult::Boolean(true));
    cache.put(make_key(3), "f".to_string(), 1, DerivedResult::Image("url".to_string()));
    cache.put(make_key(4), "f".to_string(), 1, DerivedResult::Error("error".to_string()));
    cache.put(make_key(5), "f".to_string(), 1, DerivedResult::RichText(vec![]));

    assert_eq!(cache.len(), 6);
    assert_eq!(cache.get(&make_key(0), "f", 1), Some(&DerivedResult::Text("text".to_string())));
    assert_eq!(cache.get(&make_key(1), "f", 1), Some(&DerivedResult::Number(42.0)));
    assert_eq!(cache.get(&make_key(2), "f", 1), Some(&DerivedResult::Boolean(true)));
    assert_eq!(cache.get(&make_key(3), "f", 1), Some(&DerivedResult::Image("url".to_string())));
    assert_eq!(cache.get(&make_key(4), "f", 1), Some(&DerivedResult::Error("error".to_string())));
    assert_eq!(cache.get(&make_key(5), "f", 1), Some(&DerivedResult::RichText(vec![])));
}

#[test]
fn test_capacity_one_cache() {
    let mut cache = ResultCache::new(1);

    cache.put(make_key(0), "f".to_string(), 1, DerivedResult::Text("A".to_string()));
    assert_eq!(cache.len(), 1);

    cache.put(make_key(1), "f".to_string(), 1, DerivedResult::Text("B".to_string()));
    assert_eq!(cache.len(), 1);
    assert!(cache.get(&make_key(0), "f", 1).is_none());
    assert!(cache.get(&make_key(1), "f", 1).is_some());
}

#[test]
fn test_lru_eviction_sequence() {
    let mut cache = ResultCache::new(3);

    cache.put(make_key(0), "f".to_string(), 1, DerivedResult::Text("A".to_string()));
    cache.put(make_key(1), "f".to_string(), 1, DerivedResult::Text("B".to_string()));
    cache.put(make_key(2), "f".to_string(), 1, DerivedResult::Text("C".to_string()));

    // Access row 0, then row 1 - row 2 becomes LRU
    cache.get(&make_key(0), "f", 1);
    cache.get(&make_key(1), "f", 1);

    // Evict row 2
    cache.put(make_key(3), "f".to_string(), 1, DerivedResult::Text("D".to_string()));

    assert!(cache.get(&make_key(0), "f", 1).is_some());
    assert!(cache.get(&make_key(1), "f", 1).is_some());
    assert!(cache.get(&make_key(2), "f", 1).is_none(), "Row 2 should have been evicted as LRU");
    assert!(cache.get(&make_key(3), "f", 1).is_some());
}

#[test]
fn test_put_existing_key_does_not_increase_size() {
    let mut cache = ResultCache::new(100);
    let key = make_key(0);

    cache.put(key.clone(), "f".to_string(), 1, DerivedResult::Text("A".to_string()));
    cache.put(key.clone(), "f".to_string(), 1, DerivedResult::Text("B".to_string()));
    cache.put(key.clone(), "f".to_string(), 1, DerivedResult::Text("C".to_string()));

    assert_eq!(cache.len(), 1);
}

#[test]
fn test_different_files_are_separate() {
    let mut cache = ResultCache::new(100);
    let key1 = RowKey::new("/test/cards.toml", "cards", 0);
    let key2 = RowKey::new("/test/effects.toml", "effects", 0);

    cache.put(key1.clone(), "f".to_string(), 1, DerivedResult::Text("Card".to_string()));
    cache.put(key2.clone(), "f".to_string(), 1, DerivedResult::Text("Effect".to_string()));

    assert_eq!(cache.get(&key1, "f", 1), Some(&DerivedResult::Text("Card".to_string())));
    assert_eq!(cache.get(&key2, "f", 1), Some(&DerivedResult::Text("Effect".to_string())));
}

#[test]
fn test_fill_cache_to_exact_capacity() {
    let mut cache = ResultCache::new(5);

    for i in 0..5 {
        cache.put(make_key(i), "f".to_string(), 1, DerivedResult::Number(i as f64));
    }

    assert_eq!(cache.len(), 5);

    for i in 0..5 {
        assert_eq!(cache.get(&make_key(i), "f", 1), Some(&DerivedResult::Number(i as f64)));
    }
}

#[test]
fn test_cache_after_clear_can_be_reused() {
    let mut cache = ResultCache::new(100);

    cache.put(make_key(0), "f".to_string(), 1, DerivedResult::Text("Before".to_string()));
    cache.clear();

    cache.put(make_key(0), "f".to_string(), 1, DerivedResult::Text("After".to_string()));
    assert_eq!(cache.get(&make_key(0), "f", 1), Some(&DerivedResult::Text("After".to_string())));
    assert_eq!(cache.len(), 1);
}

#[test]
fn test_invalidate_row_then_reinsert() {
    let mut cache = ResultCache::new(100);
    let key = make_key(0);

    cache.put(key.clone(), "f".to_string(), 1, DerivedResult::Text("Before".to_string()));
    cache.invalidate_row(&key);
    assert!(cache.is_empty());

    cache.put(key.clone(), "f".to_string(), 2, DerivedResult::Text("After".to_string()));
    assert_eq!(cache.get(&key, "f", 2), Some(&DerivedResult::Text("After".to_string())));
}

#[test]
fn test_generation_change_makes_old_entry_unreachable() {
    let mut cache = ResultCache::new(100);
    let tracker = GenerationTracker::new();
    let key = make_key(0);

    let gen1 = tracker.increment_generation(key.clone());
    cache.put(key.clone(), "f".to_string(), gen1, DerivedResult::Text("Gen1".to_string()));

    // New generation makes old cache entry unreachable via get_current
    let gen2 = tracker.increment_generation(key.clone());
    assert!(cache.get_current(&key, "f", &tracker).is_none());

    // Insert with new generation
    cache.put(key.clone(), "f".to_string(), gen2, DerivedResult::Text("Gen2".to_string()));
    assert_eq!(
        cache.get_current(&key, "f", &tracker),
        Some(&DerivedResult::Text("Gen2".to_string()))
    );
}
