#!/usr/bin/env python3

"""Runs `just review` stages with structured performance logging."""

from __future__ import annotations

import argparse
import os
import shlex
import socket
import subprocess
import sys
import time
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import review_perf_log
import review_scope


@dataclass(frozen=True)
class CommandSpec:
    """A concrete shell command belonging to a review step."""

    name: str
    argv: list[str]


@dataclass(frozen=True)
class StepSpec:
    """A logical review step with one or more concrete commands."""

    name: str
    commands: list[CommandSpec]


def run_id_now() -> str:
    """Builds a unique run identifier."""
    ts = time.strftime("%Y%m%dT%H%M%S", time.gmtime())
    return f"review-{ts}-{uuid.uuid4().hex[:8]}"


def git_output(args: list[str]) -> str:
    """Returns trimmed git command output or empty string on failure."""
    try:
        output = subprocess.check_output(
            ["git", *args], text=True, stderr=subprocess.DEVNULL
        )
    except subprocess.CalledProcessError:
        return ""
    return output.strip()


def git_is_dirty() -> bool:
    """Returns whether the current worktree has local modifications."""
    result = subprocess.call(
        ["git", "diff", "--quiet"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
    )
    if result != 0:
        return True
    cached_result = subprocess.call(
        ["git", "diff", "--cached", "--quiet"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    return cached_result != 0


def emit(event: str, run_id: str, payload: dict[str, Any]) -> None:
    """Appends a single structured perf event."""
    review_perf_log.append_event({"event": event, "run_id": run_id, **payload})


def command_string(argv: list[str]) -> str:
    """Renders command arguments as a shell-escaped command string."""
    return shlex.join(argv)


def console_mode() -> str:
    """Returns the configured console output mode."""
    mode = os.environ.get("REVIEW_PERF_CONSOLE", "milestones")
    if mode == "verbose":
        return "verbose"
    return "milestones"


def print_milestone(message: str) -> None:
    """Prints a high-level progress message immediately."""
    print(message, flush=True)


def print_scope_summary(scope_decision: review_scope.ScopeDecision) -> None:
    """Prints a concise summary of scope planning decisions."""
    print_milestone(
        f"[scope] mode={scope_decision.mode} source={scope_decision.changed_files_source} changed={len(scope_decision.changed_files)} domains={','.join(scope_decision.domains)}"
    )
    if scope_decision.forced_full and scope_decision.forced_full_reason:
        print_milestone(
            f"[scope] full review forced: {scope_decision.forced_full_reason}"
        )
        return

    if scope_decision.skipped_steps:
        skipped = ", ".join(
            f"{step} ({reason})"
            for step, reason in sorted(scope_decision.skipped_steps.items())
        )
        print_milestone(f"[scope] planned skips: {skipped}")
    else:
        print_milestone("[scope] no steps eligible for skip")


def run_command(
    run_id: str,
    step_name: str,
    command_index: int,
    command: CommandSpec,
    base_env: dict[str, str],
    mode: str,
) -> tuple[int, float]:
    """Runs one command and emits start/end events."""
    command_id = f"{step_name}:{command_index}"
    emit(
        "command_start",
        run_id,
        {
            "status": "started",
            "step_name": step_name,
            "command_id": command_id,
            "command_name": command.name,
            "command": command_string(command.argv),
        },
    )

    started = time.monotonic()
    env = dict(base_env)
    env["REVIEW_PERF_STEP"] = step_name
    if mode == "verbose":
        return_code = subprocess.call(command.argv, env=env)
    else:
        completed = subprocess.run(
            command.argv, env=env, capture_output=True, text=True
        )
        return_code = completed.returncode
        if return_code != 0:
            if completed.stdout:
                print(completed.stdout, end="")
            if completed.stderr:
                print(completed.stderr, end="", file=sys.stderr)
    elapsed_ms = round((time.monotonic() - started) * 1000, 3)

    emit(
        "command_end",
        run_id,
        {
            "status": "ok" if return_code == 0 else "failed",
            "step_name": step_name,
            "command_id": command_id,
            "command_name": command.name,
            "command": command_string(command.argv),
            "duration_ms": elapsed_ms,
            "exit_code": return_code,
        },
    )

    return (return_code, elapsed_ms)


def step_specs() -> list[StepSpec]:
    """Returns the review step/command execution graph."""
    return [
        StepSpec(
            "check-snapshots",
            [CommandSpec("check-snapshots", ["just", "check-snapshots"])],
        ),
        StepSpec(
            "check-format", [CommandSpec("check-format", ["just", "check-format"])]
        ),
        StepSpec(
            "check-docs-format",
            [CommandSpec("check-docs-format", ["just", "check-docs-format"])],
        ),
        StepSpec(
            "check-token-limits",
            [CommandSpec("check-token-limits", ["just", "check-token-limits"])],
        ),
        StepSpec(
            "review-scope-validate",
            [CommandSpec("review-scope-validate", ["just", "review-scope-validate"])],
        ),
        StepSpec("build", [CommandSpec("build", ["just", "build"])]),
        StepSpec("clippy", [CommandSpec("clippy", ["just", "clippy"])]),
        StepSpec(
            "style-validator",
            [CommandSpec("style-validator", ["just", "style-validator"])],
        ),
        StepSpec("rlf-lint", [CommandSpec("rlf-lint", ["just", "rlf-lint"])]),
        StepSpec("test-core", [CommandSpec("test-core", ["just", "review-core-test"])]),
        StepSpec(
            "python-test",
            [
                CommandSpec(
                    "python-test",
                    ["just", "python-test"],
                )
            ],
        ),
        StepSpec(
            "pyre-check",
            [
                CommandSpec(
                    "pyre-check",
                    ["just", "pyre-check"],
                )
            ],
        ),
        StepSpec(
            "local-unity-test",
            [CommandSpec("local-unity-test", ["just", "local-unity-test"])],
        ),
        StepSpec("parser-test", [CommandSpec("parser-test", ["just", "parser-test"])]),
        StepSpec(
            "tv-check",
            [
                CommandSpec(
                    "tv-check-rust",
                    [
                        "bash",
                        "-lc",
                        'output=$(cargo check --manifest-path rules_engine/src/tv/src-tauri/Cargo.toml 2>&1); if [ $? -eq 0 ]; then echo "TV check passed"; else echo "$output"; exit 1; fi',
                    ],
                ),
                CommandSpec(
                    "tv-check-tsc",
                    [
                        "bash",
                        "-lc",
                        'output=$(cd rules_engine/src/tv && npx tsc --noEmit 2>&1); if [ $? -eq 0 ]; then echo "TV TypeScript check passed"; else echo "$output"; exit 1; fi',
                    ],
                ),
                CommandSpec(
                    "tv-check-eslint",
                    [
                        "bash",
                        "-lc",
                        'output=$(cd rules_engine/src/tv && npx eslint src/ 2>&1); if [ $? -eq 0 ]; then echo "TV ESLint check passed"; else echo "$output"; exit 1; fi',
                    ],
                ),
            ],
        ),
        StepSpec("tv-clippy", [CommandSpec("tv-clippy", ["just", "tv-clippy"])]),
        StepSpec("tv-test", [CommandSpec("tv-test", ["just", "tv-test"])]),
        StepSpec(
            "cqs-check",
            [
                CommandSpec(
                    "cqs-check-tsc",
                    [
                        "bash",
                        "-lc",
                        'output=$(cd scripts/constructed_quest_prototype && npx tsc --noEmit 2>&1); if [ $? -eq 0 ]; then echo "CQS TypeScript check passed"; else echo "$output"; exit 1; fi',
                    ],
                ),
                CommandSpec(
                    "cqs-check-eslint",
                    [
                        "bash",
                        "-lc",
                        'output=$(cd scripts/constructed_quest_prototype && npx eslint src/ 2>&1); if [ $? -eq 0 ]; then echo "CQS ESLint check passed"; else echo "$output"; exit 1; fi',
                    ],
                ),
            ],
        ),
        StepSpec(
            "go-prototypes-check-format",
            [
                CommandSpec(
                    "go-prototypes-check-format",
                    ["just", "go-prototypes-check-format"],
                )
            ],
        ),
        StepSpec(
            "go-prototypes-typecheck",
            [
                CommandSpec(
                    "go-prototypes-typecheck",
                    ["just", "go-prototypes-typecheck"],
                )
            ],
        ),
        StepSpec(
            "go-prototypes-lint",
            [
                CommandSpec(
                    "go-prototypes-lint",
                    ["just", "go-prototypes-lint"],
                )
            ],
        ),
        StepSpec(
            "go-prototypes-test",
            [
                CommandSpec(
                    "go-prototypes-test",
                    ["just", "go-prototypes-test"],
                )
            ],
        ),
    ]


def parse_review_args() -> argparse.Namespace:
    """Parses CLI arguments for review execution mode."""
    parser = argparse.ArgumentParser(description="Run scoped review with perf logging")
    parser.add_argument("commit", nargs="?", default=None, help="Commit SHA to review")
    parser.add_argument(
        "--from", dest="from_sha", default=None, help="Start of commit range"
    )
    parser.add_argument("--to", dest="to_sha", default=None, help="End of commit range")
    parser.add_argument("--all", action="store_true", help="Run all review phases")
    return parser.parse_args()


def has_local_changes() -> bool:
    """Returns whether the worktree has modifications or untracked files."""
    if git_is_dirty():
        return True
    try:
        output = subprocess.check_output(
            ["git", "ls-files", "--others", "--exclude-standard"],
            text=True,
            stderr=subprocess.DEVNULL,
        )
    except subprocess.CalledProcessError:
        return False
    return bool(output.strip())


def get_commit_files(commit_sha: str) -> str:
    """Returns newline-separated changed files for a single commit."""
    try:
        output = subprocess.check_output(
            ["git", "diff-tree", "--no-commit-id", "--name-only", "-r", commit_sha],
            text=True,
            stderr=subprocess.PIPE,
        )
    except subprocess.CalledProcessError as exc:
        stderr_text = exc.stderr.strip() if exc.stderr else "unknown error"
        print(
            f"error: failed to get files for commit {commit_sha}: {stderr_text}",
            file=sys.stderr,
        )
        sys.exit(1)
    return output.strip()


def run_review() -> int:
    """Executes all review steps with perf instrumentation."""
    run_id = run_id_now()
    run_seq = review_perf_log.estimate_next_run_sequence()
    started = time.monotonic()
    mode = console_mode()
    all_steps = step_specs()
    step_names = [step.name for step in all_steps]

    base_env = dict(os.environ)
    base_env["REVIEW_PERF"] = "1"
    base_env["REVIEW_PERF_RUN_ID"] = run_id
    base_env.setdefault("REVIEW_PERF_SOURCE", "default")
    scope_mode = review_scope.normalize_scope_mode(base_env.get("REVIEW_SCOPE_MODE"))

    emit(
        "run_start",
        run_id,
        {
            "status": "started",
            "run_seq": run_seq,
            "source": base_env["REVIEW_PERF_SOURCE"],
            "cwd": str(Path.cwd()),
            "pid": os.getpid(),
            "host": socket.gethostname(),
            "git_commit": git_output(["rev-parse", "HEAD"]),
            "git_branch": git_output(["rev-parse", "--abbrev-ref", "HEAD"]),
            "git_dirty": git_is_dirty(),
            "detail_mode": os.environ.get("REVIEW_PERF_DETAIL", "stable"),
            "console_mode": mode,
        },
    )

    scope_event_status = "ok"
    try:
        scope_decision = review_scope.plan_review_scope(
            step_names=step_names,
            env=base_env,
            repo_root=Path.cwd(),
        )
    except Exception as exc:
        scope_event_status = "degraded"
        warning_message = f"scope planner failed: {exc}"
        emit(
            "warning",
            run_id,
            {
                "status": "degraded",
                "message": warning_message,
            },
        )
        scope_decision = review_scope.fallback_full_scope_decision(
            step_names, scope_mode, warning_message
        )

    emit(
        "scope_plan",
        run_id,
        {
            "status": scope_event_status,
            **scope_decision.event_payload(),
        },
    )

    print_scope_summary(scope_decision)

    if scope_decision.enforce and not scope_decision.forced_full:
        selected_steps = set(scope_decision.selected_steps)
        steps = [step for step in all_steps if step.name in selected_steps]
    else:
        steps = all_steps

    step_count = len(steps)
    print_milestone(f"[review] run {run_seq} started ({step_count} steps)")

    failed_step = ""
    failed_command = ""
    failed_code = 0
    step_totals_ms: dict[str, float] = {}

    for step_index, step in enumerate(steps, start=1):
        print_milestone(f"[review {step_index}/{step_count}] {step.name} started")
        emit(
            "step_start",
            run_id,
            {
                "status": "started",
                "step_name": step.name,
                "command_count": len(step.commands),
            },
        )

        step_started = time.monotonic()
        command_elapsed_ms = 0.0
        step_failed = False

        for command_index, command in enumerate(step.commands, start=1):
            return_code, elapsed_ms = run_command(
                run_id, step.name, command_index, command, base_env, mode
            )
            command_elapsed_ms += elapsed_ms
            if return_code != 0:
                failed_step = step.name
                failed_command = command.name
                failed_code = return_code
                step_failed = True
                break

        step_elapsed_ms = round((time.monotonic() - step_started) * 1000, 3)
        step_totals_ms[step.name] = step_elapsed_ms

        emit(
            "step_end",
            run_id,
            {
                "status": "failed" if step_failed else "ok",
                "step_name": step.name,
                "duration_ms": step_elapsed_ms,
                "command_duration_ms": round(command_elapsed_ms, 3),
                "failed_command": failed_command if step_failed else "",
                "exit_code": failed_code if step_failed else 0,
            },
        )

        if step_failed:
            print_milestone(
                f"[review {step_index}/{step_count}] {step.name} failed ({step_elapsed_ms / 1000:.1f}s)"
            )
        else:
            print_milestone(
                f"[review {step_index}/{step_count}] {step.name} ok ({step_elapsed_ms / 1000:.1f}s)"
            )

        if step_failed:
            break

    total_ms = round((time.monotonic() - started) * 1000, 3)
    emit(
        "run_end",
        run_id,
        {
            "status": "failed" if failed_code else "ok",
            "run_seq": run_seq,
            "duration_ms": total_ms,
            "failed_step": failed_step,
            "failed_command": failed_command,
            "exit_code": failed_code,
            "step_totals_ms": step_totals_ms,
            "source": base_env["REVIEW_PERF_SOURCE"],
        },
    )

    retain_runs = int(os.environ.get("REVIEW_PERF_RETAIN_RUNS", "1000"))
    removed = review_perf_log.prune_to_max_runs(retain_runs)
    if removed > 0:
        emit(
            "warning",
            run_id,
            {
                "status": "ok",
                "message": "pruned old review runs",
                "removed_runs": removed,
                "retained_runs": retain_runs,
            },
        )

    if failed_code:
        print_milestone(
            f"[review] failed at {failed_step}/{failed_command} ({total_ms / 1000:.1f}s)"
        )
    else:
        print_milestone(f"[review] completed successfully ({total_ms / 1000:.1f}s)")

    return failed_code


def main() -> int:
    """Entrypoint for review perf runner."""
    args = parse_review_args()

    if args.all:
        os.environ["REVIEW_SCOPE_FORCE_FULL"] = "1"
    elif args.commit:
        files = get_commit_files(args.commit)
        if not files:
            print(
                f"error: commit {args.commit} has no changed files",
                file=sys.stderr,
            )
            return 1
        os.environ["REVIEW_SCOPE_CHANGED_FILES"] = files
    elif args.from_sha or args.to_sha:
        if not args.from_sha or not args.to_sha:
            print(
                "error: --from and --to must both be specified",
                file=sys.stderr,
            )
            return 1
        os.environ["REVIEW_SCOPE_BASE_REF"] = args.from_sha
        os.environ["REVIEW_SCOPE_HEAD_REF"] = args.to_sha
    else:
        if not has_local_changes():
            print(
                "error: no modified or untracked files detected.",
                file=sys.stderr,
            )
            print("Specify what to review:", file=sys.stderr)
            print(
                "  just review              # review local changes",
                file=sys.stderr,
            )
            print(
                "  just review <commit>     # review a specific commit",
                file=sys.stderr,
            )
            print(
                "  just review --from <sha> --to <sha>  # review a commit range",
                file=sys.stderr,
            )
            print(
                "  just review --all        # run all review phases",
                file=sys.stderr,
            )
            return 1

    try:
        return run_review()
    except Exception as exc:
        print(f"review perf runner failed: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
