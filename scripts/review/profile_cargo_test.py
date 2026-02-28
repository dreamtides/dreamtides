#!/usr/bin/env python3

"""Profiles cargo test execution and emits structured performance events."""

from __future__ import annotations

import argparse
import json
import os
import re
import shlex
import subprocess
import sys
import time
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import review_perf_log

TEST_RESULT_PATTERN = re.compile(
    r"test result: .*? (?P<passed>\d+) passed; (?P<failed>\d+) failed; (?P<ignored>\d+) ignored; (?P<measured>\d+) measured; (?P<filtered>\d+) filtered out"
)
NIGHTLY_FAILURE_MARKERS = (
    "only accepted on the nightly compiler",
    "Unrecognized option: 'format'",
    "Unrecognized option: 'Z'",
)


@dataclass(frozen=True)
class TestBinary:
    """Metadata for a discovered test binary."""

    executable: str
    manifest_dir: str
    package_id: str
    target_name: str


@dataclass
class BinaryExecutionResult:
    """Execution result and parsed counts for a test binary."""

    exit_code: int
    output: str
    error_output: str
    passed: int
    failed: int
    ignored: int
    measured: int
    filtered: int

    @property
    def executed(self) -> int:
        """Returns the count of tests that were executed."""
        return self.passed + self.failed + self.ignored + self.measured


def parse_args() -> argparse.Namespace:
    """Parses CLI arguments."""
    parser = argparse.ArgumentParser(
        description="Profile cargo test execution for just review"
    )
    parser.add_argument("--manifest-path", required=True, help="Cargo manifest path")
    parser.add_argument(
        "--package", action="append", default=[], help="Cargo package name"
    )
    parser.add_argument("--workspace", action="store_true", help="Run across workspace")
    parser.add_argument(
        "--exclude", action="append", default=[], help="Excluded packages"
    )
    parser.add_argument("--quiet", action="store_true", help="Pass quiet mode")
    parser.add_argument("--test-threads", type=int, help="libtest --test-threads value")
    parser.add_argument(
        "--detail",
        choices=["stable", "nightly"],
        default=os.environ.get("REVIEW_PERF_DETAIL", "stable"),
    )
    parser.add_argument(
        "--require-match", action="store_true", help="Fail if filters match no tests"
    )
    parser.add_argument(
        "--include-doc-tests",
        action="store_true",
        default=os.environ.get("REVIEW_PERF_INCLUDE_DOC_TESTS", "0") == "1",
        help="Also execute doc tests (disabled by default to preserve just review behavior)",
    )
    parser.add_argument(
        "--step-name",
        default=os.environ.get("REVIEW_PERF_STEP", ""),
        help="Logical review step name",
    )
    parser.add_argument("filters", nargs="*", help="Optional test filters")
    args = parser.parse_args()

    if not args.workspace and not args.package:
        parser.error("one of --workspace or --package is required")

    if args.workspace and args.package:
        parser.error("--workspace cannot be combined with --package")

    return args


def should_log_perf() -> bool:
    """Returns whether perf events should be persisted."""
    return os.environ.get("REVIEW_PERF", "1") != "0"


def active_run_id() -> str:
    """Returns the active run ID used for emitted events."""
    cached = getattr(active_run_id, "_value", "")
    if cached:
        return cached

    run_id = os.environ.get("REVIEW_PERF_RUN_ID")
    if not run_id:
        run_id = f"adhoc-{uuid.uuid4().hex[:12]}"

    setattr(active_run_id, "_value", run_id)
    return run_id


def emit_event(event: str, payload: dict[str, Any]) -> None:
    """Emits a single perf event if perf logging is enabled."""
    if not should_log_perf():
        return

    record = {
        "event": event,
        "run_id": active_run_id(),
        "source": os.environ.get("REVIEW_PERF_SOURCE", "default"),
    }

    step_name = os.environ.get("REVIEW_PERF_STEP") or payload.get("step_name")
    if isinstance(step_name, str) and step_name:
        record["step_name"] = step_name

    record.update(payload)
    try:
        review_perf_log.append_event(record)
    except (
        Exception
    ) as exc:  # pragma: no cover - logging failures should not fail tests
        print(f"Warning: failed to append perf event: {exc}", file=sys.stderr)


def cargo_scope_args(args: argparse.Namespace) -> list[str]:
    """Builds cargo scope arguments based on requested package/workspace scope."""
    scope: list[str] = ["--manifest-path", args.manifest_path]
    if args.workspace:
        scope.append("--workspace")
    else:
        for package in args.package:
            scope.extend(["-p", package])
    for excluded in args.exclude:
        scope.extend(["--exclude", excluded])
    if args.quiet:
        scope.append("-q")
    return scope


def discover_test_binaries(args: argparse.Namespace) -> list[TestBinary]:
    """Compiles tests without running and returns discovered test binaries."""
    compile_cmd = [
        "cargo",
        "test",
        *cargo_scope_args(args),
        "--no-run",
        "--message-format=json",
    ]
    emit_event(
        "cargo_compile_start",
        {
            "command": shlex.join(compile_cmd),
            "status": "started",
            "detail_mode": args.detail,
        },
    )

    started = time.monotonic()
    process = subprocess.run(compile_cmd, capture_output=True, text=True)

    binaries: list[TestBinary] = []
    for line in process.stdout.splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            payload = json.loads(line)
        except json.JSONDecodeError:
            continue

        if payload.get("reason") != "compiler-artifact":
            continue
        if not payload.get("profile", {}).get("test"):
            continue

        executable = payload.get("executable")
        manifest_path = payload.get("manifest_path")
        target = payload.get("target", {})
        if not isinstance(executable, str) or not executable:
            continue
        if not isinstance(manifest_path, str) or not manifest_path:
            continue

        binaries.append(
            TestBinary(
                executable=executable,
                manifest_dir=str(Path(manifest_path).resolve().parent),
                package_id=str(payload.get("package_id", "")),
                target_name=str(target.get("name", "unknown")),
            )
        )

    stderr_output = process.stderr
    exit_code = process.returncode
    elapsed_ms = round((time.monotonic() - started) * 1000, 3)

    if exit_code != 0:
        if stderr_output:
            sys.stderr.write(stderr_output)
        emit_event(
            "cargo_compile_end",
            {
                "status": "failed",
                "duration_ms": elapsed_ms,
                "exit_code": exit_code,
            },
        )
        raise subprocess.CalledProcessError(exit_code, compile_cmd)

    deduped: list[TestBinary] = []
    seen: set[str] = set()
    for binary in binaries:
        if binary.executable in seen:
            continue
        seen.add(binary.executable)
        deduped.append(binary)

    emit_event(
        "cargo_compile_end",
        {
            "status": "ok",
            "duration_ms": elapsed_ms,
            "exit_code": 0,
            "binary_count": len(deduped),
        },
    )

    return deduped


def parse_test_result_counts(text: str) -> tuple[int, int, int, int, int]:
    """Parses libtest result summary counts from command output."""
    matches = list(TEST_RESULT_PATTERN.finditer(text))
    if not matches:
        return (0, 0, 0, 0, 0)
    match = matches[-1]
    return (
        int(match.group("passed")),
        int(match.group("failed")),
        int(match.group("ignored")),
        int(match.group("measured")),
        int(match.group("filtered")),
    )


def emit_nightly_test_case_events(lines: list[str]) -> None:
    """Emits per-test-case events from libtest nightly JSON output."""
    for line in lines:
        try:
            payload = json.loads(line)
        except json.JSONDecodeError:
            continue
        if payload.get("type") != "test":
            continue
        event = payload.get("event")
        if event not in {"ok", "failed", "ignored"}:
            continue

        duration_s = payload.get("exec_time")
        duration_ms = (
            round(float(duration_s) * 1000, 3)
            if isinstance(duration_s, (float, int))
            else None
        )
        emit_payload: dict[str, Any] = {
            "status": "ok" if event == "ok" else event,
            "test_name": payload.get("name", ""),
        }
        if duration_ms is not None:
            emit_payload["duration_ms"] = duration_ms
        emit_event("test_case_end", emit_payload)


def run_binary(
    binary: TestBinary, args: argparse.Namespace, use_nightly_json: bool
) -> BinaryExecutionResult:
    """Runs a single discovered test binary."""
    command = [binary.executable]
    if use_nightly_json:
        command.extend(["-Z", "unstable-options", "--format", "json"])
    if args.test_threads is not None:
        command.append(f"--test-threads={args.test_threads}")
    if args.quiet:
        command.append("-q")
    command.extend(args.filters)

    emit_event(
        "test_binary_start",
        {
            "status": "started",
            "binary": binary.executable,
            "target_name": binary.target_name,
            "package_id": binary.package_id,
            "command": shlex.join(command),
        },
    )

    started = time.monotonic()
    process = subprocess.Popen(
        command,
        cwd=binary.manifest_dir,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    output, error_output = process.communicate()
    elapsed_ms = round((time.monotonic() - started) * 1000, 3)

    if output:
        sys.stdout.write(output)
    if error_output:
        sys.stderr.write(error_output)

    if use_nightly_json:
        emit_nightly_test_case_events(output.splitlines())

    passed, failed, ignored, measured, filtered = parse_test_result_counts(
        f"{output}\n{error_output}"
    )

    emit_event(
        "test_binary_end",
        {
            "status": "ok" if process.returncode == 0 else "failed",
            "duration_ms": elapsed_ms,
            "binary": binary.executable,
            "target_name": binary.target_name,
            "package_id": binary.package_id,
            "exit_code": process.returncode,
            "passed": passed,
            "failed": failed,
            "ignored": ignored,
            "measured": measured,
            "filtered_out": filtered,
        },
    )

    return BinaryExecutionResult(
        exit_code=process.returncode,
        output=output,
        error_output=error_output,
        passed=passed,
        failed=failed,
        ignored=ignored,
        measured=measured,
        filtered=filtered,
    )


def run_doc_tests(args: argparse.Namespace) -> int:
    """Runs doc tests for compatibility with cargo test semantics."""
    if args.filters:
        return 0

    doc_cmd = ["cargo", "test", *cargo_scope_args(args), "--doc"]
    emit_event(
        "doc_tests_start",
        {
            "status": "started",
            "command": shlex.join(doc_cmd),
        },
    )
    started = time.monotonic()
    return_code = subprocess.call(doc_cmd)
    elapsed_ms = round((time.monotonic() - started) * 1000, 3)
    emit_event(
        "doc_tests_end",
        {
            "status": "ok" if return_code == 0 else "failed",
            "duration_ms": elapsed_ms,
            "exit_code": return_code,
        },
    )
    return return_code


def should_fallback_from_nightly(result: BinaryExecutionResult) -> bool:
    """Returns whether nightly JSON execution should fall back to stable mode."""
    if result.exit_code == 0:
        return False
    combined = f"{result.output}\n{result.error_output}"
    return any(marker in combined for marker in NIGHTLY_FAILURE_MARKERS)


def main() -> int:
    """Entrypoint for profiling cargo tests."""
    args = parse_args()

    if args.step_name:
        os.environ["REVIEW_PERF_STEP"] = args.step_name

    binaries = discover_test_binaries(args)

    total_executed = 0
    used_nightly = args.detail == "nightly"

    for binary in binaries:
        result = run_binary(binary, args, use_nightly_json=used_nightly)
        if used_nightly and should_fallback_from_nightly(result):
            emit_event(
                "warning",
                {
                    "status": "degraded",
                    "message": "nightly test JSON unavailable; rerunning binary in stable mode",
                    "binary": binary.executable,
                },
            )
            result = run_binary(binary, args, use_nightly_json=False)
            used_nightly = False

        total_executed += result.executed
        if result.exit_code != 0:
            return result.exit_code

    if args.include_doc_tests:
        doc_result = run_doc_tests(args)
        if doc_result != 0:
            return doc_result

    if args.require_match and args.filters and total_executed == 0:
        print(
            f"Error: No tests matched filters: {' '.join(args.filters)}",
            file=sys.stderr,
        )
        return 1

    print("Success")
    return 0


if __name__ == "__main__":
    sys.exit(main())
