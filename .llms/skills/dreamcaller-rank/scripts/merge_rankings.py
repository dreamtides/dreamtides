#!/usr/bin/env python3

import argparse
import csv
import math
import json
from pathlib import Path


def expected_records(path: Path) -> dict[str, str]:
    expected = {}
    for line_number, line in enumerate(path.read_text().splitlines(), start=1):
        if not line.strip():
            continue
        try:
            record = json.loads(line)
        except json.JSONDecodeError as error:
            raise SystemExit(f"{path}:{line_number}: invalid JSON: {error}") from error
        uuid = record.get("uuid")
        rendered_text = record.get("rendered_text")
        if not uuid or rendered_text is None:
            raise SystemExit(
                f"{path}:{line_number}: expected-jsonl records must include uuid and rendered_text"
            )
        if uuid in expected:
            raise SystemExit(f"{path}:{line_number}: duplicate uuid {uuid}")
        expected[uuid] = str(rendered_text)
    return expected


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Merge Dreamcaller ranking JSONL files into one strict final order."
    )
    parser.add_argument("inputs", nargs="+", type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--expected-count", type=int)
    parser.add_argument("--expected-jsonl", type=Path)
    parser.add_argument("--score-min", type=float, default=0.0)
    parser.add_argument("--score-max", type=float, default=100.0)
    parser.add_argument("--tie-break-min", type=int, default=-2)
    parser.add_argument("--tie-break-max", type=int, default=2)
    return parser.parse_args()


def read_jsonl(path: Path) -> list[dict]:
    records = []
    for line_number, line in enumerate(path.read_text().splitlines(), start=1):
        if not line.strip():
            continue
        try:
            records.append(json.loads(line))
        except json.JSONDecodeError as error:
            raise SystemExit(f"{path}:{line_number}: invalid JSON: {error}") from error
    return records


def parse_finite_number(value: object, path: Path, field: str) -> float:
    try:
        parsed = float(value)
    except (TypeError, ValueError) as error:
        raise SystemExit(f"{path}: invalid {field} {value!r}") from error
    if not math.isfinite(parsed):
        raise SystemExit(f"{path}: non-finite {field} {value!r}")
    return parsed


def parse_tie_break(value: object, path: Path) -> int:
    parsed = parse_finite_number(value, path, "tie_break")
    if not parsed.is_integer():
        raise SystemExit(f"{path}: tie_break must be an integer, got {value!r}")
    return int(parsed)


def main() -> None:
    args = parse_args()

    merged: dict[str, dict] = {}
    source_paths_by_uuid: dict[str, str] = {}
    for path in args.inputs:
        for record in read_jsonl(path):
            uuid = record.get("uuid")
            score = record.get("score")
            rendered_text = record.get("rendered_text")
            if not uuid or score is None or rendered_text is None:
                raise SystemExit(
                    f"{path}: every record must include uuid, score, and rendered_text"
                )
            parsed_score = parse_finite_number(score, path, "score")
            if parsed_score < args.score_min or parsed_score > args.score_max:
                raise SystemExit(
                    f"{path}: score {parsed_score} outside [{args.score_min}, {args.score_max}]"
                )
            parsed_tie_break = parse_tie_break(record.get("tie_break", 0), path)
            if parsed_tie_break < args.tie_break_min or parsed_tie_break > args.tie_break_max:
                raise SystemExit(
                    f"{path}: tie_break {parsed_tie_break} outside "
                    f"[{args.tie_break_min}, {args.tie_break_max}]"
                )
            merged[uuid] = {
                "uuid": uuid,
                "score": parsed_score,
                "tie_break": parsed_tie_break,
                "rendered_text": str(rendered_text),
            }
            source_paths_by_uuid[uuid] = str(path)

    if args.expected_count is not None and len(merged) != args.expected_count:
        raise SystemExit(
            f"expected {args.expected_count} cards, found {len(merged)} after merge"
        )

    if args.expected_jsonl is not None:
        expected = expected_records(args.expected_jsonl)
        missing = sorted(set(expected) - set(merged))
        unexpected = sorted(set(merged) - set(expected))
        mismatched_rendered_text = [
            uuid
            for uuid, expected_rendered_text in expected.items()
            if uuid in merged and merged[uuid]["rendered_text"] != expected_rendered_text
        ]
        if missing or unexpected or mismatched_rendered_text:
            messages = []
            if missing:
                messages.append(f"missing {len(missing)} uuids: {', '.join(missing[:10])}")
            if unexpected:
                messages.append(
                    f"unexpected {len(unexpected)} uuids: {', '.join(unexpected[:10])}"
                )
            if mismatched_rendered_text:
                sample = mismatched_rendered_text[:10]
                details = ", ".join(
                    f"{uuid} from {source_paths_by_uuid.get(uuid, 'unknown source')}"
                    for uuid in sample
                )
                messages.append(
                    "rendered_text mismatch for "
                    f"{len(mismatched_rendered_text)} uuids: {details}"
                )
            raise SystemExit("; ".join(messages))

    ordered = sorted(
        merged.values(),
        key=lambda record: (-record["score"], -record["tie_break"], record["uuid"]),
    )

    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open("w", newline="") as handle:
        writer = csv.writer(handle)
        for record in ordered:
            writer.writerow(
                [
                    record["uuid"],
                    f"{record['score']:.2f}",
                    record["rendered_text"],
                ]
            )


if __name__ == "__main__":
    main()
