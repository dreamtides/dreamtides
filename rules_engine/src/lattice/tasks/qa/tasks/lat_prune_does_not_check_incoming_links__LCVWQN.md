---
lattice-id: LCVWQN
name: lat-prune-does-not-check-incoming-links-
description: lat prune does not check for incoming links before deleting
parent-id: LC6WQN
task-type: bug
priority: 2
labels:
- testing
- prune
created-at: 2026-01-20T07:00:32.169844Z
updated-at: 2026-01-21T22:31:38.583121Z
---

## Context

Test scenario: LCDWQN (Task Lifecycle Management)
Step 6.2: Close and attempt prune with inline links

## Steps to Reproduce

1. Create two tasks MAIN and SUB
2. Add an inline link in MAIN body referencing SUB: `[SUB](lattice://SUB_ID)`
3. Close SUB task
4. Run `lat prune auth/tasks/.closed/`

## Expected Behavior

According to the test specification:
> If there are inline markdown links TO the pruned task, command should error

The prune command should detect that MAIN has a link TO SUB and refuse to prune
without `--force`.

## Actual Behavior

`lat prune` succeeds without error, leaving MAIN with a broken link to the
now-deleted SUB task. No warning or error is given about the incoming link.

## Command Output

```
Pruned 1 closed task(s)
  LBZWQN (auth/tasks/.closed/sub_task.md)
Exit code: 0
```

The MAIN task still contains the broken link after prune completes.
