#!/usr/bin/env bash
# Run `just review-verbose` in a claimed worktree slot so the main repo stays
# free for continued work. Tries unique branch names so multiple concurrent
# reviews can each claim their own slot.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

BASE="$(git -C "$REPO_ROOT" rev-parse HEAD)"

# Try branch names code-review-1 through code-review-3 (one per pool slot).
# claim rejects a branch already checked out in a slot, so we try the next.
WORKTREE=""
for i in 1 2 3; do
    if WORKTREE=$(python3 "$REPO_ROOT/scripts/abu/abu.py" worktree claim "code-review-$i" --base "$BASE" 2>/dev/null); then
        echo "Review worktree ($i): $WORKTREE" >&2
        break
    fi
    WORKTREE=""
done

if [ -z "$WORKTREE" ]; then
    echo "Error: Could not claim a review worktree (all slots busy)" >&2
    exit 1
fi

# Mark slot as busy so concurrent `claim` won't steal it.
# Staging a file makes git status show a non-?? line, which
# causes _is_worktree_available to return False.
touch "$WORKTREE/.review-lock"
git -C "$WORKTREE" add -f .review-lock
cleanup() { git -C "$WORKTREE" reset HEAD .review-lock 2>/dev/null; rm -f "$WORKTREE/.review-lock"; }
trap cleanup EXIT

cd "$WORKTREE" && just review-verbose || { osascript -e 'display dialog "Review failed" with icon stop' 2>/dev/null; exit 1; }
