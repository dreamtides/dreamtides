#!/usr/bin/env python3

"""Replace legacy rendered-card tide fields with clean-room tide assignments."""

from __future__ import annotations

import argparse
import json
import re
import sys
import tomllib
from pathlib import Path

OLD_TIDE_KEYS = {"tide", "tide-cost"}
KEY_RE = re.compile(r"^([a-z][a-z0-9-]*)\s*=\s*(.*)")
ID_LINE_RE = re.compile(r'^id\s*=\s*(".*")\s*$')


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--rendered-cards",
        type=Path,
        default=Path("rules_engine/tabula/rendered-cards.toml"),
        help="Path to rendered-cards.toml",
    )
    parser.add_argument(
        "--cards-jsonl",
        type=Path,
        default=Path("rules_engine/tabula/cards.jsonl"),
        help="Path to clean-room cards.jsonl artifact",
    )
    parser.add_argument(
        "--tides-note",
        type=Path,
        default=Path("notes/tides_v2.md"),
        help="Path to tides_v2.md",
    )
    return parser.parse_args()


def parse_toml_string(value: str) -> str:
    return tomllib.loads(f"v = {value}")["v"]


def parse_tide_ids(tides_note_text: str) -> list[str]:
    tide_ids: list[str] = []
    active_section = False

    for line in tides_note_text.splitlines():
        if line in {
            "### Structural Tides",
            "### Support Tides",
            "### Utility Tides",
        }:
            active_section = True
            continue
        if line.startswith("## "):
            active_section = False
            continue
        if active_section and line == "Structural lane notes:":
            active_section = False
            continue
        if not active_section or not line.startswith("- `"):
            continue

        tide_id = line.split("`", 2)[1]
        if tide_id not in tide_ids:
            tide_ids.append(tide_id)

    if not tide_ids:
        raise ValueError("No tide ids found in tides_v2.md")

    return tide_ids


def load_assignments(path: Path) -> tuple[dict[str, list[str]], int]:
    assignments: dict[str, list[str]] = {}
    max_tides = 0

    for line in path.read_text(encoding="utf-8").splitlines():
        row = json.loads(line)
        tides = row["tides"]
        assignments[row["uuid"]] = tides
        max_tides = max(max_tides, len(tides))

    return assignments, max_tides


def parse_card_fields(card_lines: list[str]) -> list[tuple[str, list[str]]]:
    fields: list[tuple[str, list[str]]] = []
    index = 0

    while index < len(card_lines):
        line = card_lines[index]
        match = KEY_RE.match(line)
        if not match:
            index += 1
            continue

        key = match.group(1)
        value_part = match.group(2)
        field_lines = [line]

        if '"""' in value_part and value_part.count('"""') == 1:
            index += 1
            while index < len(card_lines):
                field_lines.append(card_lines[index])
                if '"""' in card_lines[index]:
                    break
                index += 1

        fields.append((key, field_lines))
        index += 1

    return fields


def collect_card_lines(lines: list[str], start: int) -> tuple[list[str], int]:
    card_lines: list[str] = []
    in_multiline = False
    index = start

    while index < len(lines):
        line = lines[index]

        if line.strip() == "" and not in_multiline:
            return card_lines, index
        if not in_multiline and (
            line.strip() == "[[cards]]" or line.strip().startswith("[metadata")
        ):
            return card_lines, index

        card_lines.append(line)

        if line.count('"""') % 2 == 1:
            in_multiline = not in_multiline

        index += 1

    return card_lines, index


def format_tides_line(tides: list[str]) -> str:
    formatted = ", ".join(json.dumps(tide) for tide in tides)
    return f"tides = [{formatted}]"


def rewrite_card_lines(
    card_lines: list[str], assignments: dict[str, list[str]]
) -> tuple[list[str], bool]:
    fields = parse_card_fields(card_lines)
    rewritten: list[str] = []
    card_id: str | None = None

    for key, field_lines in fields:
        if key == "id":
            match = ID_LINE_RE.match(field_lines[0])
            if match is None:
                raise ValueError(f"Could not parse id field: {field_lines[0]!r}")
            card_id = parse_toml_string(match.group(1))

        if key in OLD_TIDE_KEYS or key == "tides":
            continue

        rewritten.extend(field_lines)
        if key == "id" and card_id in assignments:
            rewritten.append(format_tides_line(assignments[card_id]))

    if card_id is None:
        raise ValueError("Card entry missing id field")

    return rewritten, card_id in assignments


def skip_block(lines: list[str], index: int) -> int:
    index += 1
    while index < len(lines) and lines[index].strip() != "":
        index += 1
    while index < len(lines) and lines[index].strip() == "":
        index += 1
    return index


def format_tides_validation_rule(tide_ids: list[str]) -> list[str]:
    return [
        "[[metadata.validation_rules]]",
        'column = "tides"',
        'type = "enum"',
        "enum = [" + ", ".join(json.dumps(tide_id) for tide_id in tide_ids) + "]",
        "",
    ]


def format_columns(max_tides: int) -> list[str]:
    columns: list[tuple[str, int, bool]] = [("name", 200, True), ("id", 140, False)]
    columns.extend((f"tides[{index}]", 180, False) for index in range(max_tides))
    columns.extend(
        [
            ("rendered-text", 320, False),
            ("energy-cost", 100, False),
            ("card-type", 150, False),
            ("subtype", 200, False),
            ("rarity", 125, False),
            ("is-fast", 80, False),
        ]
    )

    result: list[str] = []
    for key, width, bold in columns:
        result.append("[[metadata.columns]]")
        result.append(f'key = "{key}"')
        result.append(f"width = {width}")
        if bold:
            result.append("bold = true")
        result.append("")

    return result


def rewrite_metadata(
    metadata_lines: list[str], tide_ids: list[str], max_tides: int
) -> list[str]:
    result: list[str] = []
    index = 0
    inserted_tides_rule = False
    replaced_columns = False

    while index < len(metadata_lines):
        line = metadata_lines[index]

        if (
            line.strip() == "[[metadata.validation_rules]]"
            and index + 1 < len(metadata_lines)
            and metadata_lines[index + 1].strip()
            in {'column = "tide"', 'column = "tide-cost"'}
        ):
            if not inserted_tides_rule:
                result.extend(format_tides_validation_rule(tide_ids))
                inserted_tides_rule = True
            index = skip_block(metadata_lines, index)
            continue

        if (
            line.strip() == "[[metadata.validation_rules]]"
            and index + 1 < len(metadata_lines)
            and metadata_lines[index + 1].strip() == 'column = "tides"'
        ):
            if inserted_tides_rule:
                index = skip_block(metadata_lines, index)
                continue
            inserted_tides_rule = True

        if line.strip() == "[[metadata.columns]]":
            if not replaced_columns:
                result.extend(format_columns(max_tides))
                replaced_columns = True
            while index < len(metadata_lines):
                stripped = metadata_lines[index].strip()
                if stripped == "[[metadata.columns]]" or stripped == "":
                    index += 1
                    continue
                if re.match(r"^(key|width|bold|frozen)\s*=", stripped):
                    index += 1
                    continue
                break
            continue

        if (
            line.strip() == "[[metadata.statistics]]"
            and index + 1 < len(metadata_lines)
            and metadata_lines[index + 1].strip() == 'column = "tide"'
        ):
            index = skip_block(metadata_lines, index)
            continue

        result.append(line)
        index += 1

    if not inserted_tides_rule:
        result.extend(format_tides_validation_rule(tide_ids))

    if not replaced_columns:
        result.extend(format_columns(max_tides))

    return result


def process_file(
    rendered_cards_path: Path,
    assignments: dict[str, list[str]],
    tide_ids: list[str],
    max_tides: int,
) -> tuple[str, int, int]:
    lines = rendered_cards_path.read_text(encoding="utf-8").split("\n")
    output_lines: list[str] = []
    assigned_count = 0
    unassigned_count = 0
    index = 0

    while index < len(lines):
        line = lines[index]

        if line.strip() == "[metadata]":
            output_lines.extend(rewrite_metadata(lines[index:], tide_ids, max_tides))
            break

        if line.strip() == "[[cards]]":
            output_lines.append(line)
            index += 1
            card_lines, next_index = collect_card_lines(lines, index)
            rewritten_lines, matched_assignment = rewrite_card_lines(
                card_lines, assignments
            )
            output_lines.extend(rewritten_lines)
            if matched_assignment:
                assigned_count += 1
            else:
                unassigned_count += 1
            if next_index < len(lines) and lines[next_index].strip() == "":
                output_lines.append(lines[next_index])
                index = next_index + 1
            else:
                index = next_index
            continue

        output_lines.append(line)
        index += 1

    return "\n".join(output_lines), assigned_count, unassigned_count


def main() -> int:
    args = parse_args()
    assignments, max_tides = load_assignments(args.cards_jsonl)
    tide_ids = parse_tide_ids(args.tides_note.read_text(encoding="utf-8"))
    output, assigned_count, unassigned_count = process_file(
        args.rendered_cards, assignments, tide_ids, max_tides
    )
    args.rendered_cards.write_text(output, encoding="utf-8")
    print(
        f"Updated {args.rendered_cards} with clean-room tides for {assigned_count} cards; "
        f"{unassigned_count} cards remain without a `tides` field."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
