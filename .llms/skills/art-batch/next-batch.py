#!/usr/bin/env python3
"""Print the next batch of unprocessed image IDs (rolling-window aware).

Usage: python3 next-batch.py [BATCH_SIZE]

Tracks three states for each image ID:
  - done:      result file exists, or listed in art-assigned.toml / skips.txt
  - in-flight: listed in /tmp/art-batch-inflight.txt but not yet done
  - remaining: neither done nor in-flight

The script automatically graduates in-flight IDs to done when their result or
skip appears, keeping the inflight file accurate without manual cleanup.

Output: one image ID per line (up to BATCH_SIZE new IDs), or "DONE" if no
remaining images exist. Newly printed IDs are appended to the inflight file.
Also prints a progress summary to stderr.
"""

import re
import sys
from pathlib import Path

BATCH_SIZE = int(sys.argv[1]) if len(sys.argv) > 1 else 5

IMAGES = Path("/tmp/art-batch-images.txt")
INFLIGHT = Path("/tmp/art-batch-inflight.txt")
ASSIGNED = (
    Path(__file__).parent.parent.parent.parent
    / "rules_engine"
    / "tabula"
    / "art-assigned.toml"
)
SKIPS = Path("/tmp/art-batch-skips.txt")
RESULTS_DIR = Path("/tmp/art-batch-results")

all_ids = [line.strip() for line in IMAGES.read_text().splitlines() if line.strip()]
total = len(all_ids)

# Collect completed IDs from all sources
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

# Load in-flight set, pruning any that have since completed
inflight = set()
if INFLIGHT.exists():
    for line in INFLIGHT.read_text().splitlines():
        s = line.strip()
        if s and s not in done:
            inflight.add(s)
    # Rewrite pruned inflight file
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
    # Append new IDs to inflight file
    with open(INFLIGHT, "a") as f:
        for img_id in batch:
            f.write(img_id + "\n")
    for img_id in batch:
        print(img_id)
