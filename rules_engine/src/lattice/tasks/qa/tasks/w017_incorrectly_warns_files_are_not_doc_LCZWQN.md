---
lattice-id: LCZWQN
name: w017-incorrectly-warns-files-are-not-doc
description: W017 incorrectly warns files are not in docs/ directory
parent-id: LC6WQN
task-type: bug
priority: 3
labels:
- check
- warning
created-at: 2026-01-20T07:05:43.152959Z
updated-at: 2026-01-21T22:31:38.599187Z
---

## Context

Test scenario: LCLWQN (Linking System and Normalization)
Part 2, Step 2.1: Link Validation

## Steps to Reproduce

1. Create a document at `docs/design/architecture.md`
2. Run `lat check`

## Expected Behavior

No W017 warning, since the file IS in docs/ (specifically docs/design/)

## Actual Behavior

```
docs/design/architecture.md:
  Warning [W017]: docs/design/architecture.md is not in tasks/ or docs/ directory
```

The file IS in docs/ subdirectory but W017 warning claims it isn't.

## Analysis

The check seems to only recognize files directly in `docs/` or `tasks/` but not
in subdirectories like `docs/design/` or `docs/api/`.
