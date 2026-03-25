#!/usr/bin/env python3
"""Filter the anonymized card pool for art-matching.

Reads cards_anonymized.txt and outputs filtered card entries by type
(character/event), cost range, tide, rarity, and subtype.

Usage:
    python3 pool-filter.py characters
    python3 pool-filter.py events
    python3 pool-filter.py characters --cost 2-4
    python3 pool-filter.py events --tide Umbra
    python3 pool-filter.py characters --tide Bloom --cost 3-5
    python3 pool-filter.py characters --cost 2-4 --rarity Rare
    python3 pool-filter.py characters --spark 0-1
    python3 pool-filter.py events --mechanic "dissolve"
    python3 pool-filter.py events --mechanic "draw"

Commands:
    characters              All anonymized character entries
    events                  All anonymized event entries

Filters (combinable):
    --cost LOW-HIGH         Energy cost range (e.g. 2-4, or just 3)
    --spark LOW-HIGH        Spark range (characters only, e.g. 1-3)
    --tide TIDE             Filter to a specific tide
    --rarity RARITY         Filter to a rarity (Common, Uncommon, Rare, Legendary)
    --mechanic KEYWORD      Filter to cards whose rules text contains keyword
    --subtype SUBTYPE       Filter to a character subtype (e.g. Warrior, "Spirit Animal",
                            Survivor, Char). Use "Char" for cards with no mechanical subtype.
"""

import re
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent
REPO_ROOT = SCRIPT_DIR.parent.parent.parent
ANON_FILE = REPO_ROOT / "cards_anonymized.txt"

MECHANICAL_SUBTYPES = {"warrior", "spirit animal", "survivor"}

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


def parse_range(s: str) -> tuple[int, int]:
    """Parse a range string like '2-4' or '3' into (low, high)."""
    if "-" in s:
        parts = s.split("-", 1)
        return int(parts[0]), int(parts[1])
    val = int(s)
    return val, val


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
    cost_range = None
    spark_range = None
    tide_filter = None
    rarity_filter = None
    mechanic_filter = None
    subtype_filter = None

    i = 0
    while i < len(args):
        if args[i] == "--cost" and i + 1 < len(args):
            cost_range = parse_range(args[i + 1])
            i += 2
        elif args[i] == "--spark" and i + 1 < len(args):
            spark_range = parse_range(args[i + 1])
            i += 2
        elif args[i] == "--tide" and i + 1 < len(args):
            tide_filter = args[i + 1].lower()
            i += 2
        elif args[i] == "--rarity" and i + 1 < len(args):
            rarity_filter = args[i + 1][0].upper()  # C, U, R, or L
            i += 2
        elif args[i] == "--mechanic" and i + 1 < len(args):
            mechanic_filter = args[i + 1].lower()
            i += 2
        elif args[i] == "--subtype" and i + 1 < len(args):
            subtype_filter = args[i + 1].lower()
            i += 2
        else:
            print(f"Unknown argument: {args[i]}")
            return

    # Read and parse all card lines
    cards = []
    for line in ANON_FILE.read_text().splitlines():
        card = parse_line(line)
        if card:
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
    if cost_range:
        low, high = cost_range
        filtered = [c for c in filtered if low <= safe_int(c["cost"], 99) <= high]

    if spark_range:
        low, high = spark_range
        filtered = [c for c in filtered if low <= safe_int(c["spark"], 0) <= high]

    if tide_filter:
        filtered = [c for c in filtered if c["tide"].lower() == tide_filter]

    if rarity_filter:
        filtered = [c for c in filtered if c["rarity"] == rarity_filter]

    if mechanic_filter:
        filtered = [c for c in filtered if mechanic_filter in c["text"].lower()]

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
        print(c["raw"])
        count += 1

    filters_desc = []
    if cost_range:
        filters_desc.append(f"cost {cost_range[0]}-{cost_range[1]}")
    if spark_range:
        filters_desc.append(f"spark {spark_range[0]}-{spark_range[1]}")
    if tide_filter:
        filters_desc.append(f"tide={tide_filter}")
    if rarity_filter:
        filters_desc.append(f"rarity={rarity_filter}")
    if mechanic_filter:
        filters_desc.append(f"mechanic contains '{mechanic_filter}'")
    if subtype_filter:
        filters_desc.append(f"subtype={subtype_filter}")
    filter_str = f" [{', '.join(filters_desc)}]" if filters_desc else ""

    print(f"\n--- {count} {label}{filter_str} ---")


if __name__ == "__main__":
    main()
