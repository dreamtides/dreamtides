#!/usr/bin/env python3
"""Check how many times a rules text has already been matched in art-assigned.toml.

Usage: python3 check-match-count.py "Exact rules text here"

Prints one of:
  PASS (0 matches)        — card is fresh, no concerns
  WARN: 1 match           — card has been used once, consider alternatives
  WARN: 2 matches         — card is popular, must justify why this art is a better fit
  FAIL: 3+ matches (N)    — card is saturated, pick a different card
"""

import re
import sys
from pathlib import Path

ASSIGNED = Path(__file__).parent.parent.parent.parent / "rules_engine" / "tabula" / "art-assigned.toml"
HARD_CAP = 3

if len(sys.argv) < 2:
    print('Usage: python3 check-match-count.py "Exact rules text"')
    sys.exit(1)

target = sys.argv[1]

count = 0
if ASSIGNED.exists():
    for line in ASSIGNED.read_text().splitlines():
        m = re.match(r'^rendered-text\s*=\s*"(.+)"', line.strip())
        if m and m.group(1) == target:
            count += 1

if count >= HARD_CAP:
    print(f"FAIL: {count} matches — this card is saturated, pick a different card")
elif count >= 2:
    print(f"WARN: {count} matches — this card is popular, you must justify why this art is a uniquely better fit than prior matches or pick a different card")
elif count >= 1:
    print(f"WARN: {count} match — this card has been used before, consider alternatives")
else:
    print("PASS (0 matches)")
