---
lattice-id: LCXWQN
name: lat-fmt-does-not-expand-shorthand-links-
description: lat fmt does not expand shorthand links in document body
task-type: bug
priority: 2
labels:
- linking
- fmt
created-at: 2026-01-20T07:03:15.611467Z
updated-at: 2026-01-20T07:03:15.611467Z
---

## Context
Test scenario: LCLWQN (Linking System and Normalization)
Part 1, Step 1.1: Shorthand Link Expansion

## Steps to Reproduce
1. Create test documents with manually-set lattice-ids
2. Add shorthand links like `[architecture doc](LTEST01)` in document body
3. Run `lat fmt`

## Expected Behavior
- Link `[architecture doc](LTEST01)` should expand to `[architecture doc](../design/architecture.md#LTEST01)`
- Link `[endpoints](../api/endpoints.md)` should get `#LTEST02` fragment appended

## Actual Behavior
- Shorthand links remain unexpanded: `[architecture doc](LTEST01)`
- Path-only links don't get fragment added
- `lat fmt` output says "2 file(s) formatted, 1 unchanged" but files with shorthand links were not modified

## Command Output
```
$ lat fmt
Formatted:
  docs/api/api.md
  docs/design/design.md

2 file(s) formatted, 1 unchanged
```

The files containing shorthand links (`docs/api/endpoints.md` and `docs/design/data_model.md`) were not formatted.