#!/usr/bin/env python3
"""Filter the anonymized card pool for art-matching.

Reads cards_anonymized.txt and outputs filtered card entries by type
(character/event), tide, and subtype.

Usage:
    python3 pool-filter.py characters
    python3 pool-filter.py events
    python3 pool-filter.py characters --tide Umbra
    python3 pool-filter.py characters --subtype Warrior
    python3 pool-filter.py characters --tide Bloom --subtype "Spirit Animal"

Commands:
    characters              All anonymized character entries
    events                  All anonymized event entries

Filters (combinable):
    --tide TIDE             Filter to a specific tide
    --subtype SUBTYPE       Filter to a character subtype (e.g. Warrior, "Spirit Animal",
                            Survivor, Char). Use "Char" for cards with no mechanical subtype.
"""

import re
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent
REPO_ROOT = SCRIPT_DIR.parent.parent.parent
ANON_FILE = REPO_ROOT / "cards_anonymized.txt"

# Import shared match counting (handles multiline TOML + batch results)
sys.path.insert(0, str(REPO_ROOT / ".llms" / "skills" / "art-batch"))
from match_counts import get_match_counts

MECHANICAL_SUBTYPES = {"warrior", "spirit animal", "survivor"}

# Cards assigned this many times or more are excluded from the pool
SATURATION_LIMIT = 4

# Pattern: TideCost | Cost●[/Spark✦] | Type[↯] | R | Rules text
LINE_RE = re.compile(
    r'^(\w+?)(\d+|\*)\s*\|\s*'             # tide + tide-cost
    r'(\d+|\*?)●(?:/(\d+|\*?)✦)?\s*\|\s*'  # cost / optional spark
    r'(.+?)(\u21AF)?\s*\|\s*'              # type + optional fast marker
    r'([CURL])\s*\|\s*'                    # rarity
    r'(.+)$'                               # rules text
)


def parse_line(line: str) -> dict | None:
    """Parse a single anonymized card line into a dict."""
    m = LINE_RE.match(line.strip())
    if not m:
        return None
    tide, tide_cost, cost, spark, card_type, fast, rarity, text = m.groups()
    card_type = card_type.strip()
    is_event = card_type == "Event"
    return {
        "tide": tide,
        "tide-cost": tide_cost,
        "cost": cost,
        "spark": spark or "",
        "type": card_type,
        "is-event": is_event,
        "is-fast": fast == "↯" if fast else False,
        "rarity": rarity,
        "text": text.strip(),
        "raw": line.strip(),
    }


def safe_int(s: str, default: int = 0) -> int:
    try:
        return int(s)
    except (ValueError, TypeError):
        return default


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        return

    command = sys.argv[1]
    args = sys.argv[2:]

    # Parse optional filters
    tide_filter = None
    subtype_filter = None

    i = 0
    while i < len(args):
        if args[i] == "--tide" and i + 1 < len(args):
            tide_filter = args[i + 1].lower()
            i += 2
        elif args[i] == "--subtype" and i + 1 < len(args):
            subtype_filter = args[i + 1].lower()
            i += 2
        else:
            print(f"Unknown argument: {args[i]}")
            return

    # Read and parse all card lines, excluding saturated cards
    match_counts = get_match_counts()
    saturated = {t for t, c in match_counts.items() if c >= SATURATION_LIMIT}
    cards = []
    for line in ANON_FILE.read_text().splitlines():
        card = parse_line(line)
        if card and card["text"] not in saturated:
            card["match_count"] = match_counts.get(card["text"], 0)
            cards.append(card)

    # Filter by command type
    if command == "characters":
        filtered = [c for c in cards if not c["is-event"]]
        label = "characters"
    elif command == "events":
        filtered = [c for c in cards if c["is-event"]]
        label = "events"
    else:
        print(f"Unknown command: {command}")
        print("Valid commands: characters, events")
        return

    # Apply optional filters
    if tide_filter:
        filtered = [c for c in filtered if c["tide"].lower() == tide_filter]

    if subtype_filter:
        if subtype_filter == "char":
            filtered = [c for c in filtered
                        if c["type"].lower() not in MECHANICAL_SUBTYPES]
        else:
            filtered = [c for c in filtered
                        if c["type"].lower() == subtype_filter]

    # Sort by tide then cost
    tide_order = {"Bloom": 0, "Arc": 1, "Ignite": 2, "Pact": 3,
                  "Umbra": 4, "Rime": 5, "Surge": 6, "Neutral": 7}
    filtered.sort(key=lambda c: (
        tide_order.get(c["tide"], 8),
        safe_int(c["cost"], 99),
    ))

    # Group by tide for display
    current_tide = None
    count = 0
    for c in filtered:
        t = c["tide"]
        if t != current_tide:
            current_tide = t
            tide_cards = [x for x in filtered if x["tide"] == t]
            print(f"\n=== {t.upper()} ({len(tide_cards)}) ===")
        mc = c["match_count"]
        if mc >= 2:
            print(f"⚠{mc}× {c['raw']}")
        elif mc == 1:
            print(f" 1× {c['raw']}")
        else:
            print(f"    {c['raw']}")
        count += 1

    filters_desc = []
    if tide_filter:
        filters_desc.append(f"tide={tide_filter}")
    if subtype_filter:
        filters_desc.append(f"subtype={subtype_filter}")
    filter_str = f" [{', '.join(filters_desc)}]" if filters_desc else ""

    print(f"\n--- {count} {label}{filter_str} ---")
    if saturated:
        print(f"({len(saturated)} cards hidden — assigned {SATURATION_LIMIT}+ times)")


if __name__ == "__main__":
    main()
