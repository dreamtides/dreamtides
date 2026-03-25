#!/usr/bin/env python3
"""Look up card art image path and description by image ID."""

import glob
import re
import sys

def main():
    if len(sys.argv) != 2:
        print("Usage: art-lookup.py <image_id>", file=sys.stderr)
        sys.exit(1)

    image_id = sys.argv[1]
    images_dir = "/Users/dthurn/Documents/shutterstock/images_clean"

    matches = glob.glob(f"{images_dir}/*-{image_id}.*")
    if not matches:
        print(f"No image file found for ID {image_id}", file=sys.stderr)
        sys.exit(1)

    path = matches[0]
    # Extract description from filename: strip prefix, ID suffix, and extension
    fname = path.rsplit("/", 1)[-1]
    fname_no_ext = fname.rsplit(".", 1)[0]
    # Remove trailing -ID
    fname_no_id = re.sub(r'-\d+$', '', fname_no_ext)
    # Remove stock-photo- / stock-vector- prefix
    desc_slug = re.sub(r'^stock-(?:photo|vector)-', '', fname_no_id)
    # Convert hyphens to spaces and title-case
    description = desc_slug.replace("-", " ").strip()

    print(f"Path: {path}")
    print(f"Description: {description}")

if __name__ == "__main__":
    main()
