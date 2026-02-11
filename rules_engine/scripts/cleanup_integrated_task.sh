#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: cleanup_integrated_task.sh [--task-id T0005] [--branch codex/task-0005] [--worktree /abs/path] [--dry-run]

Safely removes one integrated codex task worktree and branch.

Discovery order:
1) Explicit --branch / --worktree values.
2) --task-id -> branch name.
3) Auto-discover newest done task branch that:
   - exists locally
   - is already merged into master
EOF
}

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

TASK_ID=""
BRANCH=""
WORKTREE=""
DRY_RUN=0

while [ "$#" -gt 0 ]; do
  case "$1" in
    --task-id)
      TASK_ID="${2:-}"
      shift 2
      ;;
    --branch)
      BRANCH="${2:-}"
      shift 2
      ;;
    --worktree)
      WORKTREE="${2:-}"
      shift 2
      ;;
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage
      exit 1
      ;;
  esac
done

run() {
  if [ "$DRY_RUN" -eq 1 ]; then
    echo "+ $*"
  else
    "$@"
  fi
}

task_branch_from_id() {
  local id="$1"
  local num="${id//[^0-9]/}"
  if [ -z "$num" ]; then
    return 1
  fi
  echo "codex/task-$num"
}

discover_branch() {
  local candidate_ids branch
  candidate_ids="$(
    .codex/scripts/task.py list --status done \
      | awk 'NR>1 {print $1}' \
      | sort -r
  )"
  while read -r id; do
    [ -z "$id" ] && continue
    branch="$(task_branch_from_id "$id")" || continue
    git show-ref --verify --quiet "refs/heads/$branch" || continue
    git merge-base --is-ancestor "$branch" master || continue
    TASK_ID="$id"
    BRANCH="$branch"
    return 0
  done <<< "$candidate_ids"
  return 1
}

discover_worktree_for_branch() {
  local branch="$1"
  git worktree list --porcelain | awk -v b="refs/heads/$branch" '
    $1 == "worktree" { wt=$2 }
    $1 == "branch" && $2 == b { print wt; exit }
  '
}

if [ -z "$BRANCH" ] && [ -n "$TASK_ID" ]; then
  BRANCH="$(task_branch_from_id "$TASK_ID")" || {
    echo "Invalid TASK_ID: $TASK_ID" >&2
    exit 1
  }
fi

if [ -z "$BRANCH" ]; then
  discover_branch || {
    echo "No integrated codex/task-* branch found for cleanup." >&2
    exit 1
  }
fi

git show-ref --verify --quiet "refs/heads/$BRANCH" || {
  echo "Branch not found: $BRANCH" >&2
  exit 1
}

git merge-base --is-ancestor "$BRANCH" master || {
  echo "Refusing cleanup: $BRANCH is not merged into master." >&2
  exit 1
}

if [ -z "$WORKTREE" ]; then
  WORKTREE="$(discover_worktree_for_branch "$BRANCH" || true)"
fi

if [ -n "$WORKTREE" ]; then
  if [ "$WORKTREE" = "$REPO_ROOT" ]; then
    echo "Refusing to remove main repo worktree: $WORKTREE" >&2
    exit 1
  fi
  if [ -d "$WORKTREE" ]; then
    run git worktree remove "$WORKTREE"
  fi
fi

run git branch -d "$BRANCH"

echo "Cleaned branch: $BRANCH"
[ -n "$WORKTREE" ] && echo "Cleaned worktree: $WORKTREE"
