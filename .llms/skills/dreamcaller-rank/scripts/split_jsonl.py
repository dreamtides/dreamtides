#!/usr/bin/env python3

import argparse
import json
from pathlib import Path


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

    lines = []
    for line_number, raw_line in enumerate(args.input_path.read_text().splitlines(), start=1):
        if not raw_line.strip():
            continue
        try:
            record = json.loads(raw_line)
        except json.JSONDecodeError as error:
            raise SystemExit(
                f"{args.input_path}:{line_number}: invalid JSON: {error}"
            ) from error
        if "uuid" not in record:
            raise SystemExit(f"{args.input_path}:{line_number}: missing uuid")
        lines.append(raw_line)
    args.output_dir.mkdir(parents=True, exist_ok=True)

    chunk_paths = []
    for start in range(0, len(lines), args.chunk_size):
        chunk_index = start // args.chunk_size + 1
        chunk_path = args.output_dir / f"chunk-{chunk_index:03d}.jsonl"
        chunk_lines = lines[start : start + args.chunk_size]
        chunk_path.write_text("\n".join(chunk_lines) + "\n")
        chunk_paths.append(
            {
                "path": str(chunk_path),
                "records": len(chunk_lines),
                "start_index": start,
                "end_index": start + len(chunk_lines) - 1,
            }
        )

    manifest = {
        "input_path": str(args.input_path),
        "total_records": len(lines),
        "chunk_size": args.chunk_size,
        "chunks": chunk_paths,
    }
    (args.output_dir / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")


if __name__ == "__main__":
    main()
