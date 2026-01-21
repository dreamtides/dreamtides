---
lattice-id: LDVWQN
name: add-module-structure-auto-mode-overseer-
description: Add module structure for auto_mode and overseer_mode
task-type: task
priority: 0
labels:
- auto-overseer
- foundation
- structure
blocking:
- LDDWQN
- LDFWQN
- LDUWQN
- LDCWQN
created-at: 2026-01-21T04:02:53.062125Z
updated-at: 2026-01-21T04:27:51.149124Z
closed-at: 2026-01-21T04:27:51.149124Z
---

## Overview

Create the module directory structure and mod.rs files for the new auto_mode and overseer_mode modules. This is a prerequisite for all other implementation tasks.

## Implementation Steps

1. **Create auto_mode module**:
   - Create `src/auto_mode/` directory
   - Create `src/auto_mode/mod.rs` with module declarations
   - Planned submodules:
     - `auto_orchestrator.rs` - main orchestration
     - `auto_config.rs` - configuration parsing
     - `auto_workers.rs` - worker lifecycle
     - `auto_accept.rs` - accept workflow
     - `heartbeat_thread.rs` - heartbeat mechanism
     - `task_pool.rs` - task pool command execution

2. **Create overseer_mode module**:
   - Create `src/overseer_mode/` directory
   - Create `src/overseer_mode/mod.rs` with module declarations
   - Planned submodules:
     - `overseer_loop.rs` - main loop
     - `overseer_config.rs` - configuration
     - `health_monitor.rs` - daemon health monitoring
     - `remediation_prompt.rs` - prompt construction
     - `remediation_executor.rs` - execution and logging
     - `overseer_session.rs` - Claude session management

3. **Update lib.rs**:
   - Add `pub mod auto_mode;`
   - Add `pub mod overseer_mode;`

4. **Create placeholder files**:
   - Create empty .rs files for each planned submodule
   - Add TODO comments indicating purpose
   - This allows other tasks to be worked on in parallel

5. **Verify compilation**:
   - Run `just check` to ensure module structure compiles
   - Fix any module path issues

## Acceptance Criteria

- Module directories created
- mod.rs files declare all submodules
- lib.rs exports new modules
- Placeholder files exist for all planned submodules
- Project compiles with empty modules
