#!/usr/bin/env python3
"""Pre-build the image ID list for art-batch orchestration.

Writes one image ID per line to /tmp/art-batch-images.txt
"""

import os
import re

IMAGES_DIR = "/Users/dthurn/Documents/shutterstock/images_clean"

ids = []
for fname in sorted(os.listdir(IMAGES_DIR)):
    m = re.search(r'-(\d+)\.\w+$', fname)
    if m:
        ids.append(m.group(1))

with open("/tmp/art-batch-images.txt", "w") as out:
    for img_id in ids:
        out.write(img_id + "\n")

print(f"Wrote {len(ids)} image IDs to /tmp/art-batch-images.txt")
