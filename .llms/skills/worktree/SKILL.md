---
name: worktree
description: Use when implementing a feature in an isolated git worktree. Triggers on /worktree, worktree workflow, isolated branch work, or parallel development in a clean environment.
---

# Worktree Workflow

Implement a feature in an isolated worktree slot, then
release it when done.

## Claim a Slot

```bash
WORKTREE=$(python3 scripts/abu/abu.py worktree claim <branch-name>)
```

- Prints only the worktree path to stdout (diagnostics to stderr).
- Reuses a merged+clean slot, creates a new one if pool has room, or errors if all 3 slots (alpha/beta/gamma) are occupied.
- Creates the branch at `master` by default. Use `--base <ref>` for a different base.

## Work in the Worktree

**All work must happen inside `$WORKTREE`** — not just edits
and builds, but also file reads, research, and grep/glob
searches. The main repo may be changing in parallel, so
reading files from it can give stale or inconsistent results.

```bash
cd "$WORKTREE"
# edit, build, test, AND read/research here
just check && just review
```

## Rebase and Merge When Done

Before reporting complete, rebase onto current master (other
work may have landed while you were working):

```bash
cd "$WORKTREE"
git fetch /Users/dthurn/Documents/GoogleDrive/dreamtides master
git log HEAD..FETCH_HEAD --oneline   # check for new commits
git rebase FETCH_HEAD                # if any new commits exist
just fmt && just review              # re-validate after rebase
```

Then fast-forward merge into master:

```bash
cd /Users/dthurn/Documents/GoogleDrive/dreamtides
git merge --ff-only <branch-name>
```

## Release the Slot

After merging, the slot auto-reclaims on the next
`abu worktree claim` (it detects merged+clean slots). To
release immediately:

```bash
python3 scripts/abu/abu.py worktree remove <slot-name> --delete-branch
```

## Quick Reference

| Command | Purpose |
|---------|---------|
| `abu worktree claim <branch>` | Get a worktree path |
| `abu worktree list` | Show active worktrees |
| `abu worktree remove <slot>` | Free a slot manually |
| `abu worktree refresh <slot>` | Re-sync gitignored caches |
