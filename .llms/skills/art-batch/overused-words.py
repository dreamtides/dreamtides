#!/usr/bin/env python3
"""Check a proposed card name against overused words in art-assigned.toml.

Usage: python3 overused-words.py "Proposed Card Name"

Prints PASS if no word in the name is overused (3+ prior uses), or
FAIL with the offending words if any are overused.
"""

import re
import sys
from collections import Counter
from pathlib import Path

ASSIGNED = Path(__file__).parent.parent.parent.parent / "rules_engine" / "tabula" / "art-assigned.toml"
STOP_WORDS = {"the", "of", "a", "an", "in", "on", "at", "to", "and", "for", "from", "with"}

if len(sys.argv) < 2:
    print("Usage: python3 overused-words.py \"Proposed Card Name\"")
    sys.exit(1)

words: Counter[str] = Counter()
if ASSIGNED.exists():
    for line in ASSIGNED.read_text().splitlines():
        m = re.match(r'^name\s*=\s*"(.+)"', line.strip())
        if m:
            for w in m.group(1).lower().split():
                if w not in STOP_WORDS and len(w) > 2:
                    words[w] += 1

proposed = sys.argv[1]
conflicts = []
for w in proposed.lower().split():
    if w in STOP_WORDS or len(w) <= 2:
        continue
    if words[w] >= 3:
        conflicts.append(f"{w} ({words[w]} uses)")

if conflicts:
    print(f"FAIL: {', '.join(conflicts)}")
else:
    print("PASS")
