#!/usr/bin/env python3

import argparse
import csv
import json
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Merge Dreamcaller ranking JSONL files into one strict final order."
    )
    parser.add_argument("inputs", nargs="+", type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--expected-count", type=int)
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


def main() -> None:
    args = parse_args()

    merged: dict[str, dict] = {}
    for path in args.inputs:
        for record in read_jsonl(path):
            uuid = record.get("uuid")
            score = record.get("score")
            if not uuid or score is None:
                raise SystemExit(f"{path}: every record must include uuid and score")
            merged[uuid] = {
                "uuid": uuid,
                "score": float(score),
                "tie_break": float(record.get("tie_break", 0)),
            }

    if args.expected_count is not None and len(merged) != args.expected_count:
        raise SystemExit(
            f"expected {args.expected_count} cards, found {len(merged)} after merge"
        )

    ordered = sorted(
        merged.values(),
        key=lambda record: (-record["score"], -record["tie_break"], record["uuid"]),
    )

    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open("w", newline="") as handle:
        writer = csv.writer(handle)
        for record in ordered:
            writer.writerow([record["uuid"], f"{record['score']:.2f}"])


if __name__ == "__main__":
    main()
