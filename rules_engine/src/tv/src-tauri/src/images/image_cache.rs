// This module provides infrastructure for Tasks 36-38. The types and functions
// are intentionally not yet used until image fetching is implemented.
#![allow(dead_code)]

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use sha2::{Digest, Sha256};

use crate::error::error_types::TvError;

/// Default maximum cache size in bytes (100 MB).
const DEFAULT_MAX_CACHE_SIZE: u64 = 100 * 1024 * 1024;

/// Metadata file name storing cache entry information.
const CACHE_METADATA_FILE: &str = "cache_metadata.json";

/// Directory name for cached images within app data.
const CACHE_DIR_NAME: &str = "image_cache";

/// Content-addressed image cache with LRU eviction.
///
/// Images are stored by SHA-256 hash of their source URL. The cache persists
/// across application restarts and uses LRU eviction when size limit is exceeded.
pub struct ImageCache {
    cache_dir: PathBuf,
    max_size: u64,
    metadata: Arc<RwLock<CacheMetadata>>,
}

/// Metadata tracking all cache entries and their access times.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CacheMetadata {
    entries: HashMap<String, CacheEntry>,
    total_size: u64,
}

/// Metadata for a single cache entry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CacheEntry {
    url: String,
    size: u64,
    last_access: u64,
}

impl CacheMetadata {
    fn new() -> Self {
        Self { entries: HashMap::new(), total_size: 0 }
    }
}

impl ImageCache {
    /// Creates a new image cache in the specified application data directory.
    pub fn new(app_data_dir: &Path) -> Result<Self, TvError> {
        Self::with_max_size(app_data_dir, DEFAULT_MAX_CACHE_SIZE)
    }

    /// Creates a new image cache with a custom maximum size.
    pub fn with_max_size(app_data_dir: &Path, max_size: u64) -> Result<Self, TvError> {
        let cache_dir = app_data_dir.join(CACHE_DIR_NAME);
        fs::create_dir_all(&cache_dir).map_err(|e| TvError::WriteError {
            path: cache_dir.to_string_lossy().to_string(),
            message: format!("Failed to create cache directory: {e}"),
        })?;

        let metadata = Self::load_or_create_metadata(&cache_dir)?;
        Ok(Self { cache_dir, max_size, metadata: Arc::new(RwLock::new(metadata)) })
    }

    /// Returns the cache key for a given URL (SHA-256 hash).
    pub fn url_to_cache_key(url: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Gets an image from the cache by URL.
    ///
    /// Returns the cached file path if the image exists, or None if not cached.
    /// Updates the last access time for LRU tracking.
    pub fn get(&self, url: &str) -> Option<PathBuf> {
        let cache_key = Self::url_to_cache_key(url);
        let file_path = self.cache_dir.join(&cache_key);

        if !file_path.exists() {
            return None;
        }

        let mut metadata = self.metadata.write().ok()?;
        if let Some(entry) = metadata.entries.get_mut(&cache_key) {
            entry.last_access = current_timestamp();
        }
        if let Err(e) = self.save_metadata_locked(&metadata) {
            tracing::warn!(
                component = "tv.images.cache",
                error = %e,
                "Failed to update cache metadata after access"
            );
        }

        Some(file_path)
    }

    /// Stores an image in the cache.
    ///
    /// The image data is stored under a key derived from the URL hash.
    /// If the cache would exceed max size after adding this entry, LRU
    /// eviction is performed first.
    pub fn put(&self, url: &str, data: &[u8]) -> Result<PathBuf, TvError> {
        let cache_key = Self::url_to_cache_key(url);
        let file_path = self.cache_dir.join(&cache_key);
        let data_size = data.len() as u64;

        {
            let mut metadata = self.metadata.write().map_err(|_| TvError::ImageCacheCorrupt {
                cache_key: cache_key.clone(),
            })?;

            self.evict_if_needed(&mut metadata, data_size)?;

            let mut file = File::create(&file_path).map_err(|e| TvError::WriteError {
                path: file_path.to_string_lossy().to_string(),
                message: format!("Failed to create cache file: {e}"),
            })?;

            file.write_all(data).map_err(|e| TvError::WriteError {
                path: file_path.to_string_lossy().to_string(),
                message: format!("Failed to write cache file: {e}"),
            })?;

            if let Some(old_entry) = metadata.entries.get(&cache_key) {
                metadata.total_size = metadata.total_size.saturating_sub(old_entry.size);
            }

            metadata.entries.insert(
                cache_key.clone(),
                CacheEntry { url: url.to_string(), size: data_size, last_access: current_timestamp() },
            );
            metadata.total_size += data_size;

            self.save_metadata_locked(&metadata)?;
        }

        tracing::info!(
            component = "tv.images.cache",
            cache_key = %cache_key,
            size = data_size,
            "Stored image in cache"
        );

        Ok(file_path)
    }

    /// Checks if a URL is cached.
    pub fn contains(&self, url: &str) -> bool {
        let cache_key = Self::url_to_cache_key(url);
        let file_path = self.cache_dir.join(&cache_key);
        file_path.exists()
    }

    /// Returns the current total cache size in bytes.
    pub fn total_size(&self) -> u64 {
        self.metadata.read().map(|m| m.total_size).unwrap_or(0)
    }

    /// Returns the number of entries in the cache.
    pub fn entry_count(&self) -> usize {
        self.metadata.read().map(|m| m.entries.len()).unwrap_or(0)
    }

    /// Clears all entries from the cache.
    pub fn clear(&self) -> Result<(), TvError> {
        let mut metadata = self.metadata.write().map_err(|_| TvError::ImageCacheCorrupt {
            cache_key: "metadata".to_string(),
        })?;

        for (cache_key, _) in metadata.entries.drain() {
            let file_path = self.cache_dir.join(&cache_key);
            if let Err(e) = fs::remove_file(&file_path) {
                tracing::warn!(
                    component = "tv.images.cache",
                    cache_key = %cache_key,
                    error = %e,
                    "Failed to remove cache file during clear"
                );
            }
        }

        metadata.total_size = 0;
        self.save_metadata_locked(&metadata)?;

        tracing::info!(component = "tv.images.cache", "Cache cleared");
        Ok(())
    }

    /// Removes a specific URL from the cache.
    pub fn remove(&self, url: &str) -> Result<bool, TvError> {
        let cache_key = Self::url_to_cache_key(url);
        let file_path = self.cache_dir.join(&cache_key);

        let mut metadata = self.metadata.write().map_err(|_| TvError::ImageCacheCorrupt {
            cache_key: cache_key.clone(),
        })?;

        if let Some(entry) = metadata.entries.remove(&cache_key) {
            metadata.total_size = metadata.total_size.saturating_sub(entry.size);
            if let Err(e) = fs::remove_file(&file_path) {
                tracing::warn!(
                    component = "tv.images.cache",
                    cache_key = %cache_key,
                    error = %e,
                    "Failed to remove cache file"
                );
            }
            self.save_metadata_locked(&metadata)?;
            return Ok(true);
        }

        Ok(false)
    }

    /// Reads the cached image data for a URL.
    pub fn get_data(&self, url: &str) -> Result<Option<Vec<u8>>, TvError> {
        let Some(path) = self.get(url) else {
            return Ok(None);
        };

        let mut file = File::open(&path).map_err(|_e| TvError::FileNotFound {
            path: path.to_string_lossy().to_string(),
        })?;

        let mut data = Vec::new();
        file.read_to_end(&mut data).map_err(|_e| TvError::ImageCacheCorrupt {
            cache_key: Self::url_to_cache_key(url),
        })?;

        Ok(Some(data))
    }

    /// Validates cache integrity on startup.
    ///
    /// Removes entries from metadata that don't have corresponding files,
    /// and files that aren't tracked in metadata.
    pub fn validate_integrity(&self) -> Result<ValidationResult, TvError> {
        let mut metadata = self.metadata.write().map_err(|_| TvError::ImageCacheCorrupt {
            cache_key: "metadata".to_string(),
        })?;

        let mut orphaned_files_removed = 0;
        let mut missing_entries_removed = 0;
        let mut size_corrected = false;

        let entries_snapshot: Vec<String> = metadata.entries.keys().cloned().collect();
        for cache_key in entries_snapshot {
            let file_path = self.cache_dir.join(&cache_key);
            if !file_path.exists() {
                if let Some(entry) = metadata.entries.remove(&cache_key) {
                    metadata.total_size = metadata.total_size.saturating_sub(entry.size);
                    missing_entries_removed += 1;
                }
            }
        }

        if let Ok(dir_entries) = fs::read_dir(&self.cache_dir) {
            for entry in dir_entries.flatten() {
                let file_name = entry.file_name().to_string_lossy().to_string();
                if file_name == CACHE_METADATA_FILE {
                    continue;
                }
                if !metadata.entries.contains_key(&file_name) {
                    if let Err(e) = fs::remove_file(entry.path()) {
                        tracing::warn!(
                            component = "tv.images.cache",
                            file = %file_name,
                            error = %e,
                            "Failed to remove orphaned cache file"
                        );
                    } else {
                        orphaned_files_removed += 1;
                    }
                }
            }
        }

        let actual_total: u64 = metadata.entries.values().map(|e| e.size).sum();
        if actual_total != metadata.total_size {
            metadata.total_size = actual_total;
            size_corrected = true;
        }

        if missing_entries_removed > 0 || orphaned_files_removed > 0 || size_corrected {
            self.save_metadata_locked(&metadata)?;
        }

        tracing::info!(
            component = "tv.images.cache",
            orphaned_files_removed = orphaned_files_removed,
            missing_entries_removed = missing_entries_removed,
            size_corrected = size_corrected,
            "Cache integrity validation complete"
        );

        Ok(ValidationResult { orphaned_files_removed, missing_entries_removed, size_corrected })
    }

    fn evict_if_needed(&self, metadata: &mut CacheMetadata, incoming_size: u64) -> Result<(), TvError> {
        let target_size = self.max_size.saturating_sub(incoming_size);

        while metadata.total_size > target_size && !metadata.entries.is_empty() {
            let lru_key = metadata
                .entries
                .iter()
                .min_by_key(|(_, e)| e.last_access)
                .map(|(k, _)| k.clone());

            let Some(key_to_remove) = lru_key else {
                break;
            };

            if let Some(entry) = metadata.entries.remove(&key_to_remove) {
                metadata.total_size = metadata.total_size.saturating_sub(entry.size);
                let file_path = self.cache_dir.join(&key_to_remove);
                if let Err(e) = fs::remove_file(&file_path) {
                    tracing::warn!(
                        component = "tv.images.cache",
                        cache_key = %key_to_remove,
                        error = %e,
                        "Failed to remove evicted cache file"
                    );
                }
                tracing::debug!(
                    component = "tv.images.cache",
                    cache_key = %key_to_remove,
                    size = entry.size,
                    "Evicted cache entry (LRU)"
                );
            }
        }

        Ok(())
    }

    fn load_or_create_metadata(cache_dir: &Path) -> Result<CacheMetadata, TvError> {
        let metadata_path = cache_dir.join(CACHE_METADATA_FILE);

        if metadata_path.exists() {
            let content = fs::read_to_string(&metadata_path).map_err(|e| TvError::ImageCacheCorrupt {
                cache_key: format!("metadata file read error: {e}"),
            })?;

            match serde_json::from_str(&content) {
                Ok(metadata) => return Ok(metadata),
                Err(e) => {
                    tracing::warn!(
                        component = "tv.images.cache",
                        error = %e,
                        "Failed to parse cache metadata, starting fresh"
                    );
                }
            }
        }

        Ok(CacheMetadata::new())
    }

    fn save_metadata_locked(&self, metadata: &CacheMetadata) -> Result<(), TvError> {
        let metadata_path = self.cache_dir.join(CACHE_METADATA_FILE);
        let content = serde_json::to_string_pretty(metadata).map_err(|e| TvError::ImageCacheCorrupt {
            cache_key: format!("metadata serialization error: {e}"),
        })?;

        fs::write(&metadata_path, content).map_err(|e| TvError::WriteError {
            path: metadata_path.to_string_lossy().to_string(),
            message: format!("Failed to write cache metadata: {e}"),
        })?;

        Ok(())
    }
}

/// Result of cache integrity validation.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub orphaned_files_removed: usize,
    pub missing_entries_removed: usize,
    pub size_corrected: bool,
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
