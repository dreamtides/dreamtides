use std::sync::Arc;

use tv_lib::images::image_cache::ImageCache;
use tv_lib::images::image_fetcher::ImageFetcher;

fn create_cache() -> (tempfile::TempDir, Arc<ImageCache>) {
    let temp = tempfile::tempdir().expect("Should create temp dir");
    let cache = Arc::new(ImageCache::new(temp.path()).expect("Should create cache"));
    (temp, cache)
}

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
fn test_fetcher_new_creates_successfully() {
    let (_temp, cache) = create_cache();
    let fetcher = ImageFetcher::new(cache);
    assert!(fetcher.is_ok(), "Should create fetcher with default settings");
}

#[test]
fn test_fetcher_with_config_creates_successfully() {
    let (_temp, cache) = create_cache();
    let fetcher = ImageFetcher::with_config(cache, std::time::Duration::from_secs(10), 2);
    assert!(fetcher.is_ok(), "Should create fetcher with custom config");
}

#[tokio::test]
async fn test_fetch_local_valid_image() {
    let (_temp, cache) = create_cache();
    let fetcher = ImageFetcher::new(cache).expect("Should create fetcher");

    let image_dir = tempfile::tempdir().expect("Should create image dir");
    let image_path = image_dir.path().join("test.png");
    std::fs::write(&image_path, minimal_png()).expect("Should write test image");

    let result = fetcher.fetch(image_path.to_str().unwrap()).await;
    assert!(result.is_ok(), "Should successfully read local image: {result:?}");
    assert_eq!(result.unwrap(), image_path, "Should return the original local path");
}

#[tokio::test]
async fn test_fetch_local_nonexistent_file() {
    let (_temp, cache) = create_cache();
    let fetcher = ImageFetcher::new(cache).expect("Should create fetcher");

    let result = fetcher.fetch("/nonexistent/path/image.png").await;
    assert!(result.is_err(), "Should fail for nonexistent file");
    let err = result.unwrap_err();
    assert!(
        format!("{err}").contains("not found") || format!("{err}").contains("nonexistent"),
        "Error should indicate file not found: {err}"
    );
}

#[tokio::test]
async fn test_fetch_local_invalid_image_data() {
    let (_temp, cache) = create_cache();
    let fetcher = ImageFetcher::new(cache).expect("Should create fetcher");

    let image_dir = tempfile::tempdir().expect("Should create dir");
    let invalid_path = image_dir.path().join("invalid.png");
    std::fs::write(&invalid_path, b"not an image at all").expect("Should write file");

    let result = fetcher.fetch(invalid_path.to_str().unwrap()).await;
    assert!(result.is_err(), "Should fail for invalid image data");
    let err = result.unwrap_err();
    assert!(
        format!("{err}").contains("image format") || format!("{err}").contains("Unrecognized"),
        "Error should mention image format issue: {err}"
    );
}

#[tokio::test]
async fn test_fetch_local_does_not_cache() {
    let (_temp, cache) = create_cache();
    let fetcher = ImageFetcher::new(Arc::clone(&cache)).expect("Should create fetcher");

    let image_dir = tempfile::tempdir().expect("Should create dir");
    let image_path = image_dir.path().join("local.png");
    std::fs::write(&image_path, minimal_png()).expect("Should write test image");

    let path_str = image_path.to_str().unwrap();
    fetcher.fetch(path_str).await.expect("Should fetch local image");

    assert_eq!(cache.entry_count(), 0, "Local files should not be added to cache");
    assert!(!cache.contains(path_str), "Cache should not contain local path");
}

#[tokio::test]
async fn test_fetch_remote_cache_hit() {
    let (_temp, cache) = create_cache();
    let fetcher = ImageFetcher::new(Arc::clone(&cache)).expect("Should create fetcher");

    let url = "https://example.com/image.png";
    let data = minimal_png();
    let expected_path = cache.put(url, &data).expect("Should pre-populate cache");

    let result = fetcher.fetch(url).await;
    assert!(result.is_ok(), "Should return cached image: {result:?}");
    assert_eq!(result.unwrap(), expected_path, "Should return cached path");
}

#[tokio::test]
async fn test_fetch_remote_unreachable_host() {
    let (_temp, cache) = create_cache();
    let fetcher = ImageFetcher::with_config(cache, std::time::Duration::from_secs(2), 1)
        .expect("Should create fetcher");

    let result =
        fetcher.fetch("https://invalid-host-that-does-not-exist.example.invalid/image.png").await;
    assert!(result.is_err(), "Should fail for unreachable host");
}

#[tokio::test]
async fn test_fetch_cache_reference() {
    let (_temp, cache) = create_cache();
    let fetcher = ImageFetcher::new(Arc::clone(&cache)).expect("Should create fetcher");
    let fetcher_cache = fetcher.cache();

    let url = "https://example.com/test.png";
    fetcher_cache.put(url, &minimal_png()).expect("Should store via cache ref");
    assert!(fetcher_cache.contains(url), "Cache ref should reflect stored entry");
}

#[tokio::test]
async fn test_fetch_local_absolute_path() {
    let (_temp, cache) = create_cache();
    let fetcher = ImageFetcher::new(cache).expect("Should create fetcher");

    let image_dir = tempfile::tempdir().expect("Should create dir");
    let image_path = image_dir.path().join("absolute.png");
    std::fs::write(&image_path, minimal_png()).expect("Should write image");

    let abs_path = image_path.canonicalize().expect("Should canonicalize");
    let result = fetcher.fetch(abs_path.to_str().unwrap()).await;
    assert!(result.is_ok(), "Should fetch image via absolute path: {result:?}");
}

#[tokio::test]
async fn test_fetch_local_with_spaces_in_path() {
    let (_temp, cache) = create_cache();
    let fetcher = ImageFetcher::new(cache).expect("Should create fetcher");

    let image_dir = tempfile::tempdir().expect("Should create dir");
    let nested = image_dir.path().join("path with spaces");
    std::fs::create_dir_all(&nested).expect("Should create nested dir");
    let image_path = nested.join("test image.png");
    std::fs::write(&image_path, minimal_png()).expect("Should write image");

    let result = fetcher.fetch(image_path.to_str().unwrap()).await;
    assert!(result.is_ok(), "Should handle spaces in path: {result:?}");
}

#[tokio::test]
async fn test_fetch_local_empty_file() {
    let (_temp, cache) = create_cache();
    let fetcher = ImageFetcher::new(cache).expect("Should create fetcher");

    let image_dir = tempfile::tempdir().expect("Should create dir");
    let empty_path = image_dir.path().join("empty.png");
    std::fs::write(&empty_path, b"").expect("Should write empty file");

    let result = fetcher.fetch(empty_path.to_str().unwrap()).await;
    assert!(result.is_err(), "Should fail for empty file");
}
