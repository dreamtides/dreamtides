#!/usr/bin/env python3
"""Check a proposed card name against overused words.

Checks both art-assigned.toml and /tmp/art-batch-results/*.toml using
proper TOML parsing.

Usage: python3 overused-words.py "Proposed Card Name"

Prints PASS if no word in the name is overused (3+ prior uses), or
FAIL with the offending words if any are overused.
"""

import sys
import tomllib
from collections import Counter
from pathlib import Path

REPO_ROOT = Path(__file__).parent.parent.parent.parent
RENDERED = REPO_ROOT / "rules_engine" / "tabula" / "rendered-cards.toml"
ASSIGNED = REPO_ROOT / "rules_engine" / "tabula" / "art-assigned.toml"
RESULTS_DIR = Path("/tmp/art-batch-results")
BATCH_RESULTS_DIR = Path("/tmp/card-design-batch-results")
STOP_WORDS = {"the", "of", "a", "an", "in", "on", "at", "to", "and", "for", "from", "with"}
OVERUSE_THRESHOLD = 3

if len(sys.argv) < 2:
    print('Usage: python3 overused-words.py "Proposed Card Name"')
    sys.exit(1)

words: Counter[str] = Counter()


def count_names(path: Path) -> None:
    with open(path, "rb") as f:
        data = tomllib.load(f)
    for card in data.get("cards", []):
        name = card.get("name", "")
        for w in name.lower().split():
            # Strip possessives
            if w.endswith("'s") or w.endswith("\u2019s"):
                w = w[:-2]
            if w not in STOP_WORDS and len(w) > 2:
                words[w] += 1


if RENDERED.exists():
    count_names(RENDERED)
if ASSIGNED.exists():
    count_names(ASSIGNED)
for d in [RESULTS_DIR, BATCH_RESULTS_DIR]:
    if d.exists():
        for f in sorted(d.iterdir()):
            if f.suffix == ".toml":
                count_names(f)

proposed = sys.argv[1]
conflicts = []
for w in proposed.lower().split():
    w_clean = w[:-2] if (w.endswith("'s") or w.endswith("\u2019s")) else w
    if w_clean in STOP_WORDS or len(w_clean) <= 2:
        continue
    if words[w_clean] >= OVERUSE_THRESHOLD:
        conflicts.append(f"{w_clean} ({words[w_clean]} uses)")

if conflicts:
    print(f"FAIL: {', '.join(conflicts)}")
else:
    print("PASS")
