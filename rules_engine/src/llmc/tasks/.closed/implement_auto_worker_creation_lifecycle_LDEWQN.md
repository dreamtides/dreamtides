---
lattice-id: LDEWQN
name: implement-auto-worker-creation-lifecycle
description: Implement auto worker creation and lifecycle management
parent-id: LBSWQN
task-type: feature
priority: 1
labels:
- auto-mode
- auto-overseer
- foundation
blocking:
- LDHWQN
- LDQWQN
- LDGWQN
blocked-by:
- LDCWQN
- LDUWQN
created-at: 2026-01-21T04:00:03.919357Z
updated-at: 2026-01-21T22:31:38.711151Z
closed-at: 2026-01-21T05:28:06.118248Z
---

## Overview

Implement the auto worker management system that creates and tracks workers
specifically for autonomous operation (auto-1, auto-2, etc.).

## Implementation Steps

1. **Create auto_workers.rs** under `src/auto_mode/`:
   - Function to generate auto worker names: `auto-1`, `auto-2`, ..., `auto-N`
   - Function to create missing auto workers up to configured concurrency
   - Function to identify if a worker is an auto worker (by name prefix)

2. **Update State struct** in `state.rs`:
   - Add `auto_mode: bool` field to track if daemon is running in auto mode
   - Add `auto_workers: Vec<String>` to track which workers are auto-managed
   - Add `last_task_completion_unix: Option<u64>` for stall detection
   - Ensure backward compatibility (new fields should be optional/defaulted)

3. **Implement auto worker creation logic**:
   - Reuse existing `llmc add` logic internally
   - Auto workers should have `excluded_from_pool: true` set automatically
   - Create worktrees at `.worktrees/auto-1`, etc.
   - TMUX sessions named `llmc-auto-1`, etc.

4. **Add command restriction enforcement**:
   - Modify `llmc start` command to reject auto workers with clear error message
   - Ensure other commands (`attach`, `peek`, `reset`, `nuke`, `pick`) work
     normally

5. **Handle auto worker cleanup**:
   - `llmc nuke --all` should include auto workers
   - `llmc reset --all` should include auto workers
   - Document behavior in help text

## Acceptance Criteria

- Auto workers are created automatically when daemon starts in auto mode
- Auto workers cannot receive manual `llmc start` tasks
- Auto workers are correctly tracked in state
- Existing commands work with auto workers (except start)
