#!/usr/bin/env python3
"""
Display a dependency tree with ready/blocked status coloring.

A task is considered "ready" if its only open dependencies are ancestor epics
(parent containers), not sibling tasks at the same depth level.

Usage:
    ./bd_ready_tree.py <issue-id>
    ./bd_ready_tree.py dr-ymm
"""

import json
import subprocess
import sys
from dataclasses import dataclass


class Colors:
    GREEN = "\033[32m"
    YELLOW = "\033[33m"
    RED = "\033[31m"
    GRAY = "\033[90m"
    RESET = "\033[0m"
    BOLD = "\033[1m"


@dataclass
class Issue:
    id: str
    title: str
    status: str
    priority: int
    issue_type: str
    depth: int
    dependencies: list  # IDs of issues this depends on


def run_bd_command(args: list[str]) -> str:
    result = subprocess.run(
        ["bd"] + args,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print(f"Error running bd {' '.join(args)}: {result.stderr}", file=sys.stderr)
        sys.exit(1)
    return result.stdout


def get_tree_issues(root_id: str) -> dict[str, Issue]:
    """Get all issues in the dependency tree."""
    output = run_bd_command(["dep", "tree", root_id, "--direction=up", "--json"])
    data = json.loads(output)

    issues = {}
    for item in data:
        # Skip tombstones (deleted issues)
        if item.get("status") == "tombstone":
            continue
        issues[item["id"]] = Issue(
            id=item["id"],
            title=item["title"],
            status=item["status"],
            priority=item.get("priority", 2),
            issue_type=item.get("issue_type", "task"),
            depth=item.get("depth", 0),
            dependencies=[],
        )
    return issues


def get_dependencies(issue_id: str) -> list[dict]:
    """Get dependencies for a single issue."""
    output = run_bd_command(["dep", "list", issue_id, "--json"])
    if not output.strip():
        return []
    return json.loads(output)


def populate_dependencies(issues: dict[str, Issue]) -> None:
    """Populate dependency lists for all issues."""
    for issue_id, issue in issues.items():
        deps = get_dependencies(issue_id)
        issue.dependencies = [d["id"] for d in deps]


def is_ready(issue: Issue, issues: dict[str, Issue]) -> bool:
    """
    Determine if an issue is ready for work.

    An issue is ready if:
    - It's already closed, OR
    - All its open dependencies are at a lower depth (ancestors/parents)
    """
    if issue.status == "closed":
        return True

    for dep_id in issue.dependencies:
        if dep_id not in issues:
            continue
        dep = issues[dep_id]
        if dep.status == "closed":
            continue
        # If dependency is at same or higher depth, it's a real blocker
        if dep.depth >= issue.depth:
            return False

    return True


def build_tree_structure(issues: dict[str, Issue]) -> dict[str, list[str]]:
    """Build parent->children mapping from dependencies."""
    children: dict[str, list[str]] = {id: [] for id in issues}

    for issue_id, issue in issues.items():
        for dep_id in issue.dependencies:
            if dep_id in issues:
                dep = issues[dep_id]
                # If dependency is at lower depth, it's the parent
                if dep.depth < issue.depth:
                    if issue_id not in children[dep_id]:
                        children[dep_id].append(issue_id)

    return children


def get_status_symbol(issue: Issue, ready: bool) -> str:
    """Get status symbol with color."""
    if issue.status == "closed":
        return f"{Colors.GREEN}✓{Colors.RESET}"
    elif issue.status == "in_progress":
        return f"{Colors.YELLOW}◐{Colors.RESET}"
    elif ready:
        return f"{Colors.GREEN}○{Colors.RESET}"
    else:
        return f"{Colors.RED}●{Colors.RESET}"


def get_ready_tag(issue: Issue, ready: bool) -> str:
    """Get [READY] or [BLOCKED] tag with color."""
    if issue.status == "closed":
        return ""
    elif ready:
        return f" {Colors.GREEN}[READY]{Colors.RESET}"
    else:
        return f" {Colors.RED}[BLOCKED]{Colors.RESET}"


def truncate(s: str, max_len: int = 50) -> str:
    """Truncate string with ellipsis."""
    if len(s) <= max_len:
        return s
    return s[:max_len-1] + "…"


def print_tree(
    issues: dict[str, Issue],
    children: dict[str, list[str]],
    ready_status: dict[str, bool],
    root_id: str,
    prefix: str = "",
    is_last: bool = True,
) -> None:
    """Print tree recursively with box-drawing characters."""
    issue = issues[root_id]
    ready = ready_status[root_id]

    # Determine connector
    if prefix == "":
        connector = ""
    elif is_last:
        connector = "└── "
    else:
        connector = "├── "

    # Build the line
    symbol = get_status_symbol(issue, ready)
    tag = get_ready_tag(issue, ready)
    title = truncate(issue.title)
    priority = f"P{issue.priority}"

    print(f"{prefix}{connector}{symbol} {issue.id}: {title} [{priority}] ({issue.status}){tag}")

    # Print children
    child_ids = children.get(root_id, [])
    # Sort children: ready first, then by ID
    child_ids = sorted(child_ids, key=lambda x: (not ready_status[x], x))

    for i, child_id in enumerate(child_ids):
        is_last_child = (i == len(child_ids) - 1)
        if prefix == "":
            new_prefix = "    "
        elif is_last:
            new_prefix = prefix + "    "
        else:
            new_prefix = prefix + "│   "
        print_tree(issues, children, ready_status, child_id, new_prefix, is_last_child)


def main():
    if len(sys.argv) < 2:
        print("Usage: bd_ready_tree.py <issue-id>", file=sys.stderr)
        sys.exit(1)

    root_id = sys.argv[1]

    # Get all issues in tree
    issues = get_tree_issues(root_id)
    if not issues:
        print(f"No issues found for {root_id}", file=sys.stderr)
        sys.exit(1)

    # Get dependencies for each issue
    populate_dependencies(issues)

    # Calculate ready status for each issue
    ready_status = {id: is_ready(issue, issues) for id, issue in issues.items()}

    # Build tree structure
    children = build_tree_structure(issues)

    # Print legend
    print(f"🌲 Ready tree for {root_id}:\n")
    print(f"  Status: {Colors.GREEN}○{Colors.RESET} ready  {Colors.YELLOW}◐{Colors.RESET} in_progress  {Colors.RED}●{Colors.RESET} blocked  {Colors.GREEN}✓{Colors.RESET} closed\n")

    # Print tree
    print_tree(issues, children, ready_status, root_id)

    # Print summary
    ready_count = sum(1 for id, r in ready_status.items() if r and issues[id].status not in ("closed", "in_progress"))
    blocked_count = sum(1 for id, r in ready_status.items() if not r and issues[id].status not in ("closed", "in_progress"))
    print(f"\n  Ready: {ready_count}, Blocked: {blocked_count}, Total: {len(issues)}")


if __name__ == "__main__":
    main()
