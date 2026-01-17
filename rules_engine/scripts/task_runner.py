#!/usr/bin/env python3
"""
Task runner for LLMC workflow automation.

Commands:
    task_runner start              Find ready bead, assign to worker, track mapping
    task_runner start --worker X   Use specific worker
    task_runner start --immediate  Skip confirmation prompt

    task_runner review [worker]    Review pending worker (wraps llmc review)
    task_runner review opus1       Review specific worker
    task_runner review --interface vscode
    task_runner review --name-only

    task_runner accept [worker]    Accept worker (default: most recently reviewed), close bead
    task_runner accept opus1       Accept specific worker

Options:
    --verbose, -v    Enable debug logging
"""

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

LLMC_DIR = Path.home() / "llmc"
STATE_FILE = LLMC_DIR / "task_runner_state.json"
MASTER_REPO = Path.home() / "Documents" / "GoogleDrive" / "dreamtides"

VERBOSE = False


class Colors:
    GREEN = "\033[32m"
    YELLOW = "\033[33m"
    CYAN = "\033[36m"
    RED = "\033[31m"
    RESET = "\033[0m"
    BOLD = "\033[1m"


def log(msg: str) -> None:
    if VERBOSE:
        print(f"{Colors.CYAN}[DEBUG] {msg}{Colors.RESET}")


def run_command(
    args: list[str], capture: bool = True, cwd: Path | None = None
) -> subprocess.CompletedProcess:
    if capture:
        return subprocess.run(args, capture_output=True, text=True, cwd=cwd)
    else:
        return subprocess.run(args, cwd=cwd)


def load_state() -> dict:
    if not STATE_FILE.exists():
        log(f"State file does not exist: {STATE_FILE}")
        return {"worker_beads": {}, "last_reviewed_worker": None}
    try:
        with open(STATE_FILE) as f:
            data = json.load(f)
            log(f"Loaded state: {data}")
            return data
    except (json.JSONDecodeError, IOError) as e:
        log(f"Failed to load state: {e}")
        return {"worker_beads": {}, "last_reviewed_worker": None}


def save_state(state: dict) -> None:
    log(f"Saving state: {state}")
    STATE_FILE.parent.mkdir(parents=True, exist_ok=True)
    with open(STATE_FILE, "w") as f:
        json.dump(state, f, indent=2)


def find_ready_task() -> dict | None:
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
        open_tasks = [t for t in tasks if t.get("status") != "in_progress"]
        if not open_tasks:
            print("No ready tasks (all are in_progress)", file=sys.stderr)
            return None
        return open_tasks[0]
    except json.JSONDecodeError as e:
        print(f"Error parsing task list: {e}", file=sys.stderr)
        return None


def print_task_summary(task: dict) -> None:
    print(f"  ID:       {task.get('id', 'N/A')}")
    print(f"  Title:    {task.get('title', 'N/A')}")
    print(f"  Type:     {task.get('issue_type', 'N/A')}")
    print(f"  Priority: P{task.get('priority', 'N/A')}")
    print(f"  Status:   {task.get('status', 'N/A')}")


def confirm(prompt: str) -> bool:
    try:
        response = input(f"\n{Colors.YELLOW}{prompt} [y/N]: {Colors.RESET}")
        return response.lower() in ("y", "yes")
    except (KeyboardInterrupt, EOFError):
        print()
        return False


def amend_commit_with_bead(bead_id: str) -> bool:
    log(f"Amending commit with bead {bead_id}")
    result = run_command(["git", "log", "-1", "--format=%B"], cwd=MASTER_REPO)
    if result.returncode != 0:
        print(f"{Colors.RED}Failed to get commit message: {result.stderr}{Colors.RESET}", file=sys.stderr)
        return False

    original_message = result.stdout.strip()
    if bead_id in original_message:
        new_message = original_message
    else:
        new_message = f"{original_message}\n\nCloses {bead_id}"

    result = run_command(
        ["git", "commit", "--amend", "--no-edit", "-m", new_message],
        cwd=MASTER_REPO
    )
    if result.returncode != 0:
        print(f"{Colors.RED}Failed to amend commit: {result.stderr}{Colors.RESET}", file=sys.stderr)
        return False

    print(f"{Colors.GREEN}Amended commit (closes {bead_id}){Colors.RESET}")
    return True


def cmd_start(args: argparse.Namespace) -> int:
    print(f"{Colors.BOLD}Finding ready task...{Colors.RESET}")
    task = find_ready_task()
    if not task:
        return 1

    task_id = task["id"]

    print(f"{Colors.BOLD}Task:{Colors.RESET}")
    print_task_summary(task)

    if not args.immediate:
        if not confirm("Start this task?"):
            print("Cancelled.")
            return 0

    # Mark task as in_progress
    result = run_command(["bd", "update", task_id, "--status", "in_progress"])
    if result.returncode != 0:
        print(f"Warning: Failed to mark as in_progress: {result.stderr}", file=sys.stderr)

    # Start LLMC with JSON output
    cmd = ["just", "llmc", "start", "--self-review", "--json"]
    if args.worker:
        cmd.extend(["--worker", args.worker])
    else:
        cmd.extend(["--prefix", "opus"])
    cmd.extend(["--prompt-cmd", f"bd show {task_id}"])

    print(f"\n{Colors.CYAN}Running: {' '.join(cmd)}{Colors.RESET}\n")
    result = run_command(cmd, capture=True)

    if result.returncode != 0:
        print(f"{Colors.RED}llmc start failed{Colors.RESET}", file=sys.stderr)
        if result.stderr:
            print(result.stderr, file=sys.stderr)
        if result.stdout:
            print(result.stdout)
        return result.returncode

    # Parse JSON to get worker name
    try:
        data = json.loads(result.stdout)
        worker = data.get("worker")
        print(f"{Colors.GREEN}Started: {worker}{Colors.RESET}")
        print(f"  Branch: {data.get('branch', 'N/A')}")
        print(f"  Self-review: {data.get('self_review_enabled', False)}")
    except json.JSONDecodeError as e:
        print(f"{Colors.YELLOW}Warning: Could not parse output{Colors.RESET}", file=sys.stderr)
        if result.stdout:
            print(result.stdout)
        return 1

    if not worker:
        print(f"{Colors.RED}No worker in response{Colors.RESET}", file=sys.stderr)
        return 1

    # Save worker -> bead mapping
    state = load_state()
    state["worker_beads"][worker] = task_id
    save_state(state)
    print(f"{Colors.GREEN}Assigned {task_id} to {worker}{Colors.RESET}")

    return 0


def cmd_review(args: argparse.Namespace) -> int:
    # First, get worker info via JSON to track state
    json_cmd = ["just", "llmc", "review", "--json"]
    if args.worker:
        json_cmd.append(args.worker)
    result = run_command(json_cmd, capture=True)

    if result.returncode != 0:
        print(f"{Colors.RED}llmc review failed{Colors.RESET}", file=sys.stderr)
        if result.stderr:
            print(result.stderr, file=sys.stderr)
        if result.stdout:
            print(result.stdout)
        return result.returncode

    # Parse JSON to get worker and record it
    worker = None
    bead_id = None
    try:
        data = json.loads(result.stdout)
        worker = data.get("worker")
    except json.JSONDecodeError:
        print(f"{Colors.YELLOW}Warning: Could not parse review output{Colors.RESET}", file=sys.stderr)
        if result.stdout:
            print(result.stdout)

    if worker:
        # Record last reviewed worker
        state = load_state()
        state["last_reviewed_worker"] = worker
        save_state(state)
        log(f"Recorded last_reviewed_worker: {worker}")

        # Show bead assignment if known
        bead_id = state.get("worker_beads", {}).get(worker)
        if bead_id:
            print(f"{Colors.CYAN}Bead: {bead_id}{Colors.RESET}")
            print(f"To accept: task_runner accept\n")

    # Build the actual review command (without --json) for interactive diff
    cmd = ["just", "llmc", "review"]
    if args.worker:
        cmd.append(args.worker)
    if args.interface:
        cmd.extend(["--interface", args.interface])
    if args.name_only:
        cmd.append("--name-only")

    # Exec to the review command to show the diff interface
    log(f"Exec: {' '.join(cmd)}")
    os.execvp(cmd[0], cmd)


def cmd_accept(args: argparse.Namespace) -> int:
    state = load_state()
    worker = args.worker or state.get("last_reviewed_worker")

    if not worker:
        print(f"{Colors.RED}No worker specified and no recently reviewed worker. Provide a worker name or run 'task_runner review' first.{Colors.RESET}", file=sys.stderr)
        return 1

    bead_id = state.get("worker_beads", {}).get(worker)
    if not bead_id:
        print(f"{Colors.RED}Worker '{worker}' has no assigned bead.{Colors.RESET}", file=sys.stderr)
        print(f"You may need to run: just llmc accept {worker}", file=sys.stderr)
        return 1

    print(f"{Colors.BOLD}Accepting: {worker} (bead: {bead_id}){Colors.RESET}")

    # Run llmc accept
    cmd = ["just", "llmc", "accept", worker]
    print(f"{Colors.CYAN}Running: {' '.join(cmd)}{Colors.RESET}\n")
    result = run_command(cmd, capture=False)

    if result.returncode != 0:
        print(f"\n{Colors.RED}Accept failed. Bead NOT closed.{Colors.RESET}", file=sys.stderr)
        return result.returncode

    # Close the bead
    print(f"\n{Colors.BOLD}Closing bead {bead_id}...{Colors.RESET}")
    close_result = run_command(["bd", "close", bead_id])
    if close_result.returncode != 0:
        print(f"{Colors.RED}Warning: Failed to close bead: {close_result.stderr}{Colors.RESET}", file=sys.stderr)
    else:
        print(f"{Colors.GREEN}Closed {bead_id}{Colors.RESET}")

    # Amend commit with bead reference
    print(f"{Colors.BOLD}Amending commit...{Colors.RESET}")
    amend_commit_with_bead(bead_id)

    # Clear state for this worker
    if worker in state.get("worker_beads", {}):
        del state["worker_beads"][worker]
    state["last_reviewed_worker"] = None
    save_state(state)

    return 0


def main():
    global VERBOSE

    parser = argparse.ArgumentParser(
        description="Task runner for LLMC workflow automation"
    )
    parser.add_argument(
        "--verbose", "-v",
        action="store_true",
        help="Enable debug logging"
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    # start command
    start_parser = subparsers.add_parser("start", help="Start a new task")
    start_parser.add_argument(
        "--immediate", "-y",
        action="store_true",
        help="Skip confirmation"
    )
    start_parser.add_argument(
        "--worker", "-w",
        help="Specific worker (default: auto-select from opus* pool)"
    )

    # review command
    review_parser = subparsers.add_parser("review", help="Review pending work")
    review_parser.add_argument(
        "worker",
        nargs="?",
        help="Specific worker to review (default: oldest pending)"
    )
    review_parser.add_argument(
        "--interface", "-i",
        choices=["difftastic", "vscode"],
        help="Diff interface to use"
    )
    review_parser.add_argument(
        "--name-only",
        action="store_true",
        help="Show only changed file names"
    )

    # accept command
    accept_parser = subparsers.add_parser("accept", help="Accept most recently reviewed work")
    accept_parser.add_argument(
        "worker",
        nargs="?",
        help="Specific worker to accept (default: most recently reviewed)"
    )

    args = parser.parse_args()

    VERBOSE = args.verbose
    if VERBOSE:
        log(f"Arguments: {args}")

    if args.command == "start":
        sys.exit(cmd_start(args))
    elif args.command == "review":
        sys.exit(cmd_review(args))
    elif args.command == "accept":
        sys.exit(cmd_accept(args))


if __name__ == "__main__":
    main()
