#!/usr/bin/env python3

import argparse
import json
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Fail if any UUID appears in more than one ranking JSONL file."
    )
    parser.add_argument("inputs", nargs="+", type=Path)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    seen_paths = {}
    for path in args.inputs:
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
            if uuid in seen_paths:
                raise SystemExit(
                    f"duplicate uuid {uuid} in {path} and {seen_paths[uuid]}"
                )
            seen_paths[uuid] = str(path)


if __name__ == "__main__":
    main()
