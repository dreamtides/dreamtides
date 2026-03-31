#!/usr/bin/env python3
"""Build the image list for card-design-batch from a user-provided directory.

Usage: python3 build-image-list.py <IMAGES_DIR>

Writes one image filename (without extension) per line to
/tmp/card-design-batch-images.txt. Also writes a mapping file
/tmp/card-design-batch-paths.txt with "filename\tpath" per line
so subagents can locate the actual image file.
"""

import os
import re
import sys
from pathlib import Path

IMAGE_EXTENSIONS = {".png", ".jpg", ".jpeg", ".webp", ".bmp", ".tiff", ".tif"}

if len(sys.argv) < 2:
    print("Usage: python3 build-image-list.py <IMAGES_DIR>")
    sys.exit(1)

images_dir = Path(sys.argv[1])
if not images_dir.is_dir():
    print(f"ERROR: {images_dir} is not a directory")
    sys.exit(1)

entries = []
for fname in sorted(os.listdir(images_dir)):
    fpath = images_dir / fname
    if fpath.is_file() and fpath.suffix.lower() in IMAGE_EXTENSIONS:
        stem = fpath.stem
        entries.append((stem, str(fpath.resolve())))

with open("/tmp/card-design-batch-images.txt", "w") as out:
    for stem, _ in entries:
        out.write(stem + "\n")

with open("/tmp/card-design-batch-paths.txt", "w") as out:
    for stem, path in entries:
        out.write(f"{stem}\t{path}\n")

print(f"Found {len(entries)} images in {images_dir}")
