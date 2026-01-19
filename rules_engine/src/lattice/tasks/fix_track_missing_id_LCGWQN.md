---
lattice-id: LCGWQN
name: fix-track-missing-id
description: lat track fails when lattice-id field is missing from frontmatter
parent-id: LCEWQN
task-type: bug
priority: 1
created-at: 2026-01-19T05:15:00Z
updated-at: 2026-01-19T05:20:00Z
---

# lat track fails when lattice-id is missing

## Problem

When a markdown file is missing the `lattice-id` field in its frontmatter,
`lat track` fails with a parsing error instead of adding the missing ID.

## Steps to Reproduce

1. Create a markdown file with frontmatter but no `lattice-id` field
2. Run `lat track <file> "description"`
3. Command fails with: `missing field 'lattice-id'`

## Expected Behavior

The primary purpose of `lat track` is to add Lattice tracking (including the
`lattice-id`) to existing markdown files. It should handle files that don't
yet have a `lattice-id`.

## Actual Behavior

The YAML parser requires `lattice-id` to be present, creating a catch-22 where
the command meant to add the ID cannot run without the ID already present.

## Suggested Fix

The track command should:

1. Use a lenient parser that doesn't require `lattice-id`
2. Generate and insert a new `lattice-id` if missing
3. Preserve all other existing frontmatter fields

This is likely the most important bug to fix as it blocks the basic use case
for `lat track`.
