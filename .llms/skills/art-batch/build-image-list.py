#!/usr/bin/env python3
"""Pre-build the image ID list for art-batch orchestration.

Writes one image ID per line to /tmp/art-batch-images.txt
"""

import json

with open("/Users/dthurn/Documents/shutterstock/image_urls.json") as f:
    entries = json.load(f)

with open("/tmp/art-batch-images.txt", "w") as out:
    for e in entries:
        out.write(e["id"] + "\n")

print(f"Wrote {len(entries)} image IDs to /tmp/art-batch-images.txt")
