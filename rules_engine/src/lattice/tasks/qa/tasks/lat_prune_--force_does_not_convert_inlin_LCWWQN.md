---
lattice-id: LCWWQN
name: lat-prune---force-does-not-convert-inlin
description: lat prune --force does not convert inline links to plain text
parent-id: LC6WQN
task-type: bug
priority: 2
labels:
- testing
- prune
created-at: 2026-01-20T07:00:32.253744Z
updated-at: 2026-01-21T22:31:38.576704Z
---

## Context

Test scenario: LCDWQN (Task Lifecycle Management)
Step 6.3: Prune with --force

## Steps to Reproduce

1. Create two tasks MAIN and RELATED
2. Add an inline link in MAIN body referencing RELATED: `
   [RELATED_ID](lattice://RELATED_ID) `
3. Close RELATED task
4. Run `lat prune auth/tasks/.closed/ --force`

## Expected Behavior

According to the test specification:
> Use `--force` to convert links to plain text

When using `--force`, the prune command should convert any inline markdown links
that reference the pruned task into plain text.

## Actual Behavior

`lat prune --force` deletes the task but does NOT convert the inline link to
plain text. The MAIN task still contains the broken link `
[RELATED_ID](lattice://RELATED_ID) ` after the prune completes.

## Command Output

```
Pruned 1 closed task(s)
  LB3WQN (auth/tasks/.closed/related_task.md)
Exit code: 0
```

After checking the MAIN task file:
```
Depends on [LB3WQN](lattice://LB3WQN).
```

The link is still present and now broken (pointing to a non-existent document).
