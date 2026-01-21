---
lattice-id: LDJWQN
name: implement-transient-vs-hard-failure-hand
description: Implement transient vs hard failure handling in auto mode
task-type: feature
priority: 1
labels:
- auto-mode
- auto-overseer
- core
blocking:
- LDTWQN
blocked-by:
- LDIWQN
created-at: 2026-01-21T04:00:48.594696Z
updated-at: 2026-01-21T04:09:13.812358Z
---

## Overview

Implement the two-tier error handling system that distinguishes between transient failures (patrol can recover) and hard failures (trigger shutdown).

## Implementation Steps

1. **Define error categories**:
   - Transient failures (patrol attempts recovery):
     - Worker Claude Code session crashes
     - TMUX session disappears unexpectedly
   - Hard failures (immediate shutdown):
     - task_pool_command returns non-zero
     - post_accept_command returns non-zero
     - Worker enters error state after patrol retries exhausted
     - Rebase failure during accept
     - State file corruption
     - Hook IPC failures

2. **Extend patrol for auto mode**:
   - Modify patrol to track retry counts for transient failures
   - Implement retry with backoff (up to 2 retries)
   - If retry succeeds, continue normally
   - If retries exhausted, escalate to hard failure

3. **Implement escalation logic**:
   - When patrol detects transient failure in auto mode:
     - Attempt recovery (restart session, recreate worktree)
     - Track attempt count per worker
     - After 2 failed attempts, mark as hard failure
   - Hard failures trigger shutdown sequence

4. **Update worker state tracking**:
   - Add `auto_retry_count: u32` to worker record (or track in memory)
   - Reset retry count on successful task completion
   - Track which failures are retryable vs immediate shutdown

5. **Logging**:
   - Log transient failure detection and recovery attempts
   - Log escalation from transient to hard failure
   - Include full context for debugging

## Acceptance Criteria

- Single worker crash is recovered automatically by patrol
- Repeated crashes (>2) escalate to shutdown
- Hard failures trigger immediate shutdown
- Clear logging distinguishes failure types
- Existing patrol behavior unchanged for non-auto mode
