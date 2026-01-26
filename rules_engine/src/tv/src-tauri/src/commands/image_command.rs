use tauri::State;

use crate::error::error_types::TvError;
use crate::images::image_fetcher::ImageFetcherState;

/// Tauri command to fetch an image by URL.
///
/// Returns the local cache file path on success. The frontend should convert
/// this path to an asset URL using `convertFileSrc()` before passing to Univer.
#[tauri::command]
pub async fn fetch_image(
    fetcher_state: State<'_, ImageFetcherState>,
    url: String,
) -> Result<String, TvError> {
    let fetcher = fetcher_state.get().ok_or_else(|| TvError::ImageFetchError {
        url: url.clone(),
        message: "Image fetcher not initialized".to_string(),
    })?;

    let path = fetcher.fetch(&url).await?;

    tracing::debug!(
        component = "tv.commands.image",
        url = %url,
        path = %path.display(),
        "Image fetch command completed"
    );

    Ok(path.to_string_lossy().to_string())
}
