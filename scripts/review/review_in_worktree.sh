#!/usr/bin/env bash
# Run `just review-verbose` in a claimed worktree slot so the main repo stays
# free for continued work.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

BASE="$(git -C "$REPO_ROOT" rev-parse HEAD)"

# Clean up stale code-review branches from previous runs.
# Detach HEAD in unlocked slots so claim doesn't hit a branch conflict.
for slot in alpha beta gamma; do
    SLOT_PATH="$HOME/dreamtides-worktrees/$slot"
    if [ -d "$SLOT_PATH" ] && [ ! -f "$SLOT_PATH/.review-lock" ]; then
        BRANCH=$(git -C "$SLOT_PATH" branch --show-current 2>/dev/null || true)
        if [[ "$BRANCH" == code-review-* ]]; then
            git -C "$SLOT_PATH" checkout --detach 2>/dev/null || true
            git -C "$REPO_ROOT" branch -D "$BRANCH" 2>/dev/null || true
        fi
    fi
done

WORKTREE=""
for i in 1 2 3; do
    if WORKTREE=$(python3 "$REPO_ROOT/scripts/abu/abu.py" worktree claim "code-review-$i" --base "$BASE"); then
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
