#!/usr/bin/env bash
# Run `just review-verbose` in a claimed worktree slot so the main repo stays
# free for continued work.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${DREAMTIDES_REPO_ROOT:-$(cd "$SCRIPT_DIR/../.." && pwd)}"
WORKTREE_BASE="${DREAMTIDES_WORKTREE_BASE:-$HOME/dreamtides-worktrees}"
REVIEW_JUST_CMD="${REVIEW_JUST_CMD:-just}"
BASE=""
WORKTREE=""
REVIEW_BRANCH=""

is_reclaimable_slot_branch() {
    local slot="$1"
    local branch="$2"

    [[ -z "$branch" || "$branch" == code-review-* || "$branch" == "$slot" ]]
}

normalize_slot_for_review() {
    local slot="$1"
    local slot_path="$2"
    local base="$3"
    local allow_locked="${4:-false}"
    local branch=""

    if [ ! -d "$slot_path" ]; then
        return 0
    fi

    if [ -f "$slot_path/.review-lock" ] && [ "$allow_locked" != "true" ]; then
        return 0
    fi

    branch=$(git -C "$slot_path" branch --show-current 2>/dev/null || true)
    if ! is_reclaimable_slot_branch "$slot" "$branch"; then
        return 0
    fi

    rm -f "$slot_path/.review-lock"
    git -C "$slot_path" reset HEAD .review-lock >/dev/null 2>&1 || true
    git -C "$slot_path" reset --hard "$base" >/dev/null 2>&1 || return 1
    git -C "$slot_path" clean -fd >/dev/null 2>&1 || return 1
    git -C "$slot_path" checkout --detach "$base" >/dev/null 2>&1 || return 1

    if [ -n "$branch" ]; then
        git -C "$REPO_ROOT" branch -D "$branch" >/dev/null 2>&1 || true
    fi
}

main() {
    BASE="$(git -C "$REPO_ROOT" rev-parse HEAD)"

    # Normalize stale review/pool slots before claiming so the generic
    # claim logic sees them as reusable again.
    for slot in alpha beta gamma; do
        normalize_slot_for_review "$slot" "$WORKTREE_BASE/$slot" "$BASE"
    done

    for i in 1 2 3; do
        if WORKTREE=$(python3 "$REPO_ROOT/scripts/abu/abu.py" worktree claim "code-review-$i" --base "$BASE"); then
            REVIEW_BRANCH="code-review-$i"
            break
        fi
        WORKTREE=""
    done

    if [ -z "$WORKTREE" ]; then
        echo "Error: Could not claim a review worktree (all slots busy)" >&2
        exit 1
    fi

    cleanup() {
        if [ -n "$WORKTREE" ]; then
            normalize_slot_for_review "$(basename "$WORKTREE")" "$WORKTREE" "$BASE" true || true
        fi
        if [ -n "$REVIEW_BRANCH" ]; then
            git -C "$REPO_ROOT" branch -D "$REVIEW_BRANCH" >/dev/null 2>&1 || true
        fi
    }
    trap cleanup EXIT INT TERM HUP

    # Mark slot as busy so concurrent `claim` won't steal it.
    # Staging a file makes git status show a non-?? line, which
    # causes _is_worktree_available to return False.
    touch "$WORKTREE/.review-lock"
    git -C "$WORKTREE" add -f .review-lock

    cd "$WORKTREE" && "$REVIEW_JUST_CMD" review-verbose || {
        osascript -e 'display dialog "Review failed" with icon stop' 2>/dev/null
        exit 1
    }
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
    main "$@"
fi
