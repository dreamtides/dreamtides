#!/usr/bin/env python3
"""Run Dreamsign design jobs across Codex and Claude with deterministic batching."""

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
import tomllib
from collections import deque
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, TextIO

DEFAULT_MAX_CONCURRENCY = 4
DEFAULT_TIMEOUT_SECONDS = 1800
DEFAULT_CLAUDE_LOG_DIR = Path.home() / ".claude/projects/-Users-dthurn-dreamtides"
DEFAULT_CODEX_SESSIONS_DIR = Path.home() / ".codex/sessions"
DEFAULT_CLAUDE_MODEL = "opus"
DEFAULT_CODEX_MODEL = "gpt-5.4"
DEFAULT_SEED = 1
DEFAULT_MODE1_COUNT = 32
DEFAULT_MODE2_COUNT = 56
DEFAULT_MODE3A_COUNT = 56
DEFAULT_MODE3B_COUNT = 56
DEFAULT_MIN_ITEMS_PER_JOB = 1
AGENT_ORDER = ("codex", "claude")
MODE_ORDER = ("mode1", "mode2", "mode3a", "mode3b")
MODE_LABELS = {
    "mode1": "Mode 1: Dreamcaller Inspired",
    "mode2": "Mode 2: Magic the Gathering Inspired",
    "mode3a": "Mode 3A: Monster Train / Slay the Spire Inspired, Battle-Level",
    "mode3b": "Mode 3B: Monster Train / Slay the Spire Inspired, Quest-Level",
}
JOB_KEY_PATTERN = re.compile(r"Job key:\s*([a-z0-9-]+)")
REQUIRED_TOP_LEVEL_KEYS = (
    "dreamsign",
    "type",
    "ability_text",
    "justification",
    "rejected_alternatives",
)
VALID_DREAMSIGN_TYPES = {"Battle", "Quest", "Hybrid"}
REPO_ROOT = Path(__file__).resolve().parents[2]
SKILL_PATH = REPO_ROOT / ".llms/skills/dreamsign-design/SKILL.md"
DEFAULT_DREAMCALLERS_PATH = REPO_ROOT / "rules_engine/tabula/dreamcallers.toml"
DEFAULT_COMMANDERS_PATH = Path("/Users/dthurn/Documents/edhrec/commanders.txt")
DEFAULT_STS_FIGHT_PATH = Path("/Users/dthurn/Documents/slaythespire/relics-fight.txt")
DEFAULT_MT_BATTLE_PATH = Path(
    "/Users/dthurn/Documents/monstertrain/artifacts-battle.txt"
)
DEFAULT_MT_MAP_PATH = Path("/Users/dthurn/Documents/monstertrain/artifacts-map.txt")
DEFAULT_STS_OTHER_PATH = Path("/Users/dthurn/Documents/slaythespire/relics-other.txt")

DREAMSIGN_JSON_SCHEMA: dict[str, Any] = {
    "type": "object",
    "required": list(REQUIRED_TOP_LEVEL_KEYS),
    "properties": {
        "dreamsign": {"type": "string"},
        "type": {"type": "string", "enum": sorted(VALID_DREAMSIGN_TYPES)},
        "ability_text": {"type": "string"},
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
class DreamsignPromptSpec:
    """Deterministic prompt plan before provider assignment."""

    mode: str
    source_paths: tuple[str, ...]
    source_items: tuple[str, ...]


@dataclass(frozen=True)
class AgentJob:
    """One provider invocation for one Dreamsign design prompt."""

    agent_name: str
    model_name: str
    job_key: str
    mode: str
    source_paths: tuple[str, ...]
    source_items: tuple[str, ...]
    avoid_ability_texts: tuple[str, ...] = ()
    attempt: int = 1


@dataclass
class AgentAttemptResult:
    """Final state for one Dreamsign job."""

    agent_name: str
    model_name: str
    job_key: str
    mode: str
    success: bool
    parsed_json: dict[str, Any] | None
    errors: list[str]
    used_retry: bool
    attempts: int = 1
    stdout: str = ""
    stderr: str = ""
    exit_code: int | None = None


class BatchReporter:
    """Persist live progress and partial results for a Dreamsign batch."""

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
                "attempt": job.attempt,
                "active_counts": active_counts,
                "pending_counts": pending_counts,
            }
        )
        self._print(
            f"[{_timestamp()}] START {job.agent_name:<6} attempt={job.attempt} "
            f"mode={job.mode:<5} active={sum(active_counts.values())} "
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
            "attempt": result.attempts,
            "success": result.success,
            "exit_code": result.exit_code,
            "errors": result.errors,
            "stderr": result.stderr,
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
            f"mode={result.mode:<5} active={sum(active_counts.values())} "
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
                "attempt": job.attempt,
                "errors": errors,
                "active_counts": active_counts,
                "pending_counts": pending_counts,
            }
        )
        self._print(
            f"[{_timestamp()}] RETRY {job.agent_name:<6} attempt={job.attempt} "
            f"mode={job.mode:<5} active={sum(active_counts.values())} "
            f"pending={sum(pending_counts.values())} job={job.job_key}"
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
        help="Maximum concurrent Dreamsign jobs",
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
        help="Deterministic seed used for prompt generation and provider assignment",
    )
    parser.add_argument(
        "--mode1-count",
        type=int,
        default=DEFAULT_MODE1_COUNT,
        help="Total Mode 1 jobs",
    )
    parser.add_argument(
        "--mode2-count",
        type=int,
        default=DEFAULT_MODE2_COUNT,
        help="Total Mode 2 jobs",
    )
    parser.add_argument(
        "--mode3a-count",
        type=int,
        default=DEFAULT_MODE3A_COUNT,
        help="Total Mode 3A jobs",
    )
    parser.add_argument(
        "--mode3b-count",
        type=int,
        default=DEFAULT_MODE3B_COUNT,
        help="Total Mode 3B jobs",
    )
    parser.add_argument(
        "--mode2-min-per-job",
        "--mode2-pool-size",
        dest="mode2_min_per_job",
        type=int,
        default=DEFAULT_MIN_ITEMS_PER_JOB,
        help=(
            "Minimum number of MTG source cards per Mode 2 job. "
            "The runner may assign more to saturate the full file."
        ),
    )
    parser.add_argument(
        "--mode3a-sts-min-per-job",
        "--mode3a-sts-pool-size",
        dest="mode3a_sts_min_per_job",
        type=int,
        default=DEFAULT_MIN_ITEMS_PER_JOB,
        help=(
            "Minimum number of Slay the Spire battle relics per Mode 3A job. "
            "The runner may assign more to saturate the full file."
        ),
    )
    parser.add_argument(
        "--mode3a-mt-min-per-job",
        "--mode3a-mt-pool-size",
        dest="mode3a_mt_min_per_job",
        type=int,
        default=DEFAULT_MIN_ITEMS_PER_JOB,
        help=(
            "Minimum number of Monster Train battle artifacts per Mode 3A job. "
            "The runner may assign more to saturate the full file."
        ),
    )
    parser.add_argument(
        "--mode3b-sts-min-per-job",
        "--mode3b-sts-pool-size",
        dest="mode3b_sts_min_per_job",
        type=int,
        default=DEFAULT_MIN_ITEMS_PER_JOB,
        help=(
            "Minimum number of Slay the Spire quest relics per Mode 3B job. "
            "The runner may assign more to saturate the full file."
        ),
    )
    parser.add_argument(
        "--mode3b-mt-min-per-job",
        "--mode3b-mt-pool-size",
        dest="mode3b_mt_min_per_job",
        type=int,
        default=DEFAULT_MIN_ITEMS_PER_JOB,
        help=(
            "Minimum number of Monster Train map artifacts per Mode 3B job. "
            "The runner may assign more to saturate the full file."
        ),
    )
    parser.add_argument(
        "--dreamcallers-path",
        type=Path,
        default=DEFAULT_DREAMCALLERS_PATH,
        help="Path to dreamcallers.toml for Mode 1",
    )
    parser.add_argument(
        "--commanders-path",
        type=Path,
        default=DEFAULT_COMMANDERS_PATH,
        help="Path to MTG commander inspiration file for Mode 2",
    )
    parser.add_argument(
        "--sts-fight-path",
        type=Path,
        default=DEFAULT_STS_FIGHT_PATH,
        help="Path to Slay the Spire battle relic inspirations for Mode 3A",
    )
    parser.add_argument(
        "--mt-battle-path",
        type=Path,
        default=DEFAULT_MT_BATTLE_PATH,
        help="Path to Monster Train battle artifact inspirations for Mode 3A",
    )
    parser.add_argument(
        "--mt-map-path",
        type=Path,
        default=DEFAULT_MT_MAP_PATH,
        help="Path to Monster Train map artifact inspirations for Mode 3B",
    )
    parser.add_argument(
        "--sts-other-path",
        type=Path,
        default=DEFAULT_STS_OTHER_PATH,
        help="Path to Slay the Spire quest relic inspirations for Mode 3B",
    )
    parser.add_argument(
        "--avoid-ability-texts-path",
        type=Path,
        help=(
            "Optional newline-delimited file of existing Dreamsign ability texts to "
            "avoid repeating or closely paraphrasing."
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


def load_dreamcaller_specs(path: Path) -> list[DreamsignPromptSpec]:
    with path.open("rb") as handle:
        payload = tomllib.load(handle)

    dreamcallers = payload.get("dreamcaller")
    if not isinstance(dreamcallers, list):
        raise ValueError(f"Missing [[dreamcaller]] array in {path}")

    specs: list[DreamsignPromptSpec] = []
    for entry in dreamcallers:
        if not isinstance(entry, dict):
            raise ValueError(f"Unexpected dreamcaller entry in {path}")
        name = str(entry["name"])
        title = str(entry["title"])
        rendered_text = str(entry["rendered-text"])
        mandatory_tides = ", ".join(entry.get("mandatory-tides", []))
        optional_tides = ", ".join(entry.get("optional-tides", []))
        specs.append(
            DreamsignPromptSpec(
                mode="mode1",
                source_paths=(str(path),),
                source_items=(
                    f"Dreamcaller: {name}, {title}",
                    f"Rendered text: {rendered_text}",
                    f"Mandatory tides: {mandatory_tides}",
                    f"Optional tides: {optional_tides}",
                ),
            )
        )
    return specs


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


def load_optional_unique_lines(path: Path | None) -> tuple[str, ...]:
    if path is None:
        return ()
    return tuple(load_unique_lines(path))


def _arg_value(
    args: argparse.Namespace, new_name: str, legacy_name: str, default: int
) -> int:
    if hasattr(args, new_name):
        return int(getattr(args, new_name))
    if hasattr(args, legacy_name):
        return int(getattr(args, legacy_name))
    return default


def _build_saturated_chunks(
    items: list[str], *, chunk_count: int, min_per_chunk: int, rng: random.Random
) -> list[tuple[str, ...]]:
    if chunk_count < 1:
        raise ValueError("chunk_count must be at least 1")
    if min_per_chunk < 1:
        raise ValueError("min_per_chunk must be at least 1")

    shuffled = list(items)
    rng.shuffle(shuffled)
    total_slots = max(len(shuffled), chunk_count * min_per_chunk)
    assigned_items = list(shuffled)

    duplicate_pool: list[str] = []
    while len(assigned_items) < total_slots:
        if not duplicate_pool:
            duplicate_pool = list(shuffled)
            rng.shuffle(duplicate_pool)
        assigned_items.append(duplicate_pool.pop())

    size_base, size_remainder = divmod(total_slots, chunk_count)
    sizes = [
        size_base + (1 if chunk_index < size_remainder else 0)
        for chunk_index in range(chunk_count)
    ]
    rng.shuffle(sizes)
    rng.shuffle(assigned_items)

    chunks: list[tuple[str, ...]] = []
    cursor = 0
    for size in sizes:
        chunks.append(tuple(assigned_items[cursor : cursor + size]))
        cursor += size

    rng.shuffle(chunks)
    return chunks


def build_mode_specs(args: argparse.Namespace) -> dict[str, list[DreamsignPromptSpec]]:
    specs_by_mode: dict[str, list[DreamsignPromptSpec]] = {
        mode: [] for mode in MODE_ORDER
    }

    if args.mode1_count == 0:
        specs_by_mode["mode1"] = []
    else:
        mode1_specs = load_dreamcaller_specs(args.dreamcallers_path)
        if args.mode1_count != len(mode1_specs):
            raise ValueError(
                f"--mode1-count must match dreamcaller count in {args.dreamcallers_path}: "
                f"expected {len(mode1_specs)}, got {args.mode1_count}"
            )
        specs_by_mode["mode1"] = mode1_specs

    if args.mode2_count > 0:
        mode2_rng = random.Random(f"{args.seed}:mode2")
        commander_chunks = _build_saturated_chunks(
            load_unique_lines(args.commanders_path),
            chunk_count=args.mode2_count,
            min_per_chunk=_arg_value(
                args,
                "mode2_min_per_job",
                "mode2_pool_size",
                DEFAULT_MIN_ITEMS_PER_JOB,
            ),
            rng=mode2_rng,
        )
        specs_by_mode["mode2"] = [
            DreamsignPromptSpec(
                mode="mode2",
                source_paths=(str(args.commanders_path),),
                source_items=chunk,
            )
            for chunk in commander_chunks
        ]

    if args.mode3a_count > 0:
        mode3a_sts_rng = random.Random(f"{args.seed}:mode3a:sts")
        mode3a_mt_rng = random.Random(f"{args.seed}:mode3a:mt")
        mode3a_sts_chunks = _build_saturated_chunks(
            load_unique_lines(args.sts_fight_path),
            chunk_count=args.mode3a_count,
            min_per_chunk=_arg_value(
                args,
                "mode3a_sts_min_per_job",
                "mode3a_sts_pool_size",
                DEFAULT_MIN_ITEMS_PER_JOB,
            ),
            rng=mode3a_sts_rng,
        )
        mode3a_mt_chunks = _build_saturated_chunks(
            load_unique_lines(args.mt_battle_path),
            chunk_count=args.mode3a_count,
            min_per_chunk=_arg_value(
                args,
                "mode3a_mt_min_per_job",
                "mode3a_mt_pool_size",
                DEFAULT_MIN_ITEMS_PER_JOB,
            ),
            rng=mode3a_mt_rng,
        )
        specs_by_mode["mode3a"] = [
            DreamsignPromptSpec(
                mode="mode3a",
                source_paths=(str(args.sts_fight_path), str(args.mt_battle_path)),
                source_items=sts_chunk + mt_chunk,
            )
            for sts_chunk, mt_chunk in zip(
                mode3a_sts_chunks, mode3a_mt_chunks, strict=True
            )
        ]

    if args.mode3b_count > 0:
        mode3b_sts_rng = random.Random(f"{args.seed}:mode3b:sts")
        mode3b_mt_rng = random.Random(f"{args.seed}:mode3b:mt")
        mode3b_sts_chunks = _build_saturated_chunks(
            load_unique_lines(args.sts_other_path),
            chunk_count=args.mode3b_count,
            min_per_chunk=_arg_value(
                args,
                "mode3b_sts_min_per_job",
                "mode3b_sts_pool_size",
                DEFAULT_MIN_ITEMS_PER_JOB,
            ),
            rng=mode3b_sts_rng,
        )
        mode3b_mt_chunks = _build_saturated_chunks(
            load_unique_lines(args.mt_map_path),
            chunk_count=args.mode3b_count,
            min_per_chunk=_arg_value(
                args,
                "mode3b_mt_min_per_job",
                "mode3b_mt_pool_size",
                DEFAULT_MIN_ITEMS_PER_JOB,
            ),
            rng=mode3b_mt_rng,
        )
        specs_by_mode["mode3b"] = [
            DreamsignPromptSpec(
                mode="mode3b",
                source_paths=(str(args.mt_map_path), str(args.sts_other_path)),
                source_items=mt_chunk + sts_chunk,
            )
            for mt_chunk, sts_chunk in zip(
                mode3b_mt_chunks, mode3b_sts_chunks, strict=True
            )
        ]

    return specs_by_mode


def provider_counts_for_mode(mode_count: int) -> dict[str, int]:
    if mode_count % len(AGENT_ORDER) != 0:
        raise ValueError(
            f"Mode count must divide evenly across providers, got {mode_count}"
        )
    return {agent_name: mode_count // len(AGENT_ORDER) for agent_name in AGENT_ORDER}


def build_jobs(args: argparse.Namespace) -> list[AgentJob]:
    specs_by_mode = build_mode_specs(args)
    model_by_agent = {"codex": args.codex_model, "claude": args.claude_model}
    avoid_ability_texts = load_optional_unique_lines(
        getattr(args, "avoid_ability_texts_path", None)
    )
    jobs: list[AgentJob] = []

    for mode in MODE_ORDER:
        specs = list(specs_by_mode[mode])
        counts = provider_counts_for_mode(len(specs))
        assignments = [
            agent_name for agent_name in AGENT_ORDER for _ in range(counts[agent_name])
        ]
        assignment_rng = random.Random(f"{args.seed}:{mode}:providers")
        assignment_rng.shuffle(assignments)
        for index, (spec, agent_name) in enumerate(
            zip(specs, assignments, strict=True),
            start=1,
        ):
            source_slug = _slugify(spec.source_items[0])[:36]
            jobs.append(
                AgentJob(
                    agent_name=agent_name,
                    model_name=model_by_agent[agent_name],
                    job_key=f"{mode}-{index:03d}-{agent_name}-{source_slug}",
                    mode=mode,
                    source_paths=spec.source_paths,
                    source_items=spec.source_items,
                    avoid_ability_texts=avoid_ability_texts,
                )
            )

    provider_totals = {
        agent_name: sum(1 for job in jobs if job.agent_name == agent_name)
        for agent_name in AGENT_ORDER
    }
    if provider_totals["codex"] != provider_totals["claude"]:
        raise ValueError(f"Provider totals must balance, got {provider_totals}")
    if len({job.job_key for job in jobs}) != len(jobs):
        raise ValueError("Duplicate job keys generated")

    return jobs


def validate_dreamsign_result(payload: Any) -> list[str]:
    if not isinstance(payload, dict):
        return ["Top-level JSON value must be an object"]

    errors: list[str] = []
    for key in REQUIRED_TOP_LEVEL_KEYS:
        if key not in payload:
            errors.append(f"Missing top-level key: {key}")

    if (
        not isinstance(payload.get("dreamsign"), str)
        or not payload.get("dreamsign").strip()
    ):
        errors.append("dreamsign must be a non-empty string")
    if payload.get("type") not in VALID_DREAMSIGN_TYPES:
        errors.append(
            f"type must be one of: {', '.join(sorted(VALID_DREAMSIGN_TYPES))}"
        )
    if (
        not isinstance(payload.get("ability_text"), str)
        or not payload.get("ability_text").strip()
    ):
        errors.append("ability_text must be a non-empty string")
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
            "source_paths": list(job.source_paths),
            "source_items": list(job.source_items),
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
                }
                if result is not None
                else None
            ),
        }

    return {"summary": summary, "jobs": jobs_payload}


def build_agent_prompt(job: AgentJob, skill_text: str) -> str:
    sources_block = "\n".join(f"- {path}" for path in job.source_paths)
    inspirations_block = "\n".join(
        f"{index}. {item}" for index, item in enumerate(job.source_items, start=1)
    )
    avoid_block = ""
    if job.avoid_ability_texts:
        avoid_lines = "\n".join(f"- {text}" for text in job.avoid_ability_texts)
        avoid_block = (
            "Existing Dreamsign ability texts to avoid repeating or closely paraphrasing:\n"
            f"{avoid_lines}\n\n"
            "Do not reuse the same core hook, cadence, or obvious near-variant of any listed text.\n\n"
        )
    return (
        "You are generating structured JSON for one Dreamtides Dreamsign design.\n"
        "Follow the dreamsign-design skill specification below exactly.\n"
        "Use the current repository as context and perform the research the skill requires.\n"
        "Read the required docs listed in the skill before designing.\n"
        "Output exactly one valid JSON object and no markdown or prose.\n"
        "Return JSON with exactly these keys:\n"
        "- dreamsign\n"
        "- type\n"
        "- ability_text\n"
        "- justification\n"
        "- rejected_alternatives\n\n"
        f"Job key: {job.job_key}\n"
        f"Mode: {MODE_LABELS[job.mode]}\n"
        f"Provider target: {job.agent_name} ({job.model_name})\n"
        "Source files:\n"
        f"{sources_block}\n\n"
        "Inspiration input:\n"
        f"{inspirations_block}\n\n"
        f"{avoid_block}"
        "Skill specification:\n"
        f"{skill_text}"
    )


def choose_next_agent(
    *,
    pending_counts: dict[str, int],
    active_counts: dict[str, int],
    max_concurrency: int,
) -> str | None:
    available_agents = [
        agent_name
        for agent_name in AGENT_ORDER
        if pending_counts.get(agent_name, 0) > 0
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

    reporter.record_batch_start(total_jobs=len(jobs))
    reporter.record_recovered(results_by_job_key)
    reporter.write_snapshot(results_by_job_key)

    with tempfile.TemporaryDirectory(prefix="dreamsign-batch-") as temp_dir:
        temp_dir_path = Path(temp_dir)
        schema_path = temp_dir_path / "dreamsign-output-schema.json"
        schema_path.write_text(
            json.dumps(DREAMSIGN_JSON_SCHEMA, indent=2, ensure_ascii=False) + "\n",
            encoding="utf-8",
        )

        while active_tasks or any(pending_jobs.values()):
            while len(active_tasks) < max_concurrency:
                pending_counts = {
                    agent_name: len(queue) for agent_name, queue in pending_jobs.items()
                }
                next_agent = choose_next_agent(
                    pending_counts=pending_counts,
                    active_counts=active_counts,
                    max_concurrency=max_concurrency,
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
                break

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

                if not result.success and job.attempt == 1:
                    retry_job = AgentJob(
                        agent_name=job.agent_name,
                        model_name=job.model_name,
                        job_key=job.job_key,
                        mode=job.mode,
                        source_paths=job.source_paths,
                        source_items=job.source_items,
                        avoid_ability_texts=job.avoid_ability_texts,
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
            success=False,
            parsed_json=None,
            errors=[f"Runner exception: {type(error).__name__}: {error}"],
            used_retry=job.attempt > 1,
            attempts=job.attempt,
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

    return AgentAttemptResult(
        agent_name=job.agent_name,
        model_name=job.model_name,
        job_key=job.job_key,
        mode=job.mode,
        success=not errors,
        parsed_json=parsed_json,
        errors=errors,
        used_retry=job.attempt > 1,
        attempts=job.attempt,
        stdout=raw_text,
        stderr=stderr,
        exit_code=exit_code,
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
        json.dumps(DREAMSIGN_JSON_SCHEMA, ensure_ascii=False),
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

    if (
        isinstance(payload, dict)
        and payload.get("type") == "result"
        and "structured_output" in payload
    ):
        payload = payload["structured_output"]

    return validate_payload(payload)


def validate_payload(payload: Any) -> tuple[dict[str, Any] | None, list[str]]:
    errors = validate_dreamsign_result(payload)
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


def main() -> int:
    args = parse_args()
    if args.max_concurrency < 1:
        raise ValueError("--max-concurrency must be at least 1")
    for mode_name in ("mode1_count", "mode2_count", "mode3a_count", "mode3b_count"):
        if getattr(args, mode_name) < 0:
            raise ValueError(f"--{mode_name.replace('_', '-')} must be at least 0")
    if args.recover_only and not args.recover_from_logs:
        raise ValueError("--recover-only requires --recover-from-logs")

    jobs = build_jobs(args)
    if not jobs:
        raise ValueError("At least one job must be requested")
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
        )
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
