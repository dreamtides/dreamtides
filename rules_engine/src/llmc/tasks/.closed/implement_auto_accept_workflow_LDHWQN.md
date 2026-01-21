---
lattice-id: LDHWQN
name: implement-auto-accept-workflow
description: Implement auto accept workflow
parent-id: LBSWQN
task-type: feature
priority: 1
labels:
- auto-mode
- auto-overseer
- core
blocking:
- LDIWQN
blocked-by:
- LDEWQN
created-at: 2026-01-21T04:00:48.185613Z
updated-at: 2026-01-21T22:31:38.698811Z
closed-at: 2026-01-21T05:42:00.315647Z
---

## Overview

Implement the automatic acceptance workflow that merges completed worker changes
to master without human review, and executes the optional post-accept command.

## Implementation Steps

1. **Create auto_accept.rs** under `src/auto_mode/`:
   - Implement `auto_accept_worker(worker: &WorkerRecord) -> Result<()>`
   - Reuse existing accept logic from `commands/accept.rs` where possible
   - Key difference: no human review step, no interactive prompts

2. **Accept workflow steps**:
   - Verify worker is in `needs_review` or `no_changes` state
   - For `no_changes`: simply reset worker to idle, return early
   - For `needs_review`:
     - Rebase worker branch onto master
     - If rebase conflict occurs, return error (triggers shutdown)
     - Squash commits to single commit
     - Strip agent attribution ("Generated with", "Co-Authored-By")
     - Fast-forward merge to master
     - Reset worker to idle with fresh worktree

3. **Post-accept command execution**:
   - If `post_accept_command` is configured, execute it after successful merge
   - Execute in caller's shell environment (like task pool command)
   - Log stdout/stderr to `logs/post_accept.log`
   - Non-zero exit code triggers shutdown
   - Block until command completes (may be long-running tests)

4. **Update stall detection tracking**:
   - On successful accept, update `last_task_completion_unix` in state
   - This timestamp is used by overseer to detect stalls

5. **Error handling**:
   - Rebase conflicts are hard errors (shutdown)
   - Post-accept command failure is hard error (shutdown)
   - Log detailed context for all errors

## Acceptance Criteria

- Workers in needs_review are automatically merged to master
- Workers in no_changes are reset without merge
- Post-accept command runs after each successful merge
- Completion timestamps tracked for stall detection
- All errors trigger graceful shutdown with detailed logs
