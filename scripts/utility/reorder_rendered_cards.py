#!/usr/bin/env python3
"""Reorder fields in rendered-cards.toml to match desired column display order.

Also updates metadata for the multi-tide schema by replacing legacy tide fields
with array-backed tide columns and preserving the rarity-before-fast layout.

TV displays columns in TOML key discovery order, so reordering the keys
in the data entries controls visual column order.
"""

import re
import sys
from pathlib import Path

# Desired field order for card entries. Fields not listed here
# will be appended after these in their original order.
FIELD_ORDER = [
    "name",
    "id",
    "tides",
    "rendered-text",
    "energy-cost",
    "card-type",
    "subtype",
    "rarity",
    "is-fast",
    "spark",
    "image-number",
    "art-owned",
    "card-number",
]

# Regex to match the start of a TOML key = value line
KEY_RE = re.compile(r"^([a-z][a-z0-9-]*)\s*=\s*(.*)")


def parse_card_fields(card_lines: list[str]) -> list[tuple[str, list[str]]]:
    """Parse card lines into (key, lines) pairs, handling multiline strings."""
    fields: list[tuple[str, list[str]]] = []
    i = 0
    while i < len(card_lines):
        line = card_lines[i]
        match = KEY_RE.match(line)
        if not match:
            i += 1
            continue

        key = match.group(1)
        value_part = match.group(2)
        field_lines = [line]

        # Check if value starts a multiline string (triple quotes)
        if '"""' in value_part:
            # Count triple-quote occurrences in value_part
            count = value_part.count('"""')
            if count == 1:
                # Opening """ without closing — read until closing """
                i += 1
                while i < len(card_lines):
                    field_lines.append(card_lines[i])
                    if '"""' in card_lines[i]:
                        break
                    i += 1

        fields.append((key, field_lines))
        i += 1

    return fields


def reorder_card_fields(card_lines: list[str]) -> list[str]:
    """Reorder fields within a single card entry."""
    fields = parse_card_fields(card_lines)

    # Build lookup by key
    field_map: dict[str, list[str]] = {}
    ordered_keys: list[str] = []
    for key, lines in fields:
        field_map[key] = lines
        ordered_keys.append(key)

    # Build reordered output
    result: list[str] = []
    seen: set[str] = set()

    # First, add fields in the desired order
    for key in FIELD_ORDER:
        if key in field_map:
            result.extend(field_map[key])
            seen.add(key)

    # Then, add any remaining fields in their original order
    for key in ordered_keys:
        if key not in seen:
            result.extend(field_map[key])

    return result


def is_in_multiline_string(lines: list[str], index: int) -> bool:
    """Check if the line at index is inside a triple-quoted multiline string."""
    in_multiline = False
    for i in range(index):
        line = lines[i]
        count = line.count('"""')
        if count % 2 == 1:
            in_multiline = not in_multiline
    return in_multiline


def collect_card_lines(lines: list[str], start: int) -> tuple[list[str], int]:
    """Collect all lines belonging to a card entry starting at start.

    Returns (card_lines, next_index) where next_index is the line after
    the card (the blank line or next section header).
    Handles blank lines inside triple-quoted multiline strings.
    """
    card_lines: list[str] = []
    in_multiline = False
    i = start

    while i < len(lines):
        line = lines[i]

        # Check if this blank line is a card boundary (not inside multiline)
        if line.strip() == "" and not in_multiline:
            return card_lines, i

        # Check for next section header (not inside multiline)
        if not in_multiline and (
            line.strip() == "[[cards]]" or line.strip().startswith("[metadata")
        ):
            return card_lines, i

        card_lines.append(line)

        # Track multiline string state
        count = line.count('"""')
        if count % 2 == 1:
            in_multiline = not in_multiline

        i += 1

    return card_lines, i


def process_file(filepath: Path) -> str:
    """Process the TOML file, reordering card fields and updating metadata."""
    content = filepath.read_text()
    lines = content.split("\n")

    output_lines: list[str] = []

    i = 0
    while i < len(lines):
        line = lines[i]

        # Detect start of metadata section
        if line.strip() == "[metadata]":
            metadata_lines = lines[i:]
            output_lines.extend(rewrite_metadata(metadata_lines))
            break

        # Detect card header
        if line.strip() == "[[cards]]":
            output_lines.append(line)
            i += 1
            # Collect all card lines (handling multiline strings)
            card_lines, next_i = collect_card_lines(lines, i)
            output_lines.extend(reorder_card_fields(card_lines))
            # Add the blank separator line if present
            if next_i < len(lines) and lines[next_i].strip() == "":
                output_lines.append(lines[next_i])
                i = next_i + 1
            else:
                i = next_i
            continue

        output_lines.append(line)
        i += 1

    return "\n".join(output_lines)


def rewrite_metadata(metadata_lines: list[str]) -> list[str]:
    """Rewrite the metadata section for the multi-tide column layout."""
    result = []
    i = 0
    inserted_tides_rule = False

    while i < len(metadata_lines):
        line = metadata_lines[i]

        # Replace legacy tide validation rules with the new array-backed rule.
        if (
            line.strip() == "[[metadata.validation_rules]]"
            and i + 1 < len(metadata_lines)
            and metadata_lines[i + 1].strip()
            in {'column = "tide"', 'column = "tide-cost"'}
        ):
            if not inserted_tides_rule:
                result.append("[[metadata.validation_rules]]")
                result.append('column = "tides"')
                result.append('type = "enum"')
                result.append(
                    'enum = ["warrior_pressure", "warrior_bastion",'
                    ' "spirit_growth", "materialize_value",'
                    ' "materialize_tempo", "ally_formation", "fast_tempo",'
                    ' "event_chain", "prevent_control", "discard_velocity",'
                    ' "void_recursion", "abandon_furnace", "figment_swarm",'
                    ' "survivor_dissolve", "judgment_engines",'
                    ' "character_velocity", "spark_tall", "big_energy",'
                    ' "fast_setup", "hand_cycling", "reclaim_characters",'
                    ' "reclaim_events", "spark_growth",'
                    ' "spark_disruption", "go_wide_enablers",'
                    ' "leave_play_enablers", "bounce_blink_tools",'
                    ' "topdeck_setup", "void_setup",'
                    ' "judgment_repeaters", "event_setup", "copy_effects",'
                    ' "abandon_fodder", "cost_reduction", "trigger_reuse",'
                    ' "character_tutors", "cheap_curve",'
                    ' "defensive_curve", "midcurve_glue", "card_flow",'
                    ' "foresee_selection", "resource_burst",'
                    ' "cheap_removal", "premium_removal",'
                    ' "fast_interaction", "hand_disruption", "sweepers",'
                    ' "finishers", "void_denial", "discover_toolbox",'
                    ' "judgment_bodies", "materialized_staples",'
                    ' "tempo_resets", "point_pressure"]'
                )
                result.append("")
                inserted_tides_rule = True
            while i < len(metadata_lines) and metadata_lines[i].strip() != "":
                i += 1
            while i < len(metadata_lines) and metadata_lines[i].strip() == "":
                i += 1
            continue

        if (
            line.strip() == "[[metadata.validation_rules]]"
            and i + 1 < len(metadata_lines)
            and metadata_lines[i + 1].strip() == 'column = "tides"'
        ):
            if inserted_tides_rule:
                i += 1
                while i < len(metadata_lines) and metadata_lines[i].strip() != "":
                    i += 1
                while i < len(metadata_lines) and metadata_lines[i].strip() == "":
                    i += 1
                continue

            inserted_tides_rule = True

        # Replace columns section
        if line.strip() == "[[metadata.columns]]":
            # Skip all existing column entries
            while i < len(metadata_lines):
                if (
                    metadata_lines[i].strip() == "[[metadata.columns]]"
                    or metadata_lines[i].strip() == ""
                    or re.match(
                        r"^(key|width|bold|frozen)\s*=",
                        metadata_lines[i].strip(),
                    )
                ):
                    i += 1
                else:
                    break
            # Write new column entries
            columns = [
                ("name", 200, True),
                ("id", 140, False),
                ("tides[0]", 180, False),
                ("tides[1]", 180, False),
                ("tides[2]", 180, False),
                ("tides[3]", 180, False),
                ("tides[4]", 180, False),
                ("rendered-text", 320, False),
                ("energy-cost", 100, False),
                ("card-type", 150, False),
                ("subtype", 200, False),
                ("rarity", 125, False),
                ("is-fast", 80, False),
            ]
            for key, width, bold in columns:
                result.append("[[metadata.columns]]")
                result.append(f'key = "{key}"')
                result.append(f"width = {width}")
                if bold:
                    result.append("bold = true")
                result.append("")
            continue

        # Drop the old single-tide statistic.
        if (
            line.strip() == "[[metadata.statistics]]"
            and i + 1 < len(metadata_lines)
            and metadata_lines[i + 1].strip() == 'column = "tide"'
        ):
            i += 1
            while i < len(metadata_lines) and metadata_lines[i].strip() != "":
                i += 1
            while i < len(metadata_lines) and metadata_lines[i].strip() == "":
                i += 1
            continue

        result.append(line)
        i += 1

    return result


def main():
    filepath = Path("client/Assets/StreamingAssets/Tabula/rendered-cards.toml")
    if not filepath.exists():
        print(f"File not found: {filepath}", file=sys.stderr)
        sys.exit(1)

    output = process_file(filepath)
    filepath.write_text(output)
    print(f"Reordered fields in {filepath}")


if __name__ == "__main__":
    main()
