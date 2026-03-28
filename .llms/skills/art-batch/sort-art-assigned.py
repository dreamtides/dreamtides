#!/usr/bin/env python3
"""Sort art-assigned.toml by rendered-text, preserving metadata and multiline text."""

import re
import sys
import tomllib

PATH = "rules_engine/tabula/art-assigned.toml"

with open(PATH, "r") as f:
    content = f.read()

# Parse with tomllib to get canonical rendered-text values per image-number
with open(PATH, "rb") as f:
    data = tomllib.load(f)

text_by_image = {}
for card in data["cards"]:
    img = card["image-number"]
    text_by_image[img] = " ".join(card["rendered-text"].strip().split())

# Split off the [metadata] section
meta_match = re.search(r"\n(\[metadata\].*)", content, re.DOTALL)
metadata = ""
cards_content = content
if meta_match:
    metadata = "\n" + meta_match.group(1)
    cards_content = content[: meta_match.start()]

# Split into individual [[cards]] blocks
blocks = re.split(r"(?=^\[\[cards\]\])", cards_content, flags=re.MULTILINE)
blocks = [b for b in blocks if b.strip()]


def get_image_number(block):
    m = re.search(r"^image-number\s*=\s*(.+)$", block, re.MULTILINE)
    if not m:
        return 0
    val = m.group(1).strip().strip('"')
    try:
        return int(val)
    except ValueError:
        return 0


def sort_key(block):
    img = get_image_number(block)
    text = text_by_image.get(img, "")
    return (text, img)


blocks.sort(key=sort_key)

# Reassemble
output_parts = []
for block in blocks:
    output_parts.append(block.rstrip() + "\n")

output = "\n".join(output_parts) + metadata
if not output.endswith("\n"):
    output += "\n"

with open(PATH, "w") as f:
    f.write(output)

# Validate
with open(PATH, "rb") as f:
    data = tomllib.load(f)

n_cards = len(data["cards"])
has_meta = "metadata" in data
print(f"Sorted {n_cards} cards by rendered-text. Metadata: {'yes' if has_meta else 'MISSING!'}")

if not has_meta:
    print("ERROR: metadata section lost!", file=sys.stderr)
    sys.exit(1)
