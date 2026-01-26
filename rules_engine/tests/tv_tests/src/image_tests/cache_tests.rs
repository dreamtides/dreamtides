use tv_lib::images::image_cache::ImageCache;

fn make_data(size: usize) -> Vec<u8> {
    vec![0xAB; size]
}

#[test]
fn test_cache_new_creates_directory() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let _cache = ImageCache::new(temp.path()).expect("Should create cache");
    assert!(temp.path().join("image_cache").exists(), "Cache directory should be created");
}

#[test]
fn test_cache_with_max_size() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache =
        ImageCache::with_max_size(temp.path(), 1024).expect("Should create cache with custom size");
    assert_eq!(cache.total_size(), 0, "New cache should have zero size");
    assert_eq!(cache.entry_count(), 0, "New cache should have zero entries");
}

#[test]
fn test_cache_put_and_get() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = ImageCache::new(temp.path()).expect("Should create cache");
    let data = make_data(100);
    let url = "https://example.com/image.png";

    let cached_path = cache.put(url, &data).expect("Should store image");
    assert!(cached_path.exists(), "Cached file should exist on disk");

    let retrieved_path = cache.get(url);
    assert!(retrieved_path.is_some(), "Should find cached image");
    assert_eq!(retrieved_path.unwrap(), cached_path, "Retrieved path should match stored path");
}

#[test]
fn test_cache_get_miss() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = ImageCache::new(temp.path()).expect("Should create cache");

    let result = cache.get("https://example.com/nonexistent.png");
    assert!(result.is_none(), "Cache miss should return None");
}

#[test]
fn test_cache_contains() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = ImageCache::new(temp.path()).expect("Should create cache");
    let url = "https://example.com/image.png";

    assert!(!cache.contains(url), "Should not contain uncached URL");

    cache.put(url, &make_data(50)).expect("Should store image");
    assert!(cache.contains(url), "Should contain cached URL");
}

#[test]
fn test_cache_total_size_tracks_entries() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = ImageCache::new(temp.path()).expect("Should create cache");

    cache.put("https://example.com/a.png", &make_data(100)).expect("Should store first");
    assert_eq!(cache.total_size(), 100, "Total size should be 100 after first entry");

    cache.put("https://example.com/b.png", &make_data(200)).expect("Should store second");
    assert_eq!(cache.total_size(), 300, "Total size should be 300 after second entry");
}

#[test]
fn test_cache_entry_count() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = ImageCache::new(temp.path()).expect("Should create cache");

    assert_eq!(cache.entry_count(), 0, "Empty cache should have 0 entries");

    cache.put("https://example.com/a.png", &make_data(50)).expect("Should store first");
    assert_eq!(cache.entry_count(), 1, "Should have 1 entry after first put");

    cache.put("https://example.com/b.png", &make_data(50)).expect("Should store second");
    assert_eq!(cache.entry_count(), 2, "Should have 2 entries after second put");
}

#[test]
fn test_cache_put_overwrites_existing() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = ImageCache::new(temp.path()).expect("Should create cache");
    let url = "https://example.com/image.png";

    cache.put(url, &make_data(100)).expect("Should store first version");
    assert_eq!(cache.total_size(), 100, "Size should be 100");

    cache.put(url, &make_data(200)).expect("Should overwrite with second version");
    assert_eq!(cache.entry_count(), 1, "Should still have 1 entry after overwrite");
    assert_eq!(cache.total_size(), 200, "Size should be 200 after overwrite");
}

#[test]
fn test_cache_get_data() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = ImageCache::new(temp.path()).expect("Should create cache");
    let url = "https://example.com/image.png";
    let data = make_data(100);

    cache.put(url, &data).expect("Should store image");

    let retrieved = cache.get_data(url).expect("Should read data");
    assert!(retrieved.is_some(), "Should find cached data");
    assert_eq!(retrieved.unwrap(), data, "Retrieved data should match stored data");
}

#[test]
fn test_cache_get_data_miss() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = ImageCache::new(temp.path()).expect("Should create cache");

    let result = cache.get_data("https://example.com/nonexistent.png").expect("Should not error");
    assert!(result.is_none(), "Cache miss should return None for data");
}

#[test]
fn test_cache_remove() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = ImageCache::new(temp.path()).expect("Should create cache");
    let url = "https://example.com/image.png";

    cache.put(url, &make_data(100)).expect("Should store image");
    assert!(cache.contains(url), "Should contain cached URL before remove");

    let removed = cache.remove(url).expect("Should remove successfully");
    assert!(removed, "Remove should return true for existing entry");
    assert!(!cache.contains(url), "Should not contain URL after remove");
    assert_eq!(cache.entry_count(), 0, "Entry count should be 0 after remove");
    assert_eq!(cache.total_size(), 0, "Total size should be 0 after remove");
}

#[test]
fn test_cache_remove_nonexistent() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = ImageCache::new(temp.path()).expect("Should create cache");

    let removed = cache.remove("https://example.com/nonexistent.png").expect("Should not error");
    assert!(!removed, "Remove should return false for nonexistent entry");
}

#[test]
fn test_cache_clear() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = ImageCache::new(temp.path()).expect("Should create cache");

    cache.put("https://example.com/a.png", &make_data(100)).expect("Should store first");
    cache.put("https://example.com/b.png", &make_data(200)).expect("Should store second");
    cache.put("https://example.com/c.png", &make_data(300)).expect("Should store third");

    assert_eq!(cache.entry_count(), 3, "Should have 3 entries before clear");

    cache.clear().expect("Should clear successfully");
    assert_eq!(cache.entry_count(), 0, "Should have 0 entries after clear");
    assert_eq!(cache.total_size(), 0, "Should have 0 size after clear");
    assert!(!cache.contains("https://example.com/a.png"), "Should not contain a after clear");
    assert!(!cache.contains("https://example.com/b.png"), "Should not contain b after clear");
    assert!(!cache.contains("https://example.com/c.png"), "Should not contain c after clear");
}

#[test]
fn test_cache_url_to_cache_key_deterministic() {
    let url = "https://example.com/image.png";
    let key1 = ImageCache::url_to_cache_key(url);
    let key2 = ImageCache::url_to_cache_key(url);
    assert_eq!(key1, key2, "Same URL should produce same cache key");
}

#[test]
fn test_cache_url_to_cache_key_different_urls() {
    let key1 = ImageCache::url_to_cache_key("https://example.com/a.png");
    let key2 = ImageCache::url_to_cache_key("https://example.com/b.png");
    assert_ne!(key1, key2, "Different URLs should produce different cache keys");
}

#[test]
fn test_cache_url_to_cache_key_is_hex() {
    let key = ImageCache::url_to_cache_key("https://example.com/image.png");
    assert!(key.chars().all(|c| c.is_ascii_hexdigit()), "Cache key should be hexadecimal");
    assert_eq!(key.len(), 64, "SHA-256 hex should be 64 characters");
}

#[test]
fn test_lru_eviction_when_cache_full() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = ImageCache::with_max_size(temp.path(), 250).expect("Should create small cache");

    cache.put("https://example.com/first.png", &make_data(100)).expect("Should store first");
    cache.put("https://example.com/second.png", &make_data(100)).expect("Should store second");

    assert_eq!(cache.entry_count(), 2, "Should have 2 entries before eviction");

    cache
        .put("https://example.com/third.png", &make_data(100))
        .expect("Should store third, evicting first");

    assert!(cache.contains("https://example.com/third.png"), "Third should be cached");
    assert_eq!(cache.entry_count(), 2, "Should have at most 2 entries after eviction");
    assert!(cache.total_size() <= 250, "Total size should be within limit");
}

#[test]
fn test_lru_eviction_removes_least_recently_used() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = ImageCache::with_max_size(temp.path(), 250).expect("Should create small cache");

    cache.put("https://example.com/first.png", &make_data(100)).expect("Should store first");
    std::thread::sleep(std::time::Duration::from_secs(1));

    cache.put("https://example.com/second.png", &make_data(100)).expect("Should store second");
    std::thread::sleep(std::time::Duration::from_secs(1));

    cache.get("https://example.com/first.png");
    std::thread::sleep(std::time::Duration::from_secs(1));

    cache.put("https://example.com/third.png", &make_data(100)).expect("Should store third");

    assert!(
        cache.contains("https://example.com/first.png"),
        "First should remain (recently accessed)"
    );
    assert!(cache.contains("https://example.com/third.png"), "Third should be cached (just added)");
    assert!(
        !cache.contains("https://example.com/second.png"),
        "Second should be evicted (least recently used)"
    );
}

#[test]
fn test_lru_eviction_large_entry_evicts_multiple() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = ImageCache::with_max_size(temp.path(), 500).expect("Should create cache");

    cache.put("https://example.com/a.png", &make_data(100)).expect("Should store a");
    std::thread::sleep(std::time::Duration::from_secs(1));
    cache.put("https://example.com/b.png", &make_data(100)).expect("Should store b");
    std::thread::sleep(std::time::Duration::from_secs(1));
    cache.put("https://example.com/c.png", &make_data(100)).expect("Should store c");
    std::thread::sleep(std::time::Duration::from_secs(1));

    assert_eq!(cache.entry_count(), 3, "Should have 3 entries");

    cache.put("https://example.com/big.png", &make_data(400)).expect("Should store big entry");

    assert!(cache.contains("https://example.com/big.png"), "Big entry should be cached");
    assert!(cache.total_size() <= 500, "Total size should be within limit");
}

#[test]
fn test_cache_content_addressed_same_url_same_key() {
    let url = "https://example.com/image.png";
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = ImageCache::new(temp.path()).expect("Should create cache");

    let path1 = cache.put(url, &make_data(100)).expect("Should store first time");
    let path2 = cache.put(url, &make_data(100)).expect("Should store second time (overwrite)");

    assert_eq!(path1, path2, "Same URL should map to same cache file path");
    assert_eq!(cache.entry_count(), 1, "Should still have only 1 entry");
}

#[test]
fn test_cache_different_urls_different_files() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = ImageCache::new(temp.path()).expect("Should create cache");

    let path1 = cache.put("https://example.com/a.png", &make_data(50)).expect("Should store a");
    let path2 = cache.put("https://example.com/b.png", &make_data(50)).expect("Should store b");

    assert_ne!(path1, path2, "Different URLs should map to different paths");
}

#[test]
fn test_cache_persists_across_instances() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let url = "https://example.com/image.png";
    let data = make_data(100);

    {
        let cache = ImageCache::new(temp.path()).expect("Should create first cache");
        cache.put(url, &data).expect("Should store image");
    }

    let cache = ImageCache::new(temp.path()).expect("Should create second cache");
    assert!(cache.contains(url), "Cache should persist across instances");
    assert_eq!(cache.entry_count(), 1, "Entry count should persist");
    assert_eq!(cache.total_size(), 100, "Total size should persist");

    let retrieved = cache.get_data(url).expect("Should read data");
    assert_eq!(retrieved.unwrap(), data, "Persisted data should match original");
}

#[test]
fn test_validate_integrity_clean_cache() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = ImageCache::new(temp.path()).expect("Should create cache");

    cache.put("https://example.com/a.png", &make_data(100)).expect("Should store a");
    cache.put("https://example.com/b.png", &make_data(200)).expect("Should store b");

    let result = cache.validate_integrity().expect("Should validate successfully");
    assert_eq!(result.orphaned_files_removed, 0, "Clean cache should have no orphans");
    assert_eq!(result.missing_entries_removed, 0, "Clean cache should have no missing entries");
    assert!(!result.size_corrected, "Clean cache should not need size correction");
}

#[test]
fn test_validate_integrity_removes_orphaned_files() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = ImageCache::new(temp.path()).expect("Should create cache");

    let cache_dir = temp.path().join("image_cache");
    std::fs::write(cache_dir.join("orphan_file"), b"orphan data").expect("Should create orphan");

    let result = cache.validate_integrity().expect("Should validate successfully");
    assert_eq!(result.orphaned_files_removed, 1, "Should remove 1 orphaned file");
    assert!(!cache_dir.join("orphan_file").exists(), "Orphaned file should be removed");
}

#[test]
fn test_validate_integrity_removes_missing_entries() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let url = "https://example.com/image.png";

    let cache = ImageCache::new(temp.path()).expect("Should create cache");
    let cached_path = cache.put(url, &make_data(100)).expect("Should store image");

    std::fs::remove_file(&cached_path).expect("Should delete cached file");

    let result = cache.validate_integrity().expect("Should validate successfully");
    assert_eq!(result.missing_entries_removed, 1, "Should remove 1 missing entry");
    assert!(!cache.contains(url), "Should no longer contain removed entry");
    assert_eq!(cache.entry_count(), 0, "Should have 0 entries after cleanup");
    assert_eq!(cache.total_size(), 0, "Should have 0 size after cleanup");
}

#[test]
fn test_cache_empty_data() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = ImageCache::new(temp.path()).expect("Should create cache");
    let url = "https://example.com/empty.png";

    let path = cache.put(url, &[]).expect("Should store empty data");
    assert!(path.exists(), "Empty file should exist");

    let data = cache.get_data(url).expect("Should read data").expect("Should find entry");
    assert!(data.is_empty(), "Retrieved data should be empty");
    assert_eq!(cache.total_size(), 0, "Total size should be 0 for empty data");
}

#[test]
fn test_cache_large_data() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = ImageCache::new(temp.path()).expect("Should create cache");
    let url = "https://example.com/large.png";
    let data = make_data(1_000_000);

    cache.put(url, &data).expect("Should store large data");
    assert_eq!(cache.total_size(), 1_000_000, "Size should reflect large data");

    let retrieved =
        cache.get_data(url).expect("Should read large data").expect("Should find entry");
    assert_eq!(retrieved.len(), 1_000_000, "Retrieved data size should match");
}

#[test]
fn test_cache_special_characters_in_url() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = ImageCache::new(temp.path()).expect("Should create cache");
    let url = "https://example.com/path?query=value&other=foo#fragment";

    cache.put(url, &make_data(50)).expect("Should store with special chars URL");
    assert!(cache.contains(url), "Should find URL with special characters");
}

#[test]
fn test_cache_unicode_url() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = ImageCache::new(temp.path()).expect("Should create cache");
    let url = "https://example.com/\u{1F600}/image.png";

    cache.put(url, &make_data(50)).expect("Should store with unicode URL");
    assert!(cache.contains(url), "Should find URL with unicode");
}

#[test]
fn test_cache_multiple_operations_sequence() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = ImageCache::new(temp.path()).expect("Should create cache");

    cache.put("https://example.com/a.png", &make_data(100)).expect("Should store a");
    cache.put("https://example.com/b.png", &make_data(200)).expect("Should store b");
    cache.put("https://example.com/c.png", &make_data(300)).expect("Should store c");

    assert_eq!(cache.entry_count(), 3, "Should have 3 entries");
    assert_eq!(cache.total_size(), 600, "Total size should be 600");

    cache.remove("https://example.com/b.png").expect("Should remove b");
    assert_eq!(cache.entry_count(), 2, "Should have 2 entries after remove");
    assert_eq!(cache.total_size(), 400, "Total size should be 400 after remove");

    cache.put("https://example.com/d.png", &make_data(150)).expect("Should store d");
    assert_eq!(cache.entry_count(), 3, "Should have 3 entries after adding d");
    assert_eq!(cache.total_size(), 550, "Total size should be 550");

    cache.clear().expect("Should clear");
    assert_eq!(cache.entry_count(), 0, "Should have 0 entries after clear");
    assert_eq!(cache.total_size(), 0, "Should have 0 size after clear");
}

#[test]
fn test_cache_put_returns_path_under_cache_dir() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = ImageCache::new(temp.path()).expect("Should create cache");
    let url = "https://example.com/image.png";

    let path = cache.put(url, &make_data(50)).expect("Should store image");
    let cache_dir = temp.path().join("image_cache");
    assert!(path.starts_with(&cache_dir), "Cached file should be under cache directory");
}

#[test]
fn test_cache_key_matches_filename() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = ImageCache::new(temp.path()).expect("Should create cache");
    let url = "https://example.com/image.png";

    let path = cache.put(url, &make_data(50)).expect("Should store image");
    let expected_key = ImageCache::url_to_cache_key(url);
    let file_name = path.file_name().expect("Should have file name").to_string_lossy();

    assert_eq!(file_name.as_ref(), expected_key.as_str(), "File name should match cache key");
}
