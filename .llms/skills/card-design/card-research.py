#!/usr/bin/env python3
"""Card research tool for Dreamtides card design.

Queries rendered-cards.toml to find cards by tide, mechanic, subtype, name,
or keyword. Designed to be fast and targeted so card designers can explore
the existing design space without reading the entire 8000-line file.

Usage:
    python3 card-research.py tide <tide_name>
    python3 card-research.py tide-events <tide_name>
    python3 card-research.py tide-characters <tide_name>
    python3 card-research.py mechanic <keyword> [keyword2 ...]
    python3 card-research.py subtype <subtype_name>
    python3 card-research.py name <search_term>
    python3 card-research.py cost <energy_cost>
    python3 card-research.py cost-in-tide <tide_name> <energy_cost>
    python3 card-research.py stats
    python3 card-research.py similar <mechanic_description>
    python3 card-research.py where <keyword>

Examples:
    python3 card-research.py tide Rime
    python3 card-research.py tide-events Umbra
    python3 card-research.py tide-characters Bloom
    python3 card-research.py mechanic prevent
    python3 card-research.py mechanic discard kindle
    python3 card-research.py subtype Survivor
    python3 card-research.py name "Storm"
    python3 card-research.py cost 3
    python3 card-research.py cost-in-tide Umbra 3
    python3 card-research.py stats
    python3 card-research.py similar "when you discard"
    python3 card-research.py where kindle
"""

import sys
import re
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent
REPO_ROOT = SCRIPT_DIR.parent.parent.parent
RENDERED_CARDS = REPO_ROOT / "rules_engine" / "tabula" / "rendered-cards.toml"
DREAMWELL = REPO_ROOT / "rules_engine" / "tabula" / "dreamwell.toml"


def parse_toml_entries(path: Path, entry_key: str) -> list[dict]:
    """Parse a TOML file with repeated [[entry_key]] entries into a list of dicts.

    Handles multiline strings (triple-quoted) and skips [metadata] sections.
    """
    cards = []
    current: dict = {}
    in_multiline = False
    multiline_key = ""
    multiline_value = ""
    in_metadata = False

    for line in path.read_text().splitlines():
        stripped = line.strip()

        # Skip metadata sections
        if stripped == "[metadata]" or stripped.startswith("[metadata."):
            in_metadata = True
            continue
        if in_metadata:
            if stripped.startswith("[[") and not stripped.startswith("[[metadata"):
                in_metadata = False
            else:
                continue

        # Handle multiline strings
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


def parse_cards(path: Path) -> list[dict]:
    """Parse rendered-cards.toml into a list of card dicts."""
    return parse_toml_entries(path, "cards")


def parse_dreamwell(path: Path) -> list[dict]:
    """Parse dreamwell.toml into a list of card dicts."""
    return parse_toml_entries(path, "dreamwell")


def format_card(card: dict) -> str:
    """Format a single card for display."""
    name = card.get("name", "???")
    tide = card.get("tide", "")
    tide_cost = card.get("tide-cost", "")
    cost = card.get("energy-cost", "?")
    card_type = card.get("card-type", "?")
    subtype = card.get("subtype", "")
    spark = card.get("spark", "")
    fast = card.get("is-fast", "false")
    rarity = card.get("rarity", "")
    text = card.get("rendered-text", "").replace("\\n", " ").strip()

    header = f"  {name}"
    if tide:
        header += f" [{tide} {tide_cost}]"
    header += f" — {cost}●"
    if card_type == "Character":
        header += f", {spark}✦"
        if subtype and subtype != "*":
            header += f" ({subtype})"
        elif subtype == "*":
            header += " (all types)"
    else:
        header += " (Event)"
    if fast == "true":
        header += " ↯fast"
    if rarity:
        header += f" [{rarity}]"
    result = header + "\n"
    if text:
        result += f"    {text}\n"
    return result


def cmd_tide(args: list[str], cards: list[dict]):
    """Show all cards in a given tide."""
    tide = args[0] if args else ""
    matches = [c for c in cards if c.get("tide", "").lower() == tide.lower()]
    if not matches:
        print(f"No cards found for tide '{tide}'")
        print(f"Valid tides: Arc, Bloom, Ignite, Neutral, Pact, Rime, Surge, Umbra")
        return

    chars = [c for c in matches if c.get("card-type") == "Character"]
    events = [c for c in matches if c.get("card-type") == "Event"]

    chars.sort(key=lambda c: int(c.get("energy-cost", 0)))
    events.sort(key=lambda c: int(c.get("energy-cost", 0)))

    print(f"\n=== {tide.upper()} — {len(matches)} cards ({len(chars)} characters, {len(events)} events) ===\n")
    if chars:
        print("CHARACTERS:")
        for c in chars:
            print(format_card(c))
    if events:
        print("EVENTS:")
        for c in events:
            print(format_card(c))


def cmd_mechanic(args: list[str], cards: list[dict]):
    """Find cards whose rules text contains ALL given keywords."""
    if not args:
        print("Usage: card-research.py mechanic <keyword> [keyword2 ...]")
        return

    keywords = [k.lower() for k in args]
    matches = []
    for c in cards:
        text = c.get("rendered-text", "").lower()
        name = c.get("name", "").lower()
        combined = text + " " + name
        if all(k in combined for k in keywords):
            matches.append(c)

    print(f"\n=== Cards matching [{', '.join(args)}] — {len(matches)} results ===\n")
    for c in sorted(matches, key=lambda c: (c.get("tide", ""), int(c.get("energy-cost", 0)))):
        print(format_card(c))


def cmd_subtype(args: list[str], cards: list[dict]):
    """Find all cards with a given character subtype."""
    subtype = args[0] if args else ""
    matches = [c for c in cards if c.get("subtype", "").lower() == subtype.lower()]

    if not matches:
        all_subtypes = sorted(set(
            c.get("subtype", "") for c in cards
            if c.get("subtype", "") and c.get("subtype", "") != "*"
        ))
        print(f"No cards found with subtype '{subtype}'")
        print(f"Valid subtypes: {', '.join(all_subtypes)}")
        return

    print(f"\n=== {subtype} cards — {len(matches)} results ===\n")
    for c in sorted(matches, key=lambda c: (c.get("tide", ""), int(c.get("energy-cost", 0)))):
        print(format_card(c))


def cmd_name(args: list[str], cards: list[dict]):
    """Find cards whose name contains the search term."""
    term = " ".join(args).lower() if args else ""
    matches = [c for c in cards if term in c.get("name", "").lower()]

    print(f"\n=== Cards with '{' '.join(args)}' in name — {len(matches)} results ===\n")
    for c in sorted(matches, key=lambda c: c.get("name", "")):
        print(format_card(c))


def cmd_cost(args: list[str], cards: list[dict]):
    """Find all cards at a specific energy cost."""
    cost = args[0] if args else "0"
    matches = [c for c in cards if c.get("energy-cost", "") == cost]

    chars = [c for c in matches if c.get("card-type") == "Character"]
    events = [c for c in matches if c.get("card-type") == "Event"]

    print(f"\n=== Cards at {cost}● — {len(matches)} total ({len(chars)} characters, {len(events)} events) ===\n")
    for c in sorted(matches, key=lambda c: (c.get("tide", ""), c.get("name", ""))):
        print(format_card(c))


def cmd_stats(args: list[str], cards: list[dict]):
    """Show aggregate statistics about the card pool."""
    # Cards per tide
    tides: dict[str, int] = {}
    for c in cards:
        t = c.get("tide", "(none)")
        tides[t] = tides.get(t, 0) + 1

    print(f"\n=== Card Pool Statistics ({len(cards)} total cards) ===\n")
    print("Cards per tide:")
    for t in sorted(tides.keys()):
        print(f"  {t:10s}: {tides[t]}")

    # Subtypes
    subtypes: dict[str, int] = {}
    for c in cards:
        st = c.get("subtype", "")
        if st and st != "*":
            subtypes[st] = subtypes.get(st, 0) + 1
    print(f"\nCharacter subtypes ({len(subtypes)} types):")
    for st, count in sorted(subtypes.items(), key=lambda x: -x[1]):
        # Find which tides use this subtype
        tide_uses: dict[str, int] = {}
        for c in cards:
            if c.get("subtype", "") == st:
                t = c.get("tide", "?")
                tide_uses[t] = tide_uses.get(t, 0) + 1
        tide_str = ", ".join(f"{t}:{n}" for t, n in sorted(tide_uses.items(), key=lambda x: -x[1]))
        print(f"  {st:15s}: {count:3d} cards  ({tide_str})")

    # Spark curve
    print("\nAverage spark by energy cost (characters only):")
    cost_sparks: dict[int, list[int]] = {}
    for c in cards:
        if c.get("card-type") == "Character":
            try:
                cost_val = int(c.get("energy-cost", 0))
                spark_val = int(c.get("spark", 0))
                cost_sparks.setdefault(cost_val, []).append(spark_val)
            except ValueError:
                pass
    print(f"  {'Cost':>4s}  {'Count':>5s}  {'Avg':>5s}  {'Min':>3s}  {'Max':>3s}")
    for cost_val in sorted(cost_sparks.keys()):
        sparks = cost_sparks[cost_val]
        avg = sum(sparks) / len(sparks)
        print(f"  {cost_val:4d}  {len(sparks):5d}  {avg:5.1f}  {min(sparks):3d}  {max(sparks):3d}")

    # Fast cards per tide
    print("\nFast cards per tide:")
    fast_tides: dict[str, int] = {}
    for c in cards:
        if c.get("is-fast", "false") == "true":
            t = c.get("tide", "?")
            fast_tides[t] = fast_tides.get(t, 0) + 1
    for t in sorted(fast_tides.keys(), key=lambda x: -fast_tides[x]):
        print(f"  {t:10s}: {fast_tides[t]}")

    # Rarity distribution
    print("\nRarity distribution:")
    rarities: dict[str, int] = {}
    for c in cards:
        r = c.get("rarity", "?")
        rarities[r] = rarities.get(r, 0) + 1
    for r in ["Common", "Uncommon", "Rare", "Legendary", "Special"]:
        if r in rarities:
            print(f"  {r:12s}: {rarities[r]}")

    # Dreamwell summary
    if DREAMWELL.exists():
        dw_cards = parse_dreamwell(DREAMWELL)
        print(f"\nDreamwell cards ({len(dw_cards)} total):")
        for dw in dw_cards:
            name = dw.get("name", "?")
            energy = dw.get("energy-produced", "?")
            phase = dw.get("phase", "?")
            rules = dw.get("rules-text", "(none)")
            variables = dw.get("variables", "")
            print(f"  {name}: Phase {phase}, +{energy}●, effect: {rules} {variables}")


def cmd_tide_events(args: list[str], cards: list[dict]):
    """Show only events in a given tide, sorted by cost."""
    tide = args[0] if args else ""
    matches = [
        c for c in cards
        if c.get("tide", "").lower() == tide.lower() and c.get("card-type") == "Event"
    ]
    if not matches:
        print(f"No events found for tide '{tide}'")
        print(f"Valid tides: Arc, Bloom, Ignite, Neutral, Pact, Rime, Surge, Umbra")
        return

    matches.sort(key=lambda c: int(c.get("energy-cost", 0)))
    print(f"\n=== {tide.upper()} EVENTS — {len(matches)} events ===\n")
    for c in matches:
        print(format_card(c))


def cmd_tide_characters(args: list[str], cards: list[dict]):
    """Show only characters in a given tide, sorted by cost."""
    tide = args[0] if args else ""
    matches = [
        c for c in cards
        if c.get("tide", "").lower() == tide.lower() and c.get("card-type") == "Character"
    ]
    if not matches:
        print(f"No characters found for tide '{tide}'")
        print(f"Valid tides: Arc, Bloom, Ignite, Neutral, Pact, Rime, Surge, Umbra")
        return

    matches.sort(key=lambda c: int(c.get("energy-cost", 0)))
    print(f"\n=== {tide.upper()} CHARACTERS — {len(matches)} characters ===\n")
    for c in matches:
        print(format_card(c))


def cmd_cost_in_tide(args: list[str], cards: list[dict]):
    """Show cards at a specific cost within a specific tide."""
    if len(args) < 2:
        print("Usage: card-research.py cost-in-tide <tide> <cost>")
        return

    tide = args[0]
    cost = args[1]
    matches = [
        c for c in cards
        if c.get("tide", "").lower() == tide.lower() and c.get("energy-cost", "") == cost
    ]

    chars = [c for c in matches if c.get("card-type") == "Character"]
    events = [c for c in matches if c.get("card-type") == "Event"]

    print(f"\n=== {tide.upper()} at {cost}● — {len(matches)} total ({len(chars)} characters, {len(events)} events) ===\n")
    for c in sorted(matches, key=lambda c: c.get("name", "")):
        print(format_card(c))


def cmd_where(args: list[str], cards: list[dict]):
    """Show which tides use a mechanic keyword, with counts and card names."""
    if not args:
        print("Usage: card-research.py where <keyword>")
        return

    keyword = args[0].lower()
    tide_cards: dict[str, list[str]] = {}
    for c in cards:
        text = c.get("rendered-text", "").lower()
        name = c.get("name", "")
        if keyword in text or keyword in name.lower():
            t = c.get("tide", "(none)")
            tide_cards.setdefault(t, []).append(name)

    total = sum(len(v) for v in tide_cards.values())
    print(f"\n=== '{args[0]}' across tides — {total} total cards ===\n")
    for t in sorted(tide_cards.keys(), key=lambda x: -len(tide_cards[x])):
        names = tide_cards[t]
        print(f"  {t:10s}: {len(names):2d} cards")
        for n in sorted(names):
            print(f"    - {n}")
    print()


def cmd_dump(args: list[str], cards: list[dict]):
    """Dump all cards in compact one-line-per-card format for reading into context.

    Optional argument: a tide name to dump only that tide (e.g. dump Bloom).
    """
    tide_order = ["Bloom", "Arc", "Ignite", "Pact", "Umbra", "Rime", "Surge", "Neutral", ""]
    filter_tide = args[0].lower() if args else None
    by_tide: dict[str, list[dict]] = {}
    for c in cards:
        t = c.get("tide", "")
        by_tide.setdefault(t, []).append(c)

    for tide in tide_order:
        if tide not in by_tide:
            continue
        if filter_tide and tide.lower() != filter_tide:
            continue
        def safe_cost(c: dict) -> int:
            try:
                return int(c.get("energy-cost", 0))
            except ValueError:
                return 99

        tide_cards = sorted(by_tide[tide], key=lambda c: (
            0 if c.get("card-type") == "Character" else 1,
            safe_cost(c),
            c.get("name", ""),
        ))
        label = tide if tide else "(no tide)"
        chars = [c for c in tide_cards if c.get("card-type") == "Character"]
        events = [c for c in tide_cards if c.get("card-type") == "Event"]
        print(f"\n=== {label.upper()} ({len(chars)}C, {len(events)}E) ===")
        for c in tide_cards:
            name = c.get("name", "???")
            tc = c.get("tide-cost", "")
            cost = c.get("energy-cost", "?")
            spark = c.get("spark", "")
            ct = c.get("card-type", "?")
            sub = c.get("subtype", "")
            fast = "↯" if c.get("is-fast", "false") == "true" else ""
            rarity = c.get("rarity", "")[0] if c.get("rarity", "") else ""
            text = c.get("rendered-text", "").replace("\\n", " ").strip()

            if ct == "Character":
                type_str = sub if sub and sub != "*" else "Char"
                stat = f"{cost}●/{spark}✦"
            else:
                type_str = "Event"
                stat = f"{cost}●"

            print(f"{name} | {tide}{tc} | {stat} | {type_str}{fast} | {rarity} | {text}")

    # Dreamwell cards (skip when filtering to a specific tide)
    if DREAMWELL.exists() and not filter_tide:
        dw_cards = parse_dreamwell(DREAMWELL)
        print(f"\n=== DREAMWELL ({len(dw_cards)} cards) ===")
        for dw in dw_cards:
            name = dw.get("name", "?")
            energy = dw.get("energy-produced", "?")
            phase = dw.get("phase", "?")
            rules = dw.get("rules-text", "(none)")
            variables = dw.get("variables", "")
            print(f"{name} | Phase {phase} | +{energy}● | {rules} {variables}")


def cmd_similar(args: list[str], cards: list[dict]):
    """Find cards with similar rules text (case-insensitive substring match)."""
    if not args:
        print("Usage: card-research.py similar <text to search for>")
        return
    term = " ".join(args).lower()
    matches = [c for c in cards if term in c.get("rendered-text", "").lower()]
    print(f"\n=== Cards with rules text containing '{' '.join(args)}' — {len(matches)} results ===\n")
    for c in sorted(matches, key=lambda c: (c.get("tide", ""), int(c.get("energy-cost", 0)))):
        print(format_card(c))


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        return

    command = sys.argv[1]
    args = sys.argv[2:]

    cards = parse_cards(RENDERED_CARDS)

    commands = {
        "dump": cmd_dump,
        "tide": cmd_tide,
        "tide-events": cmd_tide_events,
        "tide-characters": cmd_tide_characters,
        "mechanic": cmd_mechanic,
        "subtype": cmd_subtype,
        "name": cmd_name,
        "cost": cmd_cost,
        "cost-in-tide": cmd_cost_in_tide,
        "stats": cmd_stats,
        "similar": cmd_similar,
        "where": cmd_where,
    }

    if command in commands:
        commands[command](args, cards)
    else:
        print(f"Unknown command: {command}")
        print(f"Valid commands: {', '.join(commands.keys())}")


if __name__ == "__main__":
    main()
