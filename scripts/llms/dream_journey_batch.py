#!/usr/bin/env python3
"""Run Dream Journey design jobs across Codex and Claude with deterministic batching."""

from __future__ import annotations

import argparse
import asyncio
import json
import os
import random
import re
import signal
import sys
import tempfile
import time
from collections import deque
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, TextIO

DEFAULT_MAX_CONCURRENCY = 4
DEFAULT_TIMEOUT_SECONDS = 1800
DEFAULT_LONG_BACKOFF_SECONDS = 3600
DEFAULT_SHORT_BACKOFF_SECONDS = 120

ERROR_CATEGORY_OK = "ok"
ERROR_CATEGORY_NORMAL_RETRY = "normal_retry"
ERROR_CATEGORY_SHORT_BACKOFF = "short_backoff"
ERROR_CATEGORY_LONG_BACKOFF = "long_backoff"
ERROR_CATEGORY_HARD_HALT = "hard_halt"

HARD_HALT_PATTERNS: tuple[str, ...] = (
    r"authentication_error",
    r"permission_denied_error",
    r"invalid_api_key",
    r'"(api_error_)?status"\s*:\s*40[13]\b',
    r"\bAPI key\b.*\b(not set|not found|missing|required)\b",
    r"please\s+login",
    r"not\s+authenticated",
)
LONG_BACKOFF_PATTERNS: tuple[str, ...] = (
    r"rate_limit_error",
    r"rate.limit.exceeded",
    r"quota_exceeded",
    r"insufficient_quota",
    r"usage_limit",
    r"usage limit (reached|exceeded|hit)",
    r"weekly (usage )?limit",
    r"5.?hour (usage )?limit",
    r"you'?ve hit your (usage )?limit",
    r"you have hit your (usage )?limit",
    r'"(api_error_)?status"\s*:\s*429\b',
)
SHORT_BACKOFF_PATTERNS: tuple[str, ...] = (
    r"overloaded_error",
    r'"(api_error_)?status"\s*:\s*529\b',
    r'"(api_error_)?status"\s*:\s*5\d\d\b',
    r"service_unavailable",
    r"internal_server_error",
    r"\bapi_error\b",
    r"ECONNREFUSED",
    r"ECONNRESET",
    r"connection refused",
    r"connection reset",
    r"\bdns\b.*(fail|error|resolv)",
)
DEFAULT_CLAUDE_LOG_DIR = Path.home() / ".claude/projects/-Users-dthurn-dreamtides"
DEFAULT_CODEX_SESSIONS_DIR = Path.home() / ".codex/sessions"
DEFAULT_CLAUDE_MODEL = "opus"
DEFAULT_CODEX_MODEL = "gpt-5.4"
DEFAULT_SEED = 1
DEFAULT_MODE1_PER_CATEGORY = 5
DEFAULT_MODE2A_PER_CATEGORY = 5
DEFAULT_MODE2B_PER_CATEGORY = 5
DEFAULT_MODE3_COUNT = 180
AGENT_ORDER = ("codex", "claude")
MODE_ORDER = ("mode1", "mode2a", "mode2b", "mode3")
MODE_LABELS = {
    "mode1": "Mode 1: Big Effects",
    "mode2a": "Mode 2A: Category-based",
    "mode2b": "Mode 2B: Category-based with cost",
    "mode3": "Mode 3: Third-party inspired",
}
CATEGORY_ORDER = (
    "pact",
    "reshape",
    "gamble",
    "decree",
    "crossroads",
    "horizon",
    "echo",
)
VALID_CATEGORIES = set(CATEGORY_ORDER)
JOB_KEY_PATTERN = re.compile(r"Job key:\s*([a-z0-9-]+)")
REQUIRED_TOP_LEVEL_KEYS = (
    "mode",
    "category",
    "ability_text",
    "ability_description",
    "cost_benefit",
    "justification",
    "rejected_alternatives",
)
REPO_ROOT = Path(__file__).resolve().parents[2]
SKILL_PATH = REPO_ROOT / ".llms/skills/dream-journey-design/SKILL.md"
DEFAULT_EVENTS_PATH = Path("/Users/dthurn/Documents/events/mt_sts_combined.txt")

DREAM_JOURNEY_JSON_SCHEMA: dict[str, Any] = {
    "type": "object",
    "required": list(REQUIRED_TOP_LEVEL_KEYS),
    "properties": {
        "mode": {"type": "string"},
        "category": {"type": "string", "enum": sorted(VALID_CATEGORIES)},
        "ability_text": {"type": "string"},
        "ability_description": {"type": "string"},
        "cost_benefit": {
            "anyOf": [
                {
                    "type": "object",
                    "required": ["cost", "benefit"],
                    "properties": {
                        "cost": {"type": "string"},
                        "benefit": {"type": "string"},
                    },
                    "additionalProperties": False,
                },
                {"type": "null"},
            ],
        },
        "justification": {"type": "string"},
        "rejected_alternatives": {
            "type": "array",
            "items": {"type": "string"},
            "minItems": 2,
        },
    },
    "additionalProperties": False,
}


@dataclass(frozen=True)
class AgentJob:
    """One provider invocation for one Dream Journey design prompt."""

    agent_name: str
    model_name: str
    job_key: str
    mode: str
    category: str | None
    inspiration: str | None
    attempt: int = 1


@dataclass
class AgentAttemptResult:
    """Final state for one Dream Journey job."""

    agent_name: str
    model_name: str
    job_key: str
    mode: str
    category: str | None
    success: bool
    parsed_json: dict[str, Any] | None
    errors: list[str]
    used_retry: bool
    attempts: int = 1
    stdout: str = ""
    stderr: str = ""
    exit_code: int | None = None
    error_category: str = ERROR_CATEGORY_OK


def classify_failure(stderr: str, stdout: str) -> str:
    haystack = f"{stderr}\n{stdout}"
    if _matches_any_pattern(haystack, HARD_HALT_PATTERNS):
        return ERROR_CATEGORY_HARD_HALT
    if _matches_any_pattern(haystack, LONG_BACKOFF_PATTERNS):
        return ERROR_CATEGORY_LONG_BACKOFF
    if _matches_any_pattern(haystack, SHORT_BACKOFF_PATTERNS):
        return ERROR_CATEGORY_SHORT_BACKOFF
    return ERROR_CATEGORY_NORMAL_RETRY


def _matches_any_pattern(text: str, patterns: tuple[str, ...]) -> bool:
    return any(re.search(pattern, text, re.IGNORECASE) for pattern in patterns)


class BatchReporter:
    """Persist live progress and partial results for a Dream Journey batch."""

    def __init__(
        self,
        *,
        jobs: list[AgentJob],
        output_path: Path,
        attempt_log_path: Path,
        stream: TextIO | None = None,
    ) -> None:
        self.jobs = jobs
        self.output_path = output_path
        self.attempt_log_path = attempt_log_path
        self.stream = stream or sys.stdout
        self.output_path.parent.mkdir(parents=True, exist_ok=True)
        self.attempt_log_path.parent.mkdir(parents=True, exist_ok=True)

    def record_batch_start(self, *, total_jobs: int) -> None:
        self._record_event({"event": "batch_start", "total_jobs": total_jobs})
        self._print(f"[{_timestamp()}] BATCH total_jobs={total_jobs}")

    def record_recovered(
        self, results_by_job_key: dict[str, AgentAttemptResult]
    ) -> None:
        recovered_count = sum(
            1 for result in results_by_job_key.values() if result.success
        )
        if recovered_count == 0:
            return
        self._record_event({"event": "recovered", "recovered_count": recovered_count})
        self._print(f"[{_timestamp()}] RECOVER recovered={recovered_count}")

    def record_start(
        self,
        job: AgentJob,
        *,
        active_counts: dict[str, int],
        pending_counts: dict[str, int],
    ) -> None:
        self._record_event(
            {
                "event": "start",
                "agent": job.agent_name,
                "model": job.model_name,
                "job_key": job.job_key,
                "mode": job.mode,
                "category": job.category,
                "attempt": job.attempt,
                "active_counts": active_counts,
                "pending_counts": pending_counts,
            }
        )
        self._print(
            f"[{_timestamp()}] START {job.agent_name:<6} attempt={job.attempt} "
            f"mode={job.mode:<6} active={sum(active_counts.values())} "
            f"pending={sum(pending_counts.values())} job={job.job_key}"
        )

    def record_finish(
        self,
        result: AgentAttemptResult,
        *,
        active_counts: dict[str, int],
        pending_counts: dict[str, int],
    ) -> None:
        event: dict[str, Any] = {
            "event": "finish",
            "agent": result.agent_name,
            "model": result.model_name,
            "job_key": result.job_key,
            "mode": result.mode,
            "category": result.category,
            "attempt": result.attempts,
            "success": result.success,
            "exit_code": result.exit_code,
            "errors": result.errors,
            "stderr": result.stderr,
            "error_category": result.error_category,
            "active_counts": active_counts,
            "pending_counts": pending_counts,
        }
        if not result.success:
            event["stdout"] = result.stdout
        self._record_event(event)

        status = "DONE" if result.success else "FAIL"
        detail = (
            f"errors={len(result.errors)}"
            if result.errors
            else f"exit={result.exit_code}"
        )
        self._print(
            f"[{_timestamp()}] {status:<5} {result.agent_name:<6} attempt={result.attempts} "
            f"mode={result.mode:<6} active={sum(active_counts.values())} "
            f"pending={sum(pending_counts.values())} {detail} job={result.job_key}"
        )

    def record_retry_enqueued(
        self,
        job: AgentJob,
        *,
        errors: list[str],
        active_counts: dict[str, int],
        pending_counts: dict[str, int],
    ) -> None:
        self._record_event(
            {
                "event": "retry_enqueued",
                "agent": job.agent_name,
                "model": job.model_name,
                "job_key": job.job_key,
                "mode": job.mode,
                "category": job.category,
                "attempt": job.attempt,
                "errors": errors,
                "active_counts": active_counts,
                "pending_counts": pending_counts,
            }
        )
        self._print(
            f"[{_timestamp()}] RETRY {job.agent_name:<6} attempt={job.attempt} "
            f"mode={job.mode:<6} active={sum(active_counts.values())} "
            f"pending={sum(pending_counts.values())} job={job.job_key}"
        )

    def record_cooldown(
        self,
        *,
        agent_name: str,
        error_category: str,
        duration_seconds: float,
        until_timestamp: str,
        reason: str,
    ) -> None:
        self._record_event(
            {
                "event": "cooldown",
                "agent": agent_name,
                "error_category": error_category,
                "duration_seconds": duration_seconds,
                "until": until_timestamp,
                "reason": reason,
            }
        )
        duration_label = _format_duration(duration_seconds)
        banner = "!" * 72
        self._print(banner)
        self._print(
            f"[{_timestamp()}] COOLDOWN {agent_name.upper()} "
            f"[{error_category}] — pausing this provider for {duration_label} "
            f"(until {until_timestamp})."
        )
        self._print(f"    reason: {_truncate(reason, 240)}")
        self._print(banner)

    def record_halt(self, *, agent_name: str, reason: str) -> None:
        self._record_event(
            {"event": "halt", "agent": agent_name, "reason": reason}
        )
        banner = "#" * 72
        self._print(banner)
        self._print(
            f"[{_timestamp()}] HALT {agent_name.upper()} — provider disabled for "
            "the remainder of this run (no further retries will be scheduled)."
        )
        self._print(f"    reason: {_truncate(reason, 240)}")
        self._print(banner)

    def record_waiting(self, *, seconds: float, reason: str) -> None:
        self._record_event(
            {"event": "waiting", "seconds": seconds, "reason": reason}
        )
        self._print(
            f"[{_timestamp()}] WAIT — sleeping {_format_duration(seconds)} "
            f"({reason})."
        )

    def record_batch_complete(
        self, results_by_job_key: dict[str, AgentAttemptResult]
    ) -> None:
        success_count = sum(
            1 for result in results_by_job_key.values() if result.success
        )
        failure_count = sum(
            1 for result in results_by_job_key.values() if not result.success
        )
        self._record_event(
            {
                "event": "batch_complete",
                "success_count": success_count,
                "failure_count": failure_count,
            }
        )
        self._print(
            f"[{_timestamp()}] COMPLETE success={success_count} failure={failure_count}"
        )

    def write_snapshot(self, results_by_job_key: dict[str, AgentAttemptResult]) -> None:
        _write_json_atomic(
            self.output_path,
            synthesize_results(self.jobs, results_by_job_key),
        )

    def _record_event(self, event: dict[str, Any]) -> None:
        event_with_timestamp = {"timestamp": _timestamp(), **event}
        with self.attempt_log_path.open("a", encoding="utf-8") as handle:
            handle.write(json.dumps(event_with_timestamp, ensure_ascii=False) + "\n")

    def _print(self, message: str) -> None:
        print(message, file=self.stream, flush=True)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--output",
        type=Path,
        required=True,
        help="Path to synthesized JSON output",
    )
    parser.add_argument(
        "--attempt-log",
        type=Path,
        help="Path to JSONL attempt log (defaults to <output>.attempts.jsonl)",
    )
    parser.add_argument(
        "--max-concurrency",
        type=int,
        default=DEFAULT_MAX_CONCURRENCY,
        help="Maximum concurrent Dream Journey jobs",
    )
    parser.add_argument(
        "--timeout-seconds",
        type=int,
        default=DEFAULT_TIMEOUT_SECONDS,
        help="Default per-agent timeout in seconds",
    )
    parser.add_argument(
        "--codex-timeout-seconds",
        type=int,
        help="Per-codex timeout in seconds",
    )
    parser.add_argument(
        "--claude-timeout-seconds",
        type=int,
        help="Per-claude timeout in seconds",
    )
    parser.add_argument(
        "--codex-bin",
        default="codex",
        help="Codex CLI executable name",
    )
    parser.add_argument(
        "--claude-bin",
        default="claude",
        help="Claude CLI executable name",
    )
    parser.add_argument(
        "--codex-model",
        default=DEFAULT_CODEX_MODEL,
        help="Codex model name for Codex jobs",
    )
    parser.add_argument(
        "--claude-model",
        default=DEFAULT_CLAUDE_MODEL,
        help="Claude model name for Claude jobs",
    )
    parser.add_argument(
        "--seed",
        type=int,
        default=DEFAULT_SEED,
        help="Deterministic seed used for provider assignment",
    )
    parser.add_argument(
        "--mode1-per-category",
        type=int,
        default=DEFAULT_MODE1_PER_CATEGORY,
        help="Mode 1 jobs per category (7 categories total)",
    )
    parser.add_argument(
        "--mode2a-per-category",
        type=int,
        default=DEFAULT_MODE2A_PER_CATEGORY,
        help="Mode 2A jobs per category (7 categories total)",
    )
    parser.add_argument(
        "--mode2b-per-category",
        type=int,
        default=DEFAULT_MODE2B_PER_CATEGORY,
        help="Mode 2B jobs per category (7 categories total)",
    )
    parser.add_argument(
        "--mode3-count",
        type=int,
        default=DEFAULT_MODE3_COUNT,
        help="Mode 3 jobs (one per line from --events-path)",
    )
    parser.add_argument(
        "--events-path",
        type=Path,
        default=DEFAULT_EVENTS_PATH,
        help="Path to newline-delimited third-party event inspirations for Mode 3",
    )
    parser.add_argument(
        "--long-backoff-seconds",
        type=int,
        default=DEFAULT_LONG_BACKOFF_SECONDS,
        help=(
            "Cooldown after rate-limit / usage-cap errors on a provider "
            "(default: 3600 = 1 hour)"
        ),
    )
    parser.add_argument(
        "--short-backoff-seconds",
        type=int,
        default=DEFAULT_SHORT_BACKOFF_SECONDS,
        help=(
            "Cooldown after overloaded / 5xx / connection errors on a provider "
            "(default: 120 seconds)"
        ),
    )
    parser.add_argument(
        "--recover-from-logs",
        action="store_true",
        help="Seed or reconstruct results from ~/.claude and ~/.codex session logs",
    )
    parser.add_argument(
        "--recover-only",
        action="store_true",
        help="Write recovered results from provider logs without starting new jobs",
    )
    return parser.parse_args()


def load_skill_text() -> str:
    return SKILL_PATH.read_text(encoding="utf-8")


def load_unique_lines(path: Path) -> list[str]:
    unique_lines = list(
        dict.fromkeys(
            line.strip()
            for line in path.read_text(encoding="utf-8").splitlines()
            if line.strip()
        )
    )
    if not unique_lines:
        raise ValueError(f"No usable inspiration lines found in {path}")
    return unique_lines


def _assign_providers_for_group(
    count: int, *, seed_key: str, start_offset: int = 0
) -> list[str]:
    if count < 0:
        raise ValueError("count must be non-negative")
    base = [
        AGENT_ORDER[(index + start_offset) % len(AGENT_ORDER)]
        for index in range(count)
    ]
    random.Random(seed_key).shuffle(base)
    return base


def build_jobs(args: argparse.Namespace) -> list[AgentJob]:
    model_by_agent = {"codex": args.codex_model, "claude": args.claude_model}
    jobs: list[AgentJob] = []

    category_mode_specs = (
        ("mode1", args.mode1_per_category),
        ("mode2a", args.mode2a_per_category),
        ("mode2b", args.mode2b_per_category),
    )
    for mode, per_category in category_mode_specs:
        if per_category < 0:
            raise ValueError(f"--{mode}-per-category must be at least 0")
        if per_category == 0:
            continue
        for category_index, category in enumerate(CATEGORY_ORDER):
            assignments = _assign_providers_for_group(
                per_category,
                seed_key=f"{args.seed}:{mode}:{category}",
                start_offset=category_index,
            )
            for index, agent in enumerate(assignments, start=1):
                jobs.append(
                    AgentJob(
                        agent_name=agent,
                        model_name=model_by_agent[agent],
                        job_key=f"{mode}-{category}-{index:02d}-{agent}",
                        mode=mode,
                        category=category,
                        inspiration=None,
                    )
                )

    if args.mode3_count < 0:
        raise ValueError("--mode3-count must be at least 0")
    if args.mode3_count > 0:
        lines = load_unique_lines(args.events_path)
        if args.mode3_count > len(lines):
            raise ValueError(
                f"--mode3-count ({args.mode3_count}) exceeds unique lines in "
                f"{args.events_path} ({len(lines)})"
            )
        selected_lines = lines[: args.mode3_count]
        assignments = _assign_providers_for_group(
            args.mode3_count,
            seed_key=f"{args.seed}:mode3",
        )
        for index, (line, agent) in enumerate(
            zip(selected_lines, assignments, strict=True),
            start=1,
        ):
            source_slug = _slugify(line)[:36]
            jobs.append(
                AgentJob(
                    agent_name=agent,
                    model_name=model_by_agent[agent],
                    job_key=f"mode3-{index:03d}-{agent}-{source_slug}",
                    mode="mode3",
                    category=None,
                    inspiration=line,
                )
            )

    if len({job.job_key for job in jobs}) != len(jobs):
        raise ValueError("Duplicate job keys generated")
    if not jobs:
        raise ValueError("At least one job must be requested")

    return jobs


def validate_dream_journey_result(payload: Any) -> list[str]:
    if not isinstance(payload, dict):
        return ["Top-level JSON value must be an object"]

    errors: list[str] = []
    for key in REQUIRED_TOP_LEVEL_KEYS:
        if key not in payload:
            errors.append(f"Missing top-level key: {key}")

    if not isinstance(payload.get("mode"), str) or not payload.get("mode").strip():
        errors.append("mode must be a non-empty string")
    if payload.get("category") not in VALID_CATEGORIES:
        errors.append(f"category must be one of: {', '.join(sorted(VALID_CATEGORIES))}")
    if (
        not isinstance(payload.get("ability_text"), str)
        or not payload.get("ability_text").strip()
    ):
        errors.append("ability_text must be a non-empty string")
    if (
        not isinstance(payload.get("ability_description"), str)
        or not payload.get("ability_description").strip()
    ):
        errors.append("ability_description must be a non-empty string")

    cost_benefit = payload.get("cost_benefit")
    if cost_benefit is not None:
        if not isinstance(cost_benefit, dict):
            errors.append("cost_benefit must be an object or null")
        else:
            for inner_key in ("cost", "benefit"):
                inner_value = cost_benefit.get(inner_key)
                if not isinstance(inner_value, str) or not inner_value.strip():
                    errors.append(
                        f"cost_benefit.{inner_key} must be a non-empty string"
                    )

    if (
        not isinstance(payload.get("justification"), str)
        or not payload.get("justification").strip()
    ):
        errors.append("justification must be a non-empty string")

    rejected_alternatives = payload.get("rejected_alternatives")
    if not isinstance(rejected_alternatives, list):
        errors.append("rejected_alternatives must be a list")
    else:
        if len(rejected_alternatives) < 2:
            errors.append("rejected_alternatives must contain at least 2 items")
        for index, item in enumerate(rejected_alternatives):
            if not isinstance(item, str) or not item.strip():
                errors.append(
                    f"rejected_alternatives[{index}] must be a non-empty string"
                )

    return errors


def synthesize_results(
    jobs: list[AgentJob], results_by_job_key: dict[str, AgentAttemptResult]
) -> dict[str, Any]:
    summary = {
        "total_jobs": len(jobs),
        "success_count": sum(
            1
            for job in jobs
            if job.job_key in results_by_job_key
            and results_by_job_key[job.job_key].success
        ),
        "failure_count": sum(
            1
            for job in jobs
            if job.job_key in results_by_job_key
            and not results_by_job_key[job.job_key].success
        ),
        "providers": {
            agent_name: sum(1 for job in jobs if job.agent_name == agent_name)
            for agent_name in AGENT_ORDER
        },
        "modes": {
            mode: sum(1 for job in jobs if job.mode == mode) for mode in MODE_ORDER
        },
        "categories": {
            category: sum(1 for job in jobs if job.category == category)
            for category in CATEGORY_ORDER
        },
    }

    jobs_payload: dict[str, Any] = {}
    for job in jobs:
        result = results_by_job_key.get(job.job_key)
        jobs_payload[job.job_key] = {
            "job_key": job.job_key,
            "provider": job.agent_name,
            "model": job.model_name,
            "mode": job.mode,
            "mode_label": MODE_LABELS[job.mode],
            "category": job.category,
            "inspiration": job.inspiration,
            "result": (
                result.parsed_json
                if result is not None
                and result.success
                and result.parsed_json is not None
                else None
            ),
            "verification": (
                {
                    "success": result.success,
                    "used_retry": result.used_retry,
                    "attempts": result.attempts,
                    "errors": result.errors,
                    "exit_code": result.exit_code,
                    "error_category": result.error_category,
                }
                if result is not None
                else None
            ),
        }

    return {"summary": summary, "jobs": jobs_payload}


def build_agent_prompt(job: AgentJob, skill_text: str) -> str:
    if job.mode == "mode1":
        mode_instructions = (
            f"Operate the dream-journey-design skill in Mode 1 (Big Effects).\n"
            f"Use the '{job.category}' category for this design.\n"
            "Bias toward sweeping, run-changing effects while staying in that category.\n"
        )
    elif job.mode == "mode2a":
        mode_instructions = (
            f"Operate the dream-journey-design skill in Mode 2A (Category-based).\n"
            f"Category: {job.category}\n"
        )
    elif job.mode == "mode2b":
        mode_instructions = (
            f"Operate the dream-journey-design skill in Mode 2B (Category-based with cost).\n"
            f"Category: {job.category}\n"
            "The design MUST include an explicit cost+benefit structure. "
            "Populate the cost_benefit object with 'cost' and 'benefit' fields.\n"
        )
    elif job.mode == "mode3":
        mode_instructions = (
            "Operate the dream-journey-design skill in Mode 3 (Third-party inspired).\n"
            "Inspiration (single event from another game — extract the dynamic, "
            "do not port 1:1):\n"
            f"{job.inspiration}\n"
        )
    else:
        raise ValueError(f"Unknown mode: {job.mode}")

    cost_benefit_guidance = (
        "If the design has an explicit cost+benefit structure, populate cost_benefit "
        "as an object with 'cost' and 'benefit' string fields; otherwise set it to null. "
        "In Mode 2B the cost_benefit object is required."
    )

    return (
        "You are generating structured JSON for one Dreamtides Dream Journey design.\n"
        "Follow the dream-journey-design skill specification below exactly.\n"
        "Use the current repository as context and perform the research the skill requires.\n"
        "Read the required docs listed in the skill before designing.\n"
        "Output exactly one valid JSON object and no markdown or prose.\n"
        "Return JSON with exactly these keys:\n"
        "- mode\n"
        "- category\n"
        "- ability_text\n"
        "- ability_description\n"
        "- cost_benefit\n"
        "- justification\n"
        "- rejected_alternatives\n\n"
        f"{cost_benefit_guidance}\n\n"
        f"Job key: {job.job_key}\n"
        f"Mode: {MODE_LABELS[job.mode]}\n"
        f"Provider target: {job.agent_name} ({job.model_name})\n\n"
        f"{mode_instructions}\n"
        "Skill specification:\n"
        f"{skill_text}"
    )


def choose_next_agent(
    *,
    pending_counts: dict[str, int],
    active_counts: dict[str, int],
    max_concurrency: int,
    blocked_agents: frozenset[str] = frozenset(),
) -> str | None:
    available_agents = [
        agent_name
        for agent_name in AGENT_ORDER
        if pending_counts.get(agent_name, 0) > 0 and agent_name not in blocked_agents
    ]
    if not available_agents:
        return None
    if len(available_agents) == 1:
        return available_agents[0]

    zero_active_agents = [
        agent_name
        for agent_name in available_agents
        if active_counts.get(agent_name, 0) == 0
    ]
    if zero_active_agents:
        return zero_active_agents[0]

    target_per_agent = max(1, max_concurrency // len(AGENT_ORDER))
    below_target_agents = [
        agent_name
        for agent_name in available_agents
        if active_counts.get(agent_name, 0) < target_per_agent
    ]
    if below_target_agents:
        return min(
            below_target_agents,
            key=lambda agent_name: (
                active_counts.get(agent_name, 0),
                -pending_counts.get(agent_name, 0),
                AGENT_ORDER.index(agent_name),
            ),
        )

    return min(
        available_agents,
        key=lambda agent_name: (
            active_counts.get(agent_name, 0),
            -pending_counts.get(agent_name, 0),
            AGENT_ORDER.index(agent_name),
        ),
    )


async def run_batch(
    jobs: list[AgentJob],
    *,
    max_concurrency: int,
    codex_timeout_seconds: int,
    claude_timeout_seconds: int,
    codex_bin: str,
    claude_bin: str,
    reporter: BatchReporter,
    recovered_results: dict[str, AgentAttemptResult] | None = None,
    long_backoff_seconds: float = DEFAULT_LONG_BACKOFF_SECONDS,
    short_backoff_seconds: float = DEFAULT_SHORT_BACKOFF_SECONDS,
) -> dict[str, AgentAttemptResult]:
    skill_text = load_skill_text()
    results_by_job_key = dict(recovered_results or {})
    pending_jobs: dict[str, deque[AgentJob]] = {
        agent_name: deque(
            job
            for job in jobs
            if job.agent_name == agent_name and job.job_key not in results_by_job_key
        )
        for agent_name in AGENT_ORDER
    }
    active_counts = {agent_name: 0 for agent_name in AGENT_ORDER}
    active_tasks: dict[asyncio.Task[AgentAttemptResult], AgentJob] = {}
    cooldown_until: dict[str, float] = {agent_name: 0.0 for agent_name in AGENT_ORDER}
    halted_agents: set[str] = set()

    reporter.record_batch_start(total_jobs=len(jobs))
    reporter.record_recovered(results_by_job_key)
    reporter.write_snapshot(results_by_job_key)

    with tempfile.TemporaryDirectory(prefix="dream-journey-batch-") as temp_dir:
        temp_dir_path = Path(temp_dir)
        schema_path = temp_dir_path / "dream-journey-output-schema.json"
        schema_path.write_text(
            json.dumps(DREAM_JOURNEY_JSON_SCHEMA, indent=2, ensure_ascii=False) + "\n",
            encoding="utf-8",
        )

        while active_tasks or any(pending_jobs.values()):
            now = time.monotonic()
            blocked_agents = frozenset(
                agent_name
                for agent_name in AGENT_ORDER
                if agent_name in halted_agents or cooldown_until[agent_name] > now
            )

            while len(active_tasks) < max_concurrency:
                pending_counts = {
                    agent_name: len(queue) for agent_name, queue in pending_jobs.items()
                }
                next_agent = choose_next_agent(
                    pending_counts=pending_counts,
                    active_counts=active_counts,
                    max_concurrency=max_concurrency,
                    blocked_agents=blocked_agents,
                )
                if next_agent is None:
                    break

                job = pending_jobs[next_agent].popleft()
                active_counts[job.agent_name] += 1
                reporter.record_start(
                    job,
                    active_counts=dict(active_counts),
                    pending_counts={
                        agent_name: len(queue)
                        for agent_name, queue in pending_jobs.items()
                    },
                )
                task = asyncio.create_task(
                    _run_job(
                        job,
                        skill_text=skill_text,
                        output_schema_path=schema_path,
                        temp_dir=temp_dir_path,
                        codex_timeout_seconds=codex_timeout_seconds,
                        claude_timeout_seconds=claude_timeout_seconds,
                        codex_bin=codex_bin,
                        claude_bin=claude_bin,
                    )
                )
                active_tasks[task] = job

            if not active_tasks:
                pending_with_work = [
                    agent_name
                    for agent_name in AGENT_ORDER
                    if pending_jobs[agent_name] and agent_name not in halted_agents
                ]
                if not pending_with_work:
                    break
                now = time.monotonic()
                next_deadlines = [
                    cooldown_until[agent_name]
                    for agent_name in pending_with_work
                    if cooldown_until[agent_name] > now
                ]
                if not next_deadlines:
                    continue
                sleep_seconds = min(max(0.5, min(next_deadlines) - now), 60.0)
                reporter.record_waiting(
                    seconds=sleep_seconds,
                    reason=f"all available providers cooling down ({','.join(pending_with_work)})",
                )
                await asyncio.sleep(sleep_seconds)
                continue

            completed_tasks, _ = await asyncio.wait(
                active_tasks.keys(),
                return_when=asyncio.FIRST_COMPLETED,
            )

            for task in completed_tasks:
                job = active_tasks.pop(task)
                active_counts[job.agent_name] -= 1
                result = _result_from_task(task, job)
                results_by_job_key[result.job_key] = result
                reporter.record_finish(
                    result,
                    active_counts=dict(active_counts),
                    pending_counts={
                        agent_name: len(queue)
                        for agent_name, queue in pending_jobs.items()
                    },
                )
                reporter.write_snapshot(results_by_job_key)

                should_retry = not result.success and job.attempt == 1

                if not result.success:
                    if result.error_category == ERROR_CATEGORY_HARD_HALT:
                        halted_agents.add(job.agent_name)
                        reporter.record_halt(
                            agent_name=job.agent_name,
                            reason=(result.errors[0] if result.errors else "hard_halt"),
                        )
                        should_retry = False
                    elif result.error_category in (
                        ERROR_CATEGORY_LONG_BACKOFF,
                        ERROR_CATEGORY_SHORT_BACKOFF,
                    ):
                        duration = (
                            long_backoff_seconds
                            if result.error_category == ERROR_CATEGORY_LONG_BACKOFF
                            else short_backoff_seconds
                        )
                        deadline = time.monotonic() + duration
                        cooldown_until[job.agent_name] = max(
                            cooldown_until[job.agent_name], deadline
                        )
                        reporter.record_cooldown(
                            agent_name=job.agent_name,
                            error_category=result.error_category,
                            duration_seconds=duration,
                            until_timestamp=_timestamp_plus_seconds(duration),
                            reason=(
                                result.errors[0] if result.errors else result.error_category
                            ),
                        )

                if should_retry:
                    retry_job = AgentJob(
                        agent_name=job.agent_name,
                        model_name=job.model_name,
                        job_key=job.job_key,
                        mode=job.mode,
                        category=job.category,
                        inspiration=job.inspiration,
                        attempt=2,
                    )
                    pending_jobs[retry_job.agent_name].append(retry_job)
                    reporter.record_retry_enqueued(
                        retry_job,
                        errors=result.errors,
                        active_counts=dict(active_counts),
                        pending_counts={
                            agent_name: len(queue)
                            for agent_name, queue in pending_jobs.items()
                        },
                    )

    reporter.record_batch_complete(results_by_job_key)
    reporter.write_snapshot(results_by_job_key)
    return results_by_job_key


def _result_from_task(
    task: asyncio.Task[AgentAttemptResult], job: AgentJob
) -> AgentAttemptResult:
    try:
        return task.result()
    except Exception as error:  # pragma: no cover - defensive guard
        return AgentAttemptResult(
            agent_name=job.agent_name,
            model_name=job.model_name,
            job_key=job.job_key,
            mode=job.mode,
            category=job.category,
            success=False,
            parsed_json=None,
            errors=[f"Runner exception: {type(error).__name__}: {error}"],
            used_retry=job.attempt > 1,
            attempts=job.attempt,
            error_category=ERROR_CATEGORY_NORMAL_RETRY,
        )


async def _run_job(
    job: AgentJob,
    *,
    skill_text: str,
    output_schema_path: Path,
    temp_dir: Path,
    codex_timeout_seconds: int,
    claude_timeout_seconds: int,
    codex_bin: str,
    claude_bin: str,
) -> AgentAttemptResult:
    prompt_text = build_agent_prompt(job, skill_text)
    if job.agent_name == "codex":
        raw_text, stderr, exit_code = await _run_codex(
            prompt_text=prompt_text,
            output_schema_path=output_schema_path,
            temp_dir=temp_dir,
            timeout_seconds=codex_timeout_seconds,
            codex_bin=codex_bin,
            model_name=job.model_name,
        )
    else:
        raw_text, stderr, exit_code = await _run_claude(
            prompt_text=prompt_text,
            timeout_seconds=claude_timeout_seconds,
            claude_bin=claude_bin,
            model_name=job.model_name,
        )

    parsed_json, errors = parse_and_validate_agent_output(raw_text)
    if exit_code != 0:
        errors.append(f"{job.agent_name} exited with code {exit_code}")
    should_report_stderr = bool(
        stderr.strip() and (job.agent_name != "codex" or exit_code != 0 or bool(errors))
    )
    if should_report_stderr:
        errors.append(f"{job.agent_name} stderr: {stderr.strip()}")

    success = not errors
    error_category = (
        ERROR_CATEGORY_OK if success else classify_failure(stderr, raw_text)
    )
    return AgentAttemptResult(
        agent_name=job.agent_name,
        model_name=job.model_name,
        job_key=job.job_key,
        mode=job.mode,
        category=job.category,
        success=success,
        parsed_json=parsed_json,
        errors=errors,
        used_retry=job.attempt > 1,
        attempts=job.attempt,
        stdout=raw_text,
        stderr=stderr,
        exit_code=exit_code,
        error_category=error_category,
    )


async def _run_codex(
    *,
    prompt_text: str,
    output_schema_path: Path,
    temp_dir: Path,
    timeout_seconds: int,
    codex_bin: str,
    model_name: str,
) -> tuple[str, str, int]:
    output_handle, output_name = tempfile.mkstemp(
        prefix="codex-output-",
        suffix=".json",
        dir=temp_dir,
    )
    os.close(output_handle)
    output_path = Path(output_name)
    command = build_codex_command(
        prompt_text=prompt_text,
        output_path=output_path,
        output_schema_path=output_schema_path,
        codex_bin=codex_bin,
        model_name=model_name,
    )
    stdout, stderr, exit_code = await _run_command(command, timeout_seconds)
    if output_path.exists():
        return output_path.read_text(encoding="utf-8"), stderr, exit_code
    return stdout, stderr, exit_code


async def _run_claude(
    *,
    prompt_text: str,
    timeout_seconds: int,
    claude_bin: str,
    model_name: str,
) -> tuple[str, str, int]:
    command = build_claude_command(
        prompt_text=prompt_text,
        claude_bin=claude_bin,
        model_name=model_name,
    )
    return await _run_command(command, timeout_seconds)


def build_codex_command(
    *,
    prompt_text: str,
    output_path: Path,
    output_schema_path: Path,
    codex_bin: str,
    model_name: str,
) -> list[str]:
    return [
        codex_bin,
        "exec",
        "--dangerously-bypass-approvals-and-sandbox",
        "-C",
        str(REPO_ROOT),
        "--model",
        model_name,
        "--output-schema",
        str(output_schema_path),
        "-o",
        str(output_path),
        prompt_text,
    ]


def build_claude_command(
    *, prompt_text: str, claude_bin: str, model_name: str
) -> list[str]:
    return [
        claude_bin,
        "-p",
        "--model",
        model_name,
        "--output-format",
        "json",
        "--dangerously-skip-permissions",
        "--permission-mode",
        "bypassPermissions",
        "--json-schema",
        json.dumps(DREAM_JOURNEY_JSON_SCHEMA, ensure_ascii=False),
        prompt_text,
    ]


async def _run_command(
    command: list[str], timeout_seconds: int
) -> tuple[str, str, int]:
    with tempfile.TemporaryFile() as stdout_handle, tempfile.TemporaryFile() as stderr_handle:
        process = await asyncio.create_subprocess_exec(
            *command,
            cwd=REPO_ROOT,
            stdout=stdout_handle,
            stderr=stderr_handle,
            start_new_session=True,
        )

        try:
            await asyncio.wait_for(process.wait(), timeout=timeout_seconds)
        except asyncio.TimeoutError:
            _kill_process_group(process.pid)
            await process.wait()
            stdout_text = _read_temp_output(stdout_handle)
            stderr_text = _read_temp_output(stderr_handle)
            timeout_text = f"Timed out after {timeout_seconds} seconds"
            return (
                stdout_text,
                (
                    f"{stderr_text}\n{timeout_text}".strip()
                    if stderr_text
                    else timeout_text
                ),
                124,
            )

        return_code = process.returncode
        if return_code is None:
            raise RuntimeError("subprocess finished without a return code")

        return (
            _read_temp_output(stdout_handle),
            _read_temp_output(stderr_handle),
            return_code,
        )


def _kill_process_group(process_id: int) -> None:
    try:
        os.killpg(process_id, signal.SIGKILL)
    except ProcessLookupError:
        return


def _read_temp_output(handle: Any) -> str:
    handle.seek(0)
    return handle.read().decode("utf-8", errors="replace")


def parse_and_validate_agent_output(
    raw_text: str,
) -> tuple[dict[str, Any] | None, list[str]]:
    normalized_text = raw_text.strip()
    if normalized_text.startswith("```") and normalized_text.endswith("```"):
        normalized_text = "\n".join(normalized_text.splitlines()[1:-1]).strip()

    if not normalized_text:
        return None, ["Agent produced empty output"]

    try:
        payload = json.loads(normalized_text)
    except json.JSONDecodeError as error:
        return None, [
            f"Invalid JSON: {error.msg} at line {error.lineno} column {error.colno}"
        ]

    if isinstance(payload, dict) and payload.get("is_error") is True:
        status = payload.get("api_error_status")
        message = payload.get("result") or payload.get("message") or "unknown error"
        status_part = f" (api_error_status={status})" if status is not None else ""
        return None, [f"Provider returned error{status_part}: {message}"]

    if (
        isinstance(payload, dict)
        and payload.get("type") == "result"
        and "structured_output" in payload
    ):
        payload = payload["structured_output"]

    return validate_payload(payload)


def validate_payload(payload: Any) -> tuple[dict[str, Any] | None, list[str]]:
    errors = validate_dream_journey_result(payload)
    if errors:
        return None, errors
    return payload, []


def recover_claude_results(
    *,
    jobs: list[AgentJob],
    claude_dir: Path = DEFAULT_CLAUDE_LOG_DIR,
) -> dict[str, AgentAttemptResult]:
    job_lookup = {job.job_key: job for job in jobs if job.agent_name == "claude"}
    recovered: dict[str, AgentAttemptResult] = {}

    for path in sorted(
        claude_dir.glob("*.jsonl"), key=lambda file_path: file_path.stat().st_mtime
    ):
        job_key: str | None = None
        payload: Any = None
        for entry in _iter_jsonl(path):
            if job_key is None:
                job_key = extract_job_key(json.dumps(entry, ensure_ascii=False))
            attachment = entry.get("attachment")
            if (
                isinstance(attachment, dict)
                and attachment.get("type") == "structured_output"
            ):
                payload = attachment.get("data")

        if job_key is None or payload is None or job_key not in job_lookup:
            continue

        parsed_json, errors = validate_payload(payload)
        if errors or parsed_json is None:
            continue

        job = job_lookup[job_key]
        recovered[job_key] = AgentAttemptResult(
            agent_name="claude",
            model_name=job.model_name,
            job_key=job_key,
            mode=job.mode,
            category=job.category,
            success=True,
            parsed_json=parsed_json,
            errors=[],
            used_retry=False,
            attempts=1,
            exit_code=0,
        )

    return recovered


def recover_codex_results(
    *,
    jobs: list[AgentJob],
    codex_sessions_dir: Path = DEFAULT_CODEX_SESSIONS_DIR,
) -> dict[str, AgentAttemptResult]:
    job_lookup = {job.job_key: job for job in jobs if job.agent_name == "codex"}
    recovered: dict[str, AgentAttemptResult] = {}

    for path in sorted(
        codex_sessions_dir.rglob("*.jsonl"),
        key=lambda file_path: file_path.stat().st_mtime,
    ):
        job_key: str | None = None
        candidate_output: str | None = None

        for entry in _iter_jsonl(path):
            if job_key is None:
                job_key = extract_job_key(json.dumps(entry, ensure_ascii=False))

            if entry.get("type") == "event_msg":
                payload = entry.get("payload", {})
                if (
                    isinstance(payload, dict)
                    and payload.get("type") == "task_complete"
                    and isinstance(payload.get("last_agent_message"), str)
                ):
                    candidate_output = payload["last_agent_message"]

            if entry.get("type") == "response_item":
                payload = entry.get("payload", {})
                if (
                    isinstance(payload, dict)
                    and payload.get("type") == "message"
                    and payload.get("role") == "assistant"
                ):
                    for item in payload.get("content", []):
                        if (
                            isinstance(item, dict)
                            and item.get("type") == "output_text"
                            and isinstance(item.get("text"), str)
                            and item["text"].lstrip().startswith("{")
                        ):
                            candidate_output = item["text"]

        if job_key is None or candidate_output is None or job_key not in job_lookup:
            continue

        parsed_json, errors = parse_and_validate_agent_output(candidate_output)
        if errors or parsed_json is None:
            continue

        job = job_lookup[job_key]
        recovered[job_key] = AgentAttemptResult(
            agent_name="codex",
            model_name=job.model_name,
            job_key=job_key,
            mode=job.mode,
            category=job.category,
            success=True,
            parsed_json=parsed_json,
            errors=[],
            used_retry=False,
            attempts=1,
            exit_code=0,
        )

    return recovered


def recover_results_from_logs(
    *,
    jobs: list[AgentJob],
    claude_dir: Path = DEFAULT_CLAUDE_LOG_DIR,
    codex_sessions_dir: Path = DEFAULT_CODEX_SESSIONS_DIR,
) -> dict[str, AgentAttemptResult]:
    merged: dict[str, AgentAttemptResult] = {}
    merged.update(
        recover_codex_results(jobs=jobs, codex_sessions_dir=codex_sessions_dir)
    )
    merged.update(recover_claude_results(jobs=jobs, claude_dir=claude_dir))
    return merged


def extract_job_key(text: str) -> str | None:
    match = JOB_KEY_PATTERN.search(text)
    if match is None:
        return None
    return match.group(1).strip()


def _iter_jsonl(path: Path) -> list[dict[str, Any]]:
    entries: list[dict[str, Any]] = []
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        if not line.strip():
            continue
        try:
            entry = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(entry, dict):
            entries.append(entry)
    return entries


def default_attempt_log_path(output_path: Path) -> Path:
    return Path(str(output_path) + ".attempts.jsonl")


def _write_json_atomic(path: Path, payload: dict[str, Any]) -> None:
    temp_handle, temp_name = tempfile.mkstemp(
        prefix=f"{path.name}.",
        suffix=".tmp",
        dir=path.parent,
    )
    os.close(temp_handle)
    temp_path = Path(temp_name)
    try:
        temp_path.write_text(
            json.dumps(payload, indent=2, ensure_ascii=False) + "\n",
            encoding="utf-8",
        )
        os.replace(temp_path, path)
    finally:
        if temp_path.exists():
            temp_path.unlink()


def _slugify(value: str) -> str:
    slug = re.sub(r"[^a-z0-9]+", "-", value.lower()).strip("-")
    return slug or "job"


def _timestamp() -> str:
    return (
        datetime.now(timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z")
    )


def _timestamp_plus_seconds(seconds: float) -> str:
    later = datetime.now(timezone.utc).timestamp() + seconds
    return (
        datetime.fromtimestamp(later, timezone.utc)
        .isoformat(timespec="seconds")
        .replace("+00:00", "Z")
    )


def _format_duration(seconds: float) -> str:
    seconds_int = max(0, int(round(seconds)))
    if seconds_int < 60:
        return f"{seconds_int}s"
    if seconds_int < 3600:
        return f"{seconds_int // 60}m{seconds_int % 60:02d}s"
    hours, remainder = divmod(seconds_int, 3600)
    minutes = remainder // 60
    return f"{hours}h{minutes:02d}m"


def _truncate(text: str, limit: int) -> str:
    collapsed = " ".join(text.split())
    if len(collapsed) <= limit:
        return collapsed
    return collapsed[: limit - 3] + "..."


def main() -> int:
    args = parse_args()
    if args.max_concurrency < 1:
        raise ValueError("--max-concurrency must be at least 1")
    if args.recover_only and not args.recover_from_logs:
        raise ValueError("--recover-only requires --recover-from-logs")

    jobs = build_jobs(args)
    reporter = BatchReporter(
        jobs=jobs,
        output_path=args.output,
        attempt_log_path=args.attempt_log or default_attempt_log_path(args.output),
    )
    recovered_results = (
        recover_results_from_logs(jobs=jobs) if args.recover_from_logs else None
    )
    if args.recover_only:
        reporter.record_batch_start(total_jobs=len(jobs))
        reporter.record_recovered(recovered_results or {})
        reporter.write_snapshot(recovered_results or {})
        reporter.record_batch_complete(recovered_results or {})
        return 0
    asyncio.run(
        run_batch(
            jobs,
            max_concurrency=args.max_concurrency,
            codex_timeout_seconds=args.codex_timeout_seconds or args.timeout_seconds,
            claude_timeout_seconds=args.claude_timeout_seconds or args.timeout_seconds,
            codex_bin=args.codex_bin,
            claude_bin=args.claude_bin,
            reporter=reporter,
            recovered_results=recovered_results,
            long_backoff_seconds=args.long_backoff_seconds,
            short_backoff_seconds=args.short_backoff_seconds,
        )
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
