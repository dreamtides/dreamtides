#!/usr/bin/env python3

"""Utilities for appending and retaining `just review` performance logs."""

from __future__ import annotations

import json
import os
from contextlib import contextmanager
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import fcntl


KNOWN_EVENTS = {
    "run_start",
    "run_end",
    "step_start",
    "step_end",
    "command_start",
    "command_end",
    "cargo_compile_start",
    "cargo_compile_end",
    "test_binary_start",
    "test_binary_end",
    "test_case_end",
    "doc_tests_start",
    "doc_tests_end",
    "warning",
}


class ReviewPerfLogError(Exception):
    """Raised when a review performance log operation fails."""


def utc_now_iso() -> str:
    """Returns an ISO-8601 UTC timestamp with millisecond precision."""
    return datetime.now(timezone.utc).isoformat(timespec="milliseconds").replace("+00:00", "Z")


def resolve_log_path(log_path: str | Path | None = None) -> Path:
    """Resolves the effective performance log path."""
    if log_path is not None:
        return Path(log_path)
    configured = os.environ.get("REVIEW_PERF_LOG_PATH")
    if configured:
        return Path(configured)
    return Path(".logs/review.jsonl")


def lock_path_for(log_path: str | Path | None = None) -> Path:
    """Returns the lock file path associated with a log file."""
    path = resolve_log_path(log_path)
    return path.with_suffix(f"{path.suffix}.lock")


def ensure_event_schema(event: dict[str, Any]) -> dict[str, Any]:
    """Validates and normalizes a log event payload."""
    if not isinstance(event, dict):
        raise ReviewPerfLogError("event must be a dictionary")

    normalized = dict(event)
    normalized.setdefault("ts", utc_now_iso())

    event_name = normalized.get("event")
    if not isinstance(event_name, str) or not event_name:
        raise ReviewPerfLogError("event must include non-empty string field 'event'")
    if event_name not in KNOWN_EVENTS:
        raise ReviewPerfLogError(f"unknown event '{event_name}'")

    run_id = normalized.get("run_id")
    if not isinstance(run_id, str) or not run_id:
        raise ReviewPerfLogError("event must include non-empty string field 'run_id'")

    ts = normalized.get("ts")
    if not isinstance(ts, str) or not ts:
        raise ReviewPerfLogError("event must include non-empty string field 'ts'")

    duration = normalized.get("duration_ms")
    if duration is not None and not isinstance(duration, (int, float)):
        raise ReviewPerfLogError("duration_ms must be numeric when present")

    status = normalized.get("status")
    if status is not None and not isinstance(status, str):
        raise ReviewPerfLogError("status must be a string when present")

    return normalized


@contextmanager
def locked_log(log_path: str | Path | None = None):
    """Acquires an exclusive lock for performance log mutation."""
    path = resolve_log_path(log_path)
    path.parent.mkdir(parents=True, exist_ok=True)
    lock_path = lock_path_for(path)
    with open(lock_path, "a", encoding="utf-8") as lock_file:
        fcntl.flock(lock_file.fileno(), fcntl.LOCK_EX)
        try:
            yield path
        finally:
            fcntl.flock(lock_file.fileno(), fcntl.LOCK_UN)


def append_events(events: list[dict[str, Any]], log_path: str | Path | None = None) -> int:
    """Appends multiple NDJSON events to the performance log."""
    if not events:
        return 0

    normalized = [ensure_event_schema(event) for event in events]

    with locked_log(log_path) as path:
        with open(path, "a", encoding="utf-8") as log_file:
            for event in normalized:
                log_file.write(json.dumps(event, sort_keys=True, separators=(",", ":")))
                log_file.write("\n")
            log_file.flush()
            os.fsync(log_file.fileno())

    return len(normalized)


def append_event(event: dict[str, Any], log_path: str | Path | None = None) -> None:
    """Appends a single event to the performance log."""
    append_events([event], log_path)


def estimate_next_run_sequence(log_path: str | Path | None = None) -> int:
    """Returns the next run sequence number inferred from existing logs."""
    path = resolve_log_path(log_path)
    if not path.exists():
        return 1

    max_seq = 0
    with open(path, "r", encoding="utf-8") as log_file:
        for line in log_file:
            line = line.strip()
            if not line:
                continue
            try:
                payload = json.loads(line)
            except json.JSONDecodeError:
                continue
            if payload.get("event") != "run_start":
                continue
            run_seq = payload.get("run_seq")
            if isinstance(run_seq, int):
                max_seq = max(max_seq, run_seq)

    return max_seq + 1


def prune_to_max_runs(max_runs: int, log_path: str | Path | None = None) -> int:
    """Keeps only the newest `max_runs` runs in the NDJSON log file."""
    if max_runs <= 0:
        raise ReviewPerfLogError("max_runs must be > 0")

    with locked_log(log_path) as path:
        if not path.exists():
            return 0

        lines: list[str] = []
        run_latest: dict[str, str] = {}

        with open(path, "r", encoding="utf-8") as log_file:
            for line in log_file:
                line = line.rstrip("\n")
                if not line:
                    continue
                lines.append(line)
                try:
                    payload = json.loads(line)
                except json.JSONDecodeError:
                    continue
                run_id = payload.get("run_id")
                if not isinstance(run_id, str) or not run_id:
                    continue
                ts = payload.get("ts")
                if not isinstance(ts, str) or not ts:
                    continue
                previous = run_latest.get(run_id)
                if previous is None or ts > previous:
                    run_latest[run_id] = ts

        if len(run_latest) <= max_runs:
            return 0

        sorted_runs = sorted(run_latest.items(), key=lambda item: (item[1], item[0]))
        keep_runs = {run_id for run_id, _ in sorted_runs[-max_runs:]}

        retained_lines: list[str] = []
        removed_runs: set[str] = set()
        for line in lines:
            try:
                payload = json.loads(line)
            except json.JSONDecodeError:
                retained_lines.append(line)
                continue
            run_id = payload.get("run_id")
            if isinstance(run_id, str) and run_id and run_id not in keep_runs:
                removed_runs.add(run_id)
                continue
            retained_lines.append(line)

        temp_path = path.with_suffix(f"{path.suffix}.tmp")
        with open(temp_path, "w", encoding="utf-8") as temp_file:
            for line in retained_lines:
                temp_file.write(line)
                temp_file.write("\n")
            temp_file.flush()
            os.fsync(temp_file.fileno())

        os.replace(temp_path, path)
        return len(removed_runs)
