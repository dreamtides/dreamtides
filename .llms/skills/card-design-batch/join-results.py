#!/usr/bin/env python3
"""Join all card-design-batch results into a single output file.

Usage: python3 join-results.py [OUTPUT_PATH]

Reads all .toml files from /tmp/card-design-batch-results/ and writes
a combined TOML file. Default output: /tmp/card-design-batch-output.toml
"""

import sys
import tomllib
from pathlib import Path

RESULTS_DIR = Path("/tmp/card-design-batch-results")
OUTPUT = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("/tmp/card-design-batch-output.toml")

if not RESULTS_DIR.exists():
    print("No results directory found at /tmp/card-design-batch-results/")
    sys.exit(1)

result_files = sorted(RESULTS_DIR.glob("*.toml"))
if not result_files:
    print("No result files found")
    sys.exit(0)

entries = []
for f in result_files:
    content = f.read_text().strip()
    if content:
        entries.append(content)

if not entries:
    print("No non-empty result files")
    sys.exit(0)

with open(OUTPUT, "w") as out:
    out.write("# Card Design Batch Results\n")
    out.write("# Set keep = true on designs you want to finalize.\n")
    for entry in entries:
        out.write("\n" + entry + "\n")

total_cards = 0
for f in result_files:
    with open(f, "rb") as fh:
        data = tomllib.load(fh)
    total_cards += len(data.get("cards", []))

print(f"Wrote {len(entries)} image results ({total_cards} total designs) to {OUTPUT}")
