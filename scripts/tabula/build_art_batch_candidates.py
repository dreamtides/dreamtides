#!/usr/bin/env python3
"""
Build art-batch-candidates.toml from art-assigned.toml and /tmp/art-batch-results/,
keeping only entries whose rendered-text matches one of the 90 anonymized pool cards.
"""

import sys
import tomllib
import pathlib
import glob as glob_module
import re

ANONYMIZED_FILE = pathlib.Path("/Users/dthurn/dreamtides/cards_anonymized.txt")
ART_ASSIGNED = pathlib.Path("/Users/dthurn/dreamtides/rules_engine/tabula/art-assigned.toml")
BATCH_RESULTS_DIR = pathlib.Path("/tmp/art-batch-results")
OUTPUT_FILE = pathlib.Path("/Users/dthurn/dreamtides/rules_engine/tabula/art-batch-candidates.toml")


def load_pool_rendered_texts():
    """Load the 90-card anonymized pool and return set of rendered-text strings."""
    texts = set()
    with open(ANONYMIZED_FILE, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            # Format: Tide | cost/spark | Subtype | Rarity | rendered-text
            parts = line.split(" | ", 4)
            if len(parts) >= 5:
                rendered_text = parts[4].strip()
                texts.add(rendered_text)
    return texts


def load_toml_cards(path: pathlib.Path) -> list[dict]:
    """Load [[cards]] entries from a TOML file."""
    try:
        with open(path, "rb") as f:
            data = tomllib.load(f)
        return data.get("cards", [])
    except Exception as e:
        print(f"Warning: failed to parse {path}: {e}", file=sys.stderr)
        return []


def toml_value(v) -> str:
    """Serialize a Python value back to TOML inline format."""
    if isinstance(v, bool):
        return "true" if v else "false"
    elif isinstance(v, int):
        return str(v)
    elif isinstance(v, float):
        return str(v)
    elif isinstance(v, str):
        # Escape backslashes and double-quotes
        escaped = v.replace("\\", "\\\\").replace('"', '\\"')
        return f'"{escaped}"'
    elif v is None:
        return '""'
    else:
        return f'"{v}"'


def card_to_toml_block(card: dict) -> str:
    """Render a single card dict as a [[cards]] TOML block."""
    lines = ["[[cards]]"]
    for key, val in card.items():
        lines.append(f"{key} = {toml_value(val)}")
    return "\n".join(lines)


def main():
    pool_texts = load_pool_rendered_texts()
    print(f"Loaded {len(pool_texts)} unique rendered-texts from anonymized pool.")

    # Collect all cards from art-assigned.toml
    all_cards = load_toml_cards(ART_ASSIGNED)
    print(f"Loaded {len(all_cards)} cards from art-assigned.toml.")

    # Collect all cards from /tmp/art-batch-results/*.toml
    result_files = sorted(BATCH_RESULTS_DIR.glob("*.toml"))
    batch_count = 0
    for rf in result_files:
        cards = load_toml_cards(rf)
        all_cards.extend(cards)
        batch_count += len(cards)
    print(f"Loaded {batch_count} cards from {len(result_files)} batch result files.")
    print(f"Total cards before filtering: {len(all_cards)}")

    # Filter to only pool cards, then group by rendered-text
    groups: dict[str, list[dict]] = {}
    for card in all_cards:
        rt = card.get("rendered-text", "")
        if rt in pool_texts:
            groups.setdefault(rt, []).append(card)

    # Sort groups alphabetically by rendered-text
    sorted_texts = sorted(groups.keys())

    # Write output
    with open(OUTPUT_FILE, "w", encoding="utf-8") as out:
        first = True
        for rt in sorted_texts:
            for card in groups[rt]:
                if not first:
                    out.write("\n")
                out.write(card_to_toml_block(card))
                out.write("\n")
                first = False

    total_entries = sum(len(v) for v in groups.values())
    print(f"\nWrote {total_entries} entries representing {len(sorted_texts)} unique cards to:")
    print(f"  {OUTPUT_FILE}")


if __name__ == "__main__":
    main()
