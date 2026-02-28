#!/usr/bin/env python3
"""
Script to strip 'Generated with' and 'Co-Authored-By' lines from git commit messages.
Only processes unpushed commits to avoid rewriting history that's already been shared.
"""

import subprocess
import sys
import re
import argparse


def run_command(cmd, check=True, capture_output=True):
    """Run a shell command and return the result."""
    result = subprocess.run(
        cmd, shell=True, capture_output=capture_output, text=True, check=False
    )
    if check and result.returncode != 0:
        print(f"Command failed: {cmd}", file=sys.stderr)
        print(f"Error: {result.stderr}", file=sys.stderr)
        sys.exit(1)
    return result


def get_upstream_branch():
    """Get the upstream branch for the current branch."""
    result = run_command(
        "git rev-parse --abbrev-ref --symbolic-full-name @{u}", check=False
    )
    if result.returncode == 0:
        return result.stdout.strip()

    # Fallback to origin/master if no upstream is set
    result = run_command("git rev-parse --verify origin/master", check=False)
    if result.returncode == 0:
        return "origin/master"

    # Try origin/main as another fallback
    result = run_command("git rev-parse --verify origin/main", check=False)
    if result.returncode == 0:
        return "origin/main"

    return None


def get_unpushed_commits():
    """Get list of commit hashes that haven't been pushed."""
    upstream = get_upstream_branch()
    if not upstream:
        # No remote tracking, check all commits
        result = run_command("git rev-list HEAD", check=False)
        if result.returncode != 0:
            return []
        commits = result.stdout.strip().split("\n")
        return [c for c in commits if c]

    result = run_command(f"git rev-list {upstream}..HEAD", check=False)
    if result.returncode != 0:
        return []

    commits = result.stdout.strip().split("\n")
    return [c for c in commits if c]


def get_commit_message(commit_hash):
    """Get the full commit message for a given commit."""
    result = run_command(f"git log -1 --format=%B {commit_hash}")
    return result.stdout


def should_strip_message(message):
    """Check if a commit message contains lines that should be stripped."""
    lines = message.split("\n")
    for line in lines:
        stripped = line.strip()
        if "Generated with" in stripped or stripped.startswith("Co-Authored-By:"):
            return True
    return False


def strip_message(message):
    """Remove lines containing 'Generated with' or 'Co-Authored-By' from message."""
    lines = message.split("\n")
    filtered_lines = []

    for line in lines:
        stripped = line.strip()
        # Skip lines containing 'Generated with' or starting with 'Co-Authored-By:'
        if "Generated with" in line or stripped.startswith("Co-Authored-By:"):
            continue
        filtered_lines.append(line)

    # Remove trailing empty lines
    while filtered_lines and not filtered_lines[-1].strip():
        filtered_lines.pop()

    return "\n".join(filtered_lines)


def check_commits(verbose=False):
    """Check if any unpushed commits contain the unwanted lines."""
    commits = get_unpushed_commits()
    if not commits:
        return True

    found_issues = False
    for commit in commits:
        message = get_commit_message(commit)
        if should_strip_message(message):
            found_issues = True
            short_hash = run_command(f"git log -1 --format=%h {commit}").stdout.strip()
            subject = run_command(f"git log -1 --format=%s {commit}").stdout.strip()
            print(
                f"Commit {short_hash} contains 'Generated with' or 'Co-Authored-By' lines: {subject}"
            )

    return not found_issues


def strip_commits():
    """Strip the unwanted lines from all unpushed commits."""
    commits = get_unpushed_commits()
    if not commits:
        return

    # Check if any commits need stripping
    commits_to_strip = []
    for commit in commits:
        message = get_commit_message(commit)
        if should_strip_message(message):
            commits_to_strip.append(commit)

    if not commits_to_strip:
        return

    # Get the upstream branch to rebase from
    upstream = get_upstream_branch()
    if not upstream:
        print("Warning: No upstream branch found, using all commits on current branch")
        # Get the first commit
        result = run_command("git rev-list --max-parents=0 HEAD")
        base = result.stdout.strip()
    else:
        base = upstream

    # Create a script for git filter-branch or use rebase
    # Using rebase with --exec is safer
    # Use a temporary script to amend commits during rebase
    script_content = """#!/bin/bash
COMMIT_MSG=$(git log -1 --format=%B)
if echo "$COMMIT_MSG" | grep -q "Generated with\\|Co-Authored-By:"; then
    NEW_MSG=$(echo "$COMMIT_MSG" | grep -v "Generated with" | grep -v "Co-Authored-By:")
    # Remove trailing empty lines
    NEW_MSG=$(echo "$NEW_MSG" | sed -e :a -e '/^\\s*$/d;N;ba')
    git commit --amend -m "$NEW_MSG"
fi
"""

    # Write the script
    with open("/tmp/git-strip-commit-msg.sh", "w") as f:
        f.write(script_content)

    run_command("chmod +x /tmp/git-strip-commit-msg.sh", capture_output=False)

    # Run git rebase with the script
    result = run_command(
        f"GIT_SEQUENCE_EDITOR=true git rebase -i --exec /tmp/git-strip-commit-msg.sh {base}",
        check=False,
        capture_output=False,
    )

    if result.returncode != 0:
        print(
            "\nRebase failed. You may need to resolve conflicts manually.",
            file=sys.stderr,
        )
        print("Run 'git rebase --abort' to cancel the operation.", file=sys.stderr)
        sys.exit(1)


def main():
    parser = argparse.ArgumentParser(
        description="Strip 'Generated with' and 'Co-Authored-By' lines from unpushed git commits"
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Check if commits contain unwanted lines (returns non-zero exit code if found)",
    )
    parser.add_argument("--verbose", action="store_true", help="Print verbose output")

    args = parser.parse_args()

    if args.check:
        success = check_commits(verbose=args.verbose)
        sys.exit(0 if success else 1)
    else:
        strip_commits()


if __name__ == "__main__":
    main()
