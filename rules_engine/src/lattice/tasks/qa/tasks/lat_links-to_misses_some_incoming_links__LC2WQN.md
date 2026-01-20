---
lattice-id: LC2WQN
name: lat-links-to-misses-some-incoming-links-
description: lat links-to misses some incoming links (backlink tracking incomplete)
task-type: bug
priority: 2
labels:
- links
- backlinks
created-at: 2026-01-20T07:06:11.471138Z
updated-at: 2026-01-20T07:06:11.471138Z
---

## Context
Test scenario: LCLWQN (Linking System and Normalization)
Part 3, Step 3.1: Backlink Tracking

## Steps to Reproduce
1. Create architecture.md (LBVWQN)
2. Create endpoints.md (LBWWQN) with link: `[architecture doc](../design/architecture.md#LBVWQN)`
3. Create data_model.md (LBXWQN) with link: `[link](architecture.md#LBVWQN)`
4. Run `lat links-to LBVWQN`

## Expected Behavior
Should show both:
- endpoints.md (LBWWQN)
- data_model.md (LBXWQN)

## Actual Behavior
Only shows:
```
1 document links to architecture:

  LBXWQN [doc] data-model  (body)
```

endpoints.md is missing from the backlink results despite clearly containing a link to LBVWQN.

## Verified Content
endpoints.md contains:
```
See the [architecture doc](../design/architecture.md#LBVWQN) for context.
```