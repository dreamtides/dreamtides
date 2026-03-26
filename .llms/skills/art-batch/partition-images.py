#!/usr/bin/env python3
"""Partition the image list into N lane files for parallel processing.

Usage: python3 partition-images.py [NUM_LANES]

Reads /tmp/art-batch-images.txt, checks which images are already done
(in art-assigned.toml or skips), then splits remaining images into
/tmp/art-batch-lane-0.txt through /tmp/art-batch-lane-{N-1}.txt.

Also prints summary stats.
"""

import re
import sys
from pathlib import Path

NUM_LANES = int(sys.argv[1]) if len(sys.argv) > 1 else 5

IMAGES = Path("/tmp/art-batch-images.txt")
ASSIGNED = (
    Path(__file__).parent.parent.parent.parent
    / "rules_engine"
    / "tabula"
    / "art-assigned.toml"
)
SKIPS = Path("/tmp/art-batch-skips.txt")
RESULTS_DIR = Path("/tmp/art-batch-results")

all_ids = IMAGES.read_text().splitlines()
total = len(all_ids)

done = set()
if ASSIGNED.exists():
    for line in ASSIGNED.read_text().splitlines():
        m = re.match(r"^image-number\s*=\s*(\d+)", line.strip())
        if m:
            done.add(m.group(1))
if SKIPS.exists():
    for line in SKIPS.read_text().splitlines():
        s = line.strip()
        if s:
            done.add(s)
if RESULTS_DIR.exists():
    for f in RESULTS_DIR.iterdir():
        if f.suffix == ".toml":
            done.add(f.stem)

remaining = [img_id.strip() for img_id in all_ids if img_id.strip() and img_id.strip() not in done]

# Round-robin distribution across lanes
lanes: list[list[str]] = [[] for _ in range(NUM_LANES)]
for i, img_id in enumerate(remaining):
    lanes[i % NUM_LANES].append(img_id)

for lane_idx, lane_ids in enumerate(lanes):
    path = Path(f"/tmp/art-batch-lane-{lane_idx}.txt")
    path.write_text("\n".join(lane_ids) + "\n" if lane_ids else "")

print(f"Total images: {total}")
print(f"Already done: {len(done)}")
print(f"Remaining: {len(remaining)}")
print(f"Partitioned into {NUM_LANES} lanes:")
for i, lane_ids in enumerate(lanes):
    print(f"  Lane {i}: {len(lane_ids)} images -> /tmp/art-batch-lane-{i}.txt")
