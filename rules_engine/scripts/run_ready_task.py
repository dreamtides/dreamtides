#!/usr/bin/env python3
"""
Find a ready task and run it as an LLMC prompt, or accept completed work.

Usage:
    ./run_ready_task.py              # Shows task summary and asks for confirmation
    ./run_ready_task.py --immediate  # Skips confirmation and starts immediately
    ./run_ready_task.py accept       # Accept the most recently reviewed worker
    ./run_ready_task.py accept <worker>  # Accept a specific worker's work
"""

import argparse
import json
import subprocess
import sys
from pathlib import Path

LLMC_DIR = Path.home() / "llmc"
BEAD_ASSIGNMENTS_FILE = LLMC_DIR / "bead_assignments.json"
MASTER_REPO = Path.home() / "Documents" / "GoogleDrive" / "dreamtides"


class Colors:
    GREEN = "\033[32m"
    YELLOW = "\033[33m"
    CYAN = "\033[36m"
    RED = "\033[31m"
    RESET = "\033[0m"
    BOLD = "\033[1m"


def run_command(
    args: list[str], capture: bool = True, cwd: Path | None = None
) -> subprocess.CompletedProcess:
    """Run a command and return the result."""
    if capture:
        return subprocess.run(args, capture_output=True, text=True, cwd=cwd)
    else:
        return subprocess.run(args, cwd=cwd)


def load_bead_assignments() -> dict[str, str]:
    """Load the bead assignments from disk."""
    if not BEAD_ASSIGNMENTS_FILE.exists():
        return {}
    try:
        with open(BEAD_ASSIGNMENTS_FILE) as f:
            return json.load(f)
    except (json.JSONDecodeError, IOError):
        return {}


def save_bead_assignments(assignments: dict[str, str]) -> None:
    """Save the bead assignments to disk."""
    BEAD_ASSIGNMENTS_FILE.parent.mkdir(parents=True, exist_ok=True)
    with open(BEAD_ASSIGNMENTS_FILE, "w") as f:
        json.dump(assignments, f, indent=2)


def get_most_recently_reviewed_worker() -> str | None:
    """Get the most recently reviewed worker from llmc status."""
    result = run_command(["just", "llmc", "status", "--json"])
    if result.returncode != 0:
        return None

    try:
        data = json.loads(result.stdout)
        workers = data.get("workers", [])
        # Find workers in needs_review state, sorted by time_in_state (smallest = most recent)
        reviewing = [w for w in workers if w.get("status") in ("needs_review", "reviewing")]
        if not reviewing:
            return None
        # Sort by time_in_state_secs ascending (most recent first)
        reviewing.sort(key=lambda w: w.get("time_in_state_secs", float("inf")))
        return reviewing[0].get("name")
    except json.JSONDecodeError:
        return None


def get_worker_to_accept(worker: str | None) -> tuple[str, str] | None:
    """
    Get the worker and bead ID to accept.

    If worker is specified, use that worker.
    If not specified, use the most recently reviewed worker (like llmc accept).
    Returns (worker_name, bead_id) or None if not found.
    """
    assignments = load_bead_assignments()

    if worker:
        # Specific worker requested
        if worker not in assignments:
            print(f"{Colors.RED}Worker '{worker}' has no assigned bead.{Colors.RESET}", file=sys.stderr)
            if assignments:
                print(f"Workers with assignments: {', '.join(assignments.keys())}", file=sys.stderr)
            return None
        return (worker, assignments[worker])

    # No worker specified - use most recently reviewed (like llmc accept)
    most_recent = get_most_recently_reviewed_worker()
    if not most_recent:
        print(f"{Colors.RED}No workers in needs_review state.{Colors.RESET}", file=sys.stderr)
        return None

    if most_recent not in assignments:
        print(f"{Colors.RED}Worker '{most_recent}' is ready for review but has no assigned bead.{Colors.RESET}", file=sys.stderr)
        print(f"You may need to manually run: just llmc accept {most_recent}", file=sys.stderr)
        return None

    return (most_recent, assignments[most_recent])


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


def get_started_worker() -> str | None:
    """Get the worker that was started by parsing llmc start output."""
    # We'll capture the worker name from llmc status after starting
    result = run_command(["just", "llmc", "status", "--json"])
    if result.returncode != 0:
        return None

    try:
        data = json.loads(result.stdout)
        workers = data.get("workers", [])
        # Find the most recently started working worker (smallest time_in_state)
        working = [w for w in workers if w.get("status") == "working"]
        if not working:
            return None
        # Sort by time_in_state_secs ascending (most recent first)
        working.sort(key=lambda w: w.get("time_in_state_secs", float("inf")))
        return working[0].get("name")
    except json.JSONDecodeError:
        pass
    return None


def run_llmc(task_id: str) -> tuple[int, str | None]:
    """
    Run LLMC with the task as prompt.
    Returns (return_code, worker_name).
    """
    cmd = [
        "just", "llmc", "start",
        "--self-review",
        "--prefix", "opus",
        "--prompt-cmd", f"bd show {task_id}"
    ]
    print(f"\n{Colors.CYAN}Running: {' '.join(cmd)}{Colors.RESET}\n")
    result = run_command(cmd, capture=False)

    # Get the worker that was assigned
    worker = None
    if result.returncode == 0:
        worker = get_started_worker()

    return result.returncode, worker


def amend_commit_with_bead(bead_id: str) -> bool:
    """Amend the latest commit on master to record the bead closure."""
    # Get the current commit message
    result = run_command(["git", "log", "-1", "--format=%B"], cwd=MASTER_REPO)
    if result.returncode != 0:
        print(f"{Colors.RED}Warning: Failed to get commit message: {result.stderr}{Colors.RESET}", file=sys.stderr)
        return False

    original_message = result.stdout.strip()

    # Check if bead ID is already in the message
    if bead_id in original_message:
        print(f"{Colors.YELLOW}Bead ID already in commit message, skipping amend.{Colors.RESET}")
        return True

    # Append the bead closure note
    new_message = f"{original_message}\n\nCloses {bead_id}"

    # Amend the commit
    result = run_command(
        ["git", "commit", "--amend", "-m", new_message],
        cwd=MASTER_REPO
    )
    if result.returncode != 0:
        print(f"{Colors.RED}Warning: Failed to amend commit: {result.stderr}{Colors.RESET}", file=sys.stderr)
        return False

    print(f"{Colors.GREEN}Amended commit with 'Closes {bead_id}'{Colors.RESET}")
    return True


def cmd_start(args: argparse.Namespace) -> int:
    """Handle the start command (default behavior)."""
    # Step 1: Find a ready task
    print(f"{Colors.BOLD}Finding ready task under dr-epv...{Colors.RESET}")
    task = find_ready_task()
    if not task:
        return 1

    task_id = task["id"]

    # Step 2: Show brief task summary
    print(f"{Colors.BOLD}Task Summary:{Colors.RESET}")
    print_task_summary(task)

    # Step 3: Confirm or start immediately
    if not args.immediate:
        if not confirm_start():
            print("Cancelled.")
            return 0

    # Step 4: Mark task as in_progress so it no longer appears in bd ready
    result = run_command(["bd", "update", task_id, "--status", "in_progress"])
    if result.returncode != 0:
        print(f"Warning: Failed to mark task as in_progress: {result.stderr}", file=sys.stderr)

    # Step 5: Run LLMC
    returncode, worker = run_llmc(task_id)

    # Step 6: Save the bead assignment if successful
    if returncode == 0 and worker:
        assignments = load_bead_assignments()
        assignments[worker] = task_id
        save_bead_assignments(assignments)
        print(f"{Colors.GREEN}Assigned bead {task_id} to worker {worker}{Colors.RESET}")

    return returncode


def cmd_accept(args: argparse.Namespace) -> int:
    """Handle the accept command."""
    assignment = get_worker_to_accept(args.worker)
    if not assignment:
        return 1

    worker, bead_id = assignment

    print(f"{Colors.BOLD}Accepting work from {worker} (bead: {bead_id})...{Colors.RESET}")

    # Run llmc accept
    cmd = ["just", "llmc", "accept", worker]
    print(f"{Colors.CYAN}Running: {' '.join(cmd)}{Colors.RESET}\n")
    result = run_command(cmd, capture=False)

    if result.returncode != 0:
        print(f"\n{Colors.RED}Accept failed. Bead {bead_id} NOT closed.{Colors.RESET}", file=sys.stderr)
        return result.returncode

    # Accept succeeded - close the bead
    print(f"\n{Colors.BOLD}Closing bead {bead_id}...{Colors.RESET}")
    close_result = run_command(["bd", "close", bead_id])
    if close_result.returncode != 0:
        print(f"{Colors.RED}Warning: Failed to close bead: {close_result.stderr}{Colors.RESET}", file=sys.stderr)
    else:
        print(f"{Colors.GREEN}Closed bead {bead_id}{Colors.RESET}")

    # Amend the commit to record the bead closure
    print(f"{Colors.BOLD}Amending commit...{Colors.RESET}")
    amend_commit_with_bead(bead_id)

    # Remove the assignment
    assignments = load_bead_assignments()
    if worker in assignments:
        del assignments[worker]
        save_bead_assignments(assignments)

    return 0


def main():
    parser = argparse.ArgumentParser(
        description="Find a ready task and run it as an LLMC prompt, or accept completed work"
    )
    subparsers = parser.add_subparsers(dest="command")

    # Start command (implicit default)
    start_parser = subparsers.add_parser("start", help="Start a new task (default)")
    start_parser.add_argument(
        "--immediate",
        action="store_true",
        help="Skip confirmation and start immediately"
    )

    # Accept command
    accept_parser = subparsers.add_parser("accept", help="Accept a worker's completed work")
    accept_parser.add_argument(
        "worker",
        nargs="?",
        default=None,
        help="Worker name (optional, defaults to most recently reviewed)"
    )

    # Also support --immediate at the top level for backwards compatibility
    parser.add_argument(
        "--immediate",
        action="store_true",
        help="Skip confirmation and start immediately (for start command)"
    )

    args = parser.parse_args()

    if args.command == "accept":
        sys.exit(cmd_accept(args))
    else:
        # Default to start behavior (with or without explicit "start" command)
        sys.exit(cmd_start(args))


if __name__ == "__main__":
    main()
