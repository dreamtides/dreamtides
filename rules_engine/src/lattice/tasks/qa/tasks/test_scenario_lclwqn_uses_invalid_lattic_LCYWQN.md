---
lattice-id: LCYWQN
name: test-scenario-lclwqn-uses-invalid-lattic
description: Test scenario LCLWQN uses invalid lattice-id format (LTEST01)
parent-id: LC6WQN
task-type: bug
priority: 2
labels:
- test-scenario
- documentation
created-at: 2026-01-20T07:05:17.774888Z
updated-at: 2026-01-21T22:31:38.594623Z
---

## Context

Test scenario: LCLWQN (Linking System and Normalization)
Setup section

## Issue

The test scenario setup creates documents with manually-specified lattice-ids
like `LTEST01`, `LTEST02`, `LTEST03`. These don't follow the valid Lattice ID
format (e.g., `LBVWQN` - 6 alphanumeric characters).

When attempting to index these documents:
```
Error: YAML parsing failed in /path/to/architecture.md: line 1, column 1: Invalid Lattice ID format: LTEST01
```

## Impact

- Documents with invalid IDs cannot be indexed
- All subsequent tests that rely on these documents fail
- Makes the entire test scenario unexecutable as written

## Recommended Fix

Update the test scenario to either:

1. Use `lat create` or `lat track` to generate valid IDs
2. Store the generated IDs in shell variables for use in subsequent steps
3. Avoid hardcoding specific IDs in the test setup
