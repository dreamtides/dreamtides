---
lattice-id: LC4WQN
name: lat-close-does-not-update-links-closed-t
description: lat close does not update links to closed task
parent-id: LC6WQN
task-type: bug
priority: 2
labels:
- close
- links
created-at: 2026-01-20T07:07:51.052009Z
updated-at: 2026-01-21T22:31:38.552720Z
---

## Context

Test scenario: LCLWQN (Linking System and Normalization)
Part 5, Step 5.2: Close the task

## Steps to Reproduce

1. Create task LBZWQN at `docs/tasks/important_task.md`
2. Add link in data_model.md: `[the task](../tasks/important_task.md#LBZWQN)`
3. Run `lat close LBZWQN`

## Expected Behavior

Link in data_model.md should be updated to:
`[the task](../tasks/.closed/important_task.md#LBZWQN)`

## Actual Behavior

```
Closed LBZWQN -> docs/tasks/.closed/important_task.md
```
No mention of link updates. Link remains unchanged:
`[the task](../tasks/important_task.md#LBZWQN)`

`lat check` correctly detects the stale link:
```
Warning [W010]: Line 17 has stale link path (expected ../tasks/.closed/important_task.md#LBZWQN)
```

## Note

Compare to `lat mv` which does attempt to update links (though incompletely per
separate bug).
