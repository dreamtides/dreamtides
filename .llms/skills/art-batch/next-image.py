#!/usr/bin/env python3
"""Print the next unprocessed image ID and its index.

Usage: python3 next-image.py

Reads /tmp/art-batch-images.txt for the full list and checks
rules_engine/tabula/art-assigned.toml for already-processed IDs.

Output: INDEX TOTAL IMAGE_ID
Or "DONE" if all images are processed.
"""

import re
from pathlib import Path

IMAGES = Path("/tmp/art-batch-images.txt")
ASSIGNED = Path(__file__).parent.parent.parent.parent / "rules_engine" / "tabula" / "art-assigned.toml"

all_ids = IMAGES.read_text().splitlines()
total = len(all_ids)

done = set()
if ASSIGNED.exists():
    for line in ASSIGNED.read_text().splitlines():
        m = re.match(r'^image-number\s*=\s*(\d+)', line.strip())
        if m:
            done.add(m.group(1))

for i, img_id in enumerate(all_ids):
    if img_id.strip() and img_id.strip() not in done:
        print(f"{i+1} {total} {img_id.strip()}")
        break
else:
    print("DONE")
