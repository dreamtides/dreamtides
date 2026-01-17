#!/usr/bin/env python3
"""
Find a ready task and run it as an LLMC prompt, or accept completed work.

Usage:
    ./run_ready_task.py              # Shows task summary and asks for confirmation
    ./run_ready_task.py --immediate  # Skips confirmation and starts immediately
    ./run_ready_task.py start --worker opus1  # Use specific worker (most reliable)
    ./run_ready_task.py accept       # Accept the most recently reviewed worker
    ./run_ready_task.py accept <worker>  # Accept a specific worker's work
    ./run_ready_task.py show         # Show current bead assignments
    ./run_ready_task.py --verbose    # Enable debug logging

Options:
    --verbose, -v       Enable verbose debug logging to understand mapping issues
    --worker, -w NAME   Specific worker to use (avoids auto-detection issues)

Note: When using auto-selection (no --worker), the script may mis-identify
which worker was started if multiple workers are in the 'working' state.
Use --worker for reliable bead tracking.
"""

import argparse
import json
import subprocess
import sys
from pathlib import Path

LLMC_DIR = Path.home() / "llmc"
BEAD_ASSIGNMENTS_FILE = LLMC_DIR / "bead_assignments.json"
MASTER_REPO = Path.home() / "Documents" / "GoogleDrive" / "dreamtides"

# Global verbose flag
VERBOSE = False


def log(msg: str) -> None:
    """Print a verbose log message if verbose mode is enabled."""
    if VERBOSE:
        print(f"{Colors.CYAN}[DEBUG] {msg}{Colors.RESET}")


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
        log(f"Assignments file does not exist: {BEAD_ASSIGNMENTS_FILE}")
        return {}
    try:
        with open(BEAD_ASSIGNMENTS_FILE) as f:
            data = json.load(f)
            log(f"Loaded assignments from {BEAD_ASSIGNMENTS_FILE}: {data}")
            return data
    except (json.JSONDecodeError, IOError) as e:
        log(f"Failed to load assignments: {e}")
        return {}


def save_bead_assignments(assignments: dict[str, str]) -> None:
    """Save the bead assignments to disk."""
    log(f"Saving assignments to {BEAD_ASSIGNMENTS_FILE}: {assignments}")
    BEAD_ASSIGNMENTS_FILE.parent.mkdir(parents=True, exist_ok=True)
    with open(BEAD_ASSIGNMENTS_FILE, "w") as f:
        json.dump(assignments, f, indent=2)
    log("Assignments saved successfully")


def get_most_recently_reviewed_worker() -> str | None:
    """Get the most recently reviewed worker from llmc status."""
    log("Getting most recently reviewed worker from llmc status...")
    result = run_command(["just", "llmc", "status", "--json"])
    if result.returncode != 0:
        log(f"llmc status failed: {result.stderr}")
        return None

    try:
        data = json.loads(result.stdout)
        workers = data.get("workers", [])
        log(f"Found {len(workers)} workers in status")

        # Find workers in needs_review state, sorted by time_in_state (smallest = most recent)
        reviewing = [w for w in workers if w.get("status") in ("needs_review", "reviewing")]
        log(f"Workers in needs_review/reviewing state: {[w.get('name') for w in reviewing]}")

        if not reviewing:
            log("No workers in needs_review/reviewing state")
            return None

        # Sort by time_in_state_secs ascending (most recent first)
        reviewing.sort(key=lambda w: w.get("time_in_state_secs", float("inf")))
        for w in reviewing:
            log(f"  {w.get('name')}: {w.get('time_in_state_secs')}s in state")

        selected = reviewing[0].get("name")
        log(f"Selected most recently reviewed: {selected}")
        return selected
    except json.JSONDecodeError as e:
        log(f"Failed to parse llmc status JSON: {e}")
        return None


def get_worker_to_accept(worker: str | None) -> tuple[str, str] | None:
    """
    Get the worker and bead ID to accept.

    If worker is specified, use that worker.
    If not specified, use the most recently reviewed worker (like llmc accept).
    Returns (worker_name, bead_id) or None if not found.
    """
    log(f"get_worker_to_accept called with worker={worker}")
    assignments = load_bead_assignments()
    log(f"Current assignments: {assignments}")

    if worker:
        # Specific worker requested
        log(f"Specific worker requested: {worker}")
        if worker not in assignments:
            print(f"{Colors.RED}Worker '{worker}' has no assigned bead.{Colors.RESET}", file=sys.stderr)
            if assignments:
                print(f"Workers with assignments: {', '.join(assignments.keys())}", file=sys.stderr)
            log(f"Worker {worker} not in assignments, returning None")
            return None
        log(f"Found assignment for {worker}: {assignments[worker]}")
        return (worker, assignments[worker])

    # No worker specified - use most recently reviewed (like llmc accept)
    log("No worker specified, looking for most recently reviewed")
    most_recent = get_most_recently_reviewed_worker()
    log(f"Most recently reviewed worker: {most_recent}")

    if not most_recent:
        print(f"{Colors.RED}No workers in needs_review state.{Colors.RESET}", file=sys.stderr)
        return None

    if most_recent not in assignments:
        print(f"{Colors.RED}Worker '{most_recent}' is ready for review but has no assigned bead.{Colors.RESET}", file=sys.stderr)
        print(f"You may need to manually run: just llmc accept {most_recent}", file=sys.stderr)
        log(f"Worker {most_recent} not in assignments")
        return None

    log(f"Returning assignment for {most_recent}: {assignments[most_recent]}")
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
    log("Getting started worker from llmc status...")
    result = run_command(["just", "llmc", "status", "--json"])
    if result.returncode != 0:
        log(f"llmc status failed: {result.stderr}")
        return None

    try:
        data = json.loads(result.stdout)
        workers = data.get("workers", [])
        log(f"Found {len(workers)} workers in status")

        # Find the most recently started working worker (smallest time_in_state)
        working = [w for w in workers if w.get("status") == "working"]
        log(f"Workers in 'working' state: {[w.get('name') for w in working]}")

        if not working:
            log("No workers in 'working' state")
            return None

        # Sort by time_in_state_secs ascending (most recent first)
        working.sort(key=lambda w: w.get("time_in_state_secs", float("inf")))
        for w in working:
            log(f"  {w.get('name')}: {w.get('time_in_state_secs')}s in state")

        # Warn if multiple workers are working - mapping could be wrong
        if len(working) > 1:
            print(f"{Colors.YELLOW}Warning: Multiple workers in 'working' state. "
                  f"Selecting {working[0].get('name')} (most recently started).{Colors.RESET}",
                  file=sys.stderr)
            print(f"  Workers: {', '.join(w.get('name') for w in working)}", file=sys.stderr)

        selected = working[0].get("name")
        log(f"Selected worker: {selected}")
        return selected
    except json.JSONDecodeError as e:
        log(f"Failed to parse llmc status JSON: {e}")
        pass
    return None


def run_llmc(task_id: str, worker: str | None = None) -> tuple[int, str | None]:
    """
    Run LLMC with the task as prompt.
    Returns (return_code, worker_name).
    """
    cmd = [
        "just", "llmc", "start",
        "--self-review",
    ]

    # If specific worker requested, use --worker; otherwise use --prefix
    if worker:
        cmd.extend(["--worker", worker])
    else:
        cmd.extend(["--prefix", "opus"])

    cmd.extend(["--prompt-cmd", f"bd show {task_id}"])
    print(f"\n{Colors.CYAN}Running: {' '.join(cmd)}{Colors.RESET}\n")
    log(f"Starting llmc with task_id={task_id}, worker={worker}")
    result = run_command(cmd, capture=False)
    log(f"llmc start returned: {result.returncode}")

    # Get the worker that was assigned (only needed if not explicitly specified)
    detected_worker = None
    if result.returncode == 0:
        if worker:
            # We specified the worker, so we know it
            log(f"Worker was explicitly specified: {worker}")
            detected_worker = worker
        else:
            # Need to detect which worker was started
            detected_worker = get_started_worker()
            if detected_worker:
                log(f"Determined worker from status: {detected_worker}")
            else:
                print(f"{Colors.YELLOW}Warning: Could not determine which worker was started{Colors.RESET}", file=sys.stderr)
    else:
        log(f"llmc start failed with code {result.returncode}")

    return result.returncode, detected_worker


def amend_commit_with_bead(bead_id: str) -> bool:
    """Amend the latest commit on master to record the bead closure.

    This amend is performed AFTER `bd close` to ensure the commit reflects
    the closed bead state. We always amend even if the bead ID is already
    in the message, as the purpose is to update the commit after closing.
    """
    log(f"Amending commit with bead {bead_id}")

    # Get the current commit message
    result = run_command(["git", "log", "-1", "--format=%B"], cwd=MASTER_REPO)
    if result.returncode != 0:
        print(f"{Colors.RED}Warning: Failed to get commit message: {result.stderr}{Colors.RESET}", file=sys.stderr)
        return False

    original_message = result.stdout.strip()
    log(f"Original commit message:\n{original_message}")

    # Build the new message, adding "Closes" reference if not present
    if bead_id in original_message:
        log(f"Bead ID already in message, amending without adding duplicate")
        new_message = original_message
    else:
        new_message = f"{original_message}\n\nCloses {bead_id}"
        log(f"Adding 'Closes {bead_id}' to commit message")

    # Always amend to record the closed state (even if message unchanged,
    # this updates the commit timestamp and ensures consistency)
    result = run_command(
        ["git", "commit", "--amend", "--no-edit", "-m", new_message],
        cwd=MASTER_REPO
    )
    if result.returncode != 0:
        print(f"{Colors.RED}Warning: Failed to amend commit: {result.stderr}{Colors.RESET}", file=sys.stderr)
        log(f"Amend stderr: {result.stderr}")
        return False

    print(f"{Colors.GREEN}Amended commit (bead {bead_id} closed){Colors.RESET}")
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
    # Use explicit worker if provided, otherwise auto-detect
    explicit_worker = getattr(args, 'worker', None)
    returncode, detected_worker = run_llmc(task_id, worker=explicit_worker)

    # If we specified an explicit worker, use that; otherwise use detected
    worker = explicit_worker if explicit_worker else detected_worker
    log(f"Final worker for assignment: {worker} (explicit={explicit_worker}, detected={detected_worker})")

    # Step 6: Save the bead assignment if successful
    if returncode == 0 and worker:
        log(f"llmc start succeeded, saving assignment: {worker} -> {task_id}")
        assignments = load_bead_assignments()
        assignments[worker] = task_id
        save_bead_assignments(assignments)
        print(f"{Colors.GREEN}Assigned bead {task_id} to worker {worker}{Colors.RESET}")
    elif returncode == 0:
        print(f"{Colors.YELLOW}Warning: llmc start succeeded but could not determine worker name{Colors.RESET}", file=sys.stderr)
        print(f"The bead {task_id} was NOT assigned to any worker.", file=sys.stderr)
        print(f"You may need to manually track this assignment.", file=sys.stderr)
    else:
        log(f"llmc start failed with code {returncode}, not saving assignment")

    return returncode


def cmd_show(args: argparse.Namespace) -> int:
    """Show current bead assignments."""
    assignments = load_bead_assignments()

    if not assignments:
        print("No bead assignments.")
        return 0

    print(f"{Colors.BOLD}Current bead assignments:{Colors.RESET}")
    for worker, bead_id in sorted(assignments.items()):
        print(f"  {worker}: {bead_id}")

    return 0


def cmd_accept(args: argparse.Namespace) -> int:
    """Handle the accept command."""
    log(f"cmd_accept called with worker={args.worker}")

    assignment = get_worker_to_accept(args.worker)
    if not assignment:
        log("No assignment found, returning 1")
        return 1

    worker, bead_id = assignment
    log(f"Found assignment: worker={worker}, bead_id={bead_id}")

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
    global VERBOSE

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
    start_parser.add_argument(
        "--worker", "-w",
        default=None,
        help="Specific worker to use (default: auto-select from opus* pool)"
    )

    # Accept command
    accept_parser = subparsers.add_parser("accept", help="Accept a worker's completed work")
    accept_parser.add_argument(
        "worker",
        nargs="?",
        default=None,
        help="Worker name (optional, defaults to most recently reviewed)"
    )

    # Show command
    subparsers.add_parser("show", help="Show current bead assignments")

    # Global options
    parser.add_argument(
        "--immediate",
        action="store_true",
        help="Skip confirmation and start immediately (for start command)"
    )
    parser.add_argument(
        "--verbose", "-v",
        action="store_true",
        help="Enable verbose debug logging"
    )

    args = parser.parse_args()

    # Set global verbose flag
    VERBOSE = args.verbose
    if VERBOSE:
        print(f"{Colors.CYAN}[DEBUG] Verbose logging enabled{Colors.RESET}")
        log(f"Arguments: {args}")

    if args.command == "accept":
        sys.exit(cmd_accept(args))
    elif args.command == "show":
        sys.exit(cmd_show(args))
    else:
        # Default to start behavior (with or without explicit "start" command)
        sys.exit(cmd_start(args))


if __name__ == "__main__":
    main()
