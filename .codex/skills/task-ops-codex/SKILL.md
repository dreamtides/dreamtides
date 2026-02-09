---
name: task-ops-codex
description: Operate the persistent Codex filesystem task list under .codex/tasks using .codex/scripts/task.py. Use for task queue operations including finding ready work, adding tasks, updating dependencies, marking done, validating integrity, and inspecting task details.
---

# Codex Task Operations

Use `.codex/scripts/task.py` as the single interface for task operations.

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
```

Validate store:

```bash
.codex/scripts/task.py validate
```

## Operational Patterns

### Daily loop

1. `ready --json` to select next task.
2. `get <id> --body` only for chosen task.
3. `update <id> --status in_progress` when starting.
4. `done <id>` when finished.
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
