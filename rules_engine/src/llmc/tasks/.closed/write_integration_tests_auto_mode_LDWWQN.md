---
lattice-id: LDWWQN
name: write-integration-tests-auto-mode
description: Write integration tests for auto mode
task-type: task
priority: 2
labels:
- auto-mode
- auto-overseer
- testing
blocked-by:
- LDIWQN
- LDTWQN
- LDRWQN
created-at: 2026-01-21T04:03:13.156452Z
updated-at: 2026-01-21T19:12:48.652150Z
closed-at: 2026-01-21T19:12:48.652150Z
---

## Overview

Write integration tests that verify the auto mode functionality works correctly end-to-end.

## Implementation Steps

1. **Create test infrastructure**:
   - Mock task_pool_command that returns predefined tasks
   - Mock post_accept_command that validates merged commits
   - Test fixtures for configuration

2. **Task pool command tests**:
   - Test: command returns task, worker receives it
   - Test: command returns empty, daemon waits
   - Test: command returns non-zero, daemon shuts down
   - Test: command output logged correctly

3. **Auto accept tests**:
   - Test: completed worker automatically merged to master
   - Test: no_changes worker reset without merge
   - Test: post_accept_command executed after merge
   - Test: post_accept_command failure triggers shutdown

4. **Worker lifecycle tests**:
   - Test: auto workers created on startup
   - Test: auto workers cannot receive llmc start
   - Test: auto workers reset correctly
   - Test: concurrency limit respected

5. **Error handling tests**:
   - Test: transient failure (single crash) recovered by patrol
   - Test: repeated crashes escalate to shutdown
   - Test: hard failures trigger immediate shutdown
   - Test: worktree state preserved on shutdown

6. **Heartbeat tests**:
   - Test: heartbeat file updated regularly
   - Test: daemon registration written correctly
   - Test: stale heartbeat detectable

## Acceptance Criteria

- All happy path scenarios tested
- Error conditions tested
- Tests use mocks to avoid real Claude sessions
- Tests run in reasonable time
- Tests are reliable (no flakiness)
