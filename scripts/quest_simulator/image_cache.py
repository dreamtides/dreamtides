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


def get_image_path(image_number: int) -> str | None:
    """Return a local file path for the given image number.

    Looks up the image in the TV app's image cache using the same
    URL-to-SHA256 key mapping. Returns None if not cached.
    """
    url = _URL_TEMPLATE.format(image_number=image_number)
    cache_key = hashlib.sha256(url.encode()).hexdigest()
    cached = _TV_CACHE_DIR / cache_key
    if cached.exists():
        return str(cached)
    return None
