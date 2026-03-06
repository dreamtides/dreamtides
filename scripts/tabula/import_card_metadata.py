#!/usr/bin/env python3
"""Map card import CSV boolean columns into card-metadata.toml values."""

from __future__ import annotations

import argparse
import csv
import re
import sys
from pathlib import Path
import tomllib

CSV_TO_METADATA = {
    "DIS": "submerge",
    "WAR": "ignite",
    "FLA": "flash",
    "STO": "surge",
    "SPI": "awaken",
    "FLI": "flicker",
    "SUR": "endure",
    "SAC": "shatter",
}

CARD_NAME_COLUMNS = ("CardName", "1. CardName")
TRUE_VALUES = {"1", "t", "true", "y", "yes"}

NAME_LINE_PATTERN = re.compile(r'^name\s*=\s*(".*")\s*$')
ID_LINE_PATTERN = re.compile(r'^id\s*=\s*(".*")\s*$')
CARD_ID_LINE_PATTERN = re.compile(r'^card-id\s*=\s*(".*")\s*$')
METADATA_LINE_PATTERN = re.compile(
    r"^(flash|awaken|flicker|ignite|shatter|endure|submerge|surge)\s*=\s*\d+\s*$"
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--csv",
        type=Path,
        default=Path("~/Downloads/cards_import.csv").expanduser(),
        help="Path to cards import CSV",
    )
    parser.add_argument(
        "--cards",
        type=Path,
        default=Path("rules_engine/tabula/cards.toml"),
        help="Path to cards.toml",
    )
    parser.add_argument(
        "--metadata",
        type=Path,
        default=Path("rules_engine/tabula/card-metadata.toml"),
        help="Path to card-metadata.toml",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Calculate and report changes without writing the file",
    )
    return parser.parse_args()


def parse_bool(value: str) -> bool:
    return value.strip().lower() in TRUE_VALUES


def parse_toml_string(value: str) -> str:
    return tomllib.loads(f"v = {value}")["v"]


def detect_name_column(fieldnames: list[str]) -> str:
    for candidate in CARD_NAME_COLUMNS:
        if candidate in fieldnames:
            return candidate
    expected = " or ".join(CARD_NAME_COLUMNS)
    raise ValueError(f"CSV is missing card name column ({expected})")


def load_csv_flags(path: Path) -> tuple[dict[str, dict[str, int]], list[str]]:
    with path.open("r", encoding="utf-8-sig", newline="") as handle:
        reader = csv.DictReader(handle)
        fieldnames = list(reader.fieldnames or [])
        if not fieldnames:
            raise ValueError(f"CSV has no header row: {path}")

        name_column = detect_name_column(fieldnames)
        missing_columns = [col for col in CSV_TO_METADATA if col not in fieldnames]
        if missing_columns:
            missing = ", ".join(sorted(missing_columns))
            raise ValueError(f"CSV is missing required columns: {missing}")

        card_flags: dict[str, dict[str, int]] = {}
        duplicate_names: list[str] = []

        for row in reader:
            name = (row.get(name_column) or "").strip()
            if not name:
                continue

            flags = {
                metadata_key: 1 if parse_bool(row.get(csv_key, "")) else 0
                for csv_key, metadata_key in CSV_TO_METADATA.items()
            }

            if name in card_flags and card_flags[name] != flags:
                duplicate_names.append(name)
                flags = {
                    key: max(card_flags[name][key], flags[key])
                    for key in card_flags[name]
                }

            card_flags[name] = flags

    return card_flags, sorted(set(duplicate_names))


def load_name_to_id(path: Path) -> dict[str, str]:
    name_to_id: dict[str, str] = {}
    current_name: str | None = None

    for line in path.read_text(encoding="utf-8").splitlines():
        name_match = NAME_LINE_PATTERN.match(line)
        if name_match:
            current_name = parse_toml_string(name_match.group(1))
            continue

        id_match = ID_LINE_PATTERN.match(line)
        if id_match and current_name:
            card_id = parse_toml_string(id_match.group(1))
            if current_name in name_to_id and name_to_id[current_name] != card_id:
                raise ValueError(f"Card name appears with multiple ids: {current_name}")
            name_to_id[current_name] = card_id
            current_name = None

    if not name_to_id:
        raise ValueError(f"No card name/id entries parsed from {path}")

    return name_to_id


def map_ids(
    card_flags_by_name: dict[str, dict[str, int]], name_to_id: dict[str, str]
) -> tuple[dict[str, dict[str, int]], list[str]]:
    flags_by_id: dict[str, dict[str, int]] = {}
    missing_names: list[str] = []

    for name, flags in card_flags_by_name.items():
        card_id = name_to_id.get(name)
        if not card_id:
            missing_names.append(name)
            continue
        flags_by_id[card_id] = flags

    return flags_by_id, sorted(missing_names)


def update_card_metadata(
    metadata_path: Path, flags_by_id: dict[str, dict[str, int]], dry_run: bool
) -> tuple[int, int, list[str]]:
    lines = metadata_path.read_text(encoding="utf-8").splitlines(keepends=True)
    current_card_id: str | None = None
    touched_ids: set[str] = set()
    changed_lines = 0
    new_lines: list[str] = []

    for line in lines:
        card_id_match = CARD_ID_LINE_PATTERN.match(line.strip())
        if card_id_match:
            current_card_id = parse_toml_string(card_id_match.group(1))
            new_lines.append(line)
            continue

        field_match = METADATA_LINE_PATTERN.match(line.strip())
        if field_match and current_card_id in flags_by_id:
            card_id = current_card_id
            if card_id is None:
                new_lines.append(line)
                continue
            field_name = field_match.group(1)
            updated_value = flags_by_id[card_id][field_name]
            line_ending = "\r\n" if line.endswith("\r\n") else "\n"
            updated_line = f"{field_name} = {updated_value}{line_ending}"
            if updated_line != line:
                changed_lines += 1
            new_lines.append(updated_line)
            touched_ids.add(card_id)
            continue

        new_lines.append(line)

    missing_ids = sorted(set(flags_by_id) - touched_ids)

    if not dry_run and new_lines != lines:
        metadata_path.write_text("".join(new_lines), encoding="utf-8")

    return changed_lines, len(touched_ids), missing_ids


def main() -> int:
    args = parse_args()

    card_flags_by_name, duplicate_names = load_csv_flags(args.csv)
    name_to_id = load_name_to_id(args.cards)
    flags_by_id, missing_names = map_ids(card_flags_by_name, name_to_id)
    changed_lines, touched_cards, missing_ids = update_card_metadata(
        args.metadata, flags_by_id, dry_run=args.dry_run
    )

    print(f"csv_cards={len(card_flags_by_name)}")
    print(f"resolved_card_ids={len(flags_by_id)}")
    print(f"touched_metadata_cards={touched_cards}")
    print(f"changed_lines={changed_lines}")
    print(f"missing_card_names={len(missing_names)}")
    print(f"missing_metadata_ids={len(missing_ids)}")

    if duplicate_names:
        print("duplicate_card_names:")
        for name in duplicate_names:
            print(f"  - {name}")

    if missing_names:
        print("unmatched_card_names:")
        for name in missing_names:
            print(f"  - {name}")

    if missing_ids:
        print("unmatched_metadata_ids:")
        for card_id in missing_ids:
            print(f"  - {card_id}")

    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        print(f"error: {exc}", file=sys.stderr)
        raise SystemExit(1)
