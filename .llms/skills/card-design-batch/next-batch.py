#!/usr/bin/env python3
"""Print the next batch of unprocessed image IDs for card-design-batch.

Usage: python3 next-batch.py [BATCH_SIZE]

Tracks three states for each image ID:
  - done:      result file exists in /tmp/card-design-batch-results/
  - in-flight: listed in /tmp/card-design-batch-inflight.txt but not yet done
  - remaining: neither done nor in-flight

Output: one image ID per line (up to BATCH_SIZE new IDs), or
"WAITING" if all remaining are in-flight, or "DONE" if none remain.
"""

import sys
from pathlib import Path

BATCH_SIZE = int(sys.argv[1]) if len(sys.argv) > 1 else 5

IMAGES = Path("/tmp/card-design-batch-images.txt")
INFLIGHT = Path("/tmp/card-design-batch-inflight.txt")
RESULTS_DIR = Path("/tmp/card-design-batch-results")

all_ids = [line.strip() for line in IMAGES.read_text().splitlines() if line.strip()]
total = len(all_ids)

# Collect completed IDs
done = set()
if RESULTS_DIR.exists():
    for f in RESULTS_DIR.iterdir():
        if f.suffix == ".toml":
            done.add(f.stem)

# Load in-flight set, pruning completed
inflight = set()
if INFLIGHT.exists():
    for line in INFLIGHT.read_text().splitlines():
        s = line.strip()
        if s and s not in done:
            inflight.add(s)
    INFLIGHT.write_text("\n".join(sorted(inflight)) + "\n" if inflight else "")

remaining = [img_id for img_id in all_ids if img_id not in done and img_id not in inflight]

print(
    f"Total: {total}, Done: {len(done)}, In-flight: {len(inflight)}, Remaining: {len(remaining)}",
    file=sys.stderr,
)

if not remaining:
    if inflight:
        print("WAITING")
    else:
        print("DONE")
else:
    batch = remaining[:BATCH_SIZE]
    with open(INFLIGHT, "a") as f:
        for img_id in batch:
            f.write(img_id + "\n")
    for img_id in batch:
        print(img_id)
