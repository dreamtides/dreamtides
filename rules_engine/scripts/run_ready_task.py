#!/usr/bin/env python3
"""
Find a ready task and run it as an LLMC prompt.

Usage:
    ./run_ready_task.py              # Shows task summary and asks for confirmation
    ./run_ready_task.py --immediate  # Skips confirmation and starts immediately
"""

import argparse
import json
import subprocess
import sys


class Colors:
    GREEN = "\033[32m"
    YELLOW = "\033[33m"
    CYAN = "\033[36m"
    RESET = "\033[0m"
    BOLD = "\033[1m"


def run_command(args: list[str], capture: bool = True) -> subprocess.CompletedProcess:
    """Run a command and return the result."""
    if capture:
        return subprocess.run(args, capture_output=True, text=True)
    else:
        return subprocess.run(args)


def find_ready_task() -> dict | None:
    """Find a ready task under dr-epv that is not already in_progress."""
    result = run_command(["bd", "ready", "--parent=dr-epv", "--type=task", "--json"])
    if result.returncode != 0:
        print(f"Error finding ready task: {result.stderr}", file=sys.stderr)
        return None

    output = result.stdout.strip()
    if not output:
        print("No ready tasks found under dr-epv", file=sys.stderr)
        return None

    try:
        tasks = json.loads(output)
        # Filter out tasks that are already in_progress
        open_tasks = [t for t in tasks if t.get("status") != "in_progress"]
        if not open_tasks:
            print("No ready tasks found under dr-epv (all ready tasks are in_progress)", file=sys.stderr)
            return None
        return open_tasks[0]
    except json.JSONDecodeError as e:
        print(f"Error parsing task list: {e}", file=sys.stderr)
        return None


def print_task_summary(task: dict) -> None:
    """Print a brief 5-line summary of the task."""
    print(f"  ID:       {task.get('id', 'N/A')}")
    print(f"  Title:    {task.get('title', 'N/A')}")
    print(f"  Type:     {task.get('issue_type', 'N/A')}")
    print(f"  Priority: P{task.get('priority', 'N/A')}")
    print(f"  Status:   {task.get('status', 'N/A')}")


def confirm_start() -> bool:
    """Ask user for confirmation."""
    try:
        response = input(f"\n{Colors.YELLOW}Start LLMC with this task? [y/N]: {Colors.RESET}")
        return response.lower() in ("y", "yes")
    except (KeyboardInterrupt, EOFError):
        print()
        return False


def run_llmc(task_id: str) -> int:
    """Run LLMC with the task as prompt."""
    cmd = [
        "just", "llmc", "start",
        "--self-review",
        "--prefix", "opus",
        "--prompt-cmd", f"bd show {task_id}"
    ]
    print(f"\n{Colors.CYAN}Running: {' '.join(cmd)}{Colors.RESET}\n")
    result = run_command(cmd, capture=False)
    return result.returncode


def main():
    parser = argparse.ArgumentParser(description="Find a ready task and run it as an LLMC prompt")
    parser.add_argument(
        "--immediate",
        action="store_true",
        help="Skip confirmation and start immediately"
    )
    args = parser.parse_args()

    # Step 1: Find a ready task
    print(f"{Colors.BOLD}Finding ready task under dr-epv...{Colors.RESET}")
    task = find_ready_task()
    if not task:
        sys.exit(1)

    task_id = task["id"]

    # Step 2: Show brief task summary
    print(f"{Colors.BOLD}Task Summary:{Colors.RESET}")
    print_task_summary(task)

    # Step 3: Confirm or start immediately
    if not args.immediate:
        if not confirm_start():
            print("Cancelled.")
            sys.exit(0)

    # Step 4: Mark task as in_progress so it no longer appears in bd ready
    result = run_command(["bd", "update", task_id, "--status", "in_progress"])
    if result.returncode != 0:
        print(f"Warning: Failed to mark task as in_progress: {result.stderr}", file=sys.stderr)

    # Step 5: Run LLMC
    sys.exit(run_llmc(task_id))


if __name__ == "__main__":
    main()
