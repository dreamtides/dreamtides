---
lattice-id: LCKWQN
name: fix-check-code-blocks
description: lat check detects markdown issues inside code blocks
parent-id: LCEWQN
task-type: bug
priority: 2
created-at: 2026-01-19T05:15:00Z
updated-at: 2026-01-19T05:20:00Z
---

# lat check detects headings inside code blocks

## Problem

The linter incorrectly reports W014 warnings ("heading should have blank line
before/after") for content inside code blocks that looks like markdown but
isn't.

## Steps to Reproduce

1. Create a document with a TOML code block containing section headers
2. Run `lat check`
3. Warnings appear for `[defaults]`, `[repo]`, and `[workers.adam]`

The TOML section headers like `[defaults]` are being treated as markdown.

## Expected Behavior

Content inside fenced code blocks should be completely ignored by the markdown
linter. TOML section headers should not be treated as markdown.

## Actual Behavior

The linter processes code block content as if it were markdown, incorrectly
flagging TOML `[section]` headers as markdown elements that need blank lines.

## Suggested Fix

1. Track code fence state while parsing
2. Skip all linting rules for lines inside code fences
3. Handle both backtick and tilde fences
4. Handle nested/escaped fences correctly

## Notes

This is a common issue in documentation linters. The fix should also apply to
other inline code scenarios to prevent false positives.
