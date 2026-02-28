#!/usr/bin/env python3
"""
Checks for pending insta snapshot files (.pending-snap) in the repository.
Returns exit code 1 if any are found, 0 if none are found.

These files indicate that snapshot tests have produced new output that needs
to be reviewed with 'cargo insta review' before committing.
"""

import sys
from pathlib import Path
from typing import List, Optional


def find_project_root(start_path: Path) -> Optional[Path]:
    """Find the project root by looking for a justfile."""
    current = start_path.resolve()
    while current != current.parent:
        if (current / "justfile").exists():
            return current
        current = current.parent
    return None


def find_pending_snapshots(root: Path) -> List[Path]:
    """Find all .pending-snap files in the repository."""
    # Search in the entire repository
    pending_snapshots = list(root.rglob("*.pending-snap"))
    return pending_snapshots


def main():
    project_root = find_project_root(Path(__file__).parent)
    if not project_root:
        print("Error: Could not find project root (no justfile found)", file=sys.stderr)
        sys.exit(1)

    pending_snapshots = find_pending_snapshots(project_root)

    if pending_snapshots:
        print("❌ Found pending snapshot files:", file=sys.stderr)
        for snapshot in sorted(pending_snapshots):
            # Print path relative to project root
            rel_path = snapshot.relative_to(project_root)
            print(f"  - {rel_path}", file=sys.stderr)
        print(
            "\nPlease review and accept/reject snapshots with 'cargo insta review'",
            file=sys.stderr,
        )
        print("or 'just insta' before committing.", file=sys.stderr)
        sys.exit(1)
    else:
        print("✓ No pending snapshot files found")
        sys.exit(0)


if __name__ == "__main__":
    main()
