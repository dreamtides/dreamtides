use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use reqwest::header::{
    HeaderMap, HeaderValue, ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, REFERER, USER_AGENT,
};
use tokio::sync::Semaphore;

use crate::derived::function_registry;
use crate::error::error_types::TvError;
use crate::images::image_cache::ImageCache;

/// Default timeout for HTTP image fetches.
const DEFAULT_FETCH_TIMEOUT: Duration = Duration::from_secs(30);

/// Default maximum concurrent fetches.
const DEFAULT_MAX_CONCURRENT_FETCHES: usize = 4;

/// Delay between consecutive remote image downloads to avoid overwhelming servers.
pub const DOWNLOAD_DELAY: Duration = Duration::from_millis(200);

/// Content-type prefixes that indicate valid image responses.
const IMAGE_CONTENT_TYPE_PREFIX: &str = "image/";

/// Content-type value for binary octet-stream (accepted as potentially valid).
const OCTET_STREAM_CONTENT_TYPE: &str = "application/octet-stream";

/// Returns browser-like HTTP headers for image download requests.
pub fn browser_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
        ),
    );
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("image/avif,image/webp,image/apng,image/*,*/*;q=0.8"),
    );
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
    headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip, deflate, br"));
    headers.insert(REFERER, HeaderValue::from_static("https://www.google.com/"));
    headers
}

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
            .default_headers(browser_headers())
            .pool_max_idle_per_host(max_concurrent)
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

    /// Fetches an image by URL or local path, returning a local file path.
    ///
    /// For local filesystem paths, reads and validates the file directly
    /// without caching. For remote URLs, checks the cache first and returns
    /// immediately on hit. On cache miss, fetches the image over HTTP,
    /// validates the response content-type and image data, stores it in the
    /// cache, and returns the cached file path. Uses a semaphore to limit
    /// concurrent network requests.
    pub async fn fetch(&self, url: &str) -> Result<PathBuf, TvError> {
        if is_local_path(url) {
            return self.fetch_local(url);
        }

        self.fetch_remote(url).await
    }

    /// Reads a local filesystem image directly without caching.
    fn fetch_local(&self, path_str: &str) -> Result<PathBuf, TvError> {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(TvError::FileNotFound { path: path_str.to_string() });
        }

        let data = std::fs::read(path).map_err(|e| TvError::ImageFetchError {
            url: path_str.to_string(),
            message: format!("Failed to read local file: {e}"),
        })?;

        validate_image_data(path_str, &data)?;

        tracing::debug!(
            component = "tv.images.fetcher",
            path = %path_str,
            size = data.len(),
            "Read local image file"
        );

        Ok(path.to_path_buf())
    }

    /// Fetches an image from a remote HTTP/HTTPS URL with caching.
    async fn fetch_remote(&self, url: &str) -> Result<PathBuf, TvError> {
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

        validate_content_type(url, &response)?;

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

        tokio::time::sleep(DOWNLOAD_DELAY).await;

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
    ///
    /// Also registers the image derived function with the global function
    /// registry, since it requires access to the shared image cache.
    pub fn initialize(&self, cache_dir: &std::path::Path) -> Result<(), TvError> {
        let cache = Arc::new(ImageCache::new(cache_dir)?);
        cache.validate_integrity()?;
        function_registry::register_image_derived_function(Arc::clone(&cache));
        let fetcher = ImageFetcher::new(cache)?;
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

/// Returns true if the given string looks like a local filesystem path
/// rather than an HTTP/HTTPS URL.
fn is_local_path(url: &str) -> bool {
    !url.starts_with("http://") && !url.starts_with("https://")
}

/// Validates the HTTP response content-type header indicates an image.
///
/// Accepts responses with `image/*` content types, `application/octet-stream`,
/// or missing content-type headers (falling back to image data validation).
fn validate_content_type(url: &str, response: &reqwest::Response) -> Result<(), TvError> {
    let Some(content_type) = response.headers().get(reqwest::header::CONTENT_TYPE) else {
        return Ok(());
    };

    let content_type_str = content_type.to_str().unwrap_or("");
    let lower = content_type_str.to_ascii_lowercase();
    if lower.starts_with(IMAGE_CONTENT_TYPE_PREFIX)
        || lower.starts_with(OCTET_STREAM_CONTENT_TYPE)
        || lower.is_empty()
    {
        return Ok(());
    }

    Err(TvError::ImageFetchError {
        url: url.to_string(),
        message: format!("Unexpected content-type: {content_type_str}"),
    })
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
