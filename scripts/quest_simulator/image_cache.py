"""Download and cache card images for terminal display.

Uses the same Shutterstock URL template as TV to resolve image-number
fields to image files. Images are cached locally to avoid redundant
downloads.
"""

import os
import urllib.request
from pathlib import Path

_URL_TEMPLATE = (
    "https://www.shutterstock.com/image-illustration/-260nw-{image_number}.jpg"
)

_CACHE_DIR = Path.home() / ".cache" / "dreamtides-quest" / "images"

_USER_AGENT = (
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) "
    "AppleWebKit/537.36 (KHTML, like Gecko) "
    "Chrome/124.0.0.0 Safari/537.36"
)


def get_image_path(image_number: int) -> str | None:
    """Return a local file path for the given image number.

    Downloads the image on cache miss. Returns None if the download
    fails.
    """
    _CACHE_DIR.mkdir(parents=True, exist_ok=True)
    cached = _CACHE_DIR / f"{image_number}.jpg"
    if cached.exists():
        return str(cached)

    url = _URL_TEMPLATE.format(image_number=image_number)
    try:
        req = urllib.request.Request(url, headers={"User-Agent": _USER_AGENT})
        with urllib.request.urlopen(req, timeout=30) as resp:
            data = resp.read()
        cached.write_bytes(data)
        return str(cached)
    except Exception:
        return None
