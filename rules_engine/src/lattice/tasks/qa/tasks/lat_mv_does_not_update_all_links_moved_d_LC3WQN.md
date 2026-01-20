---
lattice-id: LC3WQN
name: lat-mv-does-not-update-all-links-moved-d
description: lat mv does not update all links to moved document
task-type: bug
priority: 2
labels:
- mv
- links
created-at: 2026-01-20T07:06:51.292257Z
updated-at: 2026-01-20T07:06:51.292257Z
---

## Context
Test scenario: LCLWQN (Linking System and Normalization)
Part 4, Step 4.1: Link Updates on Move

## Steps to Reproduce
1. Create architecture.md (LBVWQN) at docs/design/
2. Create endpoints.md linking to architecture: `[architecture doc](../design/architecture.md#LBVWQN)`
3. Create data_model.md also linking to architecture: `[link](architecture.md#LBVWQN)`
4. Run `lat mv LBVWQN docs/new_location/architecture.md`

## Expected Behavior
Both endpoints.md and data_model.md should have their links updated to point to the new location.

## Actual Behavior
Output:
```
Moved LBVWQN -> docs/new_location/architecture.md
  Parent: LBTWQN -> LBSWQN
  1 link(s) updated
```

Only data_model.md was updated. endpoints.md still has the old path:
- data_model.md: `../new_location/architecture.md#LBVWQN` ✓
- endpoints.md: `../design/architecture.md#LBVWQN` ✗ (should be `../new_location/architecture.md#LBVWQN`)

`lat check` correctly detects the stale link:
```
Warning [W010]: Line 12 has stale link path (expected ../new_location/architecture.md#LBVWQN)
```

## Related
This is likely the same root cause as the backlink tracking bug - endpoints.md was also not shown by `lat links-to LBVWQN`.