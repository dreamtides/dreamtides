#!/usr/bin/env python3
"""Concatenate per-image dreamcaller result files into notes/dreamcaller-assignments.md."""

from pathlib import Path

RESULTS_DIR = Path("/tmp/dreamcaller-batch-results")
SKIPS = Path("/tmp/dreamcaller-batch-skips.txt")
OUT = Path(__file__).parent.parent.parent.parent / "notes" / "dreamcaller-assignments.md"

files = sorted(RESULTS_DIR.glob("*.md"))
parts = ["# Dreamcaller Assignments\n"]
for f in files:
    parts.append(f.read_text().rstrip() + "\n\n---\n")

OUT.write_text("\n".join(parts))

skip_count = 0
if SKIPS.exists():
    skip_count = len([l for l in SKIPS.read_text().splitlines() if l.strip()])

print(f"Wrote {len(files)} assignments to {OUT}")
print(f"Skips: {skip_count}")
