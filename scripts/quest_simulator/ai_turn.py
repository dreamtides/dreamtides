#!/usr/bin/env python3
"""Helper script for AI sub-agent interaction with the quest simulator.

Provides a simple CLI for submitting choices and reading prompts one turn
at a time via the file-based AI turn protocol.

Usage:
    python3 scripts/quest_simulator/ai_turn.py --start --seed 42
    python3 scripts/quest_simulator/ai_turn.py --choice 2
    python3 scripts/quest_simulator/ai_turn.py --choices 0,2,3
    python3 scripts/quest_simulator/ai_turn.py --confirm
    python3 scripts/quest_simulator/ai_turn.py --decline
"""

import argparse
import json
import os
import signal
import subprocess
import sys
import time
from pathlib import Path

_PROMPT_PATH = Path(".logs/quest_ai_prompt.json")
_RESPONSE_PATH = Path(".logs/quest_ai_response.json")
_PID_PATH = Path(".logs/quest_ai_pid")
_POLL_INTERVAL = 0.3
_TIMEOUT = 300  # 5 minutes


def _is_process_alive(pid: int) -> bool:
    """Check if a process with the given PID is still running."""
    try:
        os.kill(pid, 0)
        return True
    except OSError:
        return False


def _read_pid() -> int | None:
    """Read the simulator PID from the PID file."""
    try:
        return int(_PID_PATH.read_text().strip())
    except (OSError, ValueError):
        return None


def _wait_for_prompt_or_exit(min_prompt_id: int = 0) -> dict | None:
    """Wait for a prompt file with id > min_prompt_id, or simulator exit.

    Returns the prompt dict, or None if the simulator exited.
    """
    pid = _read_pid()
    deadline = time.monotonic() + _TIMEOUT
    while time.monotonic() < deadline:
        if _PROMPT_PATH.exists():
            try:
                data = json.loads(_PROMPT_PATH.read_text())
                if data.get("prompt_id", 0) > min_prompt_id:
                    return data
            except (json.JSONDecodeError, OSError):
                pass
        if pid is not None and not _is_process_alive(pid):
            time.sleep(0.5)
            if _PROMPT_PATH.exists():
                try:
                    data = json.loads(_PROMPT_PATH.read_text())
                    if data.get("prompt_id", 0) > min_prompt_id:
                        return data
                except (json.JSONDecodeError, OSError):
                    pass
            return None
        time.sleep(_POLL_INTERVAL)
    print("Error: Timed out waiting for prompt", file=sys.stderr)
    sys.exit(2)


def _print_prompt(prompt: dict) -> None:
    """Print a prompt in a clear plain-text format."""
    context = prompt.get("context", "").strip()
    if context:
        print(context)
        print()
    prompt_type = prompt.get("type", "unknown")
    options = prompt.get("options", [])
    max_sel = prompt.get("max_selections")
    prompt_id = prompt.get("prompt_id", "?")

    print(f"--- Prompt #{prompt_id} ({prompt_type}) ---")
    if prompt_type == "confirm_decline":
        print(f"  Accept: {options[0] if options else 'Accept'}")
        print(f"  Decline: {options[1] if len(options) > 1 else 'Decline'}")
        print("Use --confirm or --decline to respond.")
    else:
        for i, opt in enumerate(options):
            print(f"  [{i}] {opt}")
        if prompt_type == "multi_select" and max_sel is not None:
            print(f"  (select up to {max_sel})")
        if prompt_type == "single_select":
            print("Use --choice N to respond.")
        else:
            print("Use --choices N,N,N to respond.")


def _write_response(prompt_id: int, choice: object) -> None:
    """Write a response file for the given prompt."""
    _RESPONSE_PATH.parent.mkdir(parents=True, exist_ok=True)
    response = {"prompt_id": prompt_id, "choice": choice}
    tmp = _RESPONSE_PATH.with_suffix(".tmp")
    tmp.write_text(json.dumps(response))
    tmp.rename(_RESPONSE_PATH)


def cmd_start(args: argparse.Namespace) -> None:
    """Start the quest simulator in AI mode."""
    # Clean up stale files
    for p in (_PROMPT_PATH, _RESPONSE_PATH, _PID_PATH):
        try:
            p.unlink()
        except OSError:
            pass

    sim_args = [
        sys.executable,
        "scripts/quest_simulator/quest_sim.py",
        "--ai",
    ]
    if args.seed is not None:
        sim_args += ["--seed", str(args.seed)]

    proc = subprocess.Popen(
        sim_args,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    _PID_PATH.parent.mkdir(parents=True, exist_ok=True)
    _PID_PATH.write_text(str(proc.pid))

    prompt = _wait_for_prompt_or_exit()
    if prompt is None:
        print("Simulator exited before producing a prompt.", file=sys.stderr)
        sys.exit(1)
    _print_prompt(prompt)


def cmd_respond(choice: object) -> None:
    """Submit a response and wait for the next prompt."""
    if not _PROMPT_PATH.exists():
        print("Error: No pending prompt found.", file=sys.stderr)
        sys.exit(2)

    try:
        current = json.loads(_PROMPT_PATH.read_text())
    except (json.JSONDecodeError, OSError) as e:
        print(f"Error reading prompt: {e}", file=sys.stderr)
        sys.exit(2)

    prompt_id = current.get("prompt_id", 0)
    _write_response(prompt_id, choice)

    prompt = _wait_for_prompt_or_exit(min_prompt_id=prompt_id)
    if prompt is None:
        # Game ended — print any remaining output
        print("--- Game Over ---")
        sys.exit(1)
    _print_prompt(prompt)


def main() -> None:
    """Parse arguments and dispatch."""
    parser = argparse.ArgumentParser(description="AI turn helper for quest simulator")
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--start", action="store_true", help="Start a new game")
    group.add_argument("--choice", type=int, help="Submit a single-select choice")
    group.add_argument(
        "--choices", type=str, help="Submit multi-select choices (comma-separated)"
    )
    group.add_argument("--confirm", action="store_true", help="Submit confirm")
    group.add_argument("--decline", action="store_true", help="Submit decline")
    parser.add_argument(
        "--seed", type=int, default=None, help="Random seed (with --start)"
    )

    args = parser.parse_args()

    if args.start:
        cmd_start(args)
    elif args.choice is not None:
        cmd_respond(args.choice)
    elif args.choices is not None:
        indices = [int(x.strip()) for x in args.choices.split(",")]
        cmd_respond(indices)
    elif args.confirm:
        cmd_respond(True)
    elif args.decline:
        cmd_respond(False)


if __name__ == "__main__":
    main()
