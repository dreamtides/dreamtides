---
lattice-id: LDZWQN
name: auto-mode-daemon-crashes-when-removing-w
description: Auto mode daemon crashes when removing worktree after accept
parent-id: LB6WQN
task-type: bug
priority: 1
labels:
- llmc-auto
- auto-mode
- worktree
- crash
blocking:
- LH3WQN
created-at: 2026-01-22T04:11:33.655427Z
updated-at: 2026-01-22T05:03:53.083604Z
closed-at: 2026-01-22T05:03:53.083604Z
---

# Bug: Auto mode daemon crashes when removing worktree after accept

## Description

During the "Auto Mode Concurrency with Multiple Workers" test (LH3WQN), the auto
mode daemon crashed after successfully accepting changes from auto-1 worker. The
crash occurred during worktree removal.

## Error Message

```
{"timestamp":"2026-01-22T04:09:57.493Z","level":"ERROR","event":{"type":"error","context":"process_completed_workers","error":"Auto accept failed for worker 'auto-1': Failed to remove worktree: Failed to manually remove worktree directory: /tmp/llmc-concurrency-test-96819/.worktrees/auto-1"}}
{"timestamp":"2026-01-22T04:09:58.133Z","level":"INFO","event":{"type":"daemon_shutdown","instance_id":"d45eba9c-d7e1-4230-9a95-5a506ee1dad2","reason":"Error: Auto mode daemon shutdown due to error: Auto accept failed for worker 'auto-1': Failed to remove worktree: Failed to manually remove worktree directory: /tmp/llmc-concurrency-test-96819/.worktrees/auto-1"}}
```

## Sequence of Events

1. Auto mode started with concurrency=2
2. Both auto-1 and auto-2 workers created successfully ✓
3. Task 1 assigned to auto-1 ✓
4. Task 2 assigned to auto-2 ✓
5. auto-1 completed and accept succeeded (commit: ecb4476b) ✓
6. **CRASH**: Failed to remove worktree directory for auto-1
7. Daemon shutdown, auto-2 never completed

## Observed State After Crash

The worktree directory `/tmp/llmc-concurrency-test-96819/.worktrees/auto-1/`
still exists but only contains a `target/` subdirectory (build artifacts). This
suggests partial cleanup occurred before failure.

## Expected Behavior

The daemon should:
1. Successfully remove the worktree after accepting changes
2. Continue processing remaining workers (auto-2)
3. Not crash the entire daemon on a single worktree removal failure

## Possible Causes

- Race condition between file operations and git worktree remove
- `target/` directory may have been locked or in use
- Git worktree not properly unlinked before directory removal attempted

## Environment

- LLMC_ROOT: /tmp/llmc-concurrency-test-96819 (isolated test instance)
- concurrency: 2
- Source: ~/Documents/GoogleDrive/dreamtides

## Repro Steps

1. Create isolated LLMC instance
2. Configure auto mode with concurrency=2
3. Create task pool returning 4 simple file creation tasks
4. Run `llmc up --auto`
5. Wait for first worker to complete and be accepted
6. Observe crash during worktree removal