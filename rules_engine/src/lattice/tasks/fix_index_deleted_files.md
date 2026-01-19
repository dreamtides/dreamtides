---
lattice-id: LCMWQN
name: fix-index-deleted-files
description: Index does not refresh when files are deleted from the filesystem
task-type: bug
priority: 2
parent-id: LCEWQN
created-at: 2026-01-19T05:15:00.000000Z
updated-at: 2026-01-19T05:15:00.000000Z
---

# Index doesn't refresh when files are deleted

## Problem

When a Lattice document is deleted from the filesystem (or removed from git
staging), it continues to appear in `lat list` and other index-based queries.

## Steps to Reproduce

1. Create a document with `lat create`
2. Delete the file with `rm` or through git
3. Run `lat list`
4. The deleted document still appears in the list

## Expected Behavior

The index reconciliation should detect that a file in the index no longer
exists on disk and remove it from the index.

## Actual Behavior

The SQLite index retains entries for deleted files. The reconciliation
process doesn't check for missing files.

## Root Cause Analysis

According to the design, Lattice discovers documents via `git ls-files`. The
reconciliation should:

1. Get the current list of documents from git
2. Compare against documents in the index
3. Remove index entries for files not in git
4. Add/update entries for files in git but stale in index

The "remove entries for missing files" step appears to be missing or not
working correctly.

## Suggested Fix

1. During index reconciliation, query all document paths from the index
2. For each indexed path, verify it exists in the git file list
3. Delete index entries for paths no longer tracked by git
4. Ensure this runs opportunistically before list/show/search commands

## Impact

Stale index entries cause confusion and incorrect query results. Users may
try to reference documents that no longer exist.
