---
lattice-id: LCJWQN
name: fix-check-line-numbers
description: Line numbers in lat check warnings are incorrect or offset
parent-id: LCEWQN
task-type: bug
priority: 2
created-at: 2026-01-19T05:15:00Z
updated-at: 2026-01-19T17:47:23.504773Z
closed-at: 2026-01-19T17:47:23.504773Z
---

# Line numbers in lat check warnings are incorrect

## Problem

The line numbers reported in `lat check` warnings don't match the actual
location of issues in the file.

## Steps to Reproduce

1. Create a document with issues around line 26-33
2. Run `lat check`
3. Warnings reference lines 16, 19, 22 instead of the actual locations

## Expected Behavior

Line numbers in warnings should match the actual file content so users can
quickly navigate to and fix issues.

## Actual Behavior

There appears to be an offset or miscalculation. The reported line numbers are
consistently lower than the actual locations. This may be related to:

- YAML frontmatter not being counted
- Code blocks being counted differently
- An off-by-N error in the line tracking

## Suggested Fix

1. Add tests that verify line number accuracy
2. Ensure line counting starts from 1 (not 0)
3. Check if frontmatter lines are being excluded from the count
4. Verify code block handling doesn't skip lines

## Impact

Incorrect line numbers significantly slow down the debugging process when
fixing document issues.
