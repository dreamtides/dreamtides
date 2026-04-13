#!/usr/bin/env python3

import argparse
import math
import json
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate ranking JSONL output against an expected UUID set."
    )
    parser.add_argument("expected_jsonl", type=Path)
    parser.add_argument("output_jsonl", type=Path)
    parser.add_argument("--exact", action="store_true")
    parser.add_argument("--require-fields", nargs="*", default=[])
    parser.add_argument("--score-min", type=float, default=0.0)
    parser.add_argument("--score-max", type=float, default=100.0)
    parser.add_argument("--tie-break-min", type=int, default=-2)
    parser.add_argument("--tie-break-max", type=int, default=2)
    parser.add_argument("--require-tie-break", action="store_true")
    parser.add_argument("--max-score-adjustment", type=float)
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


def parse_finite_number(value: object, path: Path, line_number: int, field: str) -> float:
    try:
        parsed = float(value)
    except (TypeError, ValueError) as error:
        raise SystemExit(f"{path}:{line_number}: invalid {field} {value!r}") from error
    if not math.isfinite(parsed):
        raise SystemExit(f"{path}:{line_number}: non-finite {field} {value!r}")
    return parsed


def parse_tie_break(value: object, path: Path, line_number: int) -> int:
    parsed = parse_finite_number(value, path, line_number, "tie_break")
    if not parsed.is_integer():
        raise SystemExit(f"{path}:{line_number}: tie_break must be an integer, got {value!r}")
    return int(parsed)


def read_output(path: Path, args: argparse.Namespace) -> dict[str, dict]:
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
        missing_fields = [field for field in args.require_fields if field not in record]
        if missing_fields:
            raise SystemExit(
                f"{path}:{line_number}: missing required fields: {', '.join(missing_fields)}"
            )
        if uuid in output:
            raise SystemExit(f"{path}:{line_number}: duplicate output uuid {uuid}")
        parsed_score = parse_finite_number(score, path, line_number, "score")
        if parsed_score < args.score_min or parsed_score > args.score_max:
            raise SystemExit(
                f"{path}:{line_number}: score {parsed_score} outside "
                f"[{args.score_min}, {args.score_max}]"
            )
        tie_break = 0
        if "tie_break" in record:
            tie_break = parse_tie_break(record.get("tie_break"), path, line_number)
        elif args.require_tie_break:
            raise SystemExit(f"{path}:{line_number}: missing required field tie_break")
        if tie_break < args.tie_break_min or tie_break > args.tie_break_max:
            raise SystemExit(
                f"{path}:{line_number}: tie_break {tie_break} outside "
                f"[{args.tie_break_min}, {args.tie_break_max}]"
            )
        output[uuid] = {
            "score": parsed_score,
            "tie_break": tie_break,
            "rendered_text": str(rendered_text),
        }
    return output


def main() -> None:
    args = parse_args()
    expected = read_expected(args.expected_jsonl)
    output = read_output(args.output_jsonl, args)

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

    if args.max_score_adjustment is not None:
        for line_number, line in enumerate(args.expected_jsonl.read_text().splitlines(), start=1):
            if not line.strip():
                continue
            record = json.loads(line)
            uuid = record.get("uuid")
            if uuid not in output:
                continue
            if record.get("score") is None:
                raise SystemExit(
                    f"{args.expected_jsonl}:{line_number}: expected records must include score "
                    "when --max-score-adjustment is used"
                )
            baseline_score = parse_finite_number(
                record.get("score"), args.expected_jsonl, line_number, "score"
            )
            delta = abs(output[uuid]["score"] - baseline_score)
            if delta > args.max_score_adjustment:
                raise SystemExit(
                    f"{args.output_jsonl}: uuid {uuid} changed score by {delta:.2f}, "
                    f"exceeding cap {args.max_score_adjustment:.2f}"
                )


if __name__ == "__main__":
    main()
