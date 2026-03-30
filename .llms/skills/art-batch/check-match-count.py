#!/usr/bin/env python3
"""Check how many times a rules text has already been matched.

Checks both art-assigned.toml and /tmp/art-batch-results/*.toml using
proper TOML parsing (handles multiline strings correctly).

Usage:
  python3 check-match-count.py "Exact rules text here"
  echo "multiline text" | python3 check-match-count.py --stdin

For multiline rules text, always use --stdin to avoid shell mangling.

Prints one of:
  PASS (0 matches)        — card is fresh, no concerns
  PASS (1 match)          — card has one match, still open
  WARN: 2 matches         — card is at soft cap, must justify why this art is a better fit
  FAIL: 3+ matches (N)    — card is saturated (hard cap), pick a different card
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from match_counts import get_match_counts, normalize_text

HARD_CAP = 3
SOFT_CAP = 2

if "--stdin" in sys.argv:
    target = sys.stdin.read().strip()
elif len(sys.argv) >= 2:
    target = sys.argv[1].strip()
else:
    print('Usage: python3 check-match-count.py "Exact rules text"')
    print('       echo "text" | python3 check-match-count.py --stdin')
    sys.exit(1)

if not target:
    print("Error: empty rules text")
    sys.exit(1)

counts = get_match_counts()
count = counts.get(normalize_text(target), 0)

if count >= HARD_CAP:
    print(f"FAIL: {count} matches — this card is saturated (hard cap {HARD_CAP}), pick a different card")
elif count >= SOFT_CAP:
    print(f"WARN: {count} matches — this card is popular (soft cap {SOFT_CAP}), you must justify why this art is a uniquely better fit than prior matches or pick a different card")
elif count >= 1:
    print(f"PASS ({count} match{'es' if count > 1 else ''})")
else:
    print("PASS (0 matches)")
