#!/usr/bin/env python3
"""Filter the anonymized card pool for art-matching.

Reads rendered-cards.toml and outputs anonymized card entries filtered by
type (character/event), cost range, tide, and rarity. Designed for the
art-match skill so agents can quickly narrow down candidate cards.

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
    python3 pool-filter.py unassigned-characters
    python3 pool-filter.py unassigned-events

Commands:
    characters              All anonymized character entries
    events                  All anonymized event entries
    unassigned-characters   Characters without art (no image-number)
    unassigned-events       Events without art (no image-number)

Filters (combinable):
    --cost LOW-HIGH         Energy cost range (e.g. 2-4, or just 3)
    --spark LOW-HIGH        Spark range (characters only, e.g. 1-3)
    --tide TIDE             Filter to a specific tide
    --rarity RARITY         Filter to a rarity (Common, Uncommon, Rare, Legendary)
    --mechanic KEYWORD      Filter to cards whose rules text contains keyword
    --subtype SUBTYPE       Filter to a character subtype (e.g. Warrior, "Spirit Animal",
                            Survivor, Char). Use "Char" for cards with no mechanical subtype.
"""

import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent
REPO_ROOT = SCRIPT_DIR.parent.parent.parent
RENDERED_CARDS = REPO_ROOT / "rules_engine" / "tabula" / "rendered-cards.toml"

MECHANICAL_SUBTYPES = {"warrior", "spirit animal", "survivor"}


def parse_toml_entries(path: Path, entry_key: str) -> list[dict]:
    """Parse a TOML file with repeated [[entry_key]] entries into a list of dicts."""
    cards = []
    current: dict = {}
    in_multiline = False
    multiline_key = ""
    multiline_value = ""
    in_metadata = False

    for line in path.read_text().splitlines():
        stripped = line.strip()

        if stripped == "[metadata]" or stripped.startswith("[metadata."):
            in_metadata = True
            continue
        if in_metadata:
            if stripped.startswith("[[") and not stripped.startswith("[[metadata"):
                in_metadata = False
            else:
                continue

        if in_multiline:
            if '"""' in stripped:
                multiline_value += " " + stripped.replace('"""', "").strip()
                current[multiline_key] = multiline_value.strip()
                in_multiline = False
            else:
                multiline_value += " " + stripped
            continue

        if stripped == f"[[{entry_key}]]":
            if current:
                cards.append(current)
            current = {}
            continue

        if "=" in stripped and not stripped.startswith("#") and not stripped.startswith("[["):
            key, _, value = stripped.partition("=")
            key = key.strip()
            value = value.strip()
            if value.startswith('"""'):
                content = value[3:]
                if content.endswith('"""'):
                    current[key] = content[:-3].strip()
                else:
                    in_multiline = True
                    multiline_key = key
                    multiline_value = content
            elif value.startswith('"') and value.endswith('"'):
                current[key] = value[1:-1]
            else:
                current[key] = value

    if current:
        cards.append(current)
    return cards


def parse_range(s: str) -> tuple[int, int]:
    """Parse a range string like '2-4' or '3' into (low, high)."""
    if "-" in s:
        parts = s.split("-", 1)
        return int(parts[0]), int(parts[1])
    val = int(s)
    return val, val


def format_anon(card: dict) -> str:
    """Format a card in anonymized one-line format."""
    tide = card.get("tide", "")
    tc = card.get("tide-cost", "")
    cost = card.get("energy-cost", "?")
    spark = card.get("spark", "")
    ct = card.get("card-type", "?")
    sub = card.get("subtype", "")
    fast = "↯" if card.get("is-fast", "false") == "true" else ""
    rarity = card.get("rarity", "")[0] if card.get("rarity", "") else ""
    text = card.get("rendered-text", "").replace("\\n", " ").strip()

    if sub.lower() not in MECHANICAL_SUBTYPES:
        sub = ""

    if ct == "Character":
        type_str = sub if sub and sub != "*" else "Char"
        stat = f"{cost}●/{spark}✦"
    else:
        type_str = "Event"
        stat = f"{cost}●"

    return f"{tide}{tc} | {stat} | {type_str}{fast} | {rarity} | {text}"


def safe_cost(card: dict) -> int:
    try:
        return int(card.get("energy-cost", 0))
    except ValueError:
        return 99


def safe_spark(card: dict) -> int:
    try:
        return int(card.get("spark", 0))
    except ValueError:
        return 0


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
            rarity_filter = args[i + 1].lower()
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

    cards = parse_toml_entries(RENDERED_CARDS, "cards")

    # Filter by command type
    unassigned_only = command.startswith("unassigned-")
    if command in ("characters", "unassigned-characters"):
        filtered = [c for c in cards if c.get("card-type") == "Character"]
        label = "CHARACTERS"
    elif command in ("events", "unassigned-events"):
        filtered = [c for c in cards if c.get("card-type") == "Event"]
        label = "EVENTS"
    else:
        print(f"Unknown command: {command}")
        print("Valid commands: characters, events, unassigned-characters, unassigned-events")
        return

    # Filter to unassigned (no image-number) if requested
    if unassigned_only:
        filtered = [c for c in filtered if not c.get("image-number")]
        label = f"UNASSIGNED {label}"

    # Apply optional filters
    if cost_range:
        low, high = cost_range
        filtered = [c for c in filtered if low <= safe_cost(c) <= high]

    if spark_range:
        low, high = spark_range
        filtered = [c for c in filtered if low <= safe_spark(c) <= high]

    if tide_filter:
        filtered = [c for c in filtered if c.get("tide", "").lower() == tide_filter]

    if rarity_filter:
        filtered = [c for c in filtered if c.get("rarity", "").lower() == rarity_filter]

    if mechanic_filter:
        filtered = [c for c in filtered
                    if mechanic_filter in c.get("rendered-text", "").lower()]

    if subtype_filter:
        if subtype_filter == "char":
            # "Char" means cards with no mechanical subtype (not Warrior/Spirit Animal/Survivor)
            filtered = [c for c in filtered
                        if c.get("subtype", "").lower() not in MECHANICAL_SUBTYPES]
        else:
            filtered = [c for c in filtered
                        if c.get("subtype", "").lower() == subtype_filter]

    # Sort by tide then cost
    tide_order = {"Bloom": 0, "Arc": 1, "Ignite": 2, "Pact": 3,
                  "Umbra": 4, "Rime": 5, "Surge": 6, "Neutral": 7}
    filtered.sort(key=lambda c: (
        tide_order.get(c.get("tide", ""), 8),
        safe_cost(c),
    ))

    # Group by tide for display
    current_tide = None
    count = 0
    for c in filtered:
        t = c.get("tide", "")
        if t != current_tide:
            current_tide = t
            tide_cards = [x for x in filtered if x.get("tide", "") == t]
            print(f"\n=== {t.upper() or '(none)'} ({len(tide_cards)}) ===")
        print(format_anon(c))
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

    print(f"\n--- {count} {label.lower()}{filter_str} ---")


if __name__ == "__main__":
    main()
