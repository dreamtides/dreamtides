#!/usr/bin/env python3
"""Print the next batch of unprocessed image IDs.

Usage: python3 next-batch.py [BATCH_SIZE]

Reads /tmp/art-batch-images.txt and checks art-assigned.toml,
/tmp/art-batch-results/, and /tmp/art-batch-skips.txt for already-processed IDs.

Output: one image ID per line (up to BATCH_SIZE), or "DONE" if none remain.
Also prints a progress summary to stderr.
"""

import re
import sys
from pathlib import Path

BATCH_SIZE = int(sys.argv[1]) if len(sys.argv) > 1 else 5

IMAGES = Path("/tmp/art-batch-images.txt")
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

remaining = [img_id for img_id in all_ids if img_id not in done]

print(f"Total: {total}, Done: {len(done)}, Remaining: {len(remaining)}", file=sys.stderr)

if not remaining:
    print("DONE")
else:
    batch = remaining[:BATCH_SIZE]
    for img_id in batch:
        print(img_id)
