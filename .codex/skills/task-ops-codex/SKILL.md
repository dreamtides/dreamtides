---
name: task-ops-codex
description: Operate the persistent Codex filesystem task list under /tmp/codex/tasks by default using .codex/scripts/task.py. Use for task queue operations including finding ready work, adding tasks, updating dependencies, marking done, validating integrity, and inspecting task details.
---

# Codex Task Operations

Use `.codex/scripts/task.py` as the single interface for task operations.
By default the task store is `/tmp/codex/tasks`; pass `--root` to use a different root.

## Core Commands

Initialize:

```bash
.codex/scripts/task.py init
```

Find unblocked work:

```bash
.codex/scripts/task.py ready
.codex/scripts/task.py ready --json
```

Claim oldest ready task (single-step start):

```bash
.codex/scripts/task.py start
.codex/scripts/task.py start --id-only
.codex/scripts/task.py start --json --body
```

List tasks:

```bash
.codex/scripts/task.py list
.codex/scripts/task.py list --all
.codex/scripts/task.py list --status in_progress
```

Add a task from markdown file:

```bash
.codex/scripts/task.py add \
  --title "Implement spark transfer resolver" \
  --markdown-file /tmp/task.md \
  --blocked-by T0002
```

Add a task from stdin:

```bash
cat /tmp/task.md | .codex/scripts/task.py add \
  --title "Add integration tests for spark transfer" \
  --markdown-stdin
```

Get task metadata or full details:

```bash
.codex/scripts/task.py get T0007
.codex/scripts/task.py get T0007 --body
.codex/scripts/task.py get T0007 --body --json
```

Update title/status/blockers:

```bash
.codex/scripts/task.py update T0007 --status in_progress
.codex/scripts/task.py update T0007 --add-blocker T0003
.codex/scripts/task.py update T0007 --remove-blocker T0003
.codex/scripts/task.py update T0007 --set-blocked-by T0001,T0002
```

Update markdown details:

```bash
.codex/scripts/task.py update T0007 --replace-markdown-file /tmp/new.md
.codex/scripts/task.py update T0007 --append-markdown-file /tmp/notes.md
```

Mark done:

```bash
.codex/scripts/task.py done T0007
.codex/scripts/task.py finish T0007
```

Return a task to `todo`:

```bash
.codex/scripts/task.py release T0007
```

Validate store:

```bash
.codex/scripts/task.py validate
```

## Operational Patterns

### Daily loop

1. `ready --json` to select next task.
2. `start --json --body` to atomically claim oldest ready task and load details.
3. `done <id>` or `finish <id>` when finished.
4. `release <id>` if you need to put an in-progress task back to `todo`.
5. `validate` after dependency edits.

### Adding a dependency

Use when compile order or merge-risk demands it:

```bash
.codex/scripts/task.py update T0010 --add-blocker T0008
```

### Clearing blockers

```bash
.codex/scripts/task.py update T0010 --set-blocked-by ""
```

## Error Handling Behavior

The CLI fails with non-zero exit code for:

- missing task store (run `init`)
- unknown task IDs or blocker IDs
- invalid status
- self-blocking relationships
- dependency cycles
- corrupt index JSON

## Token Efficiency Rules

- prefer `ready` and filtered `list` before `get --body`
- read one task body at a time
- prefer `--json` for scripted orchestration
- run `validate` after relationship changes
