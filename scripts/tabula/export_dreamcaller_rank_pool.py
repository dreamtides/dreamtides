#!/usr/bin/env python3

"""Export an anonymized Dreamcaller ranking pool from rendered-cards.toml."""

import argparse
import json
import tomllib
from pathlib import Path

EXCLUDED_RARITIES = {"Special", "Starter"}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Export an anonymized JSONL card pool for the dreamcaller-rank skill."
        )
    )
    parser.add_argument(
        "--input",
        type=Path,
        default=Path("rules_engine/tabula/rendered-cards.toml"),
        help="Path to rendered-cards.toml",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("rules_engine/tabula/dreamcaller_rank_card_pool.jsonl"),
        help="Destination JSONL path",
    )
    return parser.parse_args()


def normalize_spark(value: object) -> int | str:
    if value == "":
        return 0
    if isinstance(value, int):
        return value
    if value == "*":
        return value
    raise SystemExit(f"Unsupported spark value: {value!r}")


def normalize_cost(value: object) -> int | str:
    if isinstance(value, int):
        return value
    if value == "*":
        return value
    raise SystemExit(f"Unsupported energy-cost value: {value!r}")


def export_row(card: dict) -> dict:
    return {
        "uuid": card["id"],
        "card_type": card["card-type"],
        "cost": normalize_cost(card["energy-cost"]),
        "spark": normalize_spark(card.get("spark", "")),
        "subtype": card.get("subtype") or None,
        "is_fast": bool(card.get("is-fast", False)),
        "rendered_text": str(card.get("rendered-text", "")),
    }


def main() -> None:
    args = parse_args()
    with args.input.open("rb") as handle:
        data = tomllib.load(handle)

    cards = data.get("cards", [])
    filtered = [
        export_row(card)
        for card in cards
        if card.get("rarity") not in EXCLUDED_RARITIES
    ]

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(
        "".join(json.dumps(row, ensure_ascii=False) + "\n" for row in filtered),
        encoding="utf-8",
    )

    print(
        f"Wrote {len(filtered)} cards to {args.output} "
        f"(excluded rarities: {', '.join(sorted(EXCLUDED_RARITIES))})"
    )


if __name__ == "__main__":
    main()
