"""Card image lookup for the quest simulator.

Resolves image-number fields to local file paths by checking the TV
app's image cache. The TV cache stores images keyed by SHA-256 of
the Shutterstock URL, so we construct the same URL and hash it to
find the cached file.
"""

import hashlib
from pathlib import Path

_URL_TEMPLATE = (
    "https://www.shutterstock.com/image-illustration/-260nw-{image_number}.jpg"
)

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


def get_image_path(image_number: int) -> str | None:
    """Return a local file path for the given image number.

    Looks up the image in the TV app's image cache using the same
    URL-to-SHA256 key mapping. Returns None if not cached.
    """
    key = _compute_cache_key(image_number)
    cached = _TV_CACHE_DIR / key
    return str(cached) if cached.exists() else None
