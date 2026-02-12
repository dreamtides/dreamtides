#!/usr/bin/env python3
"""Create symlinks from .llms/skills/ into .claude/skills/ and .codex/skills/.

For each skill directory in .llms/skills/, creates a relative symlink in both
.claude/skills/ and .codex/skills/ pointing back to the .llms/ source. Removes
stale symlinks that point into .llms/skills/ but whose target no longer exists
(e.g. after a rename). Existing non-symlink directories are skipped with a
warning.
"""

import os
import sys
from pathlib import Path

LLMS_SKILLS_PREFIX = os.path.join("..", "..", ".llms", "skills", "")


def main() -> None:
    repo_root = Path(__file__).resolve().parent.parent.parent
    source_dir = repo_root / ".llms" / "skills"
    targets = [
        repo_root / ".claude" / "skills",
        repo_root / ".codex" / "skills",
    ]

    if not source_dir.is_dir():
        print(f"Source directory not found: {source_dir}")
        sys.exit(1)

    skills = sorted(
        p.name for p in source_dir.iterdir() if p.is_dir() and not p.name.startswith(".")
    )

    for target_dir in targets:
        target_dir.mkdir(parents=True, exist_ok=True)

        # Remove stale symlinks that point into .llms/skills/ but no longer
        # resolve (i.e. the source skill was renamed or deleted).
        for entry in sorted(target_dir.iterdir()):
            if not entry.is_symlink():
                continue
            link_target = os.readlink(entry)
            if link_target.startswith(LLMS_SKILLS_PREFIX) and not entry.resolve().exists():
                entry.unlink()

        # Create symlinks for current skills.
        for skill in skills:
            link_path = target_dir / skill
            rel_target = os.path.relpath(source_dir / skill, target_dir)

            if link_path.is_symlink():
                if os.readlink(link_path) == rel_target:
                    continue
                link_path.unlink()
            elif link_path.exists():
                print(f"WARNING: skipping {link_path} (exists and is not a symlink)")
                continue

            os.symlink(rel_target, link_path)


if __name__ == "__main__":
    main()
