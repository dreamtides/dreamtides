---
lattice-id: LCUWQN
name: lat-doctor---fix-does-not-fix-wal-health
description: lat doctor --fix does not fix WAL health issues
task-type: bug
priority: 2
labels:
- doctor
- sqlite
created-at: 2026-01-20T06:52:53.816299Z
updated-at: 2026-01-20T06:52:53.816299Z
---

## Context
Discovered during manual testing of scenario LCCWQN.

## Steps to Reproduce
1. Use `lat` normally for a period of time
2. Run `lat doctor` - observe WAL Health failure
3. Run `lat doctor --fix` - issue is not fixed

## Expected Behavior
`lat doctor --fix` should either:
- Checkpoint the WAL file to reduce its size, OR
- Not report WAL size as a fixable error if it can't be fixed automatically

## Actual Behavior
- `lat doctor` reports: `WAL Health: WAL corruption detected` with message "WAL file has unusual size (3411392 bytes)"
- Suggests running `lat doctor --fix`
- Running `lat doctor --fix` does not fix the issue - same error persists

## Command Output
```
$ ls -la .lattice/
.rw-r--r--@ 618k index.sqlite
.rw-r--r--@  33k index.sqlite-shm
.rw-r--r--@ 3.4M index.sqlite-wal

$ lat doctor --fix
...
✖  WAL Health WAL corruption detected
   └─ WAL file has unusual size (3431992 bytes)
...
✖  ERRORS
  1. WAL Health: WAL corruption detected
     └─ Fix: lat doctor --fix
```

## Notes
- The WAL file is ~3.4MB vs main database of ~618KB
- "Corruption" seems like a strong term for a large WAL file - this may just need checkpointing
- Two potential issues: (1) WAL not being checkpointed during normal use, (2) --fix not implementing the fix