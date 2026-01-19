---
lattice-id: LCLWQN
name: implement-doctor
description: Implement the lat doctor command for system health diagnostics
task-type: feature
priority: 3
parent-id: LCEWQN
created-at: 2026-01-19T05:15:00.000000Z
updated-at: 2026-01-19T05:15:00.000000Z
---

# Implement lat doctor command

## Problem

The `lat doctor` command is listed in the help output but returns "doctor
command not yet implemented" when run.

## Current Behavior

```
$ lat doctor
Error: Cannot perform operation: doctor command not yet implemented
```

## Expected Behavior

The doctor command should diagnose system health issues including:

- Index integrity (SQLite database)
- Git repository state
- Configuration validity
- Orphaned resources
- Skill symlink status

## Suggested Implementation

Based on the design doc, doctor should check:

1. **Index Health**
   - Can the SQLite database be opened?
   - Are all indexed documents still present on disk?
   - Are there documents on disk not in the index?

2. **Git State**
   - Is the working directory a git repository?
   - Are there uncommitted changes to lattice documents?
   - Is the index consistent with git's view of files?

3. **Configuration**
   - Does `.lattice/config.toml` exist and parse correctly?
   - Are all configured paths accessible?

4. **Orphaned Resources**
   - Documents with no incoming links (orphans)
   - Broken links to non-existent documents
   - Duplicate lattice IDs

5. **Optional: Repair Mode**
   - `lat doctor --repair` to fix common issues
   - Rebuild index from scratch
   - Remove orphaned index entries

## Priority

Lower priority since `lat check` covers document validation. Doctor is more
for infrastructure/system issues.
