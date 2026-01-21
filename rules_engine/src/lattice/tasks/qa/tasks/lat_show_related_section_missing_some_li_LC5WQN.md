---
lattice-id: LC5WQN
name: lat-show-related-section-missing-some-li
description: lat show Related section missing some linked documents
parent-id: LC6WQN
task-type: bug
priority: 2
labels:
- show
- links
created-at: 2026-01-20T07:08:24.785759Z
updated-at: 2026-01-21T22:32:22.846296Z
---

## Context

Test scenario: LCLWQN (Linking System and Normalization)
Part 6, Step 6.1: Related Documents in Show

## Steps to Reproduce

1. Create data_model.md (LBXWQN) with links to:
   - architecture (LBVWQN)
   - endpoints (LBWWQN)
   - task (LBZWQN)
2. Run `lat show LBXWQN`

## Expected Behavior

Related section should show all 3 linked documents:

- LBVWQN (architecture)
- LBWWQN (endpoints)
- LBZWQN (task)

## Actual Behavior

```
Related (1):
  LBWWQN: endpoints - API endpoints reference [doc]
```

Only 1 of 3 linked documents shown. Architecture and task links are missing.

## JSON Output Confirms

```json
"related": [
  {
    "id": "LBWWQN",
    ...
  }
]
```
Only endpoints is in the related array.

## Related

This is likely the same root cause as the backlink tracking and link update bugs

- inconsistent link tracking.
