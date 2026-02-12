#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: setup_task_worktree.sh [--task-id T0005] [--branch codex/task-0005] [--worktree /abs/path] [--base master] [--no-bootstrap] [--dry-run]

Safely creates one codex task worktree/branch from a base ref and bootstraps
rules_engine/src/tv/node_modules for offline automation runs.

Discovery order:
1) Explicit --branch / --worktree values.
2) --task-id -> branch + default worktree path under /tmp/codex/tasks.
3) Auto-discover oldest ready task ID from task queue.

Bootstrap behavior:
- If source node_modules exists in main repo, copy it to the worktree.
- Otherwise run pnpm install in offline mode with store rooted in /tmp.
USAGE
}

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

TASK_ID=""
BRANCH=""
WORKTREE=""
BASE_REF="master"
NO_BOOTSTRAP=0
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
    --base)
      BASE_REF="${2:-}"
      shift 2
      ;;
    --no-bootstrap)
      NO_BOOTSTRAP=1
      shift
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

task_num_from_id() {
  local id="$1"
  local num="${id//[^0-9]/}"
  if [ -z "$num" ]; then
    return 1
  fi
  printf '%s' "$num"
}

task_branch_from_id() {
  local id="$1"
  local num
  num="$(task_num_from_id "$id")" || return 1
  echo "codex/task-$num"
}

default_worktree_from_id() {
  local id="$1"
  local num
  num="$(task_num_from_id "$id")" || return 1
  echo "/tmp/codex/tasks/dreamtides-task-$num"
}

discover_task_id() {
  .codex/scripts/task.py ready | awk 'NR==2 {print $1}'
}

copy_node_modules() {
  local source_dir="$1"
  local dest_dir="$2"

  if command -v rsync >/dev/null 2>&1; then
    run mkdir -p "$dest_dir"
    run rsync -a --delete "$source_dir/" "$dest_dir/"
    return
  fi

  run rm -rf "$dest_dir"
  run mkdir -p "$(dirname "$dest_dir")"
  run cp -R "$source_dir" "$dest_dir"
}

if [ -z "$TASK_ID" ] && [ -z "$BRANCH" ]; then
  TASK_ID="$(discover_task_id || true)"
fi

if [ -z "$BRANCH" ] && [ -n "$TASK_ID" ]; then
  BRANCH="$(task_branch_from_id "$TASK_ID")" || {
    echo "Invalid TASK_ID: $TASK_ID" >&2
    exit 1
  }
fi

if [ -z "$WORKTREE" ] && [ -n "$TASK_ID" ]; then
  WORKTREE="$(default_worktree_from_id "$TASK_ID")" || {
    echo "Invalid TASK_ID for worktree resolution: $TASK_ID" >&2
    exit 1
  }
fi

if [ -z "$BRANCH" ]; then
  echo "Could not resolve branch. Pass --task-id or --branch." >&2
  exit 1
fi

if [ -z "$WORKTREE" ]; then
  echo "Could not resolve worktree path. Pass --worktree or --task-id." >&2
  exit 1
fi

if [ "$WORKTREE" = "$REPO_ROOT" ]; then
  echo "Refusing to use main repo as worktree: $WORKTREE" >&2
  exit 1
fi

git show-ref --verify --quiet "refs/heads/$BASE_REF" || {
  echo "Base ref not found: $BASE_REF" >&2
  exit 1
}

run mkdir -p /tmp/codex/tasks

if [ -d "$WORKTREE" ]; then
  run git worktree remove --force "$WORKTREE"
fi

if git show-ref --verify --quiet "refs/heads/$BRANCH"; then
  run git branch -D "$BRANCH"
fi

run git worktree add "$WORKTREE" -b "$BRANCH" "$BASE_REF"

if [ "$NO_BOOTSTRAP" -eq 1 ]; then
  echo "Created branch: $BRANCH"
  echo "Created worktree: $WORKTREE"
  echo "Skipped dependency bootstrap (--no-bootstrap)."
  exit 0
fi

MASTER_TV_DIR="$REPO_ROOT/rules_engine/src/tv"
MASTER_NODE_MODULES="$MASTER_TV_DIR/node_modules"
WORKTREE_TV_DIR="$WORKTREE/rules_engine/src/tv"
WORKTREE_NODE_MODULES="$WORKTREE_TV_DIR/node_modules"

if [ -d "$WORKTREE_NODE_MODULES" ]; then
  echo "Created branch: $BRANCH"
  echo "Created worktree: $WORKTREE"
  echo "Dependencies already present: $WORKTREE_NODE_MODULES"
  exit 0
fi

if [ -d "$MASTER_NODE_MODULES" ]; then
  copy_node_modules "$MASTER_NODE_MODULES" "$WORKTREE_NODE_MODULES"
  echo "Created branch: $BRANCH"
  echo "Created worktree: $WORKTREE"
  echo "Bootstrapped dependencies by copying from: $MASTER_NODE_MODULES"
  exit 0
fi

if ! command -v pnpm >/dev/null 2>&1; then
  echo "pnpm not found and source node_modules missing at: $MASTER_NODE_MODULES" >&2
  exit 1
fi

run mkdir -p /tmp/codex/pnpm-store
run bash -lc "cd '$WORKTREE_TV_DIR' && pnpm install --offline --frozen-lockfile --store-dir /tmp/codex/pnpm-store --virtual-store-dir .pnpm"

echo "Created branch: $BRANCH"
echo "Created worktree: $WORKTREE"
echo "Bootstrapped dependencies via offline pnpm install."
