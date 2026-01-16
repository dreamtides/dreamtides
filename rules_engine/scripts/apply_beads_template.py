#!/usr/bin/env python3
"""
Apply a template to all leaf issues under a parent epic.

Leaf issues are those that have no children in the beads hierarchy.
The template file should contain {{TASK_BODY}} which will be replaced
with each issue's current description.

Usage:
    python apply_beads_template.py <parent_id> <template_file> [--dry-run]

Example:
    python scripts/apply_beads_template.py dr-epv scripts/beads_template.txt
"""

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any


def bd_command(args: list[str], check: bool = True) -> subprocess.CompletedProcess:
    """Run a bd command."""
    result = subprocess.run(
        ["bd"] + args,
        capture_output=True,
        text=True,
    )
    if check and result.returncode != 0:
        raise RuntimeError(f"bd {' '.join(args)} failed:\n{result.stderr}")
    return result


def bd_json(args: list[str]) -> Any:
    """Run a bd command and parse JSON output."""
    result = bd_command(args + ["--json"])
    return json.loads(result.stdout)


def get_issue_details(issue_id: str) -> dict:
    """Get full details for an issue including children."""
    data = bd_json(["show", issue_id])
    # bd show might return a list or single object
    if isinstance(data, list):
        return data[0] if data else {}
    return data


def get_children(issue: dict) -> list[str]:
    """
    Extract child issue IDs from an issue's dependents.

    Children are dependents with dependency_type == "parent-child".
    """
    children = []
    for dep in issue.get("dependents", []):
        if isinstance(dep, dict) and dep.get("dependency_type") == "parent-child":
            children.append(dep["id"])
    return children


def collect_leaves(root_id: str) -> list[str]:
    """
    Recursively find all leaf issues under a root.

    A leaf is an issue with no children (no parent-child dependents).
    Does not include the root itself - only its descendants.
    """
    leaves = []
    visited = set()

    def traverse(issue_id: str, is_root: bool = False):
        if issue_id in visited:
            return
        visited.add(issue_id)

        issue = get_issue_details(issue_id)
        children = get_children(issue)

        if not children and not is_root:
            # No children and not the root = leaf node
            leaves.append(issue_id)
        else:
            # Has children - traverse them
            for child_id in children:
                traverse(child_id, is_root=False)

    traverse(root_id, is_root=True)
    return leaves


def update_issue_body(issue_id: str, body: str) -> None:
    """Update an issue's description/body."""
    # Use temp file to avoid shell escaping issues
    with tempfile.NamedTemporaryFile(
        mode="w", suffix=".md", delete=False, encoding="utf-8"
    ) as f:
        f.write(body)
        temp_file = Path(f.name)

    try:
        bd_command(["update", issue_id, f"--body-file={temp_file}"])
    finally:
        temp_file.unlink(missing_ok=True)


def main():
    parser = argparse.ArgumentParser(
        description="Apply a template to all leaf issues under a parent epic."
    )
    parser.add_argument("parent_id", help="Parent issue ID (e.g., dr-epv)")
    parser.add_argument("template_file", help="Path to template file")
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show what would be done without making changes",
    )
    args = parser.parse_args()

    template_path = Path(args.template_file)
    if not template_path.exists():
        print(f"Error: Template file not found: {template_path}", file=sys.stderr)
        sys.exit(1)

    template = template_path.read_text(encoding="utf-8")

    if "{{TASK_BODY}}" not in template:
        print(
            "Warning: Template does not contain {{TASK_BODY}} placeholder",
            file=sys.stderr,
        )

    # Find leaves
    print(f"Finding leaf issues under {args.parent_id}...")
    leaves = collect_leaves(args.parent_id)

    if not leaves:
        print("No leaf issues found.")
        return

    print(f"Found {len(leaves)} leaf issue(s):")
    for leaf in leaves:
        issue = get_issue_details(leaf)
        title = issue.get("title", "")
        print(f"  {leaf}: {title}")
    print()

    if args.dry_run:
        print("[DRY RUN] Would apply template to the above issues.")
        return

    # Apply template
    success = 0
    failed = 0
    for leaf_id in leaves:
        print(f"Updating {leaf_id}...", end=" ", flush=True)
        try:
            issue = get_issue_details(leaf_id)
            current_body = issue.get("description", "")
            new_body = template.replace("{{TASK_BODY}}", current_body)
            update_issue_body(leaf_id, new_body)
            print("done")
            success += 1
        except Exception as e:
            print(f"FAILED: {e}")
            failed += 1

    print()
    print(f"Updated {success}/{len(leaves)} issues.")
    if failed > 0:
        sys.exit(1)


if __name__ == "__main__":
    main()
