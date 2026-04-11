#!/usr/bin/env python3
"""Run codex and claude dreamcaller design jobs in parallel over a prompt file."""

from __future__ import annotations

import argparse
import asyncio
import json
import os
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any

DEFAULT_MAX_CONCURRENCY = 4
DEFAULT_TIMEOUT_SECONDS = 600
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
        "--max-concurrency",
        type=int,
        default=DEFAULT_MAX_CONCURRENCY,
        help="Maximum concurrent prompt/agent jobs",
    )
    parser.add_argument(
        "--timeout-seconds",
        type=int,
        default=DEFAULT_TIMEOUT_SECONDS,
        help="Per-agent timeout in seconds",
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
        "Output exactly one valid JSON object and no markdown or prose.\n\n"
        f"Theme prompt: {prompt}\n\n"
        "Skill specification:\n"
        f"{skill_text}"
    )


async def run_batch(
    prompts: list[str],
    *,
    max_concurrency: int,
    timeout_seconds: int,
    codex_bin: str,
    claude_bin: str,
) -> dict[str, dict[str, AgentAttemptResult]]:
    skill_text = load_skill_text()
    semaphore = asyncio.Semaphore(max_concurrency)
    results_by_prompt: dict[str, dict[str, AgentAttemptResult]] = {
        prompt: {} for prompt in prompts
    }

    with tempfile.TemporaryDirectory(prefix="dreamcaller-batch-") as temp_dir:
        temp_dir_path = Path(temp_dir)
        schema_path = temp_dir_path / "dreamcaller_schema.json"
        schema_path.write_text(
            json.dumps(DREAMCALLER_JSON_SCHEMA, ensure_ascii=False, indent=2),
            encoding="utf-8",
        )

        jobs = [
            AgentJob(agent_name=agent_name, prompt=prompt)
            for prompt in prompts
            for agent_name in AGENT_ORDER
        ]

        initial_results = await asyncio.gather(
            *[
                _run_job(
                    job,
                    semaphore=semaphore,
                    timeout_seconds=timeout_seconds,
                    skill_text=skill_text,
                    schema_path=schema_path,
                    temp_dir=temp_dir_path,
                    codex_bin=codex_bin,
                    claude_bin=claude_bin,
                    used_retry=False,
                )
                for job in jobs
            ]
        )

        retry_jobs: list[AgentJob] = []
        for result in initial_results:
            results_by_prompt[result.prompt][result.agent_name] = result
            if not result.success:
                retry_jobs.append(
                    AgentJob(agent_name=result.agent_name, prompt=result.prompt)
                )

        if retry_jobs:
            retry_results = await asyncio.gather(
                *[
                    _run_job(
                        job,
                        semaphore=semaphore,
                        timeout_seconds=timeout_seconds,
                        skill_text=skill_text,
                        schema_path=schema_path,
                        temp_dir=temp_dir_path,
                        codex_bin=codex_bin,
                        claude_bin=claude_bin,
                        used_retry=True,
                    )
                    for job in retry_jobs
                ]
            )
            for result in retry_results:
                result.attempts = 2
                previous_result = results_by_prompt[result.prompt][result.agent_name]
                if result.success:
                    results_by_prompt[result.prompt][result.agent_name] = result
                    continue
                previous_result.used_retry = True
                previous_result.attempts = 2
                previous_result.errors = result.errors
                previous_result.stdout = result.stdout
                previous_result.stderr = result.stderr
                previous_result.exit_code = result.exit_code

    return results_by_prompt


async def _run_job(
    job: AgentJob,
    *,
    semaphore: asyncio.Semaphore,
    timeout_seconds: int,
    skill_text: str,
    schema_path: Path,
    temp_dir: Path,
    codex_bin: str,
    claude_bin: str,
    used_retry: bool,
) -> AgentAttemptResult:
    async with semaphore:
        prompt_text = build_agent_prompt(job.prompt, skill_text)
        if job.agent_name == "codex":
            raw_text, stderr, exit_code = await _run_codex(
                prompt_text=prompt_text,
                schema_path=schema_path,
                temp_dir=temp_dir,
                timeout_seconds=timeout_seconds,
                codex_bin=codex_bin,
            )
        else:
            raw_text, stderr, exit_code = await _run_claude(
                prompt_text=prompt_text,
                timeout_seconds=timeout_seconds,
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
            used_retry=used_retry,
            stdout=raw_text,
            stderr=stderr,
            exit_code=exit_code,
        )


async def _run_codex(
    *,
    prompt_text: str,
    schema_path: Path,
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
    command = [
        codex_bin,
        "exec",
        "--dangerously-bypass-approvals-and-sandbox",
        "-C",
        str(REPO_ROOT),
        "--output-schema",
        str(schema_path),
        "-o",
        str(output_path),
        prompt_text,
    ]
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
        "--dangerously-skip-permissions",
        "--permission-mode",
        "bypassPermissions",
        "--json-schema",
        json.dumps(DREAMCALLER_JSON_SCHEMA, ensure_ascii=False),
        prompt_text,
    ]
    return await _run_command(command, timeout_seconds)


async def _run_command(
    command: list[str], timeout_seconds: int
) -> tuple[str, str, int]:
    process = await asyncio.create_subprocess_exec(
        *command,
        cwd=REPO_ROOT,
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE,
    )

    try:
        stdout_bytes, stderr_bytes = await asyncio.wait_for(
            process.communicate(), timeout=timeout_seconds
        )
    except asyncio.TimeoutError:
        process.kill()
        await process.communicate()
        return "", f"Timed out after {timeout_seconds} seconds", 124

    return (
        stdout_bytes.decode("utf-8", errors="replace"),
        stderr_bytes.decode("utf-8", errors="replace"),
        process.returncode,
    )


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

    errors = validate_dreamcaller_result(payload)
    if errors:
        return None, errors

    return payload, []


def main() -> int:
    args = parse_args()
    if args.max_concurrency < 1:
        raise ValueError("--max-concurrency must be at least 1")

    prompts = load_prompts(args.input)
    results_by_prompt = asyncio.run(
        run_batch(
            prompts,
            max_concurrency=args.max_concurrency,
            timeout_seconds=args.timeout_seconds,
            codex_bin=args.codex_bin,
            claude_bin=args.claude_bin,
        )
    )
    synthesis = synthesize_results(prompts, results_by_prompt)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(
        json.dumps(synthesis, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
