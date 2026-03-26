#!/usr/bin/env python3
"""Print the next unprocessed image ID from a lane file.

Usage: python3 next-image.py --lane N

Reads /tmp/art-batch-lane-{N}.txt and checks /tmp/art-batch-results/
and /tmp/art-batch-skips.txt for already-processed IDs.

Output: INDEX TOTAL IMAGE_ID
Or "DONE" if all images in this lane are processed.
"""

import argparse
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("--lane", type=int, required=True)
args = parser.parse_args()

LANE_FILE = Path(f"/tmp/art-batch-lane-{args.lane}.txt")
RESULTS_DIR = Path("/tmp/art-batch-results")
SKIPS = Path("/tmp/art-batch-skips.txt")

if not LANE_FILE.exists():
    print("DONE")
    exit()

all_ids = [line.strip() for line in LANE_FILE.read_text().splitlines() if line.strip()]
total = len(all_ids)

done = set()
if RESULTS_DIR.exists():
    for f in RESULTS_DIR.iterdir():
        if f.suffix == ".toml":
            done.add(f.stem)
if SKIPS.exists():
    for line in SKIPS.read_text().splitlines():
        s = line.strip()
        if s:
            done.add(s)

for i, img_id in enumerate(all_ids):
    if img_id not in done:
        print(f"{i + 1} {total} {img_id}")
        break
else:
    print("DONE")
