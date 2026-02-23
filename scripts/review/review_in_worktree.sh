#!/usr/bin/env bash
# Run `just review-verbose` in a claimed worktree slot so the main repo stays
# free for continued work. Uses `abu worktree claim` to get a slot,
# with a fallback that resets an existing code-review slot.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

BASE="$(git -C "$REPO_ROOT" rev-parse HEAD)"

if WORKTREE=$(python3 "$REPO_ROOT/scripts/abu/abu.py" worktree claim code-review --base "$BASE" 2>/dev/null); then
    echo "Review worktree: $WORKTREE" >&2
else
    # code-review branch already checked out — find its slot and reset it
    WORKTREE=""
    for slot in alpha beta gamma; do
        SLOT_PATH="$HOME/dreamtides-worktrees/$slot"
        if [ -d "$SLOT_PATH" ] && [ "$(git -C "$SLOT_PATH" branch --show-current 2>/dev/null)" = "code-review" ]; then
            WORKTREE="$SLOT_PATH"
            break
        fi
    done
    if [ -z "$WORKTREE" ]; then
        echo "Error: Could not claim or find a review worktree" >&2
        exit 1
    fi
    echo "Reusing review worktree: $WORKTREE" >&2
    git -C "$WORKTREE" checkout -B code-review "$BASE" >/dev/null
    git -C "$WORKTREE" clean -fd >/dev/null
fi

# Mark slot as busy so concurrent `claim` won't steal it.
# Staging a file makes git status show a non-?? line, which
# causes _is_worktree_available to return False.
touch "$WORKTREE/.review-lock"
git -C "$WORKTREE" add .review-lock
cleanup() { git -C "$WORKTREE" reset HEAD .review-lock 2>/dev/null; rm -f "$WORKTREE/.review-lock"; }
trap cleanup EXIT

cd "$WORKTREE" && just review-verbose || { osascript -e 'display dialog "Review failed" with icon stop' 2>/dev/null; exit 1; }
