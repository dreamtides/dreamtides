#!/usr/bin/env python3

"""Analyzes `.logs/review.jsonl` and summarizes review performance bottlenecks."""

from __future__ import annotations

import argparse
import json
import os
import shlex
import statistics
import subprocess
import tempfile
from collections import defaultdict
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import review_perf_log


@dataclass
class RunSummary:
    """Aggregated metrics for a single review run."""

    run_id: str
    source: str = "default"
    run_seq: int | None = None
    ts_start: str = ""
    ts_end: str = ""
    status: str = "unknown"
    total_ms: float = 0.0
    git_commit: str = ""
    git_branch: str = ""
    step_ms: dict[str, float] = field(default_factory=dict)
    test_binary_ms: list[tuple[str, float, str]] = field(default_factory=list)
    compile_ms: float = 0.0
    doc_tests_ms: float = 0.0


class AnalysisError(Exception):
    """Raised when analysis cannot proceed."""


def parse_args() -> argparse.Namespace:
    """Parses analyzer CLI arguments."""
    parser = argparse.ArgumentParser(description="Analyze just review performance logs")
    parser.add_argument(
        "--log-path",
        default=os.environ.get("REVIEW_PERF_LOG_PATH", ".logs/review.jsonl"),
    )
    parser.add_argument(
        "--runs", type=int, default=25, help="Number of recent runs for trend table"
    )
    parser.add_argument(
        "--window", type=int, default=10, help="Window size for regression deltas"
    )
    parser.add_argument(
        "--top-binaries",
        type=int,
        default=10,
        help="Count of slowest binaries to display",
    )
    parser.add_argument(
        "--include-backfill",
        action="store_true",
        help="Include backfill runs in summaries",
    )
    parser.add_argument(
        "--backfill-commits",
        type=int,
        default=0,
        help="Run sampled historical backfill before analysis",
    )
    parser.add_argument(
        "--sample-every", type=int, default=10, help="Backfill sampling interval"
    )
    parser.add_argument(
        "--backfill-command",
        default="just review",
        help="Command to execute for each backfill sample",
    )
    return parser.parse_args()


def read_events(log_path: Path) -> list[dict[str, Any]]:
    """Reads log events, ignoring malformed lines."""
    if not log_path.exists():
        raise AnalysisError(f"log file does not exist: {log_path}")

    events: list[dict[str, Any]] = []
    with open(log_path, "r", encoding="utf-8") as log_file:
        for line in log_file:
            stripped = line.strip()
            if not stripped:
                continue
            try:
                payload = json.loads(stripped)
            except json.JSONDecodeError:
                continue
            if isinstance(payload, dict):
                events.append(payload)

    return events


def aggregate_runs(
    events: list[dict[str, Any]], include_backfill: bool
) -> list[RunSummary]:
    """Aggregates per-run metrics from raw events."""
    runs: dict[str, RunSummary] = {}

    for event in events:
        run_id = event.get("run_id")
        if not isinstance(run_id, str) or not run_id:
            continue

        run = runs.setdefault(run_id, RunSummary(run_id=run_id))
        event_name = event.get("event")

        if event_name == "run_start":
            run.ts_start = str(event.get("ts", ""))
            run.run_seq = (
                event.get("run_seq")
                if isinstance(event.get("run_seq"), int)
                else run.run_seq
            )
            run.source = str(event.get("source", run.source))
            run.git_commit = str(event.get("git_commit", run.git_commit))
            run.git_branch = str(event.get("git_branch", run.git_branch))
        elif event_name == "run_end":
            run.ts_end = str(event.get("ts", ""))
            run.status = str(event.get("status", run.status))
            duration = event.get("duration_ms")
            if isinstance(duration, (int, float)):
                run.total_ms = float(duration)
            run.source = str(event.get("source", run.source))
        elif event_name == "step_end":
            step_name = event.get("step_name")
            duration = event.get("duration_ms")
            if isinstance(step_name, str) and isinstance(duration, (int, float)):
                run.step_ms[step_name] = float(duration)
        elif event_name == "cargo_compile_end":
            duration = event.get("duration_ms")
            if isinstance(duration, (int, float)):
                run.compile_ms += float(duration)
        elif event_name == "doc_tests_end":
            duration = event.get("duration_ms")
            if isinstance(duration, (int, float)):
                run.doc_tests_ms += float(duration)
        elif event_name == "test_binary_end":
            duration = event.get("duration_ms")
            binary = event.get("binary")
            step_name = str(event.get("step_name", ""))
            if isinstance(duration, (int, float)) and isinstance(binary, str):
                run.test_binary_ms.append((binary, float(duration), step_name))

    summaries = [run for run in runs.values() if run.total_ms > 0]
    if not include_backfill:
        summaries = [run for run in summaries if run.source != "backfill"]

    summaries.sort(key=lambda run: (run.ts_end or run.ts_start, run.run_id))
    return summaries


def format_ms(milliseconds: float) -> str:
    """Formats milliseconds for human-readable output."""
    seconds = milliseconds / 1000.0
    if seconds >= 60:
        minutes = int(seconds // 60)
        remainder = seconds - (minutes * 60)
        return f"{minutes}m {remainder:.1f}s"
    return f"{seconds:.2f}s"


def print_latest_run(run: RunSummary, top_binaries: int) -> None:
    """Prints a detailed summary for the newest run."""
    print(f"Latest run: {run.run_id} ({run.status})")
    print(f"  total: {format_ms(run.total_ms)}")
    if run.git_commit:
        print(f"  commit: {run.git_commit[:12]}  branch: {run.git_branch}")

    if run.step_ms:
        print("\nTop slow steps:")
        for step_name, duration in sorted(
            run.step_ms.items(), key=lambda item: item[1], reverse=True
        )[:8]:
            print(f"  {step_name:<18} {format_ms(duration)}")

    if run.test_binary_ms:
        print("\nTop slow test binaries:")
        top = sorted(run.test_binary_ms, key=lambda item: item[1], reverse=True)[
            :top_binaries
        ]
        for binary, duration, step_name in top:
            short_name = os.path.basename(binary)
            step_label = f" [{step_name}]" if step_name else ""
            print(f"  {short_name:<40} {format_ms(duration)}{step_label}")

    test_run_ms = sum(duration for _, duration, _ in run.test_binary_ms)
    if run.compile_ms or test_run_ms or run.doc_tests_ms:
        print("\nTest split:")
        print(f"  compile:   {format_ms(run.compile_ms)}")
        print(f"  binaries:  {format_ms(test_run_ms)}")
        print(f"  doc tests: {format_ms(run.doc_tests_ms)}")


def print_regression_window(runs: list[RunSummary], window: int) -> None:
    """Prints windowed regression deltas for total and per-step timings."""
    if len(runs) < window * 2:
        print("\nRegression window: not enough runs for delta analysis")
        return

    previous = runs[-(window * 2) : -window]
    recent = runs[-window:]

    prev_mean = statistics.mean(run.total_ms for run in previous)
    recent_mean = statistics.mean(run.total_ms for run in recent)
    delta = recent_mean - prev_mean
    delta_pct = (delta / prev_mean * 100.0) if prev_mean else 0.0

    direction = "slower" if delta > 0 else "faster"
    print("\nRecent regression window:")
    print(
        f"  recent {window} avg: {format_ms(recent_mean)} | previous {window} avg: {format_ms(prev_mean)} | {abs(delta_pct):.2f}% {direction}"
    )

    previous_steps: defaultdict[str, list[float]] = defaultdict(list)
    recent_steps: defaultdict[str, list[float]] = defaultdict(list)

    for run in previous:
        for step_name, duration in run.step_ms.items():
            previous_steps[step_name].append(duration)

    for run in recent:
        for step_name, duration in run.step_ms.items():
            recent_steps[step_name].append(duration)

    step_deltas: list[tuple[str, float, float, float]] = []
    all_steps = set(previous_steps) | set(recent_steps)
    for step_name in all_steps:
        prev_values = previous_steps.get(step_name, [])
        recent_values = recent_steps.get(step_name, [])
        if not prev_values or not recent_values:
            continue
        prev_step_mean = statistics.mean(prev_values)
        recent_step_mean = statistics.mean(recent_values)
        step_delta = recent_step_mean - prev_step_mean
        step_pct = (step_delta / prev_step_mean * 100.0) if prev_step_mean else 0.0
        step_deltas.append((step_name, step_delta, step_pct, recent_step_mean))

    if step_deltas:
        print("\nLargest step regressions:")
        for step_name, step_delta, step_pct, recent_step_mean in sorted(
            step_deltas, key=lambda item: item[1], reverse=True
        )[:6]:
            if step_delta <= 0:
                continue
            print(
                f"  {step_name:<18} +{format_ms(step_delta)} ({step_pct:+.2f}%)  recent avg: {format_ms(recent_step_mean)}"
            )


def print_recent_table(runs: list[RunSummary], count: int) -> None:
    """Prints a compact table for recent runs."""
    print("\nRecent runs:")
    print("  seq      status   total      commit        source")
    for run in runs[-count:]:
        seq = str(run.run_seq) if run.run_seq is not None else "-"
        commit = run.git_commit[:10] if run.git_commit else "-"
        print(
            f"  {seq:<8} {run.status:<7} {format_ms(run.total_ms):<10} {commit:<12} {run.source}"
        )


def run_backfill(
    log_path: Path, commit_count: int, sample_every: int, command: str
) -> None:
    """Runs sampled backfill review executions across historical commits."""
    if commit_count <= 0:
        return

    rev_list = subprocess.check_output(
        ["git", "rev-list", "--first-parent", f"--max-count={commit_count}", "HEAD"],
        text=True,
    ).splitlines()
    sampled = [
        commit
        for index, commit in enumerate(rev_list)
        if index % max(sample_every, 1) == 0
    ]
    if rev_list and rev_list[-1] not in sampled:
        sampled.append(rev_list[-1])

    command_args = shlex.split(command)
    print(f"Running backfill for {len(sampled)} sampled commits...")

    with tempfile.TemporaryDirectory(prefix="review-backfill-") as temp_dir:
        temp_root = Path(temp_dir)
        for index, commit in enumerate(sampled, start=1):
            worktree_dir = temp_root / commit[:12]
            subprocess.check_call(
                ["git", "worktree", "add", "--detach", str(worktree_dir), commit]
            )
            try:
                env = dict(os.environ)
                env["REVIEW_PERF"] = "1"
                env["REVIEW_PERF_SOURCE"] = "backfill"
                env["REVIEW_PERF_LOG_PATH"] = str(log_path)
                env["REVIEW_PERF_RETAIN_RUNS"] = os.environ.get(
                    "REVIEW_PERF_RETAIN_RUNS", "1000"
                )
                print(
                    f"  [{index}/{len(sampled)}] {commit[:12]} -> {' '.join(command_args)}"
                )
                subprocess.check_call(command_args, cwd=worktree_dir, env=env)
            except subprocess.CalledProcessError as exc:
                print(
                    f"    backfill run failed at {commit[:12]} (exit {exc.returncode})"
                )
            finally:
                subprocess.check_call(
                    ["git", "worktree", "remove", "--force", str(worktree_dir)]
                )


def main() -> int:
    """Entrypoint for review log analysis."""
    args = parse_args()
    log_path = Path(args.log_path).resolve()

    if args.backfill_commits > 0:
        run_backfill(
            log_path, args.backfill_commits, args.sample_every, args.backfill_command
        )

    events = read_events(log_path)
    runs = aggregate_runs(events, include_backfill=args.include_backfill)
    if not runs:
        print("No completed review runs found in log.")
        return 0

    latest = runs[-1]
    print_latest_run(latest, args.top_binaries)
    print_regression_window(runs, args.window)
    print_recent_table(runs, args.runs)

    retained = int(os.environ.get("REVIEW_PERF_RETAIN_RUNS", "1000"))
    removed = review_perf_log.prune_to_max_runs(retained, log_path)
    if removed > 0:
        print(f"\nPruned {removed} old runs from log (retained {retained}).")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
