#!/usr/bin/env python3
"""Build reduced card pool for Phase 2+ art matching.

Reads art-assigned.toml to count matches per rules text, then filters
cards_anonymized.txt to only cards matched fewer than N times.

Usage: python3 build-gap-pool.py <phase> [--threshold N]

Writes to /tmp/art-batch-pool-phase-<phase>.txt
Default threshold: 2 (cards matched fewer than 2 times)
"""

import sys
from pathlib import Path

REPO = Path(__file__).parent.parent.parent.parent
ANON = REPO / "cards_anonymized.txt"

# Import shared match counting (handles multiline TOML + batch results)
sys.path.insert(0, str(Path(__file__).parent))
from match_counts import get_match_counts


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)

    phase = sys.argv[1]
    threshold = 2
    if "--threshold" in sys.argv:
        idx = sys.argv.index("--threshold")
        threshold = int(sys.argv[idx + 1])

    # Count matches per rules text across all sources
    match_counts = get_match_counts()

    # Read anonymized pool and extract rules text from each line
    anon_lines = ANON.read_text().splitlines()
    gap_lines = []
    for line in anon_lines:
        if not line.strip() or line.startswith("==="):
            continue
        # Rules text is everything after the last |
        parts = line.split("|")
        if len(parts) >= 4:
            rules_text = parts[-1].strip()
            count = match_counts.get(rules_text, 0)
            if count < threshold:
                gap_lines.append(line)

    out_path = f"/tmp/art-batch-pool-phase-{phase}.txt"
    with open(out_path, "w") as f:
        f.write("\n".join(gap_lines) + "\n")

    total_cards = sum(1 for l in anon_lines if l.strip() and not l.startswith("==="))
    matched = total_cards - len(gap_lines)
    print(f"Total cards: {total_cards}")
    print(f"Cards with >= {threshold} matches: {matched}")
    print(f"Gap cards (< {threshold} matches): {len(gap_lines)}")
    print(f"Wrote gap pool to {out_path}")

    if len(gap_lines) == 0:
        print(f"\nAll cards matched at least {threshold} times. No phase {phase} needed.")


if __name__ == "__main__":
    main()
