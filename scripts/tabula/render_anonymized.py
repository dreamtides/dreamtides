#!/usr/bin/env python3

"""Produces an anonymized text file from rendered-cards.toml."""

import tomllib
import sys
from pathlib import Path

RARITY_MAP = {"Common": "C", "Uncommon": "U", "Rare": "R", "Legendary": "L"}


def format_card(card):
    tide = card["tide"]
    tide_cost = card["tide-cost"]
    energy_cost = card["energy-cost"]
    card_type = card["card-type"]
    subtype = card.get("subtype", "")
    rarity = RARITY_MAP.get(card["rarity"], card["rarity"])
    is_fast = card.get("is-fast", False)
    rendered_text = card.get("rendered-text", "").strip()
    # Collapse multiline text to single line
    rendered_text = " ".join(rendered_text.split())

    if card_type == "Character":
        spark = card.get("spark", 0)
        if spark == "":
            spark = 0
        type_col = subtype if subtype else "Character"
        text = f"↯fast -- {rendered_text}" if is_fast else rendered_text
        return f"{tide}{tide_cost} | {energy_cost}●/{spark}✦ | {type_col} | {rarity} | {text}"
    else:
        type_col = "Fast Event" if is_fast else card_type
        return f"{tide}{tide_cost} | {energy_cost}● | {type_col} | {rarity} | {rendered_text}"


def main():
    script_dir = Path(__file__).parent
    toml_path = (
        script_dir.parent.parent / "rules_engine" / "tabula" / "rendered-cards.toml"
    )
    output_path = toml_path.parent / "rendered_cards_anonymized.txt"

    if not toml_path.exists():
        print(f"Error: {toml_path} not found", file=sys.stderr)
        sys.exit(1)

    with open(toml_path, "rb") as f:
        data = tomllib.load(f)

    lines = []
    for card in data.get("cards", []):
        lines.append(format_card(card))

    with open(output_path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")

    print(f"Wrote {len(lines)} cards to {output_path}")


if __name__ == "__main__":
    main()
