#!/usr/bin/env python3
"""Look up card art image path and description by image ID."""

import json
import glob
import sys

def main():
    if len(sys.argv) != 2:
        print("Usage: art-lookup.py <image_id>", file=sys.stderr)
        sys.exit(1)

    image_id = sys.argv[1]
    urls_path = "/Users/dthurn/Documents/shutterstock/image_urls.json"
    images_dir = "/Users/dthurn/Documents/shutterstock/images"

    with open(urls_path) as f:
        entries = json.load(f)

    entry = next((e for e in entries if e["id"] == image_id), None)
    if not entry:
        print(f"Image ID {image_id} not found in image_urls.json", file=sys.stderr)
        sys.exit(1)

    matches = glob.glob(f"{images_dir}/*-{image_id}.*")
    if not matches:
        print(f"No image file found for ID {image_id}", file=sys.stderr)
        sys.exit(1)

    print(f"Path: {matches[0]}")
    print(f"Description: {entry['alt']}")

if __name__ == "__main__":
    main()
