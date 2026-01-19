---
lattice-id: LCIWQN
name: fix-fmt-blank-lines
description: lat fmt produces output with consecutive blank lines that fail lat check
parent-id: LCEWQN
task-type: bug
priority: 2
created-at: 2026-01-19T05:15:00Z
updated-at: 2026-01-19T05:20:00Z
---

# lat fmt produces output that fails lat check

## Problem

After running `lat fmt`, some files contain consecutive blank lines that
`lat check` then warns about (W012).

## Steps to Reproduce

1. Create a document with comments in TOML configuration
2. Run `lat fmt`
3. Run `lat check`
4. Warning W012 appears about consecutive blank lines

## Expected Behavior

The formatter should produce output that passes the linter. A file should
never fail `lat check` immediately after running `lat fmt`.

## Actual Behavior

The formatter introduces blank lines around certain content (possibly trying
to add spacing around headings or comments) that violate the W012 rule about
consecutive blank lines.

## Suggested Fix

Either:

1. Ensure the formatter collapses consecutive blank lines to single blanks
2. Or adjust the logic that adds blank lines around elements to not create
   doubles
3. Run a post-processing pass to remove consecutive blank lines

The invariant "fmt output passes check" should be enforced in tests.
