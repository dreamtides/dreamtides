#!/usr/bin/env python3
"""Sort art-assigned.toml by image-number, preserving metadata and multiline text."""

import re
import sys

PATH = "rules_engine/tabula/art-assigned.toml"

with open(PATH, "r") as f:
    content = f.read()

# Split off the [metadata] section (starts with "\n[metadata]" not "[[")
meta_match = re.search(r"\n(\[metadata\].*)", content, re.DOTALL)
metadata = ""
cards_content = content
if meta_match:
    metadata = "\n" + meta_match.group(1)
    cards_content = content[: meta_match.start()]

# Split into individual [[cards]] blocks
# Each block starts with "[[cards]]" and runs until the next "[[cards]]" or end
blocks = re.split(r"(?=^\[\[cards\]\])", cards_content, flags=re.MULTILINE)
blocks = [b for b in blocks if b.strip()]

# Extract image-number from each block for sorting
def get_image_number(block):
    m = re.search(r"^image-number\s*=\s*(.+)$", block, re.MULTILINE)
    if not m:
        return 0
    val = m.group(1).strip().strip('"')
    try:
        return int(val)
    except ValueError:
        return 0

blocks.sort(key=get_image_number)

# Reassemble: ensure each block ends with exactly one blank line
output_parts = []
for block in blocks:
    output_parts.append(block.rstrip() + "\n")

output = "\n".join(output_parts) + metadata
if not output.endswith("\n"):
    output += "\n"

with open(PATH, "w") as f:
    f.write(output)

# Validate
import tomllib

with open(PATH, "rb") as f:
    data = tomllib.load(f)

n_cards = len(data["cards"])
has_meta = "metadata" in data
print(f"Sorted {n_cards} cards by image-number. Metadata: {'yes' if has_meta else 'MISSING!'}")

if not has_meta:
    print("ERROR: metadata section lost!", file=sys.stderr)
    sys.exit(1)
