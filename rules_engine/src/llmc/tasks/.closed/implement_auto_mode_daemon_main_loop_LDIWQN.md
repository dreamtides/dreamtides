---
lattice-id: LDIWQN
name: implement-auto-mode-daemon-main-loop
description: Implement auto mode daemon main loop
parent-id: LBSWQN
task-type: feature
priority: 1
labels:
- auto-mode
- auto-overseer
- core
blocking:
- LDJWQN
- LDTWQN
- LDWWQN
- LDKWQN
blocked-by:
- LDGWQN
- LDHWQN
- LDFWQN
created-at: 2026-01-21T04:00:48.442009Z
updated-at: 2026-01-21T22:31:38.704819Z
closed-at: 2026-01-21T05:51:22.412527Z
---

## Overview

Implement the main orchestration loop for auto mode that coordinates task
assignment, completion detection, and automatic acceptance.

## Implementation Steps

1. **Create auto_orchestrator.rs** under `src/auto_mode/`:
   - Main entry point: `run_auto_mode(config: &Config) -> Result<()>`
   - Orchestrates all auto mode components

2. **Startup sequence**:
   - Validate configuration (task_pool_command required)
   - Write daemon registration to `.llmc/daemon.json`
   - Create/verify auto workers exist (up to concurrency limit)
   - Start heartbeat background thread
   - Initialize dedicated log files (task_pool.log, post_accept.log, auto.log)

3. **Main loop (per cycle)**:
   - Check for shutdown signal (Ctrl-C)
   - For each idle auto worker:
     - Execute task pool command
     - If task available: assign to worker (reuse `llmc start` logic)
     - If no tasks: skip worker this cycle
     - If error: initiate shutdown
   - For each completed worker (needs_review or no_changes):
     - Execute auto accept workflow
     - If error: initiate shutdown
   - Run patrol (existing patrol logic)
   - Sleep for patrol_interval_secs

4. **Shutdown handling**:
   - On any hard error, set shutdown flag
   - Allow current operations to drain (with timeout)
   - Execute graceful shutdown (Ctrl-C to workers, then kill)
   - Preserve all worktree state
   - Clean up daemon registration file
   - Exit with non-zero code

5. **Logging to auto.log**:
   - Task assignments (worker name, task excerpt)
   - Successful accepts (worker name, commit SHA)
   - Errors with full context
   - Shutdown initiation and reason

## Acceptance Criteria

- Daemon correctly orchestrates task assignment and acceptance
- Heartbeat runs independently of main loop
- Any error triggers orderly shutdown
- Detailed logging enables debugging
- Clean startup and shutdown sequences
