#!/usr/bin/env python3
"""Print the next batch of unprocessed dreamcaller image IDs (rolling-window aware).

Usage: python3 next-batch.py [BATCH_SIZE]

Tracks three states for each image ID:
  - done:      result file exists in /tmp/dreamcaller-batch-results, or listed in skips
  - in-flight: listed in /tmp/dreamcaller-batch-inflight.txt but not yet done
  - remaining: neither done nor in-flight

Output: one image ID per line (up to BATCH_SIZE new IDs), `WAITING` if all
remaining are in-flight, or `DONE` if every image is done or skipped.
Newly printed IDs are appended to the inflight file.
"""

import sys
from pathlib import Path

BATCH_SIZE = int(sys.argv[1]) if len(sys.argv) > 1 else 5

IMAGES = Path("/tmp/dreamcaller-batch-images.txt")
INFLIGHT = Path("/tmp/dreamcaller-batch-inflight.txt")
SKIPS = Path("/tmp/dreamcaller-batch-skips.txt")
RESULTS_DIR = Path("/tmp/dreamcaller-batch-results")

all_ids = [line.strip() for line in IMAGES.read_text().splitlines() if line.strip()]
total = len(all_ids)

done = set()
if SKIPS.exists():
    for line in SKIPS.read_text().splitlines():
        s = line.strip()
        if s:
            done.add(s)
if RESULTS_DIR.exists():
    for f in RESULTS_DIR.iterdir():
        if f.suffix == ".md":
            done.add(f.stem)

inflight = set()
if INFLIGHT.exists():
    for line in INFLIGHT.read_text().splitlines():
        s = line.strip()
        if s and s not in done:
            inflight.add(s)
    INFLIGHT.write_text("\n".join(sorted(inflight)) + "\n" if inflight else "")

remaining = [i for i in all_ids if i not in done and i not in inflight]

print(
    f"Total: {total}, Done: {len(done)}, In-flight: {len(inflight)}, Remaining: {len(remaining)}",
    file=sys.stderr,
)

if not remaining:
    print("WAITING" if inflight else "DONE")
else:
    batch = remaining[:BATCH_SIZE]
    with open(INFLIGHT, "a") as f:
        for img_id in batch:
            f.write(img_id + "\n")
    for img_id in batch:
        print(img_id)
