#!/usr/bin/env python3
"""
Apply a template to all leaf issues under a parent epic.

Leaf issues are those that have no children in the beads hierarchy.
The template file should contain {{TASK_BODY}} which will be replaced
with each issue's current description.

The template uses markers to support idempotent updates:
- <!-- BEADS_TEMPLATE_START --> / <!-- BEADS_TEMPLATE_END --> wrap the entire template
- <!-- BEADS_TASK_BODY_START --> / <!-- BEADS_TASK_BODY_END --> wrap the task body

When re-running on an already-templated issue, the script extracts the preserved
task body and applies the updated template. Issues are skipped if unchanged.

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

# Markers for idempotent template application
TEMPLATE_START = "<!-- BEADS_TEMPLATE_START -->"
TEMPLATE_END = "<!-- BEADS_TEMPLATE_END -->"
TASK_BODY_START = "<!-- BEADS_TASK_BODY_START -->"
TASK_BODY_END = "<!-- BEADS_TASK_BODY_END -->"


def extract_task_body(content: str) -> str | None:
    """Extract the task body from an already-templated issue."""
    if TASK_BODY_START not in content or TASK_BODY_END not in content:
        return None

    start_idx = content.find(TASK_BODY_START)
    end_idx = content.find(TASK_BODY_END)

    if start_idx == -1 or end_idx == -1 or start_idx >= end_idx:
        return None

    body = content[start_idx + len(TASK_BODY_START) : end_idx]
    if body.startswith("\n"):
        body = body[1:]
    if body.endswith("\n"):
        body = body[:-1]
    return body


def apply_template(template: str, task_body: str) -> str:
    """Apply template to a task body, wrapping with appropriate markers."""
    marked_body = f"{TASK_BODY_START}\n{task_body}\n{TASK_BODY_END}"
    content = template.replace("{{TASK_BODY}}", marked_body)
    return f"{TEMPLATE_START}\n{content}\n{TEMPLATE_END}"


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

    # Apply template
    updated = 0
    skipped = 0
    failed = 0
    for leaf_id in leaves:
        print(f"Processing {leaf_id}...", end=" ", flush=True)
        try:
            issue = get_issue_details(leaf_id)
            current_body = issue.get("description", "")

            # Check if already templated - extract preserved task body
            extracted = extract_task_body(current_body)
            task_body = extracted if extracted is not None else current_body

            # Apply template with markers
            new_body = apply_template(template, task_body)

            # Skip if unchanged
            if new_body == current_body:
                print("unchanged")
                skipped += 1
                continue

            if args.dry_run:
                print("would update")
                updated += 1
                continue

            update_issue_body(leaf_id, new_body)
            print("updated")
            updated += 1
        except Exception as e:
            print(f"FAILED: {e}")
            failed += 1

    print()
    if args.dry_run:
        print(f"[DRY RUN] Would update {updated}, skip {skipped} unchanged.")
    else:
        print(f"Updated {updated}, skipped {skipped} unchanged, {failed} failed.")
    if failed > 0:
        sys.exit(1)


if __name__ == "__main__":
    main()
