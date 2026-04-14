#!/usr/bin/env python3

"""Prepare and merge artifacts for the tides v2 clean-room assignment run."""

from __future__ import annotations

import argparse
import json
import shutil
from collections import defaultdict
from pathlib import Path

DEFAULT_ROOT = Path("/tmp/tides-v2-clean-room")
DEFAULT_INPUT = Path("rules_engine/tabula/dreamcaller_rank_card_pool.jsonl")
DEFAULT_SHIPPING_OUTPUT = Path("rules_engine/tabula/cards.jsonl")
ROUND_DIRECTORIES = [
    "reference",
    "input",
    "round1_prelim",
    "round2_prelim",
    "round3_audit",
    "round4_patches",
    "round5_resolution",
    "merged",
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Prepare and validate clean-room artifacts for tides_v2_orchestration_plan."
        )
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    prepare_parser = subparsers.add_parser(
        "prepare",
        help="Create the clean-room workspace, assignment packet, and input shards.",
    )
    add_shared_paths(prepare_parser)
    prepare_parser.add_argument(
        "--force",
        action="store_true",
        help="Delete any existing clean-room directory before preparing a new run.",
    )
    prepare_parser.add_argument(
        "--shard-count",
        type=int,
        default=10,
        help="Number of input shards to create.",
    )
    prepare_parser.add_argument(
        "--tides-note",
        type=Path,
        default=Path("notes/tides_v2.md"),
        help="Path to tides_v2.md.",
    )
    prepare_parser.add_argument(
        "--dreamcallers-note",
        type=Path,
        default=Path("notes/dreamcallers.md"),
        help="Path to dreamcallers.md.",
    )

    round3_parser = subparsers.add_parser(
        "merge-round3",
        help="Merge round 3 pair outputs into one validated corpus file.",
    )
    add_shared_paths(round3_parser)

    queue_parser = subparsers.add_parser(
        "build-round5-queues",
        help="Merge round 4 patches and flagged round 3 records into resolution queues.",
    )
    add_shared_paths(queue_parser)
    queue_parser.add_argument(
        "--queue-count",
        type=int,
        default=5,
        help="Number of round 5 resolution queues to create.",
    )

    final_parser = subparsers.add_parser(
        "merge-final",
        help="Merge resolved round 5 outputs into the final validated shipping artifact.",
    )
    add_shared_paths(final_parser)
    final_parser.add_argument(
        "--shipping-output",
        type=Path,
        default=DEFAULT_SHIPPING_OUTPUT,
        help="Final shipping artifact path.",
    )

    return parser.parse_args()


def add_shared_paths(parser: argparse.ArgumentParser) -> None:
    parser.add_argument(
        "--root",
        type=Path,
        default=DEFAULT_ROOT,
        help="Clean-room workspace root.",
    )
    parser.add_argument(
        "--input",
        type=Path,
        default=DEFAULT_INPUT,
        help="Source card-pool JSONL path.",
    )


def read_jsonl(path: Path) -> list[dict]:
    return [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines()]


def write_jsonl(path: Path, rows: list[dict]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        "".join(json.dumps(row, ensure_ascii=False) + "\n" for row in rows),
        encoding="utf-8",
    )


def split_evenly(rows: list[dict], shard_count: int) -> list[list[dict]]:
    base_size, extra = divmod(len(rows), shard_count)
    shards: list[list[dict]] = []
    start = 0
    for index in range(shard_count):
        size = base_size + (1 if index < extra else 0)
        end = start + size
        shards.append(rows[start:end])
        start = end
    return shards


def parse_tide_ids(tides_note_text: str) -> dict[str, list[str]]:
    sections: dict[str, list[str]] = {
        "structural": [],
        "support": [],
        "utility": [],
    }
    current: str | None = None
    for line in tides_note_text.splitlines():
        if line == "### Structural Tides":
            current = "structural"
            continue
        if line == "### Support Tides":
            current = "support"
            continue
        if line == "### Utility Tides":
            current = "utility"
            continue
        if line.startswith("## "):
            current = None
            continue
        if current == "structural" and line == "Structural lane notes:":
            current = None
            continue
        if not current or not line.startswith("- `"):
            continue
        tide_id = line.split("`", 2)[1]
        if tide_id not in sections[current]:
            sections[current].append(tide_id)
    return sections


def extract_bullets(markdown: str, header: str) -> list[str]:
    lines = markdown.splitlines()
    for index, line in enumerate(lines):
        if line.strip() != header:
            continue
        bullets: list[str] = []
        pointer = index + 1
        while pointer < len(lines):
            current = lines[pointer]
            if not current.strip():
                if bullets:
                    break
                pointer += 1
                continue
            if current.startswith("## "):
                break
            if current.startswith("- "):
                bullets.append(current[2:])
                pointer += 1
                while pointer < len(lines) and lines[pointer].startswith("  "):
                    bullets[-1] += " " + lines[pointer].strip()
                    pointer += 1
                continue
            if bullets:
                break
            pointer += 1
        if bullets:
            return bullets
    raise SystemExit(f"Could not find bullet list headed by {header!r}")


def build_assignment_packet(
    tides_note_path: Path,
    dreamcallers_note_path: Path,
) -> str:
    tides_text = tides_note_path.read_text(encoding="utf-8")
    tide_ids = parse_tide_ids(tides_text)
    assignment_targets = extract_bullets(tides_text, "Assignment targets:")
    hard_rules = extract_bullets(tides_text, "Hard rules:")
    validation_bullets = extract_bullets(tides_text, "Global validation:")
    dreamcaller_lines = [
        line.strip()
        for line in dreamcallers_note_path.read_text(encoding="utf-8").splitlines()
        if line.strip() and not line.startswith("#")
    ]

    structural_line = (
        "Structural tides are independently draftable shells with real plans, real "
        "finishers, and enough curve and interaction to anchor a run."
    )
    support_line = (
        "Support tides are overlap-friendly splash packages for setup, smoothing, "
        "light recursion, and glue; they are not where shell-exclusive density payoffs "
        "or repeated-trigger rewards belong."
    )
    utility_line = (
        "Utility tides are the broad draft floor: curve, card flow, interaction, and "
        "texture that multiple shells can draft without heavy synergy demands."
    )
    dreamcaller_summary = (
        "Dreamcaller abilities point at real archetype spines already present in the "
        "game: survivor and void recursion, materialized replay loops, second-event and "
        "event-chaining turns, figment and go-wide boards, judgment-centric bodies and "
        "echo effects, warrior cost reduction, abandon ladders, discard velocity, "
        "character deployment chains, tall spark growth, and prevent-based control. "
        "Use that as context for what packages exist, not as a reason to assign a card "
        "to a tide just because one Dreamcaller would like it."
    )

    sections = [
        "# Tides V2 Assignment Packet",
        "",
        "## Tide Definition",
        (
            "A tide is a draft package, not a faction, keyword bucket, or one-card "
            "mechanic tag. Assign tides based on which packages would happily draft the "
            "card early in a run."
        ),
        "",
        "## Layer Distinction",
        f"- {structural_line}",
        f"- {support_line}",
        f"- {utility_line}",
        "",
        "## Tide IDs",
        "- Structural: "
        + ", ".join(f"`{tide_id}`" for tide_id in tide_ids["structural"]),
        "- Support: " + ", ".join(f"`{tide_id}`" for tide_id in tide_ids["support"]),
        "- Utility: " + ", ".join(f"`{tide_id}`" for tide_id in tide_ids["utility"]),
        "",
        "## Assignment Targets",
        *[f"- {bullet}" for bullet in assignment_targets],
        "",
        "## Hard Rules",
        *[f"- {bullet}" for bullet in hard_rules],
        "",
        "## Validation Bullets",
        *[f"- {bullet}" for bullet in validation_bullets],
        "",
        "## Dreamcaller Context",
        dreamcaller_summary,
        "",
        f"Dreamcaller ability count reviewed: {len(dreamcaller_lines)}.",
        "",
    ]
    return "\n".join(sections)


def prepare_workspace(args: argparse.Namespace) -> None:
    if args.root.exists():
        if not args.force:
            raise SystemExit(
                f"{args.root} already exists; rerun with --force to replace it"
            )
        shutil.rmtree(args.root)

    for directory in ROUND_DIRECTORIES:
        (args.root / directory).mkdir(parents=True, exist_ok=True)

    rows = read_jsonl(args.input)
    shards = split_evenly(rows, args.shard_count)
    for index, shard_rows in enumerate(shards, start=1):
        write_jsonl(args.root / "input" / f"shard-{index:03}.jsonl", shard_rows)

    assignment_packet = build_assignment_packet(
        args.tides_note,
        args.dreamcallers_note,
    )
    (args.root / "reference" / "assignment_packet.md").write_text(
        assignment_packet,
        encoding="utf-8",
    )

    sizes = ", ".join(str(len(shard)) for shard in shards)
    print(
        f"Prepared {args.root} with {len(rows)} cards across {len(shards)} shards "
        f"({sizes})."
    )


def load_assignments_from_directory(directory: Path) -> list[dict]:
    rows: list[dict] = []
    for path in sorted(directory.glob("*.jsonl")):
        rows.extend(read_jsonl(path))
    return rows


def validate_assignment_schema(rows: list[dict], *, minimal: bool) -> None:
    required_keys = {"uuid", "tides"}
    if not minimal:
        required_keys |= {"primary_tide", "confidence", "review_flags", "rationale"}

    for row in rows:
        missing = required_keys - row.keys()
        if missing:
            raise SystemExit(f"Assignment row missing keys {sorted(missing)}: {row}")
        if not isinstance(row["tides"], list) or not row["tides"]:
            raise SystemExit(f"Assignment row must contain non-empty tides: {row}")


def validate_against_input(rows: list[dict], input_rows: list[dict]) -> None:
    if len(rows) != len(input_rows):
        raise SystemExit(
            f"Expected {len(input_rows)} output rows, found {len(rows)} rows"
        )

    input_uuids = [row["uuid"] for row in input_rows]
    output_uuids = [row["uuid"] for row in rows]
    if output_uuids != input_uuids:
        raise SystemExit("Output UUID order does not match input order")


def merge_round3(args: argparse.Namespace) -> None:
    input_rows = read_jsonl(args.input)
    pair_rows = load_assignments_from_directory(args.root / "round3_audit")
    validate_assignment_schema(pair_rows, minimal=False)
    pair_rows_by_uuid = {row["uuid"]: row for row in pair_rows}
    if len(pair_rows_by_uuid) != len(pair_rows):
        raise SystemExit("Round 3 merge found duplicate UUIDs across pair files")
    if set(pair_rows_by_uuid) != {row["uuid"] for row in input_rows}:
        raise SystemExit("Round 3 merge did not cover the full input UUID set")
    ordered_rows = [pair_rows_by_uuid[row["uuid"]] for row in input_rows]
    validate_against_input(ordered_rows, input_rows)
    output_path = args.root / "merged" / "round3_all.jsonl"
    write_jsonl(output_path, ordered_rows)
    print(f"Wrote validated round 3 merge to {output_path}")


def merge_patch_files(directory: Path) -> dict[str, list[dict]]:
    merged: dict[str, list[dict]] = defaultdict(list)
    for path in sorted(directory.glob("*.jsonl")):
        for row in read_jsonl(path):
            if "uuid" not in row:
                raise SystemExit(f"Patch row missing uuid in {path}: {row}")
            merged[row["uuid"]].append(row)
    return merged


def build_round5_queues(args: argparse.Namespace) -> None:
    round3_rows = read_jsonl(args.root / "merged" / "round3_all.jsonl")
    patch_rows = merge_patch_files(args.root / "round4_patches")
    queue_candidates: list[dict] = []

    for row in round3_rows:
        serious_flags = {
            "too_many_tides",
            "support_vs_structural_boundary",
            "utility_tag_suspect",
        }
        flags = set(row.get("review_flags", []))
        if (
            row["uuid"] in patch_rows
            or row.get("confidence") == "low"
            or not serious_flags.isdisjoint(flags)
        ):
            queue_candidates.append(
                {
                    "uuid": row["uuid"],
                    "round3_tides": row["tides"],
                    "primary_tide": row.get("primary_tide"),
                    "confidence": row.get("confidence"),
                    "review_flags": row.get("review_flags", []),
                    "patch_suggestions": patch_rows.get(row["uuid"], []),
                }
            )

    queues = split_evenly(queue_candidates, args.queue_count)
    for index, rows in enumerate(queues, start=1):
        write_jsonl(
            args.root / "round5_resolution" / f"input-queue-{index:02}.jsonl",
            rows,
        )

    print(
        f"Built {len(queues)} round 5 queues covering {len(queue_candidates)} flagged cards."
    )


def validate_final_rows(
    final_rows: list[dict],
    input_rows: list[dict],
    tides_note_text: str,
) -> None:
    validate_assignment_schema(final_rows, minimal=True)
    validate_against_input(final_rows, input_rows)

    tide_ids = parse_tide_ids(tides_note_text)
    allowed_tides = set().union(*tide_ids.values())
    seven_plus_count = 0
    in_range_2_to_5 = 0

    for row in final_rows:
        tides = row["tides"]
        if len(tides) < 1 or len(tides) > 8:
            raise SystemExit(f"Card has invalid tide count {len(tides)}: {row['uuid']}")
        if len(set(tides)) != len(tides):
            raise SystemExit(f"Card has duplicate tides: {row['uuid']}")
        invented = set(tides) - allowed_tides
        if invented:
            raise SystemExit(
                f"Card uses unknown tides {sorted(invented)}: {row['uuid']}"
            )
        if 2 <= len(tides) <= 5:
            in_range_2_to_5 += 1
        if len(tides) >= 7:
            seven_plus_count += 1

    if seven_plus_count >= 20:
        raise SystemExit(
            f"Expected fewer than 20 cards with 7-8 tides, found {seven_plus_count}"
        )

    print(
        "Validated final rows: "
        f"{len(final_rows)} cards, {in_range_2_to_5} with 2-5 tides, "
        f"{seven_plus_count} with 7-8 tides."
    )


def merge_final(args: argparse.Namespace) -> None:
    input_rows = read_jsonl(args.input)
    round3_rows = read_jsonl(args.root / "merged" / "round3_all.jsonl")
    resolved_rows = load_assignments_from_directory(args.root / "round5_resolution")
    resolved_rows = [
        row
        for row in resolved_rows
        if "round3_tides" not in row and "patch_suggestions" not in row
    ]
    validate_assignment_schema(resolved_rows, minimal=True)

    resolved_by_uuid = {row["uuid"]: row for row in resolved_rows}
    final_rows: list[dict] = []
    for round3_row in round3_rows:
        resolved = resolved_by_uuid.get(round3_row["uuid"])
        if resolved is not None:
            final_rows.append({"uuid": resolved["uuid"], "tides": resolved["tides"]})
            continue
        final_rows.append({"uuid": round3_row["uuid"], "tides": round3_row["tides"]})

    tides_note_text = Path("notes/tides_v2.md").read_text(encoding="utf-8")
    validate_final_rows(final_rows, input_rows, tides_note_text)

    merged_output = args.root / "merged" / "final_tides.jsonl"
    write_jsonl(merged_output, final_rows)
    write_jsonl(args.shipping_output, final_rows)
    print(f"Wrote validated final merge to {merged_output}")
    print(f"Wrote shipping artifact to {args.shipping_output}")


def main() -> None:
    args = parse_args()
    if args.command == "prepare":
        prepare_workspace(args)
        return
    if args.command == "merge-round3":
        merge_round3(args)
        return
    if args.command == "build-round5-queues":
        build_round5_queues(args)
        return
    if args.command == "merge-final":
        merge_final(args)
        return
    raise SystemExit(f"Unsupported command: {args.command}")


if __name__ == "__main__":
    main()
