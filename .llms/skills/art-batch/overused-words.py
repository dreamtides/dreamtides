#!/usr/bin/env python3
"""Scan art-assigned.toml for overused words in card names.

Prints words appearing 3+ times across all card names, one per line:
  COUNT WORD
Sorted by count descending. Subagents should avoid these words.
"""

import re
from collections import Counter
from pathlib import Path

ASSIGNED = Path(__file__).parent.parent.parent.parent / "rules_engine" / "tabula" / "art-assigned.toml"
# Common English words that don't count as repetitive
STOP_WORDS = {"the", "of", "a", "an", "in", "on", "at", "to", "and", "for", "from", "with"}

words: Counter[str] = Counter()
if ASSIGNED.exists():
    for line in ASSIGNED.read_text().splitlines():
        m = re.match(r'^name\s*=\s*"(.+)"', line.strip())
        if m:
            for w in m.group(1).lower().split():
                if w not in STOP_WORDS and len(w) > 2:
                    words[w] += 1

overused = [(w, c) for w, c in words.most_common() if c >= 3]
if overused:
    for w, c in overused:
        print(f"{c} {w}")
else:
    print("NONE")
