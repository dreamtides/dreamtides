#!/usr/bin/env python3

import argparse
import json
from pathlib import Path


def load_jsonl(path: Path) -> list[dict]:
    records = []
    seen_uuids = set()
    for line_number, line in enumerate(path.read_text().splitlines(), start=1):
        if not line.strip():
            continue
        try:
            record = json.loads(line)
        except json.JSONDecodeError as error:
            raise SystemExit(f"{path}:{line_number}: invalid JSON: {error}") from error
        uuid = record.get("uuid")
        if not uuid:
            raise SystemExit(f"{path}:{line_number}: missing uuid")
        if uuid in seen_uuids:
            raise SystemExit(f"{path}:{line_number}: duplicate uuid {uuid}")
        seen_uuids.add(uuid)
        records.append(record)
    return records


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Build a deterministic calibration reference set from a card pool."
    )
    parser.add_argument("input_path", type=Path)
    parser.add_argument("output_path", type=Path)
    parser.add_argument("--count", type=int, default=24)
    return parser.parse_args()


def select_reference_records(records: list[dict], count: int) -> list[dict]:
    if count <= 0:
        raise SystemExit("--count must be positive")
    if not records:
        return []
    if count == 1:
        return [records[len(records) // 2]]
    if count >= len(records):
        return records

    indices = []
    for offset in range(count):
        index = round(offset * (len(records) - 1) / (count - 1))
        if index not in indices:
            indices.append(index)

    return [records[index] for index in indices]


def main() -> None:
    args = parse_args()
    records = load_jsonl(args.input_path)
    reference_records = select_reference_records(records, args.count)
    args.output_path.parent.mkdir(parents=True, exist_ok=True)
    args.output_path.write_text(
        "\n".join(json.dumps(record, ensure_ascii=True) for record in reference_records) + "\n"
    )


if __name__ == "__main__":
    main()
