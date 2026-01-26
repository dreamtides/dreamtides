use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Semaphore;

use crate::error::error_types::TvError;
use crate::images::image_cache::ImageCache;

/// Default timeout for HTTP image fetches.
const DEFAULT_FETCH_TIMEOUT: Duration = Duration::from_secs(30);

/// Default maximum concurrent fetches.
const DEFAULT_MAX_CONCURRENT_FETCHES: usize = 4;

/// Async image fetcher with HTTP support, image validation, and cache integration.
///
/// Fetches images from remote URLs, validates them using the `image` crate,
/// stores them in the content-addressed cache, and returns local file paths.
/// For local filesystem paths, reads directly without caching.
pub struct ImageFetcher {
    cache: Arc<ImageCache>,
    client: reqwest::Client,
    semaphore: Arc<Semaphore>,
}

impl ImageFetcher {
    /// Creates a new image fetcher with the given cache and default settings.
    pub fn new(cache: Arc<ImageCache>) -> Result<Self, TvError> {
        Self::with_config(cache, DEFAULT_FETCH_TIMEOUT, DEFAULT_MAX_CONCURRENT_FETCHES)
    }

    /// Creates a new image fetcher with custom timeout and concurrency settings.
    pub fn with_config(
        cache: Arc<ImageCache>,
        timeout: Duration,
        max_concurrent: usize,
    ) -> Result<Self, TvError> {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| TvError::ImageFetchError {
                url: String::new(),
                message: format!("Failed to create HTTP client: {e}"),
            })?;

        Ok(Self {
            cache,
            client,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
        })
    }

    /// Fetches an image by URL, returning the local cache file path.
    ///
    /// Checks the cache first and returns immediately on hit. On cache miss,
    /// fetches the image over HTTP, validates it, stores it in the cache,
    /// and returns the cached file path. Uses a semaphore to limit concurrent
    /// network requests.
    pub async fn fetch(&self, url: &str) -> Result<PathBuf, TvError> {
        if let Some(cached_path) = self.cache.get(url) {
            tracing::debug!(
                component = "tv.images.fetcher",
                url = %url,
                "Cache hit"
            );
            return Ok(cached_path);
        }

        let _permit = self.semaphore.acquire().await.map_err(|_| TvError::ImageFetchError {
            url: url.to_string(),
            message: "Fetch semaphore closed".to_string(),
        })?;

        // Double-check cache after acquiring permit (another fetch may have completed)
        if let Some(cached_path) = self.cache.get(url) {
            tracing::debug!(
                component = "tv.images.fetcher",
                url = %url,
                "Cache hit after semaphore acquire"
            );
            return Ok(cached_path);
        }

        tracing::info!(
            component = "tv.images.fetcher",
            url = %url,
            "Fetching image from remote URL"
        );

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| map_reqwest_error(url, &e))?;

        let status = response.status();
        if !status.is_success() {
            return Err(TvError::ImageFetchError {
                url: url.to_string(),
                message: format!("HTTP {status}"),
            });
        }

        let bytes = response.bytes().await.map_err(|e| TvError::ImageFetchError {
            url: url.to_string(),
            message: format!("Failed to read response body: {e}"),
        })?;

        validate_image_data(url, &bytes)?;

        let cached_path = self.cache.put(url, &bytes)?;

        tracing::info!(
            component = "tv.images.fetcher",
            url = %url,
            size = bytes.len(),
            "Image fetched and cached"
        );

        Ok(cached_path)
    }

    /// Returns a reference to the underlying cache.
    #[allow(dead_code)]
    pub fn cache(&self) -> &ImageCache {
        &self.cache
    }
}

/// Tauri-managed state wrapper for the image fetcher.
///
/// Provides thread-safe access to a shared ImageFetcher instance.
pub struct ImageFetcherState {
    fetcher: std::sync::RwLock<Option<Arc<ImageFetcher>>>,
}

impl ImageFetcherState {
    /// Creates a new uninitialized state.
    pub fn new() -> Self {
        Self { fetcher: std::sync::RwLock::new(None) }
    }

    /// Initializes the fetcher with the given cache directory.
    pub fn initialize(&self, cache_dir: &std::path::Path) -> Result<(), TvError> {
        let cache = ImageCache::new(cache_dir)?;
        cache.validate_integrity()?;
        let fetcher = ImageFetcher::new(Arc::new(cache))?;
        if let Ok(mut guard) = self.fetcher.write() {
            *guard = Some(Arc::new(fetcher));
        }
        tracing::info!(
            component = "tv.images.fetcher",
            cache_dir = %cache_dir.display(),
            "Image fetcher initialized"
        );
        Ok(())
    }

    /// Returns a clone of the fetcher Arc, if initialized.
    pub fn get(&self) -> Option<Arc<ImageFetcher>> {
        self.fetcher.read().ok().and_then(|guard| guard.clone())
    }
}

impl Default for ImageFetcherState {
    fn default() -> Self {
        Self::new()
    }
}

/// Validates that the provided bytes represent a decodable image format.
fn validate_image_data(url: &str, data: &[u8]) -> Result<(), TvError> {
    let cursor = Cursor::new(data);
    image::ImageReader::new(cursor)
        .with_guessed_format()
        .map_err(|e| TvError::ImageFetchError {
            url: url.to_string(),
            message: format!("Failed to detect image format: {e}"),
        })?
        .format()
        .ok_or_else(|| TvError::ImageFetchError {
            url: url.to_string(),
            message: "Unrecognized image format".to_string(),
        })?;
    Ok(())
}

/// Maps a reqwest error to an appropriate TvError variant.
fn map_reqwest_error(url: &str, error: &reqwest::Error) -> TvError {
    let message = if error.is_timeout() {
        "Request timed out".to_string()
    } else if error.is_connect() {
        "Connection failed".to_string()
    } else {
        format!("Network error: {error}")
    };

    TvError::ImageFetchError { url: url.to_string(), message }
}
