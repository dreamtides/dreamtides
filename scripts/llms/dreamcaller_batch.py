#!/usr/bin/env python3
"""Run codex and claude dreamcaller design jobs in parallel over a prompt file."""

from __future__ import annotations

import argparse
import asyncio
import json
import os
import re
import signal
import sys
import tempfile
from collections import deque
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, TextIO

DEFAULT_MAX_CONCURRENCY = 4
DEFAULT_TIMEOUT_SECONDS = 600
DEFAULT_CLAUDE_LOG_DIR = Path.home() / ".claude/projects/-Users-dthurn-dreamtides"
DEFAULT_CODEX_SESSIONS_DIR = Path.home() / ".codex/sessions"
REQUIRED_TOP_LEVEL_KEYS = (
    "theme",
    "brainstorm_pool",
    "final_designs",
    "selection_notes",
)
REQUIRED_BRAINSTORM_KEYS = (
    "id",
    "ability_idea",
    "interesting_note",
    "support_estimate",
    "novelty_test",
    "quality_gates",
    "is_obvious_design",
    "uses_battlefield_position",
    "hearthstone_source",
    "selected_for_final",
)
REQUIRED_FINAL_DESIGN_KEYS = (
    "id",
    "source_brainstorm_id",
    "ability_text",
    "ability_type",
    "design_rationale",
    "synergy_citations",
    "support_estimate",
    "novelty_statement",
    "inspiration_source",
    "tags",
)
REQUIRED_SELECTION_NOTE_KEYS = (
    "selected_brainstorm_ids",
    "cut_brainstorm_ids",
    "constraints_satisfied",
)
REQUIRED_CONSTRAINT_KEYS = (
    "obvious_design_count",
    "novel_design_count",
    "has_hearthstone_inspired_design",
    "has_positional_design",
    "ability_type_mix",
)
AGENT_ORDER = ("codex", "claude")
REPO_ROOT = Path(__file__).resolve().parents[2]
SKILL_PATH = REPO_ROOT / ".llms/skills/dreamcaller-design/SKILL.md"
THEME_PROMPT_PATTERN = re.compile(r"Theme prompt:\s*(.+?)(?:\\n|\n)")
PROMPT_KEY_SPLIT_PATTERN = re.compile(r"[:,(]")

DREAMCALLER_JSON_SCHEMA: dict[str, Any] = {
    "type": "object",
    "required": list(REQUIRED_TOP_LEVEL_KEYS),
    "properties": {
        "theme": {"type": "string"},
        "brainstorm_pool": {
            "type": "array",
            "items": {
                "type": "object",
                "required": list(REQUIRED_BRAINSTORM_KEYS),
                "additionalProperties": True,
            },
        },
        "final_designs": {
            "type": "array",
            "items": {
                "type": "object",
                "required": list(REQUIRED_FINAL_DESIGN_KEYS),
                "additionalProperties": True,
            },
        },
        "selection_notes": {
            "type": "object",
            "required": list(REQUIRED_SELECTION_NOTE_KEYS),
            "properties": {
                "constraints_satisfied": {
                    "type": "object",
                    "required": list(REQUIRED_CONSTRAINT_KEYS),
                    "additionalProperties": True,
                }
            },
            "additionalProperties": True,
        },
    },
    "additionalProperties": True,
}


@dataclass(frozen=True)
class AgentJob:
    """One agent invocation for one prompt."""

    agent_name: str
    prompt: str
    attempt: int = 1


@dataclass
class AgentAttemptResult:
    """Final state for one prompt/agent pair."""

    agent_name: str
    prompt: str
    success: bool
    parsed_json: dict[str, Any] | None
    errors: list[str]
    used_retry: bool
    attempts: int = 1
    stdout: str = ""
    stderr: str = ""
    exit_code: int | None = None


class BatchReporter:
    """Persist live progress and partial results for a batch run."""

    def __init__(
        self,
        *,
        prompts: list[str],
        output_path: Path,
        attempt_log_path: Path,
        stream: TextIO | None = None,
    ) -> None:
        self.prompts = prompts
        self.output_path = output_path
        self.attempt_log_path = attempt_log_path
        self.stream = stream or sys.stdout
        self.output_path.parent.mkdir(parents=True, exist_ok=True)
        self.attempt_log_path.parent.mkdir(parents=True, exist_ok=True)

    def record_batch_start(self, *, prompt_count: int, total_jobs: int) -> None:
        self._record_event(
            {
                "event": "batch_start",
                "prompt_count": prompt_count,
                "total_jobs": total_jobs,
            }
        )
        self._print(
            f"[{_timestamp()}] BATCH prompts={prompt_count} total_jobs={total_jobs}"
        )

    def record_recovered(
        self, results_by_prompt: dict[str, dict[str, AgentAttemptResult]]
    ) -> None:
        recovered_count = sum(
            1
            for agent_results in results_by_prompt.values()
            for result in agent_results.values()
            if result.success
        )
        if recovered_count == 0:
            return
        self._record_event(
            {
                "event": "recovered",
                "recovered_count": recovered_count,
            }
        )
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
                "prompt": job.prompt,
                "attempt": job.attempt,
                "active_counts": active_counts,
                "pending_counts": pending_counts,
            }
        )
        self._print(
            f"[{_timestamp()}] START {job.agent_name:<6} attempt={job.attempt} "
            f"active={sum(active_counts.values())} pending={sum(pending_counts.values())} "
            f'prompt="{_short_prompt(job.prompt)}"'
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
            "prompt": result.prompt,
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
            f"active={sum(active_counts.values())} pending={sum(pending_counts.values())} "
            f'{detail} prompt="{_short_prompt(result.prompt)}"'
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
                "prompt": job.prompt,
                "attempt": job.attempt,
                "errors": errors,
                "active_counts": active_counts,
                "pending_counts": pending_counts,
            }
        )
        self._print(
            f"[{_timestamp()}] RETRY {job.agent_name:<6} attempt={job.attempt} "
            f"active={sum(active_counts.values())} pending={sum(pending_counts.values())} "
            f'prompt="{_short_prompt(job.prompt)}"'
        )

    def record_batch_complete(
        self, results_by_prompt: dict[str, dict[str, AgentAttemptResult]]
    ) -> None:
        success_count = sum(
            1
            for agent_results in results_by_prompt.values()
            for result in agent_results.values()
            if result.success
        )
        failure_count = sum(
            1
            for agent_results in results_by_prompt.values()
            for result in agent_results.values()
            if not result.success
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

    def write_snapshot(
        self, results_by_prompt: dict[str, dict[str, AgentAttemptResult]]
    ) -> None:
        _write_json_atomic(
            self.output_path,
            synthesize_results(self.prompts, results_by_prompt),
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
        "--input",
        type=Path,
        required=True,
        help="Path to newline-delimited dreamcaller prompts",
    )
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
        help="Maximum concurrent prompt/agent jobs",
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
        "--recover-from-logs",
        action="store_true",
        help="Seed or reconstruct results from ~/.claude and ~/.codex session logs",
    )
    parser.add_argument(
        "--recover-only",
        action="store_true",
        help="Write recovered results from provider logs without starting new agent jobs",
    )
    return parser.parse_args()


def load_prompts(path: Path) -> list[str]:
    prompts: list[str] = []
    seen: set[str] = set()

    for raw_line in path.read_text(encoding="utf-8").splitlines():
        prompt = raw_line.strip()
        if not prompt:
            continue
        if prompt in seen:
            raise ValueError(f"Duplicate prompt: {prompt}")
        prompts.append(prompt)
        seen.add(prompt)

    if not prompts:
        raise ValueError(f"No prompts found in {path}")

    return prompts


def validate_dreamcaller_result(payload: Any) -> list[str]:
    errors: list[str] = []
    if not isinstance(payload, dict):
        return ["Top-level JSON value must be an object"]

    for key in REQUIRED_TOP_LEVEL_KEYS:
        if key not in payload:
            errors.append(f"Missing top-level key: {key}")

    if not isinstance(payload.get("theme"), str):
        errors.append("theme must be a string")

    errors.extend(
        _validate_object_list(
            payload.get("brainstorm_pool"),
            "brainstorm_pool",
            REQUIRED_BRAINSTORM_KEYS,
        )
    )
    errors.extend(
        _validate_object_list(
            payload.get("final_designs"),
            "final_designs",
            REQUIRED_FINAL_DESIGN_KEYS,
        )
    )
    errors.extend(_validate_selection_notes(payload.get("selection_notes")))
    return errors


def _validate_object_list(
    value: Any, key_name: str, required_keys: tuple[str, ...]
) -> list[str]:
    if not isinstance(value, list):
        return [f"{key_name} must be a list"]

    errors: list[str] = []
    for index, item in enumerate(value):
        if not isinstance(item, dict):
            errors.append(f"{key_name}[{index}] must be an object")
            continue
        for required_key in required_keys:
            if required_key not in item:
                errors.append(f"{key_name}[{index}] missing key: {required_key}")
    return errors


def _validate_selection_notes(value: Any) -> list[str]:
    if not isinstance(value, dict):
        return ["selection_notes must be an object"]

    errors: list[str] = []
    for key in REQUIRED_SELECTION_NOTE_KEYS:
        if key not in value:
            errors.append(f"selection_notes missing key: {key}")

    if not isinstance(value.get("selected_brainstorm_ids"), list):
        errors.append("selection_notes.selected_brainstorm_ids must be a list")
    if not isinstance(value.get("cut_brainstorm_ids"), list):
        errors.append("selection_notes.cut_brainstorm_ids must be a list")

    constraints = value.get("constraints_satisfied")
    if not isinstance(constraints, dict):
        errors.append("selection_notes.constraints_satisfied must be an object")
        return errors

    for key in REQUIRED_CONSTRAINT_KEYS:
        if key not in constraints:
            errors.append(f"constraints_satisfied missing key: {key}")

    if not isinstance(constraints.get("ability_type_mix"), list):
        errors.append("constraints_satisfied.ability_type_mix must be a list")

    return errors


def synthesize_results(
    prompts: list[str],
    results_by_prompt: dict[str, dict[str, AgentAttemptResult]],
) -> dict[str, Any]:
    synthesis: dict[str, Any] = {}

    for prompt in prompts:
        agent_results = results_by_prompt.get(prompt, {})
        synthesis[prompt] = {
            "prompt": prompt,
            "agents": {
                agent_name: result.parsed_json
                for agent_name, result in agent_results.items()
                if result.success and result.parsed_json is not None
            },
            "verification": {
                agent_name: {
                    "success": result.success,
                    "used_retry": result.used_retry,
                    "attempts": result.attempts,
                    "errors": result.errors,
                    "exit_code": result.exit_code,
                }
                for agent_name, result in agent_results.items()
            },
        }

    return synthesis


def load_skill_text() -> str:
    return SKILL_PATH.read_text(encoding="utf-8")


def build_agent_prompt(prompt: str, skill_text: str) -> str:
    return (
        "You are generating structured JSON for Dreamtides dreamcaller card designs.\n"
        "Follow the skill specification below exactly.\n"
        "Use the current repository as context and perform the research the skill requires.\n"
        "Output exactly one valid JSON object and no markdown or prose.\n"
        "If a file is too large for a single read, switch to chunked reads or targeted search.\n"
        "Do not repeatedly retry oversized full-file reads.\n\n"
        f"Theme prompt: {prompt}\n\n"
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
    prompts: list[str],
    *,
    max_concurrency: int,
    codex_timeout_seconds: int,
    claude_timeout_seconds: int,
    codex_bin: str,
    claude_bin: str,
    reporter: BatchReporter,
    recovered_results: dict[str, dict[str, AgentAttemptResult]] | None = None,
) -> dict[str, dict[str, AgentAttemptResult]]:
    skill_text = load_skill_text()
    results_by_prompt: dict[str, dict[str, AgentAttemptResult]] = {
        prompt: dict((recovered_results or {}).get(prompt, {})) for prompt in prompts
    }
    pending_jobs: dict[str, deque[AgentJob]] = {
        agent_name: deque(
            AgentJob(agent_name=agent_name, prompt=prompt)
            for prompt in prompts
            if agent_name not in results_by_prompt[prompt]
        )
        for agent_name in AGENT_ORDER
    }
    active_counts = {agent_name: 0 for agent_name in AGENT_ORDER}
    active_tasks: dict[asyncio.Task[AgentAttemptResult], AgentJob] = {}

    reporter.record_batch_start(
        prompt_count=len(prompts),
        total_jobs=sum(len(queue) for queue in pending_jobs.values()),
    )
    reporter.record_recovered(results_by_prompt)
    reporter.write_snapshot(results_by_prompt)

    with tempfile.TemporaryDirectory(prefix="dreamcaller-batch-") as temp_dir:
        temp_dir_path = Path(temp_dir)

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
                results_by_prompt[result.prompt][result.agent_name] = result
                reporter.record_finish(
                    result,
                    active_counts=dict(active_counts),
                    pending_counts={
                        agent_name: len(queue)
                        for agent_name, queue in pending_jobs.items()
                    },
                )
                reporter.write_snapshot(results_by_prompt)

                if not result.success and job.attempt == 1:
                    retry_job = AgentJob(
                        agent_name=job.agent_name,
                        prompt=job.prompt,
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

    reporter.record_batch_complete(results_by_prompt)
    reporter.write_snapshot(results_by_prompt)
    return results_by_prompt


def _result_from_task(
    task: asyncio.Task[AgentAttemptResult], job: AgentJob
) -> AgentAttemptResult:
    try:
        return task.result()
    except Exception as error:  # pragma: no cover - defensive guard
        return AgentAttemptResult(
            agent_name=job.agent_name,
            prompt=job.prompt,
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
    temp_dir: Path,
    codex_timeout_seconds: int,
    claude_timeout_seconds: int,
    codex_bin: str,
    claude_bin: str,
) -> AgentAttemptResult:
    prompt_text = build_agent_prompt(job.prompt, skill_text)
    if job.agent_name == "codex":
        raw_text, stderr, exit_code = await _run_codex(
            prompt_text=prompt_text,
            temp_dir=temp_dir,
            timeout_seconds=codex_timeout_seconds,
            codex_bin=codex_bin,
        )
    else:
        raw_text, stderr, exit_code = await _run_claude(
            prompt_text=prompt_text,
            timeout_seconds=claude_timeout_seconds,
            claude_bin=claude_bin,
        )

    parsed_json, errors = parse_and_validate_agent_output(raw_text)
    if exit_code != 0:
        errors.append(f"{job.agent_name} exited with code {exit_code}")
    if stderr.strip():
        errors.append(f"{job.agent_name} stderr: {stderr.strip()}")

    return AgentAttemptResult(
        agent_name=job.agent_name,
        prompt=job.prompt,
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
    temp_dir: Path,
    timeout_seconds: int,
    codex_bin: str,
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
        codex_bin=codex_bin,
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
) -> tuple[str, str, int]:
    command = [
        claude_bin,
        "-p",
        "--output-format",
        "json",
        "--dangerously-skip-permissions",
        "--permission-mode",
        "bypassPermissions",
        "--json-schema",
        json.dumps(DREAMCALLER_JSON_SCHEMA, ensure_ascii=False),
        prompt_text,
    ]
    return await _run_command(command, timeout_seconds)


def build_codex_command(
    *, prompt_text: str, output_path: Path, codex_bin: str
) -> list[str]:
    return [
        codex_bin,
        "exec",
        "--dangerously-bypass-approvals-and-sandbox",
        "-C",
        str(REPO_ROOT),
        "-o",
        str(output_path),
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

        return (
            _read_temp_output(stdout_handle),
            _read_temp_output(stderr_handle),
            process.returncode,
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
    errors = validate_dreamcaller_result(payload)
    if errors:
        return None, errors
    return payload, []


def recover_claude_results(
    *,
    prompts: list[str],
    claude_dir: Path = DEFAULT_CLAUDE_LOG_DIR,
) -> dict[str, dict[str, AgentAttemptResult]]:
    prompt_lookup = build_prompt_lookup(prompts)
    recovered: dict[str, dict[str, AgentAttemptResult]] = {
        prompt: {} for prompt in prompts
    }

    for path in sorted(
        claude_dir.glob("*.jsonl"), key=lambda file_path: file_path.stat().st_mtime
    ):
        prompt: str | None = None
        payload: Any = None
        for entry in _iter_jsonl(path):
            if prompt is None:
                prompt = extract_theme_prompt(json.dumps(entry, ensure_ascii=False))
            attachment = entry.get("attachment")
            if (
                isinstance(attachment, dict)
                and attachment.get("type") == "structured_output"
            ):
                payload = attachment.get("data")

        resolved_prompt = resolve_prompt(prompt, prompt_lookup)
        if resolved_prompt is None or payload is None:
            continue

        parsed_json, errors = validate_payload(payload)
        if errors or parsed_json is None:
            continue

        recovered[resolved_prompt]["claude"] = AgentAttemptResult(
            agent_name="claude",
            prompt=resolved_prompt,
            success=True,
            parsed_json=parsed_json,
            errors=[],
            used_retry=False,
            attempts=1,
            exit_code=0,
        )

    return {prompt: results for prompt, results in recovered.items() if results}


def recover_codex_results(
    *,
    prompts: list[str],
    codex_sessions_dir: Path = DEFAULT_CODEX_SESSIONS_DIR,
) -> dict[str, dict[str, AgentAttemptResult]]:
    prompt_lookup = build_prompt_lookup(prompts)
    recovered: dict[str, dict[str, AgentAttemptResult]] = {
        prompt: {} for prompt in prompts
    }

    for path in sorted(
        codex_sessions_dir.rglob("*.jsonl"),
        key=lambda file_path: file_path.stat().st_mtime,
    ):
        prompt: str | None = None
        candidate_output: str | None = None

        for entry in _iter_jsonl(path):
            if prompt is None:
                prompt = extract_theme_prompt(json.dumps(entry, ensure_ascii=False))

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

        resolved_prompt = resolve_prompt(prompt, prompt_lookup)
        if resolved_prompt is None or candidate_output is None:
            continue

        parsed_json, errors = parse_and_validate_agent_output(candidate_output)
        if errors or parsed_json is None:
            continue

        recovered[resolved_prompt]["codex"] = AgentAttemptResult(
            agent_name="codex",
            prompt=resolved_prompt,
            success=True,
            parsed_json=parsed_json,
            errors=[],
            used_retry=False,
            attempts=1,
            exit_code=0,
        )

    return {prompt: results for prompt, results in recovered.items() if results}


def recover_results_from_logs(
    *,
    prompts: list[str],
    claude_dir: Path = DEFAULT_CLAUDE_LOG_DIR,
    codex_sessions_dir: Path = DEFAULT_CODEX_SESSIONS_DIR,
) -> dict[str, dict[str, AgentAttemptResult]]:
    merged: dict[str, dict[str, AgentAttemptResult]] = {
        prompt: {} for prompt in prompts
    }
    for prompt, agent_results in recover_codex_results(
        prompts=prompts,
        codex_sessions_dir=codex_sessions_dir,
    ).items():
        merged[prompt].update(agent_results)
    for prompt, agent_results in recover_claude_results(
        prompts=prompts,
        claude_dir=claude_dir,
    ).items():
        merged[prompt].update(agent_results)
    return merged


def extract_theme_prompt(text: str) -> str | None:
    match = THEME_PROMPT_PATTERN.search(text)
    if match is None:
        return None
    return match.group(1).strip()


def build_prompt_lookup(prompts: list[str]) -> dict[str, str]:
    lookup = {prompt: prompt for prompt in prompts}
    normalized_counts: dict[str, int] = {}
    for prompt in prompts:
        normalized_key = normalize_prompt_key(prompt)
        normalized_counts[normalized_key] = normalized_counts.get(normalized_key, 0) + 1
    for prompt in prompts:
        normalized_key = normalize_prompt_key(prompt)
        if normalized_counts[normalized_key] == 1:
            lookup[normalized_key] = prompt
    return lookup


def resolve_prompt(prompt: str | None, prompt_lookup: dict[str, str]) -> str | None:
    if prompt is None:
        return None
    if prompt in prompt_lookup:
        return prompt_lookup[prompt]
    return prompt_lookup.get(normalize_prompt_key(prompt))


def normalize_prompt_key(prompt: str) -> str:
    prefix = PROMPT_KEY_SPLIT_PATTERN.split(prompt, maxsplit=1)[0]
    normalized = re.sub(r"\s+", " ", prefix).strip().lower()
    return normalized


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


def _timestamp() -> str:
    return (
        datetime.now(timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z")
    )


def _short_prompt(prompt: str, limit: int = 72) -> str:
    if len(prompt) <= limit:
        return prompt
    return prompt[: limit - 3] + "..."


def main() -> int:
    args = parse_args()
    if args.max_concurrency < 1:
        raise ValueError("--max-concurrency must be at least 1")
    if args.recover_only and not args.recover_from_logs:
        raise ValueError("--recover-only requires --recover-from-logs")

    prompts = load_prompts(args.input)
    reporter = BatchReporter(
        prompts=prompts,
        output_path=args.output,
        attempt_log_path=args.attempt_log or default_attempt_log_path(args.output),
    )
    recovered_results = (
        recover_results_from_logs(prompts=prompts) if args.recover_from_logs else None
    )
    if args.recover_only:
        reporter.record_batch_start(prompt_count=len(prompts), total_jobs=0)
        reporter.record_recovered(recovered_results or {})
        reporter.write_snapshot(recovered_results or {prompt: {} for prompt in prompts})
        reporter.record_batch_complete(
            recovered_results or {prompt: {} for prompt in prompts}
        )
        return 0
    asyncio.run(
        run_batch(
            prompts,
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
