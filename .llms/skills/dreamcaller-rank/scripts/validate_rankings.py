#!/usr/bin/env python3

import argparse
import json
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate ranking JSONL output against an expected UUID set."
    )
    parser.add_argument("expected_jsonl", type=Path)
    parser.add_argument("output_jsonl", type=Path)
    parser.add_argument("--exact", action="store_true")
    return parser.parse_args()


def read_expected(path: Path) -> dict[str, str]:
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
                f"{path}:{line_number}: expected records must include uuid and rendered_text"
            )
        if uuid in expected:
            raise SystemExit(f"{path}:{line_number}: duplicate expected uuid {uuid}")
        expected[uuid] = str(rendered_text)
    return expected


def read_output(path: Path) -> dict[str, dict]:
    output = {}
    for line_number, line in enumerate(path.read_text().splitlines(), start=1):
        if not line.strip():
            continue
        try:
            record = json.loads(line)
        except json.JSONDecodeError as error:
            raise SystemExit(f"{path}:{line_number}: invalid JSON: {error}") from error
        uuid = record.get("uuid")
        score = record.get("score")
        rendered_text = record.get("rendered_text")
        if not uuid or score is None or rendered_text is None:
            raise SystemExit(
                f"{path}:{line_number}: output records must include uuid, score, and rendered_text"
            )
        if uuid in output:
            raise SystemExit(f"{path}:{line_number}: duplicate output uuid {uuid}")
        output[uuid] = {
            "score": float(score),
            "rendered_text": str(rendered_text),
        }
    return output


def main() -> None:
    args = parse_args()
    expected = read_expected(args.expected_jsonl)
    output = read_output(args.output_jsonl)

    missing = sorted(set(expected) - set(output))
    unexpected = sorted(set(output) - set(expected))
    mismatched_rendered_text = [
        uuid
        for uuid, expected_rendered_text in expected.items()
        if uuid in output and output[uuid]["rendered_text"] != expected_rendered_text
    ]

    if args.exact and (missing or unexpected or mismatched_rendered_text):
        messages = []
        if missing:
            messages.append(f"missing {len(missing)} uuids: {', '.join(missing[:10])}")
        if unexpected:
            messages.append(f"unexpected {len(unexpected)} uuids: {', '.join(unexpected[:10])}")
        if mismatched_rendered_text:
            messages.append(
                "rendered_text mismatch for "
                f"{len(mismatched_rendered_text)} uuids: {', '.join(mismatched_rendered_text[:10])}"
            )
        raise SystemExit("; ".join(messages))


if __name__ == "__main__":
    main()
