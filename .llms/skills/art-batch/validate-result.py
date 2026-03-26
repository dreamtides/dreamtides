#!/usr/bin/env python3
"""Validate an art-batch result TOML file.

Usage: python3 validate-result.py /tmp/art-batch-results/IMAGEID.toml

Checks that all required fields are present and non-empty.
Prints PASS or FAIL with details.
"""

import sys
import tomllib
from pathlib import Path

REQUIRED_FIELDS = [
    "name", "id", "tide", "tide-cost", "rendered-text",
    "energy-cost", "card-type", "rarity", "is-fast", "image-number",
]

if len(sys.argv) < 2:
    print("Usage: python3 validate-result.py <file.toml>")
    sys.exit(1)

path = Path(sys.argv[1])
if not path.exists():
    print(f"FAIL: file not found: {path}")
    sys.exit(1)

with open(path, "rb") as f:
    data = tomllib.load(f)

cards = data.get("cards", [])
if not cards:
    print("FAIL: no [[cards]] entry found")
    sys.exit(1)

card = cards[0]
errors = []

for field in REQUIRED_FIELDS:
    val = card.get(field)
    if val is None:
        errors.append(f"missing field: {field}")
    elif isinstance(val, str) and not val.strip():
        errors.append(f"empty field: {field}")

# Check for unexpected fields
expected = set(REQUIRED_FIELDS) | {"subtype", "spark", "art-owned", "card-number"}
unexpected = set(card.keys()) - expected
if unexpected:
    errors.append(f"unexpected fields: {', '.join(sorted(unexpected))}")

if errors:
    print(f"FAIL: {'; '.join(errors)}")
    sys.exit(1)
else:
    print("PASS")
