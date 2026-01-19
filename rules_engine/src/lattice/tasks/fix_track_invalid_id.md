---
lattice-id: LCFWQN
name: fix-track-invalid-id
description: lat track cannot recover from invalid lattice-id values
task-type: bug
priority: 2
parent-id: LCEWQN
created-at: 2026-01-19T05:15:00.000000Z
updated-at: 2026-01-19T05:15:00.000000Z
---

# lat track cannot recover from invalid lattice-id

## Problem

When a file has an invalid `lattice-id` value like "PLACEHOLDER", `lat track`
fails to parse the file before it can fix the ID.

## Steps to Reproduce

1. Create a markdown file with frontmatter containing `lattice-id: PLACEHOLDER`
2. Run `lat track <file> "description"`
3. Command fails with: `Invalid Lattice ID format: PLACEHOLDER`

## Expected Behavior

`lat track --force` should be able to replace an invalid ID with a valid one,
or at minimum `lat track` should detect the invalid ID and offer to replace it.

## Actual Behavior

The YAML parser rejects the file before the track command can process it.

## Suggested Fix

Add special handling in the track command to:

1. First attempt to parse with a lenient parser that accepts invalid IDs
2. If an invalid ID is detected, replace it with a newly generated valid ID
3. Alternatively, add a `--force` flag that regenerates IDs unconditionally
