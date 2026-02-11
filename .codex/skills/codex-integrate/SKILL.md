---
name: codex-integrate
description: Merge one completed codex task worktree branch into master with automatic task, branch, and worktree discovery.
---

Merge exactly one completed `codex/task-*` branch into `master`.
Use this only when an external integration lock is already held.

Default behavior: auto-discover the newest mergeable done task from the task queue,
find or recreate its worktree, rebase onto `master`, run validation, fast-forward
merge, then clean up branch/worktree.

## Runbook

1. Discover merge target (`TASK_ID`, `BRANCH`, `WORKTREE`).

- Optional override: set `TASK_ID` first to force one task (example: `TASK_ID=T0005`).
- Otherwise, discovery scans done tasks newest-first and picks the first branch that:
  - exists locally as `codex/task-<num>`
  - is ahead of `master`

```bash
cd /Users/dthurn/Documents/GoogleDrive/dreamtides/
REPO_ROOT="$(pwd)"

if [ -n "${TASK_ID:-}" ]; then
  CANDIDATE_IDS="$TASK_ID"
else
  CANDIDATE_IDS="$(.codex/scripts/task.py list --status done | awk 'NR>1 {print $1}' | sort -r)"
fi

SELECTED_TASK_ID=""
SELECTED_BRANCH=""
while read -r ID; do
  [ -z "$ID" ] && continue
  NUM="${ID//[^0-9]/}"
  BRANCH="codex/task-$NUM"
  git show-ref --verify --quiet "refs/heads/$BRANCH" || continue
  AHEAD_COUNT="$(git rev-list --count master..$BRANCH)"
  [ "$AHEAD_COUNT" -gt 0 ] || continue
  SELECTED_TASK_ID="$ID"
  SELECTED_BRANCH="$BRANCH"
  break
done <<< "$CANDIDATE_IDS"

if [ -z "$SELECTED_BRANCH" ]; then
  echo "No mergeable codex/task-* branch found."
  exit 1
fi

TASK_ID="$SELECTED_TASK_ID"
BRANCH="$SELECTED_BRANCH"
TASK_NUM="${TASK_ID//[^0-9]/}"

WORKTREE="$(git worktree list --porcelain | awk -v b="refs/heads/$BRANCH" '
  $1 == "worktree" { wt=$2 }
  $1 == "branch" && $2 == b { print wt; exit }
')"

if [ -z "$WORKTREE" ]; then
  WORKTREE="../dreamtides-task-$TASK_NUM"
  git worktree add "$WORKTREE" "$BRANCH"
fi

echo "TASK_ID=$TASK_ID"
echo "BRANCH=$BRANCH"
echo "WORKTREE=$WORKTREE"
```

2. Rebase and validate in the task worktree.

```bash
git fetch "$REPO_ROOT" master
cd "$WORKTREE"
git rebase FETCH_HEAD
just fmt
just review
```

3. Fast-forward merge into `master`, then clean up.

```bash
cd "$REPO_ROOT"
git checkout master
git merge --ff-only "$BRANCH"
git worktree remove "$WORKTREE"
git branch -d "$BRANCH"
```

## Failure Handling

- If rebase or validation fails, stop and keep branch/worktree for fixes.
- Do not force-merge and do not delete the branch/worktree when unresolved.
- If worktree removal fails because it is dirty, fix or stash inside that worktree first.
