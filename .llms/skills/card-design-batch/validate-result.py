#!/usr/bin/env python3
"""Validate a card-design-batch result TOML file.

Usage: python3 validate-result.py /tmp/card-design-batch-results/STEM.toml

Checks that:
- There are exactly 5 [[cards]] entries
- Each entry has all required fields
- Each entry has keep = false
Prints PASS or FAIL with details.
"""

import sys
import tomllib
from pathlib import Path

REQUIRED_FIELDS = [
    "keep", "name", "id", "tide", "tide-cost", "rendered-text",
    "energy-cost", "card-type", "rarity", "is-fast", "image-number",
]

EXPECTED_FIELDS = set(REQUIRED_FIELDS) | {"subtype", "spark", "art-owned", "card-number"}

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
errors = []

if len(cards) != 5:
    errors.append(f"expected 5 [[cards]] entries, found {len(cards)}")

for i, card in enumerate(cards):
    prefix = f"card[{i}]"
    for field in REQUIRED_FIELDS:
        val = card.get(field)
        if val is None:
            errors.append(f"{prefix}: missing field: {field}")
        elif isinstance(val, str) and field != "spark" and field != "subtype" and not val.strip():
            errors.append(f"{prefix}: empty field: {field}")

    if card.get("keep") is not False:
        errors.append(f"{prefix}: keep must be false")

    unexpected = set(card.keys()) - EXPECTED_FIELDS
    if unexpected:
        errors.append(f"{prefix}: unexpected fields: {', '.join(sorted(unexpected))}")

if errors:
    print(f"FAIL: {'; '.join(errors)}")
    sys.exit(1)
else:
    print("PASS")
