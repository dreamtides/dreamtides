use std::sync::Arc;

use tv_lib::images::image_cache::ImageCache;
use tv_lib::images::image_fetcher::{ImageFetcher, ImageFetcherState};

/// Minimal valid 1x1 red PNG image (67 bytes).
fn minimal_png() -> Vec<u8> {
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, // IHDR chunk length
        0x49, 0x48, 0x44, 0x52, // IHDR
        0x00, 0x00, 0x00, 0x01, // width: 1
        0x00, 0x00, 0x00, 0x01, // height: 1
        0x08, 0x02, // bit depth: 8, color type: RGB
        0x00, 0x00, 0x00, // compression, filter, interlace
        0x1E, 0x92, 0x6E, 0x05, // CRC
        0x00, 0x00, 0x00, 0x0C, // IDAT chunk length
        0x49, 0x44, 0x41, 0x54, // IDAT
        0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01, // compressed
        0xE2, 0x21, 0xBC, 0x33, // CRC
        0x00, 0x00, 0x00, 0x00, // IEND chunk length
        0x49, 0x45, 0x4E, 0x44, // IEND
        0xAE, 0x42, 0x60, 0x82, // CRC
    ]
}

#[test]
fn test_state_new_returns_uninitialized() {
    let state = ImageFetcherState::new();
    assert!(state.get().is_none(), "New state should not have a fetcher");
}

#[test]
fn test_state_default_returns_uninitialized() {
    let state = ImageFetcherState::default();
    assert!(state.get().is_none(), "Default state should not have a fetcher");
}

#[test]
fn test_state_get_returns_none_before_initialize() {
    let state = ImageFetcherState::new();
    assert!(state.get().is_none());
    assert!(state.get().is_none(), "Multiple get calls should all return None");
}

#[test]
fn test_state_initialize_creates_cache_directory() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache_subdir = temp.path().join("nested").join("cache");
    assert!(!cache_subdir.exists(), "Cache directory should not exist yet");

    let _state = ImageFetcherState::new();
    let cache = Arc::new(ImageCache::new(&cache_subdir).expect("Should create cache"));
    let fetcher = ImageFetcher::new(cache);
    assert!(fetcher.is_ok(), "Should create fetcher for new cache dir");
}

#[test]
fn test_fetcher_from_cache_can_fetch_local_image() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = Arc::new(ImageCache::new(temp.path()).expect("Should create cache"));
    let fetcher = ImageFetcher::new(Arc::clone(&cache)).expect("Should create fetcher");

    let image_dir = tempfile::tempdir().expect("Should create image dir");
    let image_path = image_dir.path().join("test.png");
    std::fs::write(&image_path, minimal_png()).expect("Should write test image");

    let rt = tokio::runtime::Runtime::new().expect("Should create runtime");
    let result = rt.block_on(fetcher.fetch(image_path.to_str().unwrap()));
    assert!(result.is_ok(), "Should fetch local image: {result:?}");
    assert_eq!(result.unwrap(), image_path);
}

#[test]
fn test_fetcher_cache_integration_put_then_fetch() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = Arc::new(ImageCache::new(temp.path()).expect("Should create cache"));
    let fetcher = ImageFetcher::new(Arc::clone(&cache)).expect("Should create fetcher");

    let url = "https://example.com/cached-image.png";
    let cached_path = cache.put(url, &minimal_png()).expect("Should store in cache");

    let rt = tokio::runtime::Runtime::new().expect("Should create runtime");
    let result = rt.block_on(fetcher.fetch(url));
    assert!(result.is_ok(), "Should return cached image: {result:?}");
    assert_eq!(result.unwrap(), cached_path, "Should return the cached file path");
}

#[test]
fn test_fetcher_cache_returns_same_reference() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = Arc::new(ImageCache::new(temp.path()).expect("Should create cache"));
    let fetcher = ImageFetcher::new(Arc::clone(&cache)).expect("Should create fetcher");

    let url = "https://example.com/ref-test.png";
    fetcher.cache().put(url, &minimal_png()).expect("Should store via cache ref");
    assert!(fetcher.cache().contains(url), "Cache ref should see stored entry");
    assert!(cache.contains(url), "Original cache should see stored entry");
}

#[test]
fn test_fetcher_with_config_respects_custom_timeout() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = Arc::new(ImageCache::new(temp.path()).expect("Should create cache"));
    let fetcher = ImageFetcher::with_config(cache, std::time::Duration::from_millis(500), 1);
    assert!(fetcher.is_ok(), "Should create fetcher with short timeout");
}

#[test]
fn test_fetcher_with_config_respects_custom_concurrency() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = Arc::new(ImageCache::new(temp.path()).expect("Should create cache"));
    let fetcher = ImageFetcher::with_config(cache, std::time::Duration::from_secs(10), 8);
    assert!(fetcher.is_ok(), "Should create fetcher with custom concurrency");
}

#[tokio::test]
async fn test_fetcher_multiple_local_fetches_independent() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = Arc::new(ImageCache::new(temp.path()).expect("Should create cache"));
    let fetcher = ImageFetcher::new(Arc::clone(&cache)).expect("Should create fetcher");

    let image_dir = tempfile::tempdir().expect("Should create image dir");
    let image_a = image_dir.path().join("a.png");
    let image_b = image_dir.path().join("b.png");
    std::fs::write(&image_a, minimal_png()).expect("Should write image a");
    std::fs::write(&image_b, minimal_png()).expect("Should write image b");

    let result_a = fetcher.fetch(image_a.to_str().unwrap()).await;
    let result_b = fetcher.fetch(image_b.to_str().unwrap()).await;
    assert!(result_a.is_ok(), "Should fetch image a: {result_a:?}");
    assert!(result_b.is_ok(), "Should fetch image b: {result_b:?}");
    assert_ne!(
        result_a.unwrap(),
        result_b.unwrap(),
        "Different images should return different paths"
    );
}

#[tokio::test]
async fn test_fetcher_local_fetch_does_not_populate_cache() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = Arc::new(ImageCache::new(temp.path()).expect("Should create cache"));
    let fetcher = ImageFetcher::new(Arc::clone(&cache)).expect("Should create fetcher");

    let image_dir = tempfile::tempdir().expect("Should create image dir");
    let image_path = image_dir.path().join("nocache.png");
    std::fs::write(&image_path, minimal_png()).expect("Should write image");

    fetcher.fetch(image_path.to_str().unwrap()).await.expect("Should fetch");
    assert_eq!(cache.entry_count(), 0, "Local fetch should not add to cache");
}

#[tokio::test]
async fn test_fetcher_remote_cache_miss_for_unreachable_returns_error() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = Arc::new(ImageCache::new(temp.path()).expect("Should create cache"));
    let fetcher = ImageFetcher::with_config(cache, std::time::Duration::from_secs(2), 1)
        .expect("Should create fetcher");

    let result =
        fetcher.fetch("https://invalid-host-that-does-not-exist.example.invalid/img.png").await;
    assert!(result.is_err(), "Should fail for unreachable host");
}

#[tokio::test]
async fn test_fetcher_remote_cache_hit_avoids_network() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = Arc::new(ImageCache::new(temp.path()).expect("Should create cache"));

    let url = "https://unreachable-but-cached.example.invalid/image.png";
    let expected_path = cache.put(url, &minimal_png()).expect("Should pre-populate");

    let fetcher =
        ImageFetcher::with_config(Arc::clone(&cache), std::time::Duration::from_millis(100), 1)
            .expect("Should create fetcher");

    let result = fetcher.fetch(url).await;
    assert!(result.is_ok(), "Cached URL should return without network: {result:?}");
    assert_eq!(result.unwrap(), expected_path, "Should return cached path");
}

#[tokio::test]
async fn test_fetcher_nonexistent_local_path_returns_file_not_found() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = Arc::new(ImageCache::new(temp.path()).expect("Should create cache"));
    let fetcher = ImageFetcher::new(cache).expect("Should create fetcher");

    let result = fetcher.fetch("/nonexistent/path/to/image.png").await;
    assert!(result.is_err(), "Should fail for nonexistent file");
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("not found") || err_msg.contains("nonexistent"),
        "Error should indicate file not found: {err_msg}"
    );
}

#[tokio::test]
async fn test_fetcher_local_invalid_data_returns_error() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = Arc::new(ImageCache::new(temp.path()).expect("Should create cache"));
    let fetcher = ImageFetcher::new(cache).expect("Should create fetcher");

    let image_dir = tempfile::tempdir().expect("Should create dir");
    let bad_path = image_dir.path().join("bad.png");
    std::fs::write(&bad_path, b"not valid image data").expect("Should write file");

    let result = fetcher.fetch(bad_path.to_str().unwrap()).await;
    assert!(result.is_err(), "Should fail for invalid image data");
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("image format") || err_msg.contains("Unrecognized"),
        "Error should mention image format: {err_msg}"
    );
}

#[tokio::test]
async fn test_fetcher_empty_local_file_returns_error() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = Arc::new(ImageCache::new(temp.path()).expect("Should create cache"));
    let fetcher = ImageFetcher::new(cache).expect("Should create fetcher");

    let image_dir = tempfile::tempdir().expect("Should create dir");
    let empty_path = image_dir.path().join("empty.png");
    std::fs::write(&empty_path, b"").expect("Should write empty file");

    let result = fetcher.fetch(empty_path.to_str().unwrap()).await;
    assert!(result.is_err(), "Should fail for empty file");
}

#[tokio::test]
async fn test_fetcher_concurrent_cache_hits() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = Arc::new(ImageCache::new(temp.path()).expect("Should create cache"));

    let url = "https://example.com/concurrent-test.png";
    let expected_path = cache.put(url, &minimal_png()).expect("Should store");

    let fetcher = Arc::new(ImageFetcher::new(Arc::clone(&cache)).expect("Should create fetcher"));

    let mut handles = Vec::new();
    for _ in 0..5 {
        let fetcher_clone = Arc::clone(&fetcher);
        let url_clone = url.to_string();
        handles.push(tokio::spawn(async move { fetcher_clone.fetch(&url_clone).await }));
    }

    for handle in handles {
        let result = handle.await.expect("Task should not panic");
        assert!(result.is_ok(), "Concurrent fetch should succeed: {result:?}");
        assert_eq!(result.unwrap(), expected_path, "All should return same cached path");
    }
}

#[tokio::test]
async fn test_fetcher_local_path_with_spaces() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = Arc::new(ImageCache::new(temp.path()).expect("Should create cache"));
    let fetcher = ImageFetcher::new(cache).expect("Should create fetcher");

    let image_dir = tempfile::tempdir().expect("Should create dir");
    let nested = image_dir.path().join("path with spaces");
    std::fs::create_dir_all(&nested).expect("Should create dir");
    let image_path = nested.join("my image.png");
    std::fs::write(&image_path, minimal_png()).expect("Should write image");

    let result = fetcher.fetch(image_path.to_str().unwrap()).await;
    assert!(result.is_ok(), "Should handle spaces in path: {result:?}");
}

#[tokio::test]
async fn test_fetcher_local_absolute_path() {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = Arc::new(ImageCache::new(temp.path()).expect("Should create cache"));
    let fetcher = ImageFetcher::new(cache).expect("Should create fetcher");

    let image_dir = tempfile::tempdir().expect("Should create dir");
    let image_path = image_dir.path().join("absolute.png");
    std::fs::write(&image_path, minimal_png()).expect("Should write image");

    let abs_path = image_path.canonicalize().expect("Should canonicalize");
    let result = fetcher.fetch(abs_path.to_str().unwrap()).await;
    assert!(result.is_ok(), "Should fetch via absolute path: {result:?}");
}
