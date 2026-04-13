#!/usr/bin/env python3

import argparse
import json
from pathlib import Path


def load_jsonl(path: Path) -> list[dict]:
    records = []
    seen_uuids = set()
    for line_number, raw_line in enumerate(path.read_text().splitlines(), start=1):
        if not raw_line.strip():
            continue
        try:
            record = json.loads(raw_line)
        except json.JSONDecodeError as error:
            raise SystemExit(f"{path}:{line_number}: invalid JSON: {error}") from error
        uuid = record.get("uuid")
        if not uuid:
            raise SystemExit(f"{path}:{line_number}: missing uuid")
        if uuid in seen_uuids:
            raise SystemExit(f"{path}:{line_number}: duplicate uuid {uuid}")
        if "rendered_text" not in record:
            raise SystemExit(f"{path}:{line_number}: missing rendered_text")
        seen_uuids.add(uuid)
        records.append(record)
    return records


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Split a JSONL card pool into evenly sized chunk files."
    )
    parser.add_argument("input_path", type=Path)
    parser.add_argument("output_dir", type=Path)
    parser.add_argument("--chunk-size", type=int, default=60)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    if args.chunk_size <= 0:
        raise SystemExit("--chunk-size must be positive")

    records = load_jsonl(args.input_path)
    args.output_dir.mkdir(parents=True, exist_ok=True)

    chunk_paths = []
    for start in range(0, len(records), args.chunk_size):
        chunk_index = start // args.chunk_size + 1
        chunk_path = args.output_dir / f"chunk-{chunk_index:03d}.jsonl"
        chunk_records = records[start : start + args.chunk_size]
        chunk_path.write_text(
            "\n".join(json.dumps(record, ensure_ascii=True) for record in chunk_records) + "\n"
        )
        chunk_paths.append(
            {
                "path": str(chunk_path),
                "records": len(chunk_records),
                "start_index": start,
                "end_index": start + len(chunk_records) - 1,
                "uuids": [record["uuid"] for record in chunk_records],
            }
        )

    manifest = {
        "input_path": str(args.input_path),
        "total_records": len(records),
        "chunk_size": args.chunk_size,
        "uuids": [record["uuid"] for record in records],
        "chunks": chunk_paths,
    }
    (args.output_dir / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")


if __name__ == "__main__":
    main()
