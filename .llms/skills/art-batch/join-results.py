#!/usr/bin/env python3
"""Join per-image result files into art-assigned.toml.

Usage: python3 join-results.py

Reads all .toml files from /tmp/art-batch-results/ and appends their
contents to rules_engine/tabula/art-assigned.toml.
"""

from pathlib import Path

RESULTS_DIR = Path("/tmp/art-batch-results")
ASSIGNED = (
    Path(__file__).parent.parent.parent.parent
    / "rules_engine"
    / "tabula"
    / "art-assigned.toml"
)

if not RESULTS_DIR.exists():
    print("No results directory found at /tmp/art-batch-results/")
    exit(1)

result_files = sorted(RESULTS_DIR.glob("*.toml"))
if not result_files:
    print("No result files found")
    exit(0)

entries = []
for f in result_files:
    content = f.read_text().strip()
    if content:
        entries.append(content)

if not entries:
    print("No non-empty result files")
    exit(0)

with open(ASSIGNED, "a") as out:
    out.write("\n")
    for entry in entries:
        out.write("\n" + entry + "\n")

print(f"Appended {len(entries)} entries to {ASSIGNED}")
print(f"Skips: {len(list(Path('/tmp/art-batch-skips.txt').read_text().splitlines())) if Path('/tmp/art-batch-skips.txt').exists() else 0}")
