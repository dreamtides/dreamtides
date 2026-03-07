"""Card image lookup for the quest simulator.

Resolves image-number fields to local file paths by checking the TV
app's image cache. The TV cache stores images keyed by SHA-256 of
the Shutterstock URL, so we construct the same URL and hash it to
find the cached file.
"""

import hashlib
import time
import urllib.error
import urllib.request
from pathlib import Path

_URL_TEMPLATE = (
    "https://www.shutterstock.com/image-illustration/-260nw-{image_number}.jpg"
)

_FETCH_HEADERS = {
    "User-Agent": (
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) "
        "AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36"
    ),
    "Accept": "image/avif,image/webp,image/apng,image/*,*/*;q=0.8",
    "Accept-Language": "en-US,en;q=0.9",
    "Referer": "https://www.google.com/",
}

_DOWNLOAD_DELAY = 0.2  # seconds between downloads

_TV_CACHE_DIR = (
    Path.home() / "Library" / "Caches" / "io.github.dreamtides.tv" / "image_cache"
)


def _compute_cache_key(image_number: int) -> str:
    url = _URL_TEMPLATE.format(image_number=image_number)
    return hashlib.sha256(url.encode()).hexdigest()


def get_cache_dir() -> Path:
    """Return the TV image cache directory."""
    return _TV_CACHE_DIR


def get_image_cache_key(image_number: int) -> str | None:
    """Return the SHA256 cache key for the given image number, or None if not cached."""
    key = _compute_cache_key(image_number)
    return key if (_TV_CACHE_DIR / key).exists() else None


def ensure_image_cached(image_number: int) -> str | None:
    """Download the image if not cached, then return the cache key.

    Uses browser-like headers matching the TV app's fetch strategy.
    Returns None on any network or I/O error.
    """
    key = _compute_cache_key(image_number)
    cache_path = _TV_CACHE_DIR / key
    if cache_path.exists():
        return key
    url = _URL_TEMPLATE.format(image_number=image_number)
    try:
        req = urllib.request.Request(url, headers=_FETCH_HEADERS)
        with urllib.request.urlopen(req, timeout=15) as response:
            data = response.read()
        _TV_CACHE_DIR.mkdir(parents=True, exist_ok=True)
        tmp = cache_path.with_suffix(".tmp")
        tmp.write_bytes(data)
        tmp.rename(cache_path)
        time.sleep(_DOWNLOAD_DELAY)
        return key
    except (urllib.error.URLError, OSError):
        return None


def get_image_path(image_number: int) -> str | None:
    """Return a local file path for the given image number.

    Looks up the image in the TV app's image cache using the same
    URL-to-SHA256 key mapping. Returns None if not cached.
    """
    key = _compute_cache_key(image_number)
    cached = _TV_CACHE_DIR / key
    return str(cached) if cached.exists() else None
